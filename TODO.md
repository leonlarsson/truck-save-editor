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

- [ ] Fleet (trucks and trailers: repair, odometer, plates), Garages (sizes, upgrade all), Drivers (skills, max all), Map (visited cities, unlock dealers and recruitment agencies), Jobs.
- [ ] All data: every unit and field, searchable and editable.

## Later

- [ ] Web version (GPUI's web target, or a page around the core compiled to wasm).
- [ ] American Truck Simulator: test on real ATS saves (the code is written for both from the start).
- [ ] A final product name (Truck Save Editor for now, in `APP_NAME`; never a game name, those are SCS trademarks), icon, signed Windows builds, releases on GitHub.
