# Releasing

A release is a version tag. Pushing `v0.1.0` runs `.github/workflows/release.yml`, which builds the Windows app, and makes a **draft** release with the .exe, its checksum and notes. Nothing is public until the owner publishes the draft. Releases are the owner's call.

## What a release holds

- `truck-save-editor-<version>-windows-x64.exe`: the app, on its own (no installer, nothing to unzip). Everything it needs is inside it, the licences of the crates built into it too (Settings → About → Open-source licences; `crates/app/assets/licenses.txt`, made by `mise run licenses` with cargo-about, and checked by CI to match the crates).
- `SHA256SUMS.txt`: the .exe's checksum.
- A build provenance attestation for it (GitHub's, signed with Sigstore): `gh attestation verify <file> --repo leonlarsson/truck-save-editor` proves the file came from this repository's release workflow, at the tagged commit.
- Notes: the hand-written `docs/release-notes/<version>.md` (without its opening comment) when there is one, otherwise drafted from the commit messages by git-cliff (`cliff.toml`: `feat`, `fix` and `polish` commits); then the download text from `.github/release-footer.md`.

The .exe carries its version, product name and copyright in its file details (`crates/app/build.rs`), and the C runtime inside it (`.cargo/config.toml`), so it starts on any Windows 10 or 11.

## Steps

1. **Check it in game.** Changes to how saves are written need the owner's in-game check (see `AGENTS.md`). Note what's still untried in `TODO.md`.
2. **Write the notes**: `docs/release-notes/<version>.md`, for players (see the earlier versions; `mise run release-notes` shows the commits to start from). New, Changed, Fixed; only what changed since the last release (check the last notes so nothing from them is repeated), and no claim the editor doesn't fully meet. The release workflow uses this file as the release's notes, and adds the Download and Check the download sections (`.github/release-footer.md`); without it, the notes are drafted from the commits. A `Headline:` line in its opening comment adds to the release's title (`Headline: American Truck Simulator support` → "Truck Save Editor 0.4.0: American Truck Simulator support"), for a release with one big change.
3. **Take the pictures**: `scripts/screenshots.sh <slot copy> <version> [ATS slot copy]` (about a minute, from made-up demo profiles; see `AGENTS.md` → Verifying). It refreshes the README's pictures in `docs/screenshots/` and puts the release's in `docs/release-notes/<version>/`. Look at every one, and keep only those the notes use.
4. **Commit in this order**, so the notes can link pictures that exist:
   1. the work itself (`feat:`, `fix:` …);
   2. the pictures and the README (`docs:`);
   3. the release: put the pictures' commit into the notes' links (`https://raw.githubusercontent.com/leonlarsson/truck-save-editor/<commit>/docs/release-notes/<version>/<name>.png`, check each loads), **bump the version** in the root `Cargo.toml` (`[workspace.package] version`), run `cargo check` so `Cargo.lock` follows and `mise run licenses` so the licence list does (it names the editor's own crates with their version; CI fails if it's behind), run `mise run check`, and commit as `chore: release <version>`.

   Push, and wait for **CI on `main`** to pass: the release workflow builds and packages, but doesn't run the tests.
5. **Tag**: `git tag v<version>`, then `git push origin v<version>` (a tag isn't sent by a plain `git push`). The workflow refuses a tag that doesn't match the version, and builds with `--locked`.
6. **Try the draft**: the workflow makes a **draft** release, titled "Truck Save Editor <version>" (and the notes' headline, if they have one), with the .exe, `SHA256SUMS.txt` and the notes. Until it's published its address reads `…/releases/tag/untagged-…`, and only the owner sees it. Download the .exe on a Windows PC, run it, open a save and look around, and read the notes as a player would.
7. **Publish**: on GitHub, or `gh release edit v<version> --draft=false --latest`. Check it's marked Latest and the title is "Truck Save Editor <version>", with the headline if the notes have one. The notes and title can still be edited afterwards; the files and tag can't (see Immutable releases). A version with a dash (`0.2.0-beta.1`) is marked as a pre-release.
8. **Afterwards**: offer a short forum post for the release (plain BBCode, short bullets; the forum doesn't show inline pictures), and note the release in `TODO.md`.

Once published, a release reaches people through the editor: it asks GitHub for the latest release when it starts (`crates/app/src/updates.rs`) and offers the download. GitHub's "latest release" leaves out drafts and pre-releases, so only a full release is announced.

To try the whole thing without releasing, run the workflow by hand (Actions → Release → Run workflow): it builds, packages and writes the notes as for a release and keeps them as a workflow artifact, with no release and no attestation.

## Repository settings

- **Immutable releases** are on (Settings → General → Releases, turned on 2026-11-22): once published, a release's files and tag can't be changed or deleted, and GitHub attests the release itself. A draft can still be edited, replaced or deleted, so try it before publishing. A published release with a mistake is fixed by a new version (0.1.1), never by replacing files; its tag can't be used again.

## Code signing

Releases aren't code-signed yet, so Windows SmartScreen warns on first runs ("Windows protected your PC" → More info → Run anyway). The options, as of November 2026:

- **SignPath Foundation**: free for open-source projects (MIT qualifies). The certificate is in the Foundation's name; it requires two-factor login for everyone with access, the line "Free code signing provided by SignPath.io, certificate by SignPath Foundation" on releases, and approval. It signs builds from GitHub Actions. The likely choice once there's a release to show them.
- **Azure Artifact Signing** (Trusted Signing): about $10 a month, but for individual developers only in the USA and Canada.
- **A certificate from a CA** (DigiCert, Sectigo …): a few hundred euros a year, in the owner's name. Since 2024 none of them skip SmartScreen's reputation building.
