# AGENTS.md

Guide for AI agents (and humans) working in this repo.

## What this is

**Truck Save Editor** (a working name) is a save editor for **Euro Truck Simulator 2**: a desktop app written in Rust with [GPUI](https://www.gpui.rs/) (Zed's UI framework). Windows is the platform that matters, because that's where the game and its saves are. American Truck Simulator runs on the same engine and uses the same file formats, so the code is written for both from the start (a `Game` value carries what differs: folders, Steam app id, currency, units, names), while ETS2 ships first.

The bar: **the best save editor anyone has used.** The ones out there are broken or ugly. Polish, safety and ease of use come before the number of features.

## Read first

- `docs/sii-format.md`: the game's file formats (encryption, binary and text SII), which the whole app rests on.
- `docs/decisions.md`: why things are the way they are. **Check it before structural changes.**
- `TODO.md`: agreed, planned work, in order.

## Product principles

1. **Simple first, everything reachable.** The first screen has the big levers: money, level and XP, skills, ADR, HQ city. Then sections (Fleet, Trailers, Garages, Drivers, Map, Jobs). Then **All data**, a searchable view of every unit and field in the save, so nothing is out of reach.
2. **Never lose a save.** Back up before every write, refuse to write while the game is running, show what changed before saving, and make restoring a backup one click. Steam Cloud uploads broken saves too, so trust is the product.
3. **Speak the game's language.** Labels say what the game says ("High value cargo", not `heavy`; "Level 42", not 812,400 points; "€ 2,500,000", not a bare number). Raw field names only show in All data.
4. **Don't make people hunt for files.** Find Steam Cloud and local profiles by themselves, show profile names (decoded from the hex folder name) and slots with their in-game preview, date and money.
5. **Never freeze.** Loading, decoding and saving run in the background with progress. Long lists are virtualized.
6. **Consistent wording.** One idea, one name, everywhere it shows: buttons, hints, menus, docs.
7. **Don't hide what isn't available.** Show it disabled with the reason, and how to make it available.
8. **For everyone.** Every action is reachable by keyboard, state is never shown by colour alone, and light and dark themes both look finished.

## Commands (mise)

Tools are managed by [mise](https://mise.jdx.dev/) (`mise.toml`): Rust, its Windows target and `cargo-xwin`. Don't install tools with rustup or `cargo install`; add them to `mise.toml`.

| Command | What it does |
|---|---|
| `mise run check` | Format check, Clippy (warnings are errors) and all tests |
| `mise run test` | `cargo test` for the workspace |
| `mise run fmt` | Format everything |
| `mise run app` | Run the desktop app in WSL (WSLg, software rendering: slow, fine for checking looks) |
| `mise run win` | Build the Windows `.exe` from WSL with `cargo-xwin` |

Before finishing a change, run `mise run check`, and add or update tests for what changed.

## Layout

```
crates/
  sii/     the file formats: ScsC decrypt, binary SII decode, text SII parse + write. Bytes in, bytes out. No UI, no disk.
  save/    the ETS2 save: profiles and slots on disk, the document model, typed access (bank, economy, vehicles …), edits, backups, writing.
  app/     the GPUI desktop app: screens, the UI kit (src/ui/), theme.
docs/      sii-format.md, decisions.md
reference/ the Python prototypes the format was worked out with
fixtures/  copies of real saves for local tests (gitignored, never committed)
```

Dependencies point downward: `app` → `save` → `sii`. **`sii` and `save` have no UI dependencies and must build for `wasm32-unknown-unknown`** (a web version later). Disk access in `save` (finding profiles, backups, writing) sits behind its `native` feature, so the rest builds for wasm.

## Rust conventions

- Edition 2024, `rustfmt` defaults, Clippy clean with `-D warnings`.
- **No `unwrap`, `expect` or `panic!` in `sii` and `save`** outside tests: return `Result` with typed errors (`thiserror`). A damaged or unexpected save must give a clear message ("Unknown value type 0x44 in unit `vehicle` at byte 120,331"), never a crash. In `app`, only for real invariants, with a comment saying why it can't fail.
- No `unsafe`.
- **Pin `gpui` and `gpui-component` to exact versions** (`=0.2.2`): their APIs change between releases. Learn the API from their examples and source *for the pinned version*, not from memory or blog posts. Upgrading them is a change of its own.
- **Clear over clever.** The owner is learning Rust alongside this project. Prefer plain code to macro tricks, write types out where they help reading, and comment the *why* (including Rust-specific whys, like a borrow that forces a clone).
- Names say what they hold. Booleans read as statements: `is_running`, `has_loans`, `needs_backup`.
- The product name lives in one constant, `APP_NAME`. Never hard-code it elsewhere.

## The save model

- A document keeps **units in file order and fields in their order**. Writing an unedited document gives back the same data. Edits change values in place; never reorder or drop units.
- Units point to each other by id (`economy.bank` → the `bank` unit). The document keeps an index by id.
- Screens use **typed access** from `save` (`Bank`, `Economy`, `Vehicle`, …), not raw field names scattered through UI code. All data is the only screen that works on raw units and fields.
- **Every change goes through one place** that records it (unit, field, old value, new value). That one list drives undo/redo, the changes shown before saving and the dirty state. The UI never mutates the document directly.
- `info.sii` (the summary the game's load menu shows) is updated to match when the fields it mirrors change (money, XP …).

## GPUI app conventions

- **Build screens from the UI kit** in `crates/app/src/ui/` (our own components, wrapping `gpui-component` where it helps). Screens don't style raw elements ad hoc. A missing component goes in the kit.
- **Colours, spacing, radii, type sizes and durations come only from theme tokens** (`ui/theme.rs`), defined for light and dark. Check both after changing the kit.
- Loading, decoding and saving run on the background executor; the UI shows progress and stays responsive.
- Keyboard shortcuts are GPUI actions with their key bindings registered in one place.
- One entity holds an open save; views observe it.

## Safety: real saves and personal data

- Saves live on Windows, reached from WSL through `/mnt/c`: Steam Cloud saves under `…\Steam\userdata\<steam id>\227300\remote\profiles\<hex name>\save\<slot>\`, local ones under `Documents\Euro Truck Simulator 2\profiles\`.
- **Never write to real save folders during development.** Copy saves into `fixtures/` (gitignored; `$ETS2_FIXTURES` points to it) and work on the copies. Write to a real slot only with the owner's explicit go-ahead, and back it up first.
- **`profile.sii` holds the World of Trucks login and an encrypted password.** Never print it, log it, copy it into fixtures or commit it.
- **Commit nothing from a real save**: no save files, decoded text, snippets, unit ids, Steam ids or profile names. Committed tests use small synthetic samples; tests on real fixtures skip when `$ETS2_FIXTURES` is missing or empty.
- The game overwrites saves when it exits, so edits made while it runs are lost.
- Run Python on save data with `python3 -I`.

## Verifying

- **Formats:** round-trip tests (decrypt → decode → write text → parse gives an equal document) on synthetic samples, and on fixtures when present.
- **UI:** `mise run app` under WSLg for looks and flow. Windows behaviour (paths, fonts, DPI, speed) is only real in the Windows build.
- **In game:** only the owner can check. Ask with the exact slot, what was changed and what to look for, and say how to restore the backup.

## Reference prototypes

`reference/sii_decrypt.py` and `reference/bsii_decode.py` decoded a real 1.62 save without errors. Until the Rust tests cover a case, they're the source of truth when the two disagree.

## Commits and PRs

Never add AI attribution: no `Co-Authored-By: Claude …` (or any other AI) trailer, no `Claude-Session` line, no "Generated with Claude Code" line in commit messages or PR descriptions. Commit as the configured git user only. Use conventional prefixes (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`). Don't hard-wrap commit message bodies; use one line per bullet. Never commit or push without the owner's explicit go-ahead, every time.

## Decisions

For anything that changes structure, dependencies, file formats or conventions, add a short dated entry to `docs/decisions.md`: what was decided, why, and what else was considered. Don't rewrite old entries; add a new one that replaces them.
