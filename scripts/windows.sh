#!/usr/bin/env bash
# Builds (and optionally runs) the Windows app from WSL.
#
# GPUI compiles its DirectX shaders with fxc.exe from the Windows SDK, which only runs on Windows,
# so the build itself runs there: this copies the source to a folder on the Windows drive (cargo
# builds badly from \\wsl$ paths) and runs Windows' own cargo on it.
#
# Needs on Windows: Rust (rustup) and Visual Studio Build Tools with "Desktop development with C++".
# Usage: scripts/windows.sh build | run
set -euo pipefail

mode="${1:-build}"
repo="$(cd "$(dirname "$0")/.." && pwd)"
win_home="$(wslpath "$(cmd.exe /c 'echo %USERPROFILE%' 2>/dev/null | tr -d '\r')")"
dest="$win_home/.truck-save-editor-build"

mkdir -p "$dest"
rsync -a --delete --exclude /target --exclude /fixtures --exclude /.git "$repo/" "$dest/"

cd "$dest"
# WSL keeps the Windows PATH it saw when it started, which misses a Rust installed since, so put
# rustup's folder first. On the first build, rustup downloads the Rust version in rust-toolchain.toml.
# The plugin first (`tse_live.dll`, for Live): the app's build puts it inside the .exe (build.rs).
cmd.exe /c "set PATH=%USERPROFILE%\\.cargo\\bin;%PATH% && cargo build --release -p live-plugin && cargo build --release -p app" 2>&1

exe="$dest/target/release/truck-save-editor.exe"
echo "Built: $(wslpath -w "$exe")"
if [ "$mode" = "run" ]; then
    # Its output goes nowhere: left attached, whatever reads this script's output (a pipe, a
    # tool waiting for it) would wait until the editor is closed.
    "$exe" >/dev/null 2>&1 &
fi
