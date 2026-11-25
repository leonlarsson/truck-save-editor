# Decisions

Why things are the way they are, newest last. For anything that changes structure, dependencies, file formats or conventions, add a dated entry: what was decided, why, and what else was considered. Don't rewrite old entries; add a new one that replaces them and mark the old one **Replaced by** a link.

---

## 2026-11-21: A decisions log, not ADR files

One file of short entries instead of a folder of Architecture Decision Records. A small project needs the *why* written down, not the ceremony. If this file grows past easy reading, split it into ADRs then.

## 2026-11-21: Rust and GPUI for the desktop app

**Decided:** the app is written in Rust, with its UI in GPUI (`gpui` + `gpui-component`), pinned to exact versions.

**Why:** the goal is the best product, built on Rust. GPUI renders on the GPU and can look and feel as polished as Zed. Zed has reached 1.0, and Discord announced (November 2026) that its new Rust desktop client uses GPUI, so it has serious backing and its Windows support will keep improving. Upstream GPUI also has an early web target, which may give a web version from the same code later.

**Considered:**
- *Tauri with a web UI:* the most mature UI toolkit, and the same UI runs on the web. Not chosen because the owner wants the app in Rust, and the web part would be TypeScript. This is the fallback if GPUI fails the test below.
- *egui:* quick to build, but hard to make look outstanding.
- *iced, Slint:* smaller ecosystems for tables and forms, and less momentum than GPUI.

**Risks, checked first:** building for Windows from WSL with `cargo-xwin`, and whether `gpui-component` can reach the polish we want (inputs, a 2,000-row virtual table, theming). A short test app settles both before real UI work (`TODO.md`).

## 2026-11-21: Three crates; the core has no UI and builds for wasm

**Decided:** a Cargo workspace with `sii` (file formats, bytes in and bytes out), `save` (the ETS2 save: model, edits, disk) and `app` (GPUI). `sii` and `save` have no UI dependencies and build for `wasm32-unknown-unknown`; disk access in `save` sits behind a `native` feature.

**Why:** the format code is the heart of the app and must be testable on its own. Keeping it UI-free leaves a web version possible (GPUI's web target, or a small web page around the core). `sii` knows nothing about ETS2, so American Truck Simulator can reuse it.

## 2026-11-21: Write saves as plain-text SII

**Decided (needs an in-game check):** saves are written back as plain-text SII, without re-encrypting or re-encoding them as binary.

**Why:** the game reads all three forms, so text is simplest and can be checked by eye. Writing binary would mean reproducing the game's structure ids and lookup tables exactly, for no gain.

**Open:** confirm that game 1.62 loads it, before any editing UI is built on it (`TODO.md`). If it doesn't, write binary SII and encrypt it.

## 2026-11-21: Safe saving is part of the product

**Decided:** every write backs up the slot first (`game.sii.bak-<timestamp>` next to it, plus the old `info.sii`), refuses to write while the game is running, shows the list of changes first, and offers to restore any backup.

**Why:** the game overwrites saves when it exits, and Steam Cloud uploads broken saves as readily as good ones. One lost save and nobody trusts the tool again.

## 2026-11-21: mise manages the tools

**Decided:** Rust (with Clippy, rustfmt and the `x86_64-pc-windows-msvc` target) and `cargo-xwin` are pinned in `mise.toml`, and the common commands are mise tasks.

**Why:** one file says what the project needs, on any machine, with no ad hoc `rustup` or `cargo install`.

## 2026-11-21: Written for ETS2 and ATS, ETS2 first

**Decided:** nothing in the code assumes ETS2 where ATS would differ. What differs between the games (Documents folder name, Steam app id 227300 vs 270880, currency, kilometres vs miles, map and company names, features like buses) lives in one `Game` value in `save`. ETS2 is built, tested and shipped first; ATS follows once someone checks it on real ATS saves.

**Why:** both games run on the same SCS engine with the same save formats and encryption key, so supporting both costs little if it's designed in now, and a lot if it's bolted on later. Twice the audience for an open-source tool.

## 2026-11-21: GPUI through GPUI Kit

**Decided:** the app depends on `gpui-kit` alone (pinned exactly, `=0.10.0`), which pins the matching GPUI snapshot (`gpui-pre`) and brings the styled components (`gpui-component`), unstyled behaviour (`gpui-base`) and Lucide icons. Its two agent skills (API and Coding Guides; Design Guides) are copied into `.claude/skills/` from the same release.

**Why:** `gpui` 0.2.2 on crates.io is older than the GPUI that `gpui-component` builds on, so the two can't be mixed; `gpui-kit` exists to keep them in step. It also brings data tables with virtual scrolling, inputs, forms, theming, AccessKit accessibility and WebAssembly support, which a save editor needs and plain GPUI lacks. It's used in a shipped commercial app (Longbridge Pro).

## 2026-11-21: Windows builds run on Windows

**Decided:** the Windows `.exe` is built by Windows' own Rust toolchain. From WSL, `mise run win` copies the source to the Windows drive and runs `cargo` there (`scripts/windows.sh`). `rust-toolchain.toml` pins the same Rust version as `mise.toml`. Release builds will come from GitHub Actions on a Windows runner.

**Why:** GPUI's Windows renderer compiles its DirectX shaders with `fxc.exe` from the Windows SDK at build time. Its build script only does that when the build itself runs on Windows, so cross-compiling from Linux with `cargo-xwin` fails (`shaders_bytes.rs` missing), and debug builds read the shader source from the build machine's disk at runtime. GPUI Fast has the same build script. Zed builds its Windows releases on Windows too.

**Considered:** patching GPUI's build script to call `fxc.exe` through WSL interop. Rejected: a fork of a dependency that changes weekly.

**Replaces:** the `cargo-xwin` plan in "mise manages the tools" (it's removed from `mise.toml`).

## 2026-11-21: GPUI passes its test

**Decided:** GPUI (through GPUI Kit) stays. The test app (sidebar shell, Overview with amount fields and level meters, Fleet with a 2,000-row sortable table, light and dark themes) builds and runs on Windows through `scripts/windows.sh`, and under WSLg. Tauri is no longer the fallback.

## 2026-11-21: Text through Fluent, English first

**Decided:** every word the app shows lives in `crates/app/locales/en.ftl` (Mozilla's Fluent format), looked up with `t!`. English is the only language for now; adding one is adding a file. The game's own names (cities, cargo, DLC) will come from the game's language files.

**Why:** localizing later means hunting down hundreds of strings and sentences glued from pieces. Fluent handles plural forms and word order per language (Polish and Czech have several plural forms), which simpler key-value systems (like `rust-i18n`, which GPUI Kit uses) get wrong.

## 2026-11-21: Never rebuild units, stop on the unknown

**Decided:** recorded in `docs/compatibility.md`: saves are edited in place (unknown units, fields and values pass through untouched); unknown binary types stop reading instead of being guessed at; a fixture corpus per game version and a weekly game-update check keep it honest.

**Why:** the two known ways to damage a save: rebuilding units from fixed field lists silently drops data the game has added, and a crash mid-write can empty the file.

## 2026-11-21: Read the game's own data from its archives

**Decided:** level tables, rank names, skill limits and the game's text (city names, with accents) are read at runtime from the installed game's `.scs` archives (`hashfs` and `gamedata` crates), found through Steam's library list. Built-in fallbacks (game 1.62's level table, skill limits) cover a missing install.

**Why:** the save holds raw XP and city tokens; showing levels, ranks and proper names needs the game's data, and reading it from the player's own install keeps it right when SCS changes it, works in every language the game has, and keeps SCS's text and artwork out of this open-source repo. The HashFS reader follows `scs_tools`' published format description (MIT) and uses the same CityHash64 variant as the game.

**Considered:** copying tables and names into the repo (goes stale, and redistributes SCS's text); asking players to extract files themselves (friction).

## 2026-11-21: The game's icons, drawn from its textures

**Decided:** `hashfs` decompresses GDeflate (compression `0x30`, ported from `scs_tools`, MIT; see `THIRD_PARTY_NOTICES.md`) and cuts a texture's first image out of its GPU upload layout. The pixel format is checked against the data's size, since some format codes aren't in the known table. `gamedata` decodes BC formats with `bcdec_rs` (a safe pure-Rust port of bcdec), reads the icon materials (`ui.sdf.rfx`: four channels, one colour each from `aux[1..4]`) and draws an icon at a given pixel size. The app draws each icon at the exact device-pixel size it covers, cached, and gives skill icons the theme's colours. ADR placards keep the game's colours over the dark panel the game draws them on.

**Why:** the game's icons make the screens speak the game's language at a glance, and drawing them from the player's install keeps SCS's artwork out of the repo. Drawing at the exact size keeps them crisp at any display scale; letting GPUI scale a 260-pixel texture down blurs or aliases it.

**Considered:** extracting the icons once into the repo (SCS's artwork; not ours to redistribute); Lucide stand-ins (fine as the fallback, but generic); a full DDS export (not needed: only the first image is shown).

## 2026-11-21: Undo and redo as whole copies of the pending changes

**Decided:** everything not saved yet lives in one value in the session (`Pending`: the career, repairs, garages, drivers, cities, All data edits). Every change goes through `Session::change`, which keeps the value from before; Undo and Redo swap whole copies. Discard is a change like any other, so it can be undone. Saving starts a new history. Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y work in the workspace's key context, so a focused text field keeps its own undo.

**Why:** the pending changes are small (a career and a few lists of ids), so copies cost nothing, and a copy can't get out of step with what it undoes the way hand-written inverse operations can. Undoing a Discard turns an easy misclick into nothing.

**Considered:** a log of operations with inverses (more code, and every new kind of change needs its inverse written and tested); undo across saves (the history would describe changes against a file that's no longer on disk).

## 2026-11-21: Where dealers and recruitment agencies come from

**Decided:** dealer cities are read at runtime from the installed game's city definitions (`vehicle_brands[]`), across the base game and every map DLC archive. Recruitment-agency cities ship as a list read off the 1.62 map, filtered at runtime to the cities the install defines. Unlocking only adds tokens of cities the game says have one.

**Why:** the city definitions are a stable, documented-by-use format, so reading them keeps dealers right as maps change. Agencies exist only in the binary map sectors, whose format is versioned and was decoded by searching for known tokens: too fragile to run on players' machines. A list that's missing a new city leaves that agency locked, which is harmless; a guess that adds tokens for cities without one might not be.

**Considered:** unlocking every city (unknown what the game does with a dealer token for a city without a dealer); scanning the map at startup (slow, and breaks when the map format changes).

## 2026-11-21: Every page reads one preview of the save

**Decided:** the session builds the save as it would be written (the loaded save with every pending change applied, the same `prepared()` saving uses) when a page first asks after a change, and keeps it until the next change. Fleet, Garages, Drivers, Map and All data read from it. An All data edit that changes the career (money, experience, skills, HQ) becomes a career change instead of a raw edit, found by applying it to a copy and reading the career back.

**Why:** pages used to overlay their own pending changes on the save as loaded, so a repair didn't show in All data and an All data edit of money didn't show on the Overview: two pictures of one save. One preview makes them agree by construction, and every new kind of change shows everywhere without page code. Building it costs one document copy (about 30 ms on a big save, release build), at most once per change.

**Considered:** overlays per page (the old way; every page must know every change); applying changes to the loaded save directly (loses the "as on disk" side that Undo, Discard and the changes list need).

## 2026-11-21: Read every save back before calling it saved

**Decided:** after writing a slot (or a new slot), the editor reads it back from disk and checks it writes the same text as what was meant to be written. If not, the slot is put back from the backup just made (a new slot is removed) and the save is reported as failed. The comparison is of text, not documents: in the text format an empty array (`loans: 0`) reads back as a single value, which only the game's schema tells apart, and both write the same text. A fixture test runs the check on every real save.

**Why:** "never lose a save" needs more than a backup: a write that didn't land (a full disk, an antivirus, a bug) should be caught before the game, or Steam Cloud, ever sees it.

**Considered:** comparing bytes only (doesn't prove the file parses); comparing documents (fails on every real save because of the empty-array ambiguity, as the fixture test showed).

## 2026-11-21: Moving to another garage moves the whole slot

**Decided:** a garage's `vehicles[i]` and `drivers[i]` are one slot. Moving a truck or a driver to another of the company's garages moves its slot (truck and driver together) into the first slot there with neither, and leaves the old one empty. The slot holding the player (`driver_player`) doesn't move. Nothing else changes: the driver's `current_city` stays where they are, and the game sends them on from there.

**Why:** in current saves every driver's `assigned_truck` and `adopted_truck` is `null`; the pairing is the slot index (checked on every filled slot of a real 1.62 save). The driver also caches values about the truck in their slot (`slot_truck_mass` and others), which stay right only if the pair stays together.

**Considered:** moving a truck or driver alone into a half-filled slot (pairs them with someone new and leaves the cached values stale); asking which slot (the game itself doesn't number them).

## 2026-11-21: Cancelling a job leaves its units in the file

**Decided:** cancelling the current job sets `player.current_job: null` and unhitches the company trailer (`player.assigned_trailer` in older saves, the `player_vehicles` unit's `trailer` in newer ones). The `player_job` unit and the company trailer stay in the file with nothing pointing at them. Only jobs in the player's own truck with a company trailer (freight market, external contracts) can be cancelled, which is every job in the real saves looked at. Finishing a job isn't offered; removing its deadline is.

**Why:** the editor never drops units (writing an unedited save gives the same file back, and edits change values only). When the game delivers a job, the job and trailer units disappear from its next save, so the game keeps only what's reachable; unreferenced units should go the same way. This is to be confirmed in game. Finishing would mean inventing the pay (it isn't stored; the game works it out on delivery), the delivery log entry, XP and statistics.

**Considered:** removing the two units (the first edit that drops units, for no known gain); quick jobs, own-trailer jobs and special transports (no save to learn their shape from yet; shown with Cancel off and the reason).

## 2026-11-22: How releases are built

**Decided:** a version tag builds a draft release on GitHub Actions (`docs/releasing.md`): a portable zip (no installer) with the .exe, license and notices; SHA-256 checksums; a build provenance attestation for the zip and the .exe; notes drafted from `feat`, `fix` and `polish` commits by git-cliff. The tag must match the version in `Cargo.toml`, the build uses `--locked`, and the .exe links the C runtime statically (`.cargo/config.toml`) and carries its version and copyright in its file details (`build.rs`, which now also holds the product name). The owner tries the draft and publishes it. Releases aren't code-signed for now.

**Why:** a save editor is a single program with its settings in the user's folder, so an installer adds nothing but a step. The CI build needed `VCRUNTIME140.dll`, which not every PC has. Attestations and checksums are free and let anyone check a download came from this repository, which matters while the .exe is unsigned. Signing isn't free for an individual in Sweden (Microsoft's service takes individuals only in the USA and Canada); SignPath Foundation's free signing for open source is the next step once there's a release to show them.

**Considered:** an MSI or MSIX installer (Start menu entry, winget; possible later, winget also takes portable zips); the bare .exe as the download (the license and notices should travel with it); GitHub's generated notes (they list pull requests, and work here lands on `main` directly); buying a certificate (a few hundred euros a year, and since 2024 even EV certificates build SmartScreen reputation over time like the rest).

## 2026-11-22: Steam Cloud profiles are saved over existing slots only

**Decided:** for a profile in Steam Cloud, the save dialog turns Save as new slot off and says why; saving over an existing slot (backed up first) is the way. To keep a save as it is, the player makes a new save in the game and edits that one.

**Why:** the first in-game check showed the game lists only the save files Steam knows (Steam's `remotecache.vdf`), and Steam only learns of files the game writes. A slot folder the editor made was written correctly but never appeared in the game. Files Steam already knows (an existing slot, `profile.sii`) are picked up when changed and uploaded.

**Considered:** registering new files with Steam through its API (would mean running as the game); telling the player to turn Steam Cloud off for the profile (works, but moves their saves; a choice for them, not a requirement of the editor).

## 2026-11-22: A save is one tree; units an edit cuts off are removed

**Decided:** replaces "Cancelling a job leaves its units in the file" (2026-11-21). After the changes are applied, every unit no longer reachable from the `economy` unit is removed (`save::drop_unreachable`, in `Session::prepared`), and cancelling a job removes the job and its company trailer with their parts. Nothing else is ever removed, and the order of the rest is kept.

**Why:** the in-game check refused the save with the cancelled job: "There are multiple unit trees in the file … Unable to create economy unit!". The game reads a save as one tree from the economy and treats unreferenced units as a second tree. A fixture test confirms every save the game wrote is exactly one tree (all units reachable), so removing only what's cut off makes the file look like the game's own after a finished job.

**Considered:** keeping the units and pointing something at them (there's nothing in the game that would); refusing edits that cut units off (cancelling a job has to).

## 2026-11-22: The editor checks GitHub for a newer version

**Decided:** when it starts, the editor asks GitHub's API for the repository's latest release (`updates.rs`, with `ureq`) and, if it's newer, says so on the start screen, in the sidebar and in Settings, with a link to the release. It's on by default and can be turned off in Settings. Updating stays a download and a swap of the .exe.

**Why:** a game update can change the save format, and then players need the fixed editor quickly; a portable .exe can't tell them otherwise, and a version without the check can never learn of the next one, so it ships in the first release. One request to GitHub is the only network use; it sends the editor's name and version as its agent and nothing about the player or their saves.

**Considered:** updating in place with one click (download, check the release's checksums, swap the .exe and restart: a likely next step); an installer with automatic updates (Velopack; would replace the portable zip); no check (players wouldn't know).

## 2026-11-22: Removing from the company

**Decided:** a hired driver, a truck without a driver, a trailer not in use and an empty garage (not the headquarters) can be removed, and the roads driven forgotten (`save::remove`, `save::forget_roads`). Each removes its entry from the player's list and the lists that run alongside it (named explicitly per kind; an unknown list of the same length stops the removal instead of guessing), clears it from garages, removes a driver's scheduled events, and leaves what nothing points at any more to `drop_unreachable`. A removed garage takes the shape of the unowned ones. Nothing pays: they're called Remove, not Sell. "Discover every road" isn't offered.

**Why:** players asked to tidy fleets and maps. The lists alongside differ by game version (`driver_flags` and `driver_undrivable_truck_timers` in 1.62, `driver_quit_warned` in older saves, sometimes kept empty), so they're named, and checked on every real save by a fixture test. Every discovered road is a map item id; listing all of them means reading the map's binary sector files, whose format changes with game versions.

**Considered:** paying like the game's sale (its price rules aren't in the save); removing a truck with its driver in one go (two clear steps are easier to undo and to explain); discovering roads from the map sectors (fragile).

## 2026-11-22: Profile tools, and the game's own settings

**Decided:** from the start screen, a profile can be backed up whole (copied into the backups folder), copied under a new name, and have its settings (`config.cfg`) copied over another profile's; the profile's name can be changed with the company's. Settings → In the game turns the developer console and convoys of up to 128 trucks on and off in the game's `config.cfg`. Every write backs up what it replaces and is refused while the game runs.

**Why:** a copy is made as a local profile, because the game lists local profiles from the disk while Steam Cloud only shows files the game wrote (found in the in-game check). The copy's World of Trucks login is emptied, so two profiles never share one, and nothing reads it. Renaming changes `profile_name` only: the folder keeps its name, since Steam Cloud knows the files under it. The game writes `config.cfg` and profiles when it exits, so changes made while it runs would be lost.

**Considered:** zipping backups (a plain folder can be opened and restored by hand, like slot backups); renaming the profile's folder (breaks Steam Cloud); keeping the login in a copy (two profiles linked to one account).

## 2026-11-22: The player's truck and where it stands

**Decided:** a truck in a garage can be made the player's (the two slots swap drivers, the player's vehicles point at it, and its garage becomes the headquarters, where the player's slot is). The player can be moved in three ways, none of which guesses coordinates: ask the game to take them to a city they've been to (`schedule_quick_travel`, with the truck's city set) or to the headquarters (`schedule_transfer_to_hq`) when the save loads; or take the exact spot from another save of the same profile. None of it works during a job; moving to another save's spot also needs the trailer unhitched.

**Why:** a position is an exact placement on a road (`(x, y, z) (w; x, y, z)`), which only the game knows how to choose; the map's road data is in its binary sector files. The game's own travel picks the spot itself, and another save's spot is one the game made. Both are untested in game as of this entry: the quick-travel fields are understood from their names and where they sit.

**Considered:** a map to click on (needs the road network from the binary map files); coordinates typed by hand (no way to know a valid one).

## 2026-11-22: German and Polish, drafted by machine and marked for review

**Decided:** the app speaks German and Polish as well as English and Swedish. Both files were drafted by machine translation, with the terms checked against the game's own German and Polish text (its `locale.scs`: Auflieger, Frachtaufträge, Rynek przesyłek, the skill names), and say at the top that they need a native speaker's review. German writes numbers `2.480.350`; an amount typed with points reads them as thousands when they group by three, otherwise as a decimal point (`2.5m`). In Polish, a name that would follow a preposition goes after a colon instead (`Garaż: Kraków`), because Polish would decline it and a template can't.

**Why:** they're two of the game's largest player languages, and a draft that uses the game's words is more useful than English while a native review is pending.

**Considered:** waiting for native translators (no one to ask yet); leaving the game's terms to the translation (the first draft guessed several wrong, such as Anhänger for Auflieger).

## 2026-11-22: Keeping an edited copy by saving over another slot

**Decided:** the save dialog has Save over another…, next to Save as new slot: it lists the profile's other saves (newest first), asks to confirm, backs the chosen one up, writes the edited save into it under that save's own name (with the original's preview pictures), checks it, and carries on editing it. The open save stays as it was. It works for Steam Cloud profiles, where Save as new slot is off.

**Why:** researching how to make a new save the game loads: a local profile lists any slot folder on disk, but a Steam Cloud profile lists only files Steam knows (`remotecache.vdf`), and Steam learns of files only when the game writes them. So in the cloud the editor can't create a save, but it can fill one the game made: the player saves to a new slot in the game (or uses one they don't need), then saves over it here. A local copy of the whole profile (Profile tools) is the other way.

**Considered:** adding entries to `remotecache.vdf` (Steam's own bookkeeping, undocumented, and a wrong entry could make Steam delete or overwrite saves); building a save from nothing (a new career needs the game's own starting state, which only the game writes).

## 2026-11-22: The start screen says where saves come from; folders can be added

**Decided:** the start screen says which profile's saves it shows, how many, and where they're kept (Steam Cloud or this PC), with Show the folder. Below the saves, it lists every folder it looked in (Steam's cloud folder for each Steam account, the game's folders in Documents, folders added) and what each held; with saves found, folders that aren't there fold into one line. Profiles without saves are listed and say how to make one. A folder can be added by choosing it or dropping it on the window: a profiles folder, a profile, one save or the game's folder in Documents all work out to the profiles folder around them. Added folders are kept in `settings.txt` (one `profile_dir =` line each) and searched for ETS2 unless their path names ATS. The game's own saves show the newest three, with Show more.

**Why:** a list alone doesn't help when it's empty or missing a save: the player can't tell whether the editor looked in the wrong place, the game keeps the profile somewhere else, or there's nothing yet. Saying where it looked turns "No saves found" into something to act on, and adding a folder covers backups, other drives and unusual setups without a settings page. Dozens of autosaves pushed everything else far down.

**Considered:** a settings section for folders (the start screen is where the problem shows); asking for a single save file to open (the editor works on profiles: backups, Save as and Profile tools need the folders around a save).

## 2026-11-22: Reporting a problem opens a filled-in GitHub issue

**Decided:** Report a problem (Settings → About, the palette, the no-saves screen) and Report this problem (on error messages the player can't fix) open a new GitHub issue in the browser, with a short template, the editor's version, the system, the language and, from an error message, the error. Nothing is sent from the editor. The issue text is English and lives in `report.rs`, not the locale files, since it's written for the people fixing it.

**Why:** the editor already uses no account and sends nothing; a prefilled link keeps that, and puts what's needed to reproduce a problem in front of the player to check (an error can name a file) before they post it.

**Considered:** sending reports from the editor (needs a server and consent, and would send data); an issue form template (its fields can't be filled from a link the same way).

## 2026-11-22: A save on its own opens where it is

**Decided:** a save folder outside any profile (copied to the desktop, downloaded), dropped on the start screen or chosen with Add a folder…, opens straight away and saves back where it is, backed up first like any save. It's a profile of its own (`Location::Loose`, "A save on its own"): no profile file, so no company name; no other slots, so Save as new slot is off with the reason and Save over another… has nothing to offer. A save inside a profile's `save` folder still adds that profile's folder instead.

**Why:** the first version refused such saves because the editor was built around profiles, but nothing about editing a save needs one: the profile only adds the company name and the slots around it. Refusing sent people to move files around by hand.

**Considered:** refusing with a message (the first version); copying the save into a profile first (writes where the player didn't ask).

## 2026-11-22: Quick travel when the save loads, instead of trips to a city or the headquarters

**Decided:** the Map page has one button, Quick travel when the save loads, in place of Take me to a city… and Take me to headquarters. It sets the player's `schedule_quick_travel` (and turns `schedule_transfer_to_hq` off), and writes no city. It's off during a job, and for saves without the fields. This replaces the trip part of "The player's truck and where it stands" (2026-11-22).

**Why:** tried in game on a Steam Cloud profile with a truck and a coach. With `schedule_quick_travel`, the game says "Game change detected" (unfinished deliveries cancelled without penalty, the vehicle moved to a safe place) and shows its quick travel map once for each of the player's vehicles, with every visited city to pick, at no cost. The city the editor wrote into the vehicle's `stored_vehicle_city` wasn't the destination: the player picks. `schedule_transfer_to_hq` did nothing at all (the player stayed where they were). So the honest feature is the game's own free quick travel, with the choice made in the game, and the button says what the player will see.

**Considered:** keeping a city picker (the game ignores it); keeping Take me to headquarters (it does nothing).

## 2026-11-22: Discovering every road from the game's map

**Decided:** Map → Discover every road adds the id of every road and junction on the map to the save's discovered roads (`economy.discovered_items[]`), keeping those driven. The ids are read from the installed game, in the background after the rest of its data: each map sector's `.base` file (in `base_map.scs` and every DLC archive), where items start with their type, id and bounding box. Rather than reading every kind of map item, roads (type 3) and prefabs (type 4) are recognised where they start: the type, a non-zero id, then a box whose low corner is below its high one and of a plausible size. The whole map takes about 1.5 s.

**Why:** discovered roads are ids of map items, generated by SCS's map editor, not numbers that can be counted out, and only the map files hold them. Reading every item kind would mean following a binary format with dozens of layouts that changes between versions; recognising the two kinds wanted needs only the item's start.

**Considered:** parsing the sector format fully (far more code, and more to break); copying another save's discovered roads (only as good as that save). Checked in game the same day: the map showed 100.00 % explored in every country. The scan finds more items than the game counts (about 179,000), and the game ignores the rest.

## 2026-11-22: The road map, from the game's own map files

**Decided:** Map → Road map draws every road and junction as a dot, the ones driven in the accent colour, with the player's truck, their garages (the headquarters marked) and city names. Everything comes from the same scan of the installed game's map sectors as Discover every road: roads (type 3) and prefabs (4) by their box; city areas (12) and garages (22) the same way, each naming its city with a token 57 bytes into the item. A city's name goes at the middle of its biggest area; only the save's own cities are named (other areas in the files decode to other words). It's drawn in square tiles at fixed zoom levels, each drawn once in the background and kept, so moving and zooming reuse what's drawn; a missing tile is stood in for by a coarser one, cut to its square.

**Why:** the save only holds which roads were driven, by id; the map files are the only source of where anything is. Tiles because drawing a picture of the whole view on every move flickered (edges went blank, dots changed size).

**Considered:** drawing each dot as a shape every frame (hundreds of thousands: too slow); roads as lines (needs each road's end points, deeper in its item than the scan reads); city positions from the save (it has none).

## 2026-11-22: A crash is written down and offered for reporting

**Decided:** a panic hook (`crash.rs`, installed first thing) writes `crash.txt` next to the settings: the version, the system, the error, where it happened and a backtrace, with paths under the home folder written from `~` so the user's name isn't in it. On the next start, the start screen says the editor closed unexpectedly, with Report this problem (a GitHub issue filled in with the report, for the user to check and post) and Dismiss; either removes the file.

**Why:** a crash on someone else's PC otherwise ends with the window gone and nothing learned. The report is sent only by the user, through GitHub, like any other report.

**Considered:** sending crash reports automatically (needs a server and consent, and sends data without asking).

## 2026-11-22: The release is the .exe alone, with every licence inside it

**Decided:** a release ships `truck-save-editor-<version>-windows-x64.exe` on its own (with its checksum and attestation), not a zip. The licences of everything built into it go inside it: cargo-about (from Embark Studios, pinned in `mise.toml`) lists every crate's licence for the Windows build into `crates/app/assets/licenses.txt` (`mise run licenses`, from `about.toml` and `about.hbs`), the app embeds it with `THIRD_PARTY_NOTICES.md`, and Settings → About → Open-source licences shows them. CI regenerates the list on every push and fails if the committed one is out of date. Accepted licences are the permissive ones in `about.toml` and MPL-2.0 (only `option-ext`, unmodified).

**Why:** one file to download and run is simplest for players, and licences inside the .exe go wherever it's copied; the zip held only the editor's own and the adapted code's notices, not the crates' (MIT and Apache-2.0 both ask for their notices with binary copies).

**Considered:** keeping the zip with a generated licences file next to the .exe (the licences only travel with the zip); writing our own licence collector over `cargo metadata` (more to maintain, and easy to get subtly wrong).

## 2026-11-23: The map's things open their panels; "Road map" becomes the Map view

**Decided:** on Map, the switch between the table and the picture reads **List | Map** (it was Cities | Road map), and the code calls the picture the map view (`pages/map_view.rs`, `MapView`, `map-view-*` messages). On the map, a click (let go within 4 px of where it was pressed, so a drag isn't a click) opens what's under the pointer, in the order things are drawn: a city's name opens a new city panel (`details/city.rs`), the truck its truck panel, a garage its garage panel. The city panel lists visits, the truck dealer, the recruitment agency and the garage, each with its action beside it (Mark as visited, Unlock, Open); the list of cities opens it too, by double-click or Enter. Single-city visits are a pending set of their own (`visited_cities`, applied by `save::visit_cities`), shown with Visit every city as one line, "N more cities visited".

**Why:** "Road map" read as a product roadmap and named the roads, not the map; List | Map says what each half is. A map that shows cities and garages but can't open them is a dead end, and the panels already exist for garages and trucks. The list opening the same panel keeps every action reachable by keyboard.

**Considered:** Cities | Map and Table | Map (the owner picked List | Map); a city card drawn on the map itself (a second kind of popup next to the panels everything else uses).

## 2026-11-23: Cities can be unvisited, and dealers and agencies locked again

**Decided:** the city panel turns each fact both ways: Mark as visited or Mark as not visited, Unlock or Lock. Un-visiting takes the city out of `economy.visited_cities` with its count (`save::unvisit_cities`); locking takes it out of `unlocked_dealers` or `unlocked_recruitments` (`save::lock`). Pending, they're sets of their own, applied after the visits and unlocks, and each direction first takes back the other (visiting a city asked to be unvisited brings its count back), so going back and forth ends with no change. Visit every city and Unlock every dealer or agency clear any such asks for their cities. The save dialog shows the cities as "N of M cities visited" before and after, whichever way they went.

**Why:** the owner asked why only one direction was possible; the save holds both as plain lists, so there's no reason not to. Showing both counts reads the same for more and for fewer.

**Considered:** a per-city map of wanted states in place of four sets (simpler to apply, but the existing all-at-once changes are sets, and the change list would need rewriting).

## 2026-11-23: The sidebar: the save's menu at the top, the editor's settings apart from the game's

**Decided:** the save's card at the top of the sidebar opens a menu: Switch save and Backups (they were buttons at the foot). Settings splits in two: **Game settings**, a page under Advanced (`pages/game_settings.rs`) with the game's folder and the game's own `config.cfg` settings; and **Settings**, the editor's own (language, appearance, backups folder, updates, reset, about), at the sidebar's foot beside light or dark, with Ctrl+, (Ctrl+1…9 stay the nine pages in the groups, Game settings the ninth). The start screen offers both, as side panels. A folder's path, shown on both pages, is `ui::path_text`.

**Why:** Switch save at the foot was hard to find, and it's about the open save, which is what the card at the top names: the usual place for a workspace's own menu. The editor's settings and the game's are different things (one changes the editor, the other the installed game), and the game folder is about the game. Settings at the foot, apart from the save's pages, is where most desktop apps put it.

**Considered:** a Switch save button under the card (visible, but one more row above the pages); both settings pages under Advanced (Settings isn't about the save); the game folder with the editor's settings (the game never sees it, but it's about where the game is). The owner picked the save menu and the game folder in Game settings.

## 2026-11-23: Coach services are edited through the game's coach job log

**Decided:** the coach career's numbers (services, passengers, average and best satisfaction) are read the way the game's statistics screen works them out: from `economy.bus_job_log`, one `bus_job_log_entry` per service. The game keeps no totals for them, so editing changes the logged services: Perfect satisfaction sets every service's `satisfaction` to 1, and a new passengers total is spread over the services in proportion to what each carried (whole passengers, the remainder to the first). The coach distance is a stored total, the second item of `economy.total_distances_by_mode[]`, and is set directly. Discover every bus stop sets `discovered: true` on every `bus_stop` in `economy.bus_stops[]`. It's all on Overview, beside the coach experience and skills; Statistics shows the numbers.

**Why:** there's nothing else to edit: the statistics are computed from the log. Changing logged services keeps the number of services real; adding made-up services would write log entries the game never made.

**Considered:** editing each logged service in a table (more control, more to get wrong); adding services to raise the count (the owner picked the simple actions).

## 2026-11-23: The profile picture is the profile folder's avatar.png, written with the profile

**Decided:** the game reads a profile's own picture from `avatar.png` in the profile folder (it has its own "change avatar" too). The editor makes one from any PNG, JPEG or BMP (the largest square from the middle, 256 × 256, `avatar.rs`) and keeps it as a pending change like the profile's name: listed before saving, undone with Undo, written by `save::save_profile` together with `profile.sii`, after the profile folder is backed up (the old picture with it), checked after writing, and put back if anything fails. Removing it deletes the file, so the game shows the profile's face again. The `image` crate gets its PNG, JPEG and BMP decoders.

**Why:** the profile folder is where the game looks; writing it with the profile file keeps one backup and one way to restore. 256 is a guess at the size the game writes (it isn't in the game's data files); the in-game check settles it.

**Considered:** writing the picture straight away (outside the save, Undo and backups); keeping the picture's own size (a huge photo in the profile folder).

## 2026-11-23: Paint jobs and colours come from the game's own paint job definitions

**Decided:** a vehicle's paint is its `vehicle_paint_job_accessory`: `data_path` names the paint job, and `base_color` and `mask_r/g/b_color` are its colours, linear light from 0 to 1 (shown and picked as sRGB, converted both ways in `save::paint`). The paint jobs offered are the model's `accessory_paint_job_data` definitions in every installed archive (`gamedata::model_paint_jobs`), one per name, in the version whose `suitable_for` names one of the vehicle's parts (`<file>.<model>.<folder>`, worked out from its accessories) or names none. Choosing one sets `data_path`, `refund` (its price, as the game keeps it) and the colours it comes in; a colour can then be changed where the definition doesn't lock it (a missing lock is a lock). The saved colours are `economy.user_colors[]`, 32 slots of `0xAABBGGRR` in sRGB (0 is empty), worked out from a save whose saved colours equal colours on its vehicles exactly once converted; they're shared as `#RRGGBB` text (`-` for an empty slot).

**Why:** the definitions say what the game allows: which paint jobs a model has, which fit its cabin, and which colours can change. Writing only what they allow keeps vehicles the game can draw.

**Considered:** listing every definition (the same paint job three times, one per cabin height, two of them wrong for the vehicle); letting every colour change (the game ignores or overrides locked ones).

## 2026-11-23: Making a job copies what the game chose for the cargo

**Decided:** a made job fills a free offer slot of the source company (`company.volatile.<company>.<city>`'s `job_offer[]`, a slot with `cargo: null`), else the offer that expires soonest. What's offered is what the game allows: the companies that ship the cargo and those that take it come from the game's company definitions (`def/company/<company>/{out,in}/*.sii`, `gamedata::company_cargo`), and the trailer (`trailer_variant`, `trailer_definition`), its `units_count`, `fill_ratio` and `company_truck` are copied from an offer in the save that carries the same cargo, so only cargo the save has an offer for can be made. The route is an offer's between the same cities (with its ferry); otherwise the straight line between the cities on the map, scaled by the middle ratio of the save's own offers' distances to their straight lines. A made job lasts a week of game time and is a pending change like any other.

**Why:** a job the game didn't make must still be one it could have made; copying its own choices (trailer, load, truck) and its own company lists keeps it so. Self-scaling the distance avoids guessing how the map's metres relate to the market's kilometres.

**Considered:** building trailers from the cargo's body types in the game's data (more cargo, but a trailer the game might never pair with it); a fixed map scale (unknown, and different per game).

## 2026-11-23: Saved colours are 8 sets of four colours

**Decided:** `economy.user_colors[]` is read and written as 8 saved colours of 4 slots each (slots 0–3, 4–7 …), each slot `0xAABBGGRR` in sRGB as before. A saved colour is a set of up to four colours (the game saves a paint job's colours together); a single colour uses the first slot of its set. The Saved colours panel shows 8 rows of four pickers; the text form joins a set's colours with `/` (`#FF0000/#00FF00/#0000FF #E0FF00`), and the paint pickers offer every colour of every set as quick picks. Replaces the 32 single colours of the paint entry above.

**Why:** seen in game: three colours written to slots 0–2 showed as one striped swatch, and a save whose slots 0 and 4 hold one colour each shows two plain swatches.

## 2026-11-23: Paint jobs and parts only from DLCs the save was made with

**Decided:** paint jobs, engines and transmissions are offered only from the base game and the DLCs the save lists in `info.sii` (`dlc|…` and `rdlc|…` dependencies: the DLCs the game had active when it saved). Each definition carries its DLC's id, read from its archive's manifest (`<archive>.manifest.sii`, a `dlc_package` with `id`, `gamedata::archive_dlc`); the base game's have none. A vehicle's current paint job or part stays in its list whatever it comes from, so the list shows what it has.

**Why:** SCS ships every DLC's archive with the game, owned or not, and Steam checks ownership when the game runs; seen in game: a paint job from a DLC the owner doesn't have was drawn, while the game's own shop didn't offer it. Offering paid DLC content to players who haven't bought it would make the editor a way around paying for DLCs, which it mustn't be (and the game may drop removable DLC content anyway).

**Considered:** offering everything installed (hands out paid content); asking Steam who owns what (needs Steam, and the save's list already says it).

## 2026-11-23: No profile picture

**Decided:** the profile picture is taken out: no `avatar.png` is written, `save::save_profile` (profile file and picture together) is gone again for `save_profile_file`, and the `image` crate is back to no decoders. Replaces "The profile picture is the profile folder's avatar.png" above.

**Why:** checked in game: with the editor's `avatar.png` in the profile folder, Career Selection still showed the profile's face, and the game's own screen for it (My Drivers → Driver Details) only offers the game's faces. The game has no picture of the player's own for a profile, so the feature could never work.

**Considered:** a face picker writing `profile.sii`'s `face` and `male` (the game already does it; the owner left it out).

## 2026-11-23: The city list is the game's, not only the save's garages

**Decided:** the editor's cities are the save's (a garage unit for every city that has one, owned or not, and the visited list) together with the game's own city definitions (`city_data` in `def/city.sii` and each map DLC's `city.*.sii`), kept to the base game's and those of the map DLCs the save was made with, and, once the map is read, to those with a city area on it. The workspace gives the session that list (`Session::set_game_cities`); `save::cities_including` merges it with the save's, and Visit every city covers it.

**Why:** small towns have no garage, so a list made from garages missed about 160 of the game's cities: on the map, in the list, the finder and Visit every city. Every map DLC's files ship with the game, owned or not, hence the filter by the save's DLCs; a few definitions are ports and border posts with no area on the map.

**Considered:** the map's city areas alone (they also name places that aren't cities); the save's companies (a town without companies would still be missing).

## 2026-11-23: Cities are counted as the game counts them

**Decided:** of the game's cities (see the entry above), only those with a company or a garage in the save count (`save::cities_with_business`); the save's own city tokens that aren't game cities (multiplayer's `truckersmp`, event places `x_…`) are left out once the game's list is known. Visit every city visits exactly the counted cities. Adds to "The city list is the game's" above.

**Why:** checked against a save's Career Progress: the defined cities of the save's map DLCs, on the map, with a company or a garage, came to exactly the game's count, and its real visited cities to exactly the game's discovered count. The cities left out are ferry ports and a village with no business at all, and the tokens are places the game visited but doesn't count.

## 2026-11-23: Viewpoints and photo trophies: the game's photo album, read only for photo trophies

**Decided:** the Map page gets a third view, Gallery: every viewpoint and photo trophy from the game's own photo album (`def/photo_album/photo_album.sii`, `viewpoints.sui`, `landmarks.sui` in `def.scs`; `gamedata::PhotoAlbum`), kept to the base game's and the save's map DLCs (each item names its `dlc_id`). Viewpoints come from the save: `economy.discovered_cutscene_items[]` (the viewpoint's cutscene map item, the album item's `objects_uid`) with `discovered_cutscene_items_states[]`, 1 found but not watched, 2 watched (what Career Progress counts). They can be marked visited (state 2) or not found (removed from both lists), one at a time or all (Visit every viewpoint); other cutscenes in the lists are kept as they are (`save::set_viewpoints`). Photo trophies come from the profile's `album/landmarks.sii` (`album_landmarks_data.landmark_id`), else its `landmarks.<id>.png` pictures, and are only shown. On the map, viewpoints are cutscene items (type 46) whose flags' type is 0, and photo trophies are map overlays (type 18) of type 4 named by the trophy's id (`gamedata::MapMarker`).

**Why:** the save and profile hold exactly what the game's Career Progress shows (checked: the same counts). A photo trophy is a photo the player took, kept for the whole profile, not one save; marking one captured would mean writing a capture without a photo, outside the save the player opened. The state meanings and the map item layout aren't documented by SCS; they come from the community's map format notes and fit the game's own lists exactly (every viewpoint's item and every photo trophy's overlay found).

**Considered:** a page of its own (the gallery is about places, so it sits with the map); marking photo trophies captured (left out, as above); finding viewpoints by scanning the map's cutscenes alone (they include other cutscenes, so the album's list decides).

## 2026-11-23: The map is a sidebar group: Cities, World map, Gallery

**Decided:** the Map page is split into three pages under a Map group in the sidebar: Cities (the list, visits, dealers and agencies), World map (the game's name for its map: roads, the truck's position, Find a city) and Gallery (viewpoints and photo trophies). A panel's Show on map opens World map from any page (`workspace::show_on_world_map`). Replaces the Gallery's place as a third view on the Map page (entry above). Ctrl+1…9 now reach Overview to Gallery; All data and Game settings are reached from the sidebar and the palette.

**Why:** one sidebar item named Map led to cities, roads, the truck and the gallery behind a List | Map | Gallery switch, which said little; the owner suggested Map as the group. Each page now holds one thing, named as the game names it.

**Considered:** an Exploration page with Cities | Map | Gallery tabs (the game's Career Progress heading); naming the switch Cities | Map | Gallery on one page.

## 2026-11-23: Coach career is a page of its own; pages have no Ctrl+number shortcuts

**Decided:** everything about the coach career moves to a Coach career page under Career, shown in the sidebar and the palette only for a save with a coach career: coach experience and skills, the coach services (passengers and distance to edit, Perfect satisfaction, Discover every bus stop) and every service in the coach job log (`save::coach_services`). Overview and Statistics are the truck career's; Fleet keeps its Trucks, Coaches and Trailers. Palette commands about coaches open Coach career. Pages lose their Ctrl+1…9 shortcuts (replaces that part of "The map is a sidebar group" above); the palette (Ctrl+K) and the sidebar reach them, and Settings keeps Ctrl+,.

**Why:** the two careers were mixed on Overview (both experience cards side by side, truck skills, ADR, coach skills and coach services one after another). A Truck | Coach switch in the sidebar was tried and dropped: most of the app (Jobs, Garages, Drivers, the map) doesn't change with it, so a global switch promised more than it did. Number shortcuts shifted whenever a page was added, and stopped at nine.

**Considered:** the global switch (above); tabs on each page with both careers (the coach career spread over three pages); splitting Career into Trucking and Coaching with the company on Overview (a bigger reshuffle).

## 2026-11-23: Screenshots come from a script and a made-up demo profile

**Decided:** `scripts/screenshots.sh` takes every README picture (light and dark) and a release's pictures in one run of the dev build. `scripts/demo-profile.py` turns a fixture copy into a demo profile ("Leon") with invented money, experience, HQ city, truck position, plates, save, company and profile names, and photo trophies. The app drives itself through a script (`TSE_SCRIPT`: pages, themes, palette commands by their labels, waits), and asks for each picture through a request file the shell script answers by capturing the window (the app can't capture itself). Development builds also build the crates that read the game's files (`sii`, `hashfs`, `gamedata`, `save`) optimised, like the dependencies.

**Why:** pictures of a real save would put the player's own values in a public repository, and taking them by hand (or by restarting the app and typing per picture) took about ten minutes and broke when layouts moved. A run now takes about half a minute; the optimised crates bring the game's data and map from about 18 seconds to a few in a development build.

**Considered:** keystrokes and clicks at coordinates (slow, and coordinates break with layouts); restarting per picture (each start reloads the game's data); rendering pictures headless in tests (GPUI's test platform draws nothing).

## 2026-11-23: The release workflow uses the hand-written notes and names the release

**Decided:** the release workflow takes a release's notes from `docs/release-notes/<version>.md` when it exists (without its opening comment), else drafts them from the commits with git-cliff as before, and adds the download and checksum sections either way. It titles the release "<product name> <version>", reading the name from `PRODUCT_NAME` in `crates/app/build.rs`. `docs/releasing.md` lists the whole release in order, with what went wrong before.

**Why:** publishing 0.3.0, the notes had to be pasted over the drafted ones by hand, and the release came out titled "v0.3.0" where the earlier ones read "Truck Save Editor 0.2.0", fixed only after publishing. Steps that have to be remembered get forgotten.

**Considered:** a checklist alone (it was there, and the title still slipped); writing the product name into the workflow (the name is written in one place only).

## 2026-11-24: Tooltips wrap: `.tip(text)` instead of `.tooltip(text)`

**Decided:** every tooltip is set with `.tip(text)` (`crates/app/src/ui/tooltip.rs`), which breaks the text into lines of about 60 characters at spaces and then sets the component's own tooltip. Implemented for Button and Checkbox, the components the app gives tooltips to.

**Why:** the kit draws a tooltip as wide as its text, so a long explanation (why a button is off, what an action does) ran off the screen as one line. The kit's Tooltip can take a maximum width, but its components build their tooltip themselves and, in the pinned version, offer no way to pass a styled one.

**Considered:** setting a width per tooltip (not possible through the components); shortening every tooltip (the explanations are the point); patching the kit (it's pinned and upgraded as a change of its own).

## 2026-11-24: The start screen picks the game first, with its logo

**Decided:** the start screen has a game picker above the profile picker: a card for Euro Truck Simulator 2 and one for American Truck Simulator, each with how many profiles and saves were found for it, and the game's logo read from its own install (`material/ui/logo/ets2_logo` in `base.scs`; ATS keeps its logo under the same name). A game with nothing found is shown, off, saying so. The profile picker below shows only the chosen game's profiles.

**Why:** the owner asked for a picker of its own for the game, then the profile, and for the games' logos. Mixing both games' profiles in one row (ATS ones marked "· ATS") read as one list. The logos come from the player's install, like the game's other icons, so none of SCS's artwork is in the repository (as the README says); a game that isn't installed shows its name without a logo.

**Considered:** keeping the logos in the repository (the owner allowed it; it would also show a logo for a game that isn't installed, but the README promises no SCS artwork is included, and the logos are SCS's trademarks).

## 2026-11-25: Recruitment agencies can't be unlocked, only locked

**Decided:** the editor no longer unlocks recruitment agencies, in either game: Unlock every agency and the city panel's agency Unlock are disabled with the reason (the game has to find an agency itself: drive past it), and the palette command and Quick start's choice are gone. Locking an agency stays, as the repair: lock it, then drive past it in the game. `Session::can_unlock` is the one rule. Dealers unlock as before.

**Why:** tested in ATS with the owner (2026-11-25): an agency unlocked in the save (`economy.unlocked_recruitments`) before the game discovered it shows on the game's map, but hiring online asks to discover an agency first, and entering it gives a black screen that Esc doesn't leave. The same save with only the editor's other changes (garage upgrade, experience, money, dealer) works; locking the agency and driving past it fixes it. Players report the same two symptoms in ETS2 with saves whose agencies were unlocked. Every field the game saves on discovery (the agency, one driver moved from `driver_pool` to `drivers_offer`, `screen_access_list`, `discovered_items`) was compared between an unlocked and a discovered agency and matched, so what the game does on discovery isn't in anything the editor can write.

**Considered:** writing the driver offer as the game does (it makes one itself when it loads the save, and that doesn't help); keeping the unlock with a warning (it damages the save until the player repairs it by hand).

## 2026-11-25: Copying a truck or trailer

**Decided:** a truck's or trailer's panel has Copy to a garage, a menu of the company's garages with room (an empty truck slot; for a trailer, fewer trailers than truck slots). `save::copy_vehicle` copies the vehicle and every unit that's part of it (accessories of every kind, a road train's other trailers) under new anonymous ids, appended at the end of the save, and links the copy as the game links one it sells: `player.trucks` with an empty profit log in `player.truck_profit_logs`, or `player.trailers` with an empty usage log in `player.trailer_utilization_logs`, and the garage's slot or trailer list. A trailer's `trailer_def` stays shared. New ids come from `Document::unused_nameless_ids`: after the highest anonymous id in the save, 8 apart, as the game's addresses are. The copy keeps plate, odometer, wear and fuel. It's a pending change like the others (Undo, the save dialog), applied after moves and before vehicle edits, so an edit made to a copy finds it.

**Why:** the first step towards adding trucks and trailers from the editor; a copy needs no knowledge of the game's dealer configurations, and works for both games. The shape was read off saves where the game had just sold a truck and a trailer (ATS) and off ETS2 fleets (204 trucks, 204 profit logs; `driver_undrivable_truck_timers` runs with drivers, not trucks, so a truck doesn't touch it). A part of a kind the code doesn't know stops the copy rather than guessing.

**Considered:** inserting the copies next to the original (units' order doesn't matter to the game's references, and appending keeps every other unit where it was); a fresh plate for the copy (the game allows duplicates, and the plate can be edited right after).

## 2026-11-25: Buying a truck from a dealer's stock

**Decided:** Fleet has Buy a truck…, also in the palette: a side panel listing the trucks the game's dealers sell ready-made (`gamedata::dealer_trucks`: every `/def/vehicle/truck_dealer/<brand>/*.sii` in the base game's definitions and every DLC's, ETS2's right-hand-drive copies left out), those of DLCs the save was made with, by model, chassis, engine and price, and the garages with a free truck slot. `save::buy_truck` gives the truck the dealer file's accessories under new ids, each with `refund` (the part's `price` from its definition, as the game writes it), and takes the truck's own fields from a truck already in the save (the company's, else a dealer's used truck; every save has those), started as new: wear 0 with the per-axle lists empty, a full tank, odometer and trip counters 0, no passengers, bought at the save's game time. Its empty profit log is made like one of the company's trucks' or a garage's. It's linked in as a copy is, and nothing is paid. A pending change like copies, applied right after them.

**Why:** a dealer file is a truck as the game sells it, written almost as a save keeps one, so buying needs no knowledge of which parts fit which model. Checked against the game: the VNL 2018 860 stock offer comes to $194,200, the price the game's dealer showed; refunds matched the parts' prices on a truck the game had just sold; 200 of an ETS2 fleet's trucks carry empty `wheels_wear` lists, so a truck without per-axle wear is one the game writes itself. Taking the fields from another truck rather than a fixed list keeps whatever a later game version adds (`docs/compatibility.md`).

**Considered:** the truck configurator's choices (`truck_desktop`), to build any combination of parts (later, with the dealer's offers as the start); charging the price (the editor already sets money directly; a checkbox could come later); a fresh plate (the other truck's is kept, and can be edited right after).

## 2026-11-25: Buying a trailer from the dealer's stock

**Decided:** Fleet's button follows the list shown (Buy a truck… on Trucks, Buy a trailer… on Trailers; both in the palette), one side panel for both (`pages/buy.rs`). `gamedata::dealer_trailers` reads every `/def/vehicle/trailer_dealer/<brand>/*.sii`: a trailer, or a road train's chain (each trailer pointing at the next as `slave_trailer`), with their accessories priced as trucks' are, and the front trailer's type from `/def/vehicle/trailer_defs/`. `save::buy_trailer` copies the whole chain under new ids; only the front trailer carries the type, the trailers behind it get `trailer_definition: null` (as the game writes a road train it sold); the type is the save's own `trailer_def` of that name (`source_name`) or a new one made from the game's definition plus the fields saves add (as another type in the save has them, else `trailer: ""` and `country_validity: 0`, as the game wrote them), listed in `player.trailer_defs`. Each trailer takes its fields from a trailer in the save started as new (no wear, cargo or damage, not oversize, offsets 0, not private); in a save with no trailer at all, the dealer file's fields alone. The front trailer goes in `player.trailers` with an empty usage log, and in the garage's trailers (fewer than its truck slots, as for copies).

**Why:** completes buying from the dealers for both kinds; the shape was read off a road train the game had just sold (ATS). The no-template case (a new career that never owned a trailer) leans on the game filling in missing fields as it does for its own dealer files, which needs checking in the game.

**Considered:** refusing to buy a trailer into a save without one (safe, but the first trailer is exactly when buying one matters).

## 2026-11-25: Added trucks and trailers get plates of their own

**Decided:** a bought or copied truck or trailer gets a new plate in the format of the one its fields came from: the region after `|` and any formatting tags kept, each letter and digit replaced (letters without I, O and Q), picked from the vehicle's new id so the preview and the save agree (`save/src/plates.rs`). Replaces "the copy keeps its plate" and "the other truck's plate is kept" in the two entries above. The Buy lists show each vehicle on two lines (`ui::DetailChoice`: the name, then chassis, engine and price), and a road train's count of trailers (a dolly, with no body, doesn't count).

**Why:** checked in the app: every bought or copied vehicle came out with the same plate as the company's own truck or trailer, which reads as a mistake in every list. A plate in the right format for its region is what the game's dealers hand out too. The one-line list cut off the price on a panel this wide.

**Considered:** a used dealer truck's plate (unique, but trailers have no used stock); no plate (unknown what the game does with a truck without one).

## 2026-11-25: "Add a truck", not "Buy a truck"

**Decided:** the dealer panels and their buttons say Add (Add a truck…, Add a trailer…, Add), and the description says nothing is paid and the money stays as it is. The code keeps "buy" for what it does with the game's data (`buy_truck`, `dealer_trucks`: the dealer's stock, bought as the game would, without paying).

**Why:** the owner asked whether Buy takes money; it doesn't, so the word promised something the editor doesn't do. Money is set on Overview.

**Considered:** keeping Buy with a Pay for it checkbox that takes the dealer's price (closer to the game's rules; can come later if asked).

## 2026-11-25: Earned screens open when a save is written; agencies can be unlocked again

**Decided:** `economy.screen_access_list` is the list of game screens a career hasn't unlocked yet; the game takes one off when the career first earns it. When a save is prepared for writing, `save::unlock_earned_screens` takes off the ones the career has earned by what it holds: Truck Manager, truck upgrades and diagnostics and the freight market (`screen.truckman`, `truck.upgrades`, `truck.diag`, `jobs.company`) with a truck of its own; Trailer Manager and trailer upgrades (`screen.trailerman`, `trailer.upgrades`) with a trailer of its own; the agency screen (`screen.recruit`) with an unlocked agency. Nothing else is touched. Recruitment agencies can be unlocked again (reverts "Recruitment agencies can't be unlocked, only locked", above): the palette command, Quick start's choice and the city panel's Unlock are back.

**Why:** a trailer added to a save that never had one showed in the garage, but the game said "To access the Trailer Manager you need to buy your first trailer"; `screen.trailerman` was still on the list. Comparing the owner's saves before and after the game sold a truck and a trailer showed which screens it takes off for them, and the agency repaired by a drive past differed from the broken one only by `screen.recruit` coming off: the black screen and the refused online hiring were that one locked screen, not something the game keeps out of the save, as the earlier entry concluded.

**Considered:** taking the screens off only for vehicles the editor adds in this session (the game's saves the editor wrote earlier would stay broken until edited again with that change).

## 2026-11-25: Plates made the game's way

**Decided:** a truck or trailer the editor adds or copies gets a plate the way the game makes one: from the `license_plate_data` of its garage's country or state (`/def/country/<country>/license_plates.sii`: per kind of vehicle, `templates[]` whose digits pick characters from `def0[]`…`def9[]`, everything else, tags included, kept), read with the game's data (`gamedata::make_plate`, `GameData::new_plate`). The app makes the plates when the player presses Add or Copy (seeded by the clock) and keeps them with the change, so the preview and the save agree; a road train gets one per trailer, dollies included, as the game gives them. Without the game's data, the earlier plate of the same format with new characters stays as the fallback (`save/src/plates.rs`). Replaces "Added trucks and trailers get plates of their own", above, where the game's data is there.

**Why:** in a save with nothing to take a plate from (a new career: the dealers' used trucks carry empty plates, and dealer files none), the added truck and trailer came out with blank plates in the game. The game's own formats give the right plate for the region in every case, with ETS2's layout tags and badges.

**Considered:** the region of the company's headquarters (the garage's is where the vehicle lives, and what a dealer there would register it in).

## 2026-11-25: Adding and copying coaches

**Decided:** Fleet's Coaches list (shown for any save with a coach career, even without a coach) has Add a coach…, also in the palette for coach careers: the same panel lists the coach dealer's ready-made coaches (`/def/vehicle/bus_dealer/<brand>/*.sii`, Coaches DLC, read with the truck dealers', `DealerTruck::is_coach`), with no garage to pick. A coach's panel has Copy coach. `save::buy_coach` and `save::copy_coach` put the coach only into `player.buses`: no garage, no profit log, `license_plate_type: bus`, its fields from the player's coach (else a truck) started as new, and a coach plate made the game's way for the headquarters' country.

**Why:** the owner bought a coach at the game's coach dealer and saved: the game had added it to `player.buses` and nothing else, its plate a Swedish coach plate (the headquarters are in Sweden).

**Considered:** a garage for coaches (the game keeps none); the coach's current city for its plate (a new coach has none until it's used).

## 2026-11-25: American Truck Simulator is supported in full

**Decided:** the notice that ATS support is new (on the start screen and on every page of an ATS save) is gone, for 0.4.0; the README names both games.

**Why:** every part of the editor reads ATS's own data, and the owner checked the edits in ATS (money and experience, dealers and agencies, garages, adding and copying trucks and trailers, a career without a truck or trailer of its own).
