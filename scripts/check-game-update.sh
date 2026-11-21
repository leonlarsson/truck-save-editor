#!/usr/bin/env bash
# Checks whether a game has a newer version than the one this editor was last verified against
# (compat/tested-versions.json), and opens a GitHub issue with the checklist if so. Run weekly by
# .github/workflows/game-update.yml; `DRY_RUN=1` prints instead of opening issues.
#
# The version comes from Steam: the public branch's build id (api.steamcmd.net, unofficial but open
# and keyless) matched to SCS's per-version `temporary_1_NN` branches, else the newest "1.NN Update
# Release" title in the game's Steam news. See docs/compatibility.md.
set -euo pipefail

cd "$(dirname "$0")/.."
tested=compat/tested-versions.json

newer() { [ "$1" != "$2" ] && [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1)" = "$1" ]; }

for game in $(jq -r 'keys[]' "$tested"); do
    name=$(jq -r ".$game.name" "$tested")
    app_id=$(jq -r ".$game.app_id" "$tested")
    tested_version=$(jq -r ".$game.version" "$tested")

    version=""
    if info=$(curl -fsS --retry 3 -A "truck-save-editor-update-check" "https://api.steamcmd.net/v1/info/$app_id"); then
        branches=$(jq ".data[\"$app_id\"].depots.branches" <<<"$info")
        public=$(jq -r '.public.buildid' <<<"$branches")
        version=$(jq -r --arg build "$public" \
            'to_entries[] | select(.key | startswith("temporary_1_")) | select(.value.buildid == $build) | .key' \
            <<<"$branches" | sed 's/^temporary_//; s/_/./' | sort -V | tail -1)
    fi
    if [ -z "$version" ]; then
        news=$(curl -fsS --retry 3 "https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=$app_id&count=30&feeds=steam_community_announcements")
        version=$(jq -r '.appnews.newsitems[].title' <<<"$news" | grep -E 'Update Release' | grep -v -i beta \
            | grep -oE '1\.[0-9]+' | sort -V | tail -1 || true)
    fi
    if [ -z "$version" ]; then
        echo "$name: couldn't tell the current version" >&2
        continue
    fi
    echo "$name: current $version, verified $tested_version"
    if ! newer "$version" "$tested_version"; then
        continue
    fi

    title="Game update: $name $version"
    body=$(cat <<BODY
$name $version is out; the editor was last verified with $tested_version. Check it still works (see \`docs/compatibility.md\`):

- [ ] Update the game, play a few minutes, and save to a spare slot.
- [ ] Copy that save into \`fixtures/\` and run \`mise run check\` (round trip, career, fleet, edited fields).
- [ ] Run the inventory diff against the previous version's save (\`cargo run -p sii --example inventory -- old/game.sii new/game.sii\`) and note what changed in \`docs/compatibility.md\`.
- [ ] Change money with the editor in a copy of the new save, save, and load it in the game.
- [ ] Update \`compat/tested-versions.json\` and \`save::NEWEST_TESTED_SAVE_VERSION\`.
BODY
)
    if [ "${DRY_RUN:-}" = 1 ]; then
        printf 'Would open: %s\n%s\n' "$title" "$body"
        continue
    fi
    if [ -n "$(gh issue list --state all --search "\"$title\" in:title" --json number --jq '.[].number')" ]; then
        echo "An issue already exists: $title"
        continue
    fi
    gh issue create --title "$title" --body "$body"
done
