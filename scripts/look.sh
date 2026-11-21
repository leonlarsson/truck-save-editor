#!/usr/bin/env bash
# Look at and drive the running app from WSL, for checking UI changes by eye (and by agents).
#
# The app runs under WSLg, whose windows are ordinary Windows windows, so these PowerShell helpers
# find the app's window by its title, capture just that window, and click or type into it.
#
#   scripts/look.sh start [ENV=VALUE ...]   restart the dev build, e.g. TSE_PAGE=fleet TSE_THEME=dark
#   scripts/look.sh shot <file.png>         capture the app window
#   scripts/look.sh click <x> <y>           left-click at a point relative to the window's top-left
#   scripts/look.sh keys <text>             type, in SendKeys syntax ({ESC}, {ENTER}, {TAB}, ^a …)
#   scripts/look.sh stop                    close the app
set -euo pipefail

title="Truck Save Editor"
repo="$(cd "$(dirname "$0")/.." && pwd)"
win_temp="$(cmd.exe /c 'echo %TEMP%' 2>/dev/null | tr -d '\r')"
temp="$(wslpath "$win_temp")"
cp "$repo/scripts/look/shot.ps1" "$repo/scripts/look/input.ps1" "$temp/"
ps() { powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$win_temp\\$1" "${@:2}" | tr -d '\r'; }

case "${1:-}" in
    start)
        shift
        pkill -x truck-save-edit || true
        (cd "$repo" && cargo build -q -p app)
        (cd "$repo" && env "$@" ./target/debug/truck-save-editor >/dev/null 2>&1 &)
        sleep 4
        ;;
    shot)
        for _ in 1 2 3 4 5 6 7 8 9 10; do
            if ps shot.ps1 -Title "$title" -Out "$win_temp\\tse-shot.png" | grep -q '^OK'; then
                cp "$temp/tse-shot.png" "$2"
                echo "$2"
                exit 0
            fi
            sleep 1
        done
        echo "The app window wasn't found." >&2
        exit 1
        ;;
    click) ps input.ps1 -Title "$title" -Action click -X "$2" -Y "$3" ;;
    keys) ps input.ps1 -Title "$title" -Action keys -Keys "$2" ;;
    stop) pkill -x truck-save-edit || true ;;
    *) sed -n '2,12p' "$0"; exit 1 ;;
esac
