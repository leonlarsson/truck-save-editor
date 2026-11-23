#!/usr/bin/env bash
# Takes the README's screenshots (light and dark) and a release's pictures (dark) of a made-up demo
# profile, so they show nothing of the player's own. Runs the dev build under WSLg.
#
#   scripts/screenshots.sh <slot folder> [release version]
#
# <slot folder> is a copy of a save under fixtures/ (never a live save): one with a coach career
# and a map's worth of driving shows the most. scripts/demo-profile.py turns it into a profile
# "Leon" with invented money, experience, plates and names, in a temporary folder. The README's
# pictures go to docs/screenshots/<page>-<theme>.png; with a version, that release's go to
# docs/release-notes/<version>/. Look at every picture before committing it.
#
# One launch: the app drives itself through steps (scripts/drive.sh, crates/app/src/dev_script.rs):
# pages, themes, palette commands, and a picture where asked.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
source_slot="${1:?usage: scripts/screenshots.sh <slot folder> [release version]}"
version="${2:-}"

# The editor's own settings say English while the pictures are taken, and go back afterwards.
settings="$HOME/.config/Truck Save Editor/settings.txt"
work="$(mktemp -d)"
cleanup() {
    if [ -f "$work/settings.txt" ]; then cp "$work/settings.txt" "$settings"; fi
    rm -rf "$work"
}
trap cleanup EXIT
if [ -f "$settings" ]; then
    cp "$settings" "$work/settings.txt"
    sed -i 's/^language = .*/language = en/' "$settings"
fi

# The demo profile.
(cd "$repo" && cargo build -q -p sii --example decode && cargo build -q -p app)
"$repo/target/debug/examples/decode" "$source_slot/game.sii" "$work/game.txt" >/dev/null
"$repo/target/debug/examples/decode" "$source_slot/info.sii" "$work/info.txt" >/dev/null
slot="$(python3 -I "$repo/scripts/demo-profile.py" "$work/game.txt" "$work/info.txt" "$work/profiles")"

# What the app does. A picture's name is its path under docs/, without .png.
pages="overview statistics coach-career fleet world-map gallery"
{
    for theme in light dark; do
        echo "theme $theme"
        for page in $pages; do
            echo "page $page"
            # World map draws its tiles in the background.
            if [ "$page" = world-map ]; then echo "wait 2500"; fi
            echo "shot screenshots/$page-$theme"
        done
    done
    if [ -n "$version" ]; then
        notes="release-notes/$version"
        for page in coach-career gallery; do
            echo "page $page"
            echo "shot $notes/$page"
        done
        echo "page world-map"
        echo "wait 1500"
        echo "shot $notes/world-map"
        echo "page fleet"
        echo "run Saved colours"
        echo "shot $notes/saved-colours"
        echo "close"
        # Make a job reads the game's companies when it first opens: thousands of small files,
        # several seconds from a Windows drive in WSL.
        echo "page jobs"
        echo "run Make a job"
        echo "wait 10000"
        echo "shot $notes/make-a-job"
        echo "close"
    fi
    # The save dialog last, with a few changes to show (nothing is written: it's only opened).
    echo "page overview"
    echo "run Visit every city"
    echo "run Visit every viewpoint"
    echo "run Max all drivers"
    echo "run Save…"
    echo "shot screenshots/save-dark"
    echo "close"
    echo "theme light"
    echo "run Save…"
    echo "shot screenshots/save-light"
    echo "close"
} >"$work/script.txt"
# The app drives itself through the steps; drive.sh saves each picture under docs/.
"$repo/scripts/drive.sh" "$slot" "$work/script.txt" "$repo/docs" \
    TSE_PAGE=overview TSE_THEME=light TSE_FIND_CITY=hamburg
