# TODO

Agreed work, in order. Each step should end with something the owner can see or try.

## 1. Foundations

- [x] Cargo workspace (edition 2024) with `crates/sii`, `crates/save`, `crates/app`; mise tasks working.
- [x] **GPUI test app:** a window with a styled money field, a skills picker and a virtual table of 2,000 rows, in light and dark. Run it under WSLg, and build it for Windows with `mise run win` and run it there. Decides whether GPUI stays (`docs/decisions.md`).

## 2. Reading saves (`sii`)

- [x] ScsC decrypt, binary SII decode, text SII parse, text SII write.
- [x] Round-trip tests on synthetic samples (committed) and on fixtures (local, skipped when missing).
- [x] Decoding a large save takes well under a second in a release build.

## 3. The in-game check

- [x] Write a save back as text and load it in game (2026-11-22: money and XP changed, Steam Cloud profile; the first try found the rotation format, now fixed and tested).
- [x] In game (2026-11-22): a truck moved to another garage with its driver, and a cancelled freight market job (after the fix that removes its units), both on a Steam Cloud profile.
- [ ] In game: a save on a local (non-cloud) profile written as a new slot.
- [ ] In game: removing a driver, a truck, a trailer and a garage; forgetting the roads (2026-11-22, not tried in game yet).
- [ ] In game: trailer cargo and empty weight, Allow everywhere, fuel over 100 %, Remove badge, coloured plates (2026-11-22, not tried in game yet).
- [ ] In game: a renamed profile, a copied profile in the game's list, copied settings, the developer console and 128-truck convoys (2026-11-22, not tried in game yet).
- [x] Engines and transmissions from the game's data for a truck's model (2026-11-22; not tried in game yet).
- [x] Make another truck your own; take the player to a city, the headquarters or another save's spot (2026-11-22).
- [ ] In game: Drive it yourself, Take me to a city, Take me to headquarters, another save's spot. The trips rely on schedule_quick_travel and schedule_transfer_to_hq doing what their names say.
- [x] Make another trailer your own (Pull it yourself, 2026-11-22; not tried in game yet).
- [ ] In game: Your day (Rest, Clear the record, Repair the cargo, the clock buttons), Pull it yourself (2026-11-22, not tried in game yet).
- [ ] Report a problem: Settings → About, the palette, the no-saves screen, and the button on error messages open a filled-in GitHub issue (2026-11-22, check on Windows that the browser opens with the text).
- [ ] In game: picked rows (bulk Repair, Fill the tanks, Move to garage, Remove, Max their skills, Make them large) and Save over another… on a Steam Cloud profile (2026-11-22, not tried in game yet).

## 4. Picking a save

- [x] Find Steam Cloud and local profiles; decode profile names; list slots with preview image, date, money and level from `info.sii`.
- [x] Load in the background with progress.
- [x] Say what's shown and where from: the profile's saves, where they're kept and their folder; every folder looked in and what it held; the game's own saves folded to the newest three (2026-11-22).
- [x] No saves, or a profile without saves: say what to do. Add a folder (chosen or dropped on the window: a profiles folder, a profile, one save, or the game's folder in Documents), kept in the settings (2026-11-22).
- [ ] On Windows: Add a folder… and dropping a folder on the window (the dialog and drag and drop only run for real there), and Show the folder opening Explorer.

## 5. Overview

- [x] Money, level and XP, skills, ADR, HQ city: shown, then editable, with "max" buttons.
- [x] Changes list, undo and redo.

## 6. Saving

- [x] Backup, game-running check, changes shown before writing, write `game.sii` + `info.sii`, restore a backup.

## 7. Sections

- [x] Fleet (trucks and coaches, repair), Garages (buy all, upgrade to large, HQ), Drivers (skills, max all).
- [x] Map: visit every city.
- [x] Map: unlock every dealer and recruitment agency (dealers from the city definitions; agencies from a list read off the 1.62 map, see `docs/sii-format.md`).
- [ ] Re-read the recruitment-agency list from the map when a map DLC adds cities (part of the game-update checklist).
- [x] Trailers (owned, with wear and repair, doubles included), odometers, plates, fuel.
- [x] Jobs: the cargo market (every offer, sortable, searchable, the game's company and cargo names), and keep every offer a week.
- [x] Jobs: the current job (cancel with no fee, remove its deadline; finishing isn't offered, see `docs/decisions.md`), and cargo icons.
- [ ] Cancelling quick jobs (a company truck), jobs in your own trailer and special transports, once there's a save to learn their shape from.
- [x] Detail panels: one truck, coach or trailer (plate, odometer, fuel, each part's wear), driver (experience, skills, ADR), garage (size, what's in it).
- [x] Detail panels: move a truck or driver to another garage (the whole slot moves; see `docs/decisions.md`).
- [x] All data: every unit and field, editable (shape-checked).
- [x] All data: edits show on the typed pages (and typed changes in All data), and search across every value.
- [x] Reading the game's archives (HashFS v1/v2) and its data: levels, ranks, skill limits, text and city names (`hashfs`, `gamedata`).
- [x] The game's own icons for skills and ADR classes: GDeflate and texture decoding in `hashfs`, the icon materials in `gamedata`, drawn at exact pixel size by `GameIconView`. Nothing of SCS's is bundled; without the game, the rows show no icon.
- [x] Brand logos and the game's model names in Fleet.
- [x] Company logos and cargo icons on Jobs (loaded as rows show, kept small).
- [x] Pictures from DLC archives too (some cargo icons live there).
- [x] Swedish, and the game's names in the app's language; Settings (language, appearance, game folder, backups).
- [x] German and Polish (2026-11-22), machine-drafted with the game's own terms.
- [ ] A native speaker reviews `de.ftl` and `pl.ftl`. More languages: a `Language`, a `locales/<code>.ftl` (the completeness test lists what's missing).
- [x] Coach garages: coaches have none. Fleet says where each is: driven now (`player.assigned_vehicles`) or parked in a city (`player.my_vehicles[]` → `stored_vehicle_city`).
- [x] HQ city: pick from the cities the company has garages in (Garages page).
- [x] City names from the game's city definitions (`city_name_localized`), not only by token: a few tokens (`vasteraas`) differ from their text keys.
- [x] Driver faces: `driver.<n>` shows the game's face `material/ui/driver/<n>.mat`.
- [ ] Driver names: the game reads `/locale/<lang>/driver_names.sii` when there is one, but ships none, and the default names aren't in any data file. Look again (maybe inside the executable's resources).

- [x] Company name (in `profile.sii`, backed up and checked like a slot; the login in the same file is never touched).

## Keeping up with the game

- [x] Inventory diff: list every unit kind, field and value type in a save, and compare two saves' lists (`docs/compatibility.md`).
- [x] Weekly GitHub Actions check for new game versions (steamcmd.net build id + Steam News); opens an issue with the checklist.
- [ ] Fixtures from more game versions and forms: text saves, Coaches, ATS.
- [x] Read the save `version` and warn when it's newer than any version the editor was tested with.

## Starting fresh

- [x] Save as a new slot: write the edited save into an empty slot instead of over the original.
- [x] Quick start for a fresh career ("Rich start", "Everything unlocked", or your own picks), applied as one change to a new profile made in the game.
- [x] Save over another save of the profile: the way to keep an edited copy in Steam Cloud, where new slots don't show (2026-11-22).
- [ ] Experiment (owner, in game): the game starts a career from `def/initial_save/normal/game.sii` in `def.scs`, a few units with `economy.new_game: true` that the game fills in itself. Does a save made of just that template, saved over a spare slot of an existing profile, start a fresh career in that profile? If it does, "Start over" could be a button.

## Later

- [ ] American Truck Simulator: test on real ATS saves (the code is written for both from the start). Until then, ATS profiles and saves show a warning (2026-11-22). Still ETS2-only: the game's data (city names, ranks, engines, icons, the DLC check), distances in miles, and Settings (the game's folder, whether it's running, its config.cfg).
- [x] App icon (`scripts/make-icon.py`, embedded in the .exe) and a release workflow (version tag → draft release).
- [ ] A final product name (Truck Save Editor for now, in `APP_NAME`; never a game name, those are SCS trademarks), signed Windows builds (see `docs/releasing.md` → Code signing), the first release (the owner's call; the pipeline is ready, `docs/releasing.md`).
- [ ] Web version, on hold (the owner's call, 2026-11-21): the desktop app comes first. The core crates stay free of UI and disk access so it stays possible (GPUI's web target, or a page around the core compiled to wasm), but no work on it until it's picked up again.
