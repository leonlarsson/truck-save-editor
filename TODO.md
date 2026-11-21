# TODO

Agreed work, in order. Each step should end with something the owner can see or try.

## 1. Foundations

- [ ] Cargo workspace (edition 2024) with `crates/sii`, `crates/save`, `crates/app`; mise tasks working.
- [ ] **GPUI test app:** a window with a styled money field, a skills picker and a virtual table of 2,000 rows, in light and dark. Run it under WSLg, and build it for Windows with `mise run win` and run it there. Decides whether GPUI stays (`docs/decisions.md`).

## 2. Reading saves (`sii`)

- [ ] ScsC decrypt, binary SII decode, text SII parse, text SII write.
- [ ] Round-trip tests on synthetic samples (committed) and on fixtures (local, skipped when missing).
- [ ] Decoding a large save takes well under a second in a release build.

## 3. The in-game check

- [ ] Write a decoded save back unchanged, as text, into a backed-up slot; the owner loads it in game. Then the same with money changed (and `info.sii` updated). Confirms the plan in `docs/sii-format.md` → *Writing saves back*.

## 4. Picking a save

- [ ] Find Steam Cloud and local profiles; decode profile names; list slots with preview image, date, money and level from `info.sii`.
- [ ] Load in the background with progress.

## 5. Overview

- [ ] Money, level and XP, skills, ADR, HQ city: shown, then editable, with "max" buttons.
- [ ] Changes list, undo and redo.

## 6. Saving

- [ ] Backup, game-running check, changes shown before writing, write `game.sii` + `info.sii`, restore a backup.

## 7. Sections

- [x] Fleet (trucks and coaches, repair), Garages (buy all, upgrade to large, HQ), Drivers (skills, max all).
- [ ] Map: visit every city (`economy.visited_cities[]` + `visited_cities_count[]`; every city has a garage unit), unlock every dealer and recruitment agency (city tokens in `unlocked_dealers[]` / `unlocked_recruitments[]`; the full lists must come from the game's data).
- [ ] Trailers, odometers, plates; jobs.
- [x] All data: every unit and field, editable (shape-checked).
- [ ] All data: show All data edits on the typed pages too (an edited money_account in Overview), and search values across units.
- [x] Reading the game's archives (HashFS v1/v2) and its data: levels, ranks, skill limits, text and city names (`hashfs`, `gamedata`).
- [ ] The game's own icons (skills, ADR classes, brands): a texture decoder (.tobj, GDeflate-compressed DDS). Never commit or bundle SCS artwork; fall back to Lucide icons when the game isn't found.
- [ ] Use the game's language for its names when the app gets more languages (gamedata already takes a language code).
- [ ] Coach garages: coaches show "No garage"; find where the save keeps them.
- [x] HQ city: pick from the cities the company has garages in (Garages page).
- [ ] City names from the game's city definitions (`city_name_localized`), not only by token: a few tokens (`vasteraas`) differ from their text keys.
- [ ] Driver names (not found in `def/` yet; maybe generated or in another archive).

## Keeping up with the game

- [x] Inventory diff: list every unit kind, field and value type in a save, and compare two saves' lists (`docs/compatibility.md`).
- [x] Weekly GitHub Actions check for new game versions (steamcmd.net build id + Steam News); opens an issue with the checklist.
- [ ] Fixtures from more game versions and forms: text saves, Coaches, ATS.
- [ ] Read the save `version` and warn when it's newer than any version the editor was tested with.

## Starting fresh

- [x] Save as a new slot: write the edited save into an empty slot instead of over the original.
- [ ] Presets for a fresh start ("Rich start": money, all skills and ADR, a garage), applied to a new profile made in the game.

## Later

- [ ] Web version (GPUI's web target, or a page around the core compiled to wasm).
- [ ] American Truck Simulator: test on real ATS saves (the code is written for both from the start).
- [ ] A final product name (Truck Save Editor for now, in `APP_NAME`; never a game name, those are SCS trademarks), icon, signed Windows builds, releases on GitHub.
