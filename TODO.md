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

- [ ] Write a decoded save back unchanged, as text, into a backed-up slot; the owner loads it in game. Then the same with money changed (and `info.sii` updated). Confirms the plan in `docs/sii-format.md` → *Writing saves back*.

## 4. Picking a save

- [x] Find Steam Cloud and local profiles; decode profile names; list slots with preview image, date, money and level from `info.sii`.
- [x] Load in the background with progress.

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
- [ ] Jobs: the current job (finish, cancel), and cargo icons.
- [x] Detail panels: one truck, coach or trailer (plate, odometer, fuel, each part's wear), driver (experience, skills, ADR), garage (size, what's in it).
- [ ] Detail panels: move a truck or driver to another garage.
- [x] All data: every unit and field, editable (shape-checked).
- [x] All data: edits show on the typed pages (and typed changes in All data), and search across every value.
- [x] Reading the game's archives (HashFS v1/v2) and its data: levels, ranks, skill limits, text and city names (`hashfs`, `gamedata`).
- [x] The game's own icons for skills and ADR classes: GDeflate and texture decoding in `hashfs`, the icon materials in `gamedata`, drawn at exact pixel size by `GameIconView`. Nothing of SCS's is bundled; without the game, the rows show no icon.
- [x] Brand logos and the game's model names in Fleet.
- [ ] Cargo icons (Jobs), company logos, from the same decoder.
- [x] Swedish, and the game's names in the app's language; Settings (language, appearance, game folder, backups).
- [ ] More languages: a `Language`, a `locales/<code>.ftl` (the completeness test lists what's missing).
- [x] Coach garages: coaches have none. Fleet says where each is: driven now (`player.assigned_vehicles`) or parked in a city (`player.my_vehicles[]` → `stored_vehicle_city`).
- [x] HQ city: pick from the cities the company has garages in (Garages page).
- [x] City names from the game's city definitions (`city_name_localized`), not only by token: a few tokens (`vasteraas`) differ from their text keys.
- [x] Driver faces: `driver.<n>` shows the game's face `material/ui/driver/<n>.mat`.
- [ ] Driver names: the game reads `/locale/<lang>/driver_names.sii` when there is one, but ships none, and the default names aren't in any data file. Look again (maybe inside the executable's resources).

## Keeping up with the game

- [x] Inventory diff: list every unit kind, field and value type in a save, and compare two saves' lists (`docs/compatibility.md`).
- [x] Weekly GitHub Actions check for new game versions (steamcmd.net build id + Steam News); opens an issue with the checklist.
- [ ] Fixtures from more game versions and forms: text saves, Coaches, ATS.
- [x] Read the save `version` and warn when it's newer than any version the editor was tested with.

## Starting fresh

- [x] Save as a new slot: write the edited save into an empty slot instead of over the original.
- [x] Quick start for a fresh career ("Rich start", "Everything unlocked", or your own picks), applied as one change to a new profile made in the game.

## Later

- [ ] Web version (GPUI's web target, or a page around the core compiled to wasm).
- [ ] American Truck Simulator: test on real ATS saves (the code is written for both from the start).
- [x] App icon (`scripts/make-icon.py`, embedded in the .exe) and a release workflow (version tag → draft release).
- [ ] A final product name (Truck Save Editor for now, in `APP_NAME`; never a game name, those are SCS trademarks), signed Windows builds, the first release (the owner's call).
