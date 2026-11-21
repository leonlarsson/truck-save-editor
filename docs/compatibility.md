# Staying compatible with game updates

SCS never published the save format, and it changes with game updates. This page collects what is known to change, why other editors broke, and what this one does about it. Research from November 2026; sources are linked so claims can be rechecked.

## What changes between game versions

- **New value types in binary SII.** Game 1.42 (save version 56) added types that broke SII_Decrypt ("unsupported value type") until it learned `0x2F` and `0x07`; `0x32` came later. SII_Decrypt is now discontinued. ([forum](https://steamcommunity.com/app/270880/discussions/0/3115920924805543459), [commits](https://github.com/TheLazyTomcat/SII_Decrypt/commits/master))
- **Binary SII versions.** Version 1 stores placements as 7 floats; version 2 and later as 8, with a hidden "bias" for x and z ([type docs](https://raw.githubusercontent.com/TheLazyTomcat/SII_Decrypt/master/Documents/Binary%20SII%20-%20Types.txt)). When version 3 appeared isn't documented; it predates 1.62.
- **Fields come and go every few versions.** Around save version 97 (2026): `economy.game_time_initial` can be `nil`, `economy_event.param` uses the full u32 range, `player.my_vehicles` was added, `company.state_change_time` was removed, `profile.sii` grew from 17 to 21 `user_data` entries. ([TS SE Tool #148](https://github.com/LIPtoH/TS-SE-Tool/issues/148))
- **New features add units.** Coaches (game 1.62's DLC, November 2026) added owned coaches, passenger jobs and `bus_experience_points`; 1.60 added fatigue, radio and a job widget, which likely added fields. ([SCS blog](https://blog.scssoft.com/2026/08/coaches-vehicle-ownership-job-schedule.html))
- **`info.sii` gains fields** (1.41 went from save version 53 to 55 with only `info.sii` changes), and should agree with `game.sii`. ([forum](https://steamcommunity.com/app/227300/discussions/0/3046108212586865712))
- **Profiles upgrade one way.** A newer game upgrades a profile on load; an older game can't read it. The game keeps `profiles(<version>).bak` copies. ([SCS forum](https://forum.scssoft.com/viewtopic.php?t=294989))
- **Not changed so far:** the `ScsC` encryption key and layout; the game loading plain-text `game.sii` (widely reported in 2026, never officially confirmed, see `TODO.md`).

Known save versions (no official table exists): 53→55 at 1.41, 56 at 1.42, 61–74 around 1.43–1.49, 97 in 2026 (ATS), 105 at ETS2 1.62.

## Why other editors broke

- **They rebuilt units from a fixed list of fields.** TS SE Tool wrote each unit from its own schema: on newer saves it silently dropped 322 fields the game had added and wrote 454 it no longer used. ([#148](https://github.com/LIPtoH/TS-SE-Tool/issues/148))
- **A crash mid-write destroyed the save.** An unexpected `nil` threw while writing, and because the file was truncated on open, `game.sii` was left empty.
- **Hard limits on versions or values**, like only accepting save versions 61–74, or signed integers for values that became unsigned.
- **Unknown value types stopped decoding** and the tools weren't updated in time; maintainers lost interest. ([#144](https://github.com/LIPtoH/TS-SE-Tool/issues/144), [#140](https://github.com/LIPtoH/TS-SE-Tool/issues/140))
- **Steam Cloud** profiles weren't found, or broken saves were synced up.

## What this editor does

1. **Never rebuild, only change.** A save is kept as every unit and field exactly as read, in order (`sii::Document`). Edits change named fields in place; everything else, including units, fields and values the editor has never heard of, is written back untouched. Writing an unedited save gives back identical text (tested on real saves).
2. **Values stay text.** Values are kept in their text form, so `nil`, huge numbers and new formats pass through untouched; typed access parses only what it edits, with wide types (`i64`).
3. **Stop on the unknown, don't guess.** An unknown binary value type or binary version stops reading with a clear message (unit, field, byte offset) instead of producing a damaged save.
4. **Typed edits are few and checked.** Money, experience, skills and the like go through `save::Career` and friends, each reading and writing its own fields; a missing field is an error the player sees, not a crash.
5. **Writing is safe.** The game must be closed; the whole slot is backed up outside the game's folders first; files are written beside the old ones and moved into place; `info.sii` is updated to match.

## How we know it still works

- **A fixture corpus**: at least one real save per game version and form (binary v2 and v3, text, with and without Coaches), kept locally in `fixtures/` (never committed; personal data). `sii` and `save` run every fixture: read, write as text, read again, compare; career and fleet read; unchanged careers write back byte-identical.
- **Synthetic tests** (committed) for every value type, damaged input, and each edit.
- **An inventory diff** for new game versions (planned): list every unit kind, field and value type in a save, and compare a new version's list with the previous one. New fields are fine (they're kept); renamed or removed fields the editor edits mean work.
- **A weekly check for game updates** (planned): a GitHub Actions job reads the public branch's build id from `https://api.steamcmd.net/v1/info/227300` (and `270880` for ATS; unofficial but open and keyless) and the Steam News API (`ISteamNews/GetNewsForApp`, titles like "Euro Truck Simulator 2: 1.63 Update Release"). When a new version appears it opens an issue: add a fixture from the new version, run the corpus and the inventory diff, and check a written save loads in game. Build ids alone can change without a new version, so the job matches the version from the news or the `temporary_1_NN` branch names. SteamDB forbids scraping and isn't used.
