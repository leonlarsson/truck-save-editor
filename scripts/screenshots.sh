#!/usr/bin/env bash
# Takes the README's screenshots (light and dark) and a release's pictures (dark) of a made-up demo
# profile, so they show nothing of the player's own. Runs the dev build under WSLg.
#
#   scripts/screenshots.sh <slot folder> [release version] [ATS slot folder]
#
# <slot folder> is a copy of a save under fixtures/ (never a live save): one with a coach career
# and a map's worth of driving shows the most. scripts/demo-profile.py turns it into a profile
# "Leon" with invented money, experience, plates and names, in a temporary folder. The README's
# pictures go to docs/screenshots/<page>-<theme>.png; with a version, that release's go to
# docs/release-notes/<version>/. Look at every picture before committing it.
#
# With an ATS slot copy too, the release gets ATS pictures (Overview, Jobs), from a demo ATS
# profile made the same way. The start screen's pictures list only the demo profiles (each with an
# autosave too), as on a PC with just them: both games' with an ATS copy, else ETS2's.
#
# One launch per game: the app drives itself through steps (scripts/drive.sh, crates/app/src/dev_script.rs):
# pages, themes, palette commands, and a picture where asked.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
source_slot="${1:?usage: scripts/screenshots.sh <slot folder> [release version]}"
version="${2:-}"
ats_slot="${3:-}"

# The editor's own settings say English while the pictures are taken, and go back afterwards.
settings="$HOME/.config/Truck Save Editor/settings.txt"
work="$(mktemp -d)"
# The demo profiles, in folders named as the games name theirs: the start screen shows the
# folder's path, and an added folder's game is told by its name. Made for the run, removed after.
ets2_profiles="/tmp/Euro Truck Simulator 2/profiles"
ats_profiles="/tmp/American Truck Simulator/profiles"
for dir in "$ets2_profiles" "$ats_profiles"; do
    if [ -e "$(dirname "$dir")" ]; then
        echo "$(dirname "$dir") is in the way: move it first" >&2
        exit 1
    fi
done
cleanup() {
    rm -rf "$(dirname "$ets2_profiles")" "$(dirname "$ats_profiles")"
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
# ATS's demo first, in a folder named after the game (that's how the start screen tells an added
# folder's game), so ETS2's, made after it, is the newest and the one the start screen picks.
ats=""
if [ -n "$ats_slot" ]; then
    mkdir -p "$work/ats"
    "$repo/target/debug/examples/decode" "$ats_slot/game.sii" "$work/ats/game.txt" >/dev/null
    "$repo/target/debug/examples/decode" "$ats_slot/info.sii" "$work/ats/info.txt" >/dev/null
    ats="$(python3 -I "$repo/scripts/demo-profile.py" "$work/ats/game.txt" "$work/ats/info.txt" \
        "$ats_profiles" ats)"
fi
slot="$(python3 -I "$repo/scripts/demo-profile.py" "$work/game.txt" "$work/info.txt" "$ets2_profiles")"
# An autosave beside each demo save (the same save, without a name), for the start screen.
for demo in "$slot" ${ats:+"$ats"}; do
    cp -r "$demo" "$(dirname "$demo")/autosave"
    sed -i 's/^ name: .*/ name: ""/' "$(dirname "$demo")/autosave/info.sii"
done
touch "$slot/game.sii" "$slot/info.sii"

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

# The start screen, with only the demo profiles to list (TSE_PROFILE_DIRS_ONLY).
profile_dirs="$ets2_profiles"
if [ -n "$ats" ]; then profile_dirs="$profile_dirs:$ats_profiles"; fi
{
    echo "run Switch save"
    echo "wait 1500"
    echo "theme light"
    echo "shot screenshots/start-light"
    echo "theme dark"
    echo "shot screenshots/start-dark"
    if [ -n "$version" ]; then echo "shot release-notes/$version/start"; fi
} >"$work/start-script.txt"
"$repo/scripts/drive.sh" "$slot" "$work/start-script.txt" "$repo/docs" \
    TSE_PAGE=overview TSE_THEME=light TSE_PROFILE_DIRS="$profile_dirs" TSE_PROFILE_DIRS_ONLY=1

# ATS: a few release pictures of a demo ATS profile.
if [ -n "$ats" ] && [ -n "$version" ]; then
    notes="release-notes/$version"
    {
        echo "page overview"
        echo "shot $notes/ats-overview"
        # Jobs draws the companies' logos as they load.
        echo "page jobs"
        echo "wait 1500"
        echo "shot $notes/ats-jobs"
    } >"$work/ats-script.txt"
    "$repo/scripts/drive.sh" "$ats" "$work/ats-script.txt" "$repo/docs" \
        TSE_GAME=ats TSE_PAGE=overview TSE_THEME=dark
fi
