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
cmd.exe /c "cargo build --release -p app" 2>&1

exe="$dest/target/release/truck-save-editor.exe"
echo "Built: $(wslpath -w "$exe")"
if [ "$mode" = "run" ]; then
    "$exe" &
fi
