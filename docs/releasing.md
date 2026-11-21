# Releasing

A release is a version tag. Pushing `v0.1.0` runs `.github/workflows/release.yml`, which builds the Windows app, and makes a **draft** release with the zip, checksums and notes. Nothing is public until the owner publishes the draft. Releases are the owner's call.

## What a release holds

- `truck-save-editor-<version>-windows-x64.zip`: the portable app (`truck-save-editor.exe`, no installer) with `LICENSE`, `THIRD_PARTY_NOTICES.md` and `README.md`.
- `SHA256SUMS.txt`: checksums of the zip and of the .exe inside it.
- A build provenance attestation for both (GitHub's, signed with Sigstore): `gh attestation verify <file> --repo leonlarsson/truck-save-editor` proves the file came from this repository's release workflow, at the tagged commit.
- Notes: drafted from the commit messages by git-cliff (`cliff.toml`: `feat`, `fix` and `polish` commits), with the download text from `.github/release-footer.md`. Edit them into something a player wants to read before publishing.

The .exe carries its version, product name and copyright in its file details (`crates/app/build.rs`), and the C runtime inside it (`.cargo/config.toml`), so it starts on any Windows 10 or 11.

## Steps

1. **Check it in game.** Changes to how saves are written need the owner's in-game check (see `AGENTS.md`).
2. **Bump the version** in the root `Cargo.toml` (`[workspace.package] version`), then run `cargo check` so `Cargo.lock` follows. The release builds with `--locked`, and fails if the lock file is behind.
3. **Preview the notes**: `mise run release-notes`.
4. **Commit and tag**: `git commit -am "chore: release 0.1.0"`, `git tag v0.1.0`, `git push origin main v0.1.0`. The workflow refuses a tag that doesn't match the version.
5. **Try the draft**: when the workflow finishes, download the zip from the draft release on a Windows PC, unzip it, open a save and look around.
6. **Edit the notes and publish.** A version with a dash (`0.2.0-beta.1`) is marked as a pre-release.

To try the whole thing without releasing, run the workflow by hand (Actions → Release → Run workflow): it builds, packages and writes the notes as for a release and keeps them as a workflow artifact, with no release and no attestation.

## Repository settings

- **Immutable releases** (Settings → General → Releases): once published, a release's files and tag can't be changed or deleted, and GitHub attests the release itself. The draft-then-publish flow above works with it.

## Code signing

Releases aren't code-signed yet, so Windows SmartScreen warns on first runs ("Windows protected your PC" → More info → Run anyway). The options, as of November 2026:

- **SignPath Foundation**: free for open-source projects (MIT qualifies). The certificate is in the Foundation's name; it requires two-factor login for everyone with access, the line "Free code signing provided by SignPath.io, certificate by SignPath Foundation" on releases, and approval. It signs builds from GitHub Actions. The likely choice once there's a release to show them.
- **Azure Artifact Signing** (Trusted Signing): about $10 a month, but for individual developers only in the USA and Canada.
- **A certificate from a CA** (DigiCert, Sectigo …): a few hundred euros a year, in the owner's name. Since 2024 none of them skip SmartScreen's reputation building.
