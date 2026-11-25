# AGENTS.md

Guide for AI agents (and humans) working in this repo.

## What this is

**Truck Save Editor** is a save editor for **Euro Truck Simulator 2** and **American Truck Simulator**: a desktop app written in Rust with [GPUI](https://www.gpui.rs/) (Zed's UI framework) through [GPUI Kit](https://gpui-kit.com) (`gpui-kit`: GPUI plus a styled component library). Windows is the platform that matters, because that's where the game and its saves are. The two games run on the same engine and use the same file formats, so the code serves both (a `Game` value carries what differs: folders, Steam app id, currency, units, names).

The bar: **the best save editor anyone has used.** Polish, safety and ease of use come before the number of features.

## Read first

- `docs/sii-format.md`: the game's file formats (encryption, binary and text SII), which the whole app rests on.
- `docs/compatibility.md`: what changes between game versions, what can break a save, and the rules that keep this one working. **Read it before changing how saves are read or written.**
- `docs/decisions.md`: why things are the way they are. **Check it before structural changes.**
- **Before any UI work**, the two GPUI Kit skills in `.claude/skills/` (copied from the pinned release): `gpui-kit` (API, Coding Guides) and `gpui-kit-design-guides` (Design Guides). They are rules, not inspiration. Never invent an API: check the source of the pinned version in `~/.cargo/registry/src/*/gpui-component-<version>/`.
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

Tools are managed by [mise](https://mise.jdx.dev/) (`mise.toml`). The Windows build uses Windows' own Rust (rustup, pinned by `rust-toolchain.toml`) and Visual Studio Build Tools. Don't install tools with rustup or `cargo install`; add them to `mise.toml`.

| Command | What it does |
|---|---|
| `mise run check` | Format check, Clippy (warnings are errors) and all tests |
| `mise run test` | `cargo test` for the workspace |
| `mise run fmt` | Format everything |
| `mise run app` | Run the desktop app in WSL (WSLg, software rendering: slow, fine for checking looks) |
| `mise run win` / `mise run win-run` | Build the Windows `.exe` on Windows, driven from WSL (`scripts/windows.sh`), and start it |
| `mise run release-notes` | Preview the next release's notes, drafted from the commits (`cliff.toml`) |

Before finishing a change, run `mise run check`, and add or update tests for what changed. CI (`.github/workflows/ci.yml`) checks the save-format crates on Linux and builds and tests the app on Windows on every push; each build's `.exe` is downloadable from the run. `.github/workflows/release.yml` builds a version tag (`v*`) into a draft release for the owner to publish (steps in `docs/releasing.md`; `mise run release-notes` previews the notes); tags and releases are the owner's call. `.github/workflows/game-update.yml` checks weekly for a new game version (`scripts/check-game-update.sh`) and opens an issue with the checklist.

## Layout

```
crates/
  sii/     the file formats: ScsC decrypt, binary SII decode, text SII parse + write. Bytes in, bytes out. No UI, no disk.
  save/    the ETS2 save: profiles and slots on disk, the document model, typed access (bank, economy, vehicles …), edits, backups, writing.
  hashfs/  the game's .scs archives (HashFS v1 and v2): list directories, read files and textures. Read-only.
  gamedata/ the installed game's own data via hashfs: level tables, rank names, skill limits, the game's text (city names) in a language, its icons. Never copied into this repo.
  app/     the GPUI desktop app: screens, the UI kit (src/ui/), theme, locales/ (the app's own text).
docs/      sii-format.md, decisions.md
reference/ the Python prototypes the format was worked out with
fixtures/  copies of real saves for local tests (gitignored, never committed)
```

Dependencies point downward: `app` → `gamedata` → `save` → `sii`, and `gamedata` → `hashfs`. **`sii` and `save` have no UI dependencies and must build for `wasm32-unknown-unknown`** (a web version later). Disk access in `save` (finding profiles, backups, writing) sits behind its `native` feature, so the rest builds for wasm.

## Rust conventions

- Edition 2024, `rustfmt` defaults, Clippy clean with `-D warnings`.
- **Every word the app shows comes from `crates/app/locales/en.ftl`** through `t!(cx, "key", …)` (`crates/app/src/i18n.rs`); words that depend on data (slot names, skills, "2 hours ago") have helpers in `labels.rs`. Never build sentences from pieces or plural with `if count == 1`: Fluent picks plural forms per language. The `sii` and `save` crates hold no UI wording. A test checks every key used in code has English text, and that every other language (`sv.ftl` Swedish, `de.ftl` German, `pl.ftl` Polish) has every English message with the same variables: a new message needs all of them too. German and Polish were machine-drafted with the game's own terms (from its `locale.scs`) and await native review; Polish has four plural forms (`one`, `few`, `many`, `other`), and names go after a colon rather than a preposition, since Polish would decline them.
- **No `unwrap`, `expect` or `panic!` in `sii` and `save`** outside tests: return `Result` with typed errors (`thiserror`). A damaged or unexpected save must give a clear message ("Unknown value type 0x44 in unit `vehicle` at byte 120,331"), never a crash. In `app`, only for real invariants, with a comment saying why it can't fail.
- No `unsafe`.
- **`gpui-kit` is the only UI dependency, pinned to an exact version** (`=0.10.0`; it pins the matching GPUI itself). Its API changes between releases (they come weekly). Learn it from the skills, examples and source *for the pinned version*, not from memory or blog posts. Upgrading is a change of its own, and copies the skills again from the matching tag.
- **Clear over clever.** The owner is learning Rust alongside this project. Prefer plain code to macro tricks, write types out where they help reading, and comment the *why* (including Rust-specific whys, like a borrow that forces a clone).
- Names say what they hold. Booleans read as statements: `is_running`, `has_loans`, `needs_backup`.
- The product name is written in one place, `PRODUCT_NAME` in `crates/app/build.rs`; code reads it as `APP_NAME`, and the Windows .exe carries it in its details. Never hard-code it elsewhere.

## The save model

- A document keeps **units in file order and fields in their order**. Writing an unedited document gives back the same data. Edits change values in place; never reorder units. A save is **one tree** from the `economy` unit: the game refuses units nothing points at ("There are multiple unit trees"), so units an edit cuts off are removed (`save::drop_unreachable`, run when a slot is prepared), and nothing else is.
- Units point to each other by id (`economy.bank` → the `bank` unit). The document keeps an index by id.
- Screens use **typed access** from `save` (`Bank`, `Economy`, `Vehicle`, …), not raw field names scattered through UI code. All data is the only screen that works on raw units and fields.
- **Every change goes through one place** that records it (unit, field, old value, new value). That one list drives undo/redo, the changes shown before saving and the dirty state. The UI never mutates the document directly.
- `info.sii` (the summary the game's load menu shows) is updated to match when the fields it mirrors change (money, XP …).

## GPUI app conventions

- **One `Session` entity holds the open save** (`crates/app/src/session.rs`): what was loaded, and every pending change (career, repairs, garage upgrades, maxed drivers, visited cities, All data edits). Pages read from it and change it only through its methods; it counts and describes the changes for the header and the save dialog, and `prepared()` builds the slot as it would be written. A new kind of change gets a field in its `Pending` value (changed only through `Session::change`, which is what makes Undo and Redo work), a line in `changes()`, its count in `change_count()`, and its application in `prepared()`.
- **Pages** (`crates/app/src/pages/`): `open_save` (the start screen: the profile's saves, where they're from, every folder looked in, and folders the player added), `overview`, `coach_career` (shown only for a save with a coach career), `fleet`, `garages`, `drivers`, `jobs`, the sidebar's Map group (`cities`, `world_map` with its `map_view`, and `gallery`), `all_data`, `game_settings` (the game's folder and its `config.cfg`), `settings` (the editor's own, at the sidebar's foot), and `details/` (the side panels for one truck, trailer, driver, garage or city, opened by double-click, Enter or a row's Details button). Each observes the session and redraws from it; every page reads the session's preview of the save, so they all agree.
- **The kit** (`crates/app/src/ui/`): `Section`, `PageHeader`, `AmountField` (amounts with 2.5m / +1m), `LevelPicker`, `StepButtons`, `TextTable` (a virtualized table of text), `GameIconView` (one of the game's icons, drawn at exact pixel size), `FilterBar` (search and toggles above a table; every table has one).
- **The command palette** (`crates/app/src/palette.rs`, Ctrl+K): a new page or bulk action gets a `PaletteCommand` too.
- **Errors the player can't fix** (opening, preparing, saving, restoring) use `report::error_notification`, which adds Report this problem: a GitHub issue filled in with the error and the editor's version, opened in the browser for the player to check and post.
- **The game's own data** (levels, ranks, city names) comes from `game_data.rs`, loaded in the background at startup; always have a fallback for when it isn't there.

- **Build screens from the UI kit** in `crates/app/src/ui/` (our own components, wrapping `gpui-component` where it helps). Screens don't style raw elements ad hoc. A missing component goes in the kit.
- **Colours, spacing, radii, type sizes and durations come only from theme tokens** (`ui/theme.rs`), defined for light and dark. Check both after changing the kit.
- Loading, decoding and saving run on the background executor; the UI shows progress and stays responsive.
- Keyboard shortcuts are GPUI actions with their key bindings registered in one place (`bind_keys` in `main.rs`): Ctrl+K palette, Ctrl+Z / Ctrl+Shift+Z undo and redo (anywhere, panels included), Ctrl+, Settings, Enter on a table row opens its panel. Icon-only buttons get an `accessibility_label` as well as a tooltip. Tooltips are set with `.tip(text)` (`ui/tooltip.rs`), never `.tooltip(…)`: it breaks long text into lines, since the kit draws a tooltip as wide as its text.
- One entity holds an open save; views observe it.

## Safety: real saves and personal data

- Saves live on Windows, reached from WSL through `/mnt/c`: Steam Cloud saves under `…\Steam\userdata\<steam id>\227300\remote\profiles\<hex name>\save\<slot>\`, local ones under `Documents\Euro Truck Simulator 2\profiles\`.
- **Never write to real save folders during development.** Copy saves into `fixtures/` (gitignored; `$ETS2_FIXTURES` points to it) and work on the copies. Write to a real slot only with the owner's explicit go-ahead, and back it up first.
- **`profile.sii` holds the World of Trucks login and an encrypted password.** Never print it, log it, copy it into fixtures or commit it.
- **Commit nothing from a real save**: no save files, decoded text, snippets, unit ids, Steam ids or profile names, and no values copied out of one (save names, money, XP, timestamps, plates, home city), not even into test samples, doc comments or commit messages. Invent sample values instead. Committed tests use small synthetic samples; tests on real fixtures skip when `$ETS2_FIXTURES` is missing or empty, and print nothing personal.
- The game overwrites saves when it exits, so edits made while it runs are lost.
- Run Python on save data with `python3 -I`.

## Verifying

- **Formats:** round-trip tests (decrypt → decode → write text → parse gives an equal document) on synthetic samples, and on fixtures when present.
- **UI by eye:** `scripts/drive.sh <slot copy> <steps file> <out folder>` is the quick way: the app drives itself through steps (`page fleet`, `theme dark`, `run <palette command>`, `open truck` (or `trailer`, `coach`: the first one's panel), `close`, `wait <ms>`, `shot <name>`) once its data has loaded, saves the pictures and quits, in seconds. `scripts/look.sh` starts the dev build under WSLg, captures just its window to a PNG, and clicks or types into it (`start`, `shot`, `click`, `park`, `keys`, `stop`), for what steps can't reach yet (a row's panel, dragging). In development builds, `TSE_PAGE=fleet` opens a page on launch and `TSE_THEME=dark` (or `light`) picks the theme. `TSE_FAKE_UPDATE=0.9.0` pretends a newer version is out (no network). `TSE_PROFILE_DIRS` adds folders of profiles to look in, and `TSE_PROFILE_DIRS_ONLY=1` looks only there (the start screen as on a PC without the game). `TSE_CRASH=1` crashes on purpose, to see the crash report written (`crash.rs`) and offered on the next start. `TSE_SLOT=<slot folder>` opens a slot copy directly (`TSE_GAME=ats` for an ATS one), `TSE_FIND_CITY=hamburg` opens World map on a city, `TSE_READY_FILE=<path>` is written once the game's data and map have loaded, and `TSE_SCRIPT=<file>` drives the app through steps (pages, themes, palette commands, pictures; `crates/app/src/dev_script.rs`). Look at every changed screen in both themes, and drive the flow you changed (click, type, Esc, click away) before calling it done. Windows behaviour (paths, fonts, DPI, speed) is only real in the Windows build (`mise run win-run`).
- **Screenshots:** `scripts/screenshots.sh <slot copy> [version] [ATS slot copy]` takes every README picture (light and dark: the pages, the save dialog, the start screen) and a release's (dark) in one run of about a minute, from made-up demo profiles (`scripts/demo-profile.py`: invented money, experience, plates and names), so no picture shows a real save. Every picture in the README comes from it: a new one gets a step there, never a hand-taken shot. Rerun it when a page shown in the README changes, and look at every picture before committing.
- **UI integration tests** (`#[gpui_kit::test]`, see the `gpui-kit` skill's `test.md`): give each user action its own `cx.update_window(…)` followed by `cx.run_until_parked()`, because entity events are only delivered between updates; and call `window.activate_window()` first, because a headless window starts inactive and an inactive window gets no focus or blur events. `crates/app/src/ui/amount_field.rs` has a worked example.
- **Design:** check every changed screen in both themes, and run the Design Guides' review checklist (`.claude/skills/gpui-kit-design-guides`).
- **In game:** only the owner can check. Ask with the exact slot, what was changed and what to look for, and say how to restore the backup.

## Reference prototypes

`reference/sii_decrypt.py` and `reference/bsii_decode.py` decoded a real 1.62 save without errors. Until the Rust tests cover a case, they're the source of truth when the two disagree.

## Commits and PRs

Never add AI attribution: no `Co-Authored-By: Claude …` (or any other AI) trailer, no `Claude-Session` line, no "Generated with Claude Code" line in commit messages or PR descriptions. Commit as the configured git user only. Use conventional prefixes (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`). Don't hard-wrap commit message bodies; use one line per bullet. The owner allows committing and pushing routine work to `main` as you go. Ask first for anything destructive or unusual: force pushes, rewriting history, deleting branches or tags, releases.

## Decisions

For anything that changes structure, dependencies, file formats or conventions, add a short dated entry to `docs/decisions.md`: what was decided, why, and what else was considered. Don't rewrite old entries; add a new one that replaces them.
