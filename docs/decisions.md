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
