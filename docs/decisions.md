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

**Why:** research into why other editors broke: rebuilding units from fixed field lists silently dropped data the game had added, and crashes mid-write emptied saves.

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

**Considered:** unlocking every city (other editors do; unknown what the game does with a dealer token for a city without a dealer); scanning the map at startup (slow, and breaks when the map format changes).

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
