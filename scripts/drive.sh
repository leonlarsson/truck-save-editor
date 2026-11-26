#!/usr/bin/env bash
# Drives the dev build under WSLg through a list of steps and saves the pictures it asks for: the
# quick way to look at the app (for checks while developing, and scripts/screenshots.sh).
#
#   scripts/drive.sh <slot folder> <steps file> <out folder> [ENV=VALUE ...]
#
# <slot folder> is a copy of a save (fixtures/…, never a live save). The steps (one per line; see
# crates/app/src/dev_script.rs) run once the game's data and map have loaded:
#
#   page gallery | theme dark | run Visit every city | pick 3 | close | wait 1500 | shot gallery-dark
#
# `shot <name>` saves the app's window (less its frame) as <out folder>/<name>.png; a name can
# hold folders. Extra ENV=VALUE pairs go to the app (TSE_THEME=dark, TSE_FIND_CITY=hamburg …).
# A run of a few pictures takes seconds; the app quits when the steps are done.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
slot="${1:?usage: scripts/drive.sh <slot folder> <steps file> <out folder> [ENV=VALUE ...]}"
steps="${2:?usage: scripts/drive.sh <slot folder> <steps file> <out folder> [ENV=VALUE ...]}"
out="${3:?usage: scripts/drive.sh <slot folder> <steps file> <out folder> [ENV=VALUE ...]}"
shift 3
slot="$(cd "$slot" && pwd)"
steps="$(cd "$(dirname "$steps")" && pwd)/$(basename "$steps")"
mkdir -p "$out"

talk="$(mktemp -d)"
cleanup() {
    pkill -x truck-save-edit 2>/dev/null || true
    rm -rf "$talk"
}
trap cleanup EXIT

(cd "$repo" && cargo build -q -p app)
pkill -x truck-save-edit 2>/dev/null || true
(cd "$repo" && env TSE_SLOT="$slot" TSE_WINDOW=1280x800 TSE_SCRIPT="$steps" TSE_SHOT_DIR="$talk" \
    "$@" ./target/debug/truck-save-editor >"$talk/log.txt" 2>&1 &)

# The app sits in a 21-pixel frame in its 1280×800 window: a picture is the window less that.
started=$(date +%s)
parked=""
while [ ! -f "$talk/done" ]; do
    if [ -f "$talk/request" ]; then
        # The pointer off the window first, so nothing shows as hovered.
        if [ -z "$parked" ]; then
            "$repo/scripts/look.sh" park >/dev/null && parked=1
            sleep 0.3
        fi
        name="$(cat "$talk/request")"
        mkdir -p "$(dirname "$out/$name")"
        "$repo/scripts/look.sh" shot "$out/$name.png" 21 21 1238 758 >/dev/null
        echo "$out/$name.png"
        rm -f "$talk/request"
    fi
    if [ $(($(date +%s) - started)) -gt 180 ]; then
        echo "The app didn't finish its steps in three minutes. Its output:" >&2
        cat "$talk/log.txt" >&2
        exit 1
    fi
    sleep 0.1
done
# Lines the app skipped or couldn't carry out.
grep "TSE_SCRIPT" "$talk/log.txt" >&2 || true
echo "Done in $(($(date +%s) - started)) s."
