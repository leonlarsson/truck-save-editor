# Truck Save Editor

A save editor for **Euro Truck Simulator 2** and **American Truck Simulator**: fast, safe and pleasant to use.

> **Early development.** It works on real saves, but writing saves hasn't been checked in the game yet. Try it on a copy (Save as new slot) and keep the backups.

## What it does

- **Finds your saves by itself**: Steam Cloud and local profiles, every slot with when it was saved and its money.
- **The big levers first**: money, truck and coach experience shown as levels with the game's own rank names, every truck and coach skill, ADR classes.
- **Coaches as a first-class career**: coach experience, coach skills, and coaches in the fleet.
- **Your company**: every truck and coach (repair them all), every garage (buy them all, upgrade to large), your headquarters, every hired driver (max them all).
- **Everything else**: All data shows every unit and value in the save, editable, with checks so a typo can't break the save.
- **Never lose a save**: every change listed before saving, a backup of the slot before every write (kept outside the game's folders), Save as new slot to leave the original untouched, and no writing while the game is running.
- **Uses the game's own data** from your install: level tables, rank names, skill limits, city names (Malmö, Kraków), and the skill icons and ADR placards, so it stays right when the game changes.
- **In English and Swedish**, with the game's own names (cities, cargo, ranks) in the same language, and a Settings page for the language, light or dark, and the game's folder.
- **Undo and redo** (Ctrl+Z, Ctrl+Shift+Z) for every change, Ctrl+S to review and save, Discard included, a **command palette** (Ctrl+K) for every page and bulk action, and the whole app usable from the keyboard (Ctrl+1…8 for pages, Enter to open a row).

Built in Rust with [GPUI](https://www.gpui.rs/) through [GPUI Kit](https://gpui-kit.com). Windows first; ETS2 first, then ATS.

## Staying compatible

Game updates have broken other editors. This one keeps every part of a save it doesn't understand exactly as it was, stops instead of guessing at anything new, and checks every week whether the game has updated. How, and why, is in [`docs/compatibility.md`](docs/compatibility.md).

## Contributing

Read [`AGENTS.md`](AGENTS.md) (for humans too), [`docs/sii-format.md`](docs/sii-format.md) for how the game's save files work, and [`docs/decisions.md`](docs/decisions.md) for why things are the way they are.

## License

[MIT](LICENSE). Not affiliated with or endorsed by SCS Software. Euro Truck Simulator 2 and American Truck Simulator are trademarks of SCS Software. The editor reads the game's data from your own install; none of SCS's text or artwork is included.
