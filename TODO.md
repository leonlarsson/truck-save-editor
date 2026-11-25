# TODO

Agreed work, in order. Each step should end with something the owner can see or try.

## 1. Foundations

- [x] Cargo workspace (edition 2024) with `crates/sii`, `crates/save`, `crates/app`; mise tasks working.
- [x] **GPUI test app:** a window with a styled money field, a skills picker and a virtual table of 2,000 rows, in light and dark. Run it under WSLg, and build it for Windows with `mise run win` and run it there. Decides whether GPUI stays (`docs/decisions.md`).

## 2. Reading saves (`sii`)

- [x] ScsC decrypt, binary SII decode, text SII parse, text SII write.
- [x] Round-trip tests on synthetic samples (committed) and on fixtures (local, skipped when missing).
- [x] Decoding a large save takes well under a second in a release build.

## 3. The in-game check

- [x] Write a save back as text and load it in game (2026-11-22: money and XP changed, Steam Cloud profile; the first try found the rotation format, now fixed and tested).
- [x] In game (2026-11-22): a truck moved to another garage with its driver, and a cancelled freight market job (after the fix that removes its units), both on a Steam Cloud profile.
- [ ] In game: a save on a local (non-cloud) profile written as a new slot.
- [x] In game (2026-11-22): moving a truck with its driver (Move to garage), removing a driver, a trailer and a garage, and forgetting the roads, on a Steam Cloud profile.
- [ ] In game: removing a truck.
- [x] The save dialog groups changes under what they belong to (the career, each truck, driver or garage, you, the map …), one labelled row per change (2026-11-22).
- [x] In game (2026-11-22): fuel over 100 % (150 % gave about 1.5 tanks of range); removing the side (engine) badge.
- [x] In game (2026-11-22): coloured plates (as Belgian plates), front and back, and plate text; the truck carrying the World of Trucks plate always gets that plate back on load, so its plate fields are off with the reason.
- [ ] In game: trailer cargo weight (only shows while the trailer is on a job) and empty weight; whether a coloured plate can keep its own country.
- [ ] A picker for each badge spot (front `f_badge`, sides `badge`, chassis logo `f_chs_logo`, others) from the game's own accessories, with "none" where the game has one, instead of only removing the side badge.
- [x] In game (2026-11-22): backing up a profile, a copied profile in the game's list (no World of Trucks link, as meant), copied settings (an option changed in the copy went back), the developer console.
- [ ] In game: a renamed profile, 128-truck convoys.
- [x] Engines and transmissions from the game's data for a truck's model (2026-11-22; works in game).
- [x] Make another truck your own; take the player to a city, the headquarters or another save's spot (2026-11-22).
- [x] In game (2026-11-22): Drive it yourself (the truck appears where the player was), with an engine and transmission swap.
- [x] In game (2026-11-22): `schedule_quick_travel` makes the game say "Game change detected" and offer its quick travel map for each of the player's vehicles (any visited city, free); the city the editor wrote wasn't used. `schedule_transfer_to_hq` did nothing. Now one button, Quick travel when the save loads; Take me to a city and Take me to headquarters are gone.
- [x] In game (2026-11-22): another save's spot (Where another save is…) puts the truck exactly there, after reading and writing the active `assigned_vehicles` entry as well as the `my_vehicles` one. The quick travel button itself (one change since) is yet to be clicked in game.
- [x] Make another trailer your own (Pull it yourself, 2026-11-22; not tried in game yet).
- [x] In game (2026-11-22): Rest and the clock (Evening) on a Steam Cloud profile.
- [ ] In game: Clear the record, Repair the cargo (none to try yet), Pull it yourself.
- [x] Report a problem opens a filled-in GitHub issue on Windows (2026-11-22).
- [x] In game (2026-11-22): Max their skills on two picked drivers.
- [x] In game (2026-11-22): Save over another… on a Steam Cloud profile (the chosen save got the changes and kept its name; the original stayed; no cloud prompt).
- [x] In game (2026-11-22): picked rows: Max their skills, Move to garage.
- [ ] In game: picked rows: Repair, Fill the tanks, Remove, Make them large (nothing to repair, fill or enlarge on the test save) (2026-11-22, not tried in game yet).

## 4. Picking a save

- [x] Find Steam Cloud and local profiles; decode profile names; list slots with preview image, date, money and level from `info.sii`.
- [x] Load in the background with progress.
- [x] Say what's shown and where from: the profile's saves, where they're kept and their folder; every folder looked in and what it held; the game's own saves folded to the newest three (2026-11-22).
- [x] No saves, or a profile without saves: say what to do. Add a folder (chosen or dropped on the window: a profiles folder, a profile, one save, or the game's folder in Documents), kept in the settings (2026-11-22).
- [x] A save on its own (outside a profile), dropped or chosen, opens and saves where it is (2026-11-22).
- [x] On Windows (2026-11-22): Add a folder…, dropping a folder (a loose save opens where it is), Show the folder.

## 5. Overview

- [x] Money, level and XP, skills, ADR, HQ city: shown, then editable, with "max" buttons.
- [x] Changes list, undo and redo.

## 6. Saving

- [x] Backup, game-running check, changes shown before writing, write `game.sii` + `info.sii`, restore a backup.

## 7. Sections

- [x] Fleet (trucks and coaches, repair), Garages (buy all, upgrade to large, HQ), Drivers (skills, max all).
- [x] Map: visit every city.
- [x] Map: unlock every dealer and recruitment agency (dealers from the city definitions; agencies from a list read off the 1.62 map, see `docs/sii-format.md`).
- [ ] Re-read the recruitment-agency list from the map when a map DLC adds cities (part of the game-update checklist).
- [x] Trailers (owned, with wear and repair, doubles included), odometers, plates, fuel.
- [x] Jobs: the cargo market (every offer, sortable, searchable, the game's company and cargo names), and keep every offer a week.
- [x] Jobs: the current job (cancel with no fee, remove its deadline; finishing isn't offered, see `docs/decisions.md`), and cargo icons.
- [ ] Cancelling quick jobs (a company truck), jobs in your own trailer and special transports, once there's a save to learn their shape from.
- [x] Detail panels: one truck, coach or trailer (plate, odometer, fuel, each part's wear), driver (experience, skills, ADR), garage (size, what's in it).
- [x] Detail panels: move a truck or driver to another garage (the whole slot moves; see `docs/decisions.md`).
- [x] All data: every unit and field, editable (shape-checked).
- [x] All data: edits show on the typed pages (and typed changes in All data), and search across every value.
- [x] Reading the game's archives (HashFS v1/v2) and its data: levels, ranks, skill limits, text and city names (`hashfs`, `gamedata`).
- [x] The game's own icons for skills and ADR classes: GDeflate and texture decoding in `hashfs`, the icon materials in `gamedata`, drawn at exact pixel size by `GameIconView`. Nothing of SCS's is bundled; without the game, the rows show no icon.
- [x] Brand logos and the game's model names in Fleet.
- [x] Company logos and cargo icons on Jobs (loaded as rows show, kept small).
- [x] Pictures from DLC archives too (some cargo icons live there).
- [x] Swedish, and the game's names in the app's language; Settings (language, appearance, game folder, backups).
- [x] German and Polish (2026-11-22), machine-drafted with the game's own terms.
- [ ] A native speaker reviews `de.ftl` and `pl.ftl`. More languages: a `Language`, a `locales/<code>.ftl` (the completeness test lists what's missing).
- [x] Coach garages: coaches have none. Fleet says where each is: driven now (`player.assigned_vehicles`) or parked in a city (`player.my_vehicles[]` → `stored_vehicle_city`).
- [x] HQ city: pick from the cities the company has garages in (Garages page).
- [x] City names from the game's city definitions (`city_name_localized`), not only by token: a few tokens (`vasteraas`) differ from their text keys.
- [x] Driver faces: `driver.<n>` shows the game's face `material/ui/driver/<n>.mat`.
- [ ] Driver names: the game reads `/locale/<lang>/driver_names.sii` when there is one, but ships none, and the default names aren't in any data file. Look again (maybe inside the executable's resources).

- [x] Company name (in `profile.sii`, backed up and checked like a slot; the login in the same file is never touched).

## Keeping up with the game

- [x] Inventory diff: list every unit kind, field and value type in a save, and compare two saves' lists (`docs/compatibility.md`).
- [x] Weekly GitHub Actions check for new game versions (steamcmd.net build id + Steam News); opens an issue with the checklist.
- [ ] Fixtures from more game versions and forms: text saves, Coaches, ATS.
- [x] Read the save `version` and warn when it's newer than any version the editor was tested with.

## Starting fresh

- [x] Save as a new slot: write the edited save into an empty slot instead of over the original.
- [x] Quick start for a fresh career ("Rich start", "Everything unlocked", or your own picks), applied as one change to a new profile made in the game.
- [x] Save over another save of the profile: the way to keep an edited copy in Steam Cloud, where new slots don't show (2026-11-22).
- [x] Duplicate a save from the start screen, unchanged (2026-11-24): a copy in a new slot, "<name> (copy)", written and checked like any save. Local profiles only: on Steam Cloud a slot the editor makes never shows in the game, so the button is off there and says what to do instead (a new slot in the game, or a local copy of the profile).
- [ ] In game (owner): a duplicated save on a local profile shows in the game's list and loads.
- [ ] Experiment (owner, in game): the game starts a career from `def/initial_save/normal/game.sii` in `def.scs`, a few units with `economy.new_game: true` that the game fills in itself. Does a save made of just that template, saved over a spare slot of an existing profile, start a fresh career in that profile? If it does, "Start over" could be a button.

- [x] In game (2026-11-22): Discover every road gives 100.00 % explored, in every country. The scan finds more items than the game counts (335,623 vs about 179,000); the extra ids are ignored.
- [x] Map → Map view: a dot for each road or junction (from the same scan's boxes), driven ones in the accent colour, and the player's truck (2026-11-22).
- [x] Map view: zoom (wheel, + and −, Whole map) and drag to move; drawn sharp for the area shown (2026-11-22).
- [x] Map view: city names (visited ones brighter, bigger cities first), the player's garages (2026-11-22).
- [x] Map view: click a city for its panel (visited, dealer, agency, garage, each with its action), a garage or the truck for theirs; the list of cities opens the same city panel (double-click or Enter). The switch reads List | Map, not Cities | Road map (2026-11-23).
- [x] City panel: mark a city as not visited, lock its dealer or agency again (2026-11-23).
- [ ] In game (owner): one city marked visited, one marked not visited, one dealer unlocked and one locked, saved as a new slot. Do they show as such in game, and does the game count a city again when driven through?
- [ ] In game (owner): a truck with its engine dragged to about 50 % and its wheels to about 25 % (wear sliders), saved as a new slot. Does the game show that wear, and does its own repair clear it?
- [x] Every city, not only those with a garage (2026-11-23): small towns have no garage unit, so the city list, the map's names and Visit every city missed them. Now the game's own city list (its map DLCs the save was made with, on the map) joins the save's.
- [x] The editor counts cities as the game's Career Progress does (2026-11-23): a city counts when it has a company or a garage (ferry ports and villages without either don't), and the save's non-city tokens (`truckersmp`, event places `x_…`) aren't cities.
- [ ] Map view: roads as lines (needs each road's ends, deeper in its item).

## More to edit (2026-11-23)

- [x] Coach services: the numbers the game's statistics show (worked out from its coach job log), the passengers transported and coach distance to set, Perfect satisfaction for every logged service, Discover every bus stop.
- [x] In game (owner, 2026-11-23): Perfect satisfaction, Discover every bus stop, passengers and coach distance all show in Career Progress → Coach Services and on the map.
- [x] Profile picture: taken out again (2026-11-23). The game only offers its own faces for a profile (My Drivers → Driver Details), never reads a picture of the player's own, and showed the face with `avatar.png` in place.
- [x] Paint in the truck and trailer panel: the paint job, from the game's own list for the model (one per name, in the version that fits the vehicle's cabin), and the colours the paint job lets you change, with the saved colours as quick picks.
- [x] Saved colours (Fleet → Saved colours…, and the palette): the 32 colours saved in the game's paint screen, each editable, copied as text to share and pasted from text.
- [x] In game (owner, 2026-11-23): a custom red on a truck and another paint job (Meteor, from the Super Stripes DLC) on a trailer show right. The trailer's paint shop didn't list Meteor: it's from the Super Stripes DLC, which the owner doesn't own. Paint jobs, engines and transmissions are now offered only from DLCs the save was made with.
- [x] Saved colours are 8 sets of 4, not 32 singles (seen in game, 2026-11-23: three colours in one group showed as one striped swatch); the editor now edits them as 8 sets. Confirmed in game: three single colours pasted show as three plain swatches.
- [x] Make a job (Jobs → Make a job…, and the palette): from a company that ships the cargo to one that takes it (the game's company definitions), on a trailer an offer in the save carries that cargo on (with its load and company truck), standard, important or urgent, for a week. The route is an offer's between the same cities, else the map's straight line scaled by the save's own offers.
- [x] In game (owner, 2026-11-23): both made jobs show on the freight market with their cargo, urgency and a week to go; the game plans the route itself (a ferry included where needed), and the pay follows the distance written. Not yet taken and delivered.
- [x] Gallery (the sidebar's Map group: Cities, World map, Gallery; 2026-11-23): every viewpoint and photo trophy from the game's own photo album (`def/photo_album`), with the player's progress (viewpoints from the save, photo trophies from the profile's album, read only), filters for what's left, and markers on the map; Visit every viewpoint (and the palette), or one at a time in its panel (Mark as visited, Mark as not found). Counts checked against a save's Career Progress: exact.
- [x] In game (owner, 2026-11-23): Visit every viewpoint shows 137/137 in Career Progress and every viewpoint as visited on the map; photo trophies unchanged. Not yet tried: Mark as not found.
- [x] Coach career (2026-11-23): a page of its own under Career, only for a save with a coach career: coach experience and skills, coach services, and every service in the coach job log. Overview and Statistics are the truck career's. Pages no longer have Ctrl+number shortcuts.
- [x] Truck dealers counted as the game counts them (144, not 146): Edirne and Veliko Tarnovo list a dealer brand but have no dealer on the map (2026-11-23).
- [ ] Recruitment agencies: the editor counts 138, the game 136 on the owner's save. Not Russia (three Russian cities have one), not the cities the game leaves out. The current map has 138 agency prefabs (`/prefab/recruitment_agency/`), and placing each in its nearest city gives 137: two cities the game doesn't count are still unknown. Next: assign each prefab to the city area box it stands in (not the nearest centre), then compare with the save's `unlocked_recruitments`.
- [ ] Say where things are kept: viewpoints are per save (`game.sii`), photo trophies per profile (its `album` folder). The Gallery says so now (2026-11-23); the rest of the app doesn't yet. Say it where it matters (the Gallery, panels, the save dialog), and look at what else is per profile rather than per save.
- [ ] Make a job: the trailer list names trailers by the game's codes (`Container, singl 3e 40`); the game's own trailer names would read better, if its data has them.

## Next features (2026-11-24)

- [ ] Add trucks and trailers from the app, in both games: buy any model the installed game has (as its dealer configures it) into a garage slot, and copy one you already own.
  - [x] Copy a truck or trailer into a garage with room (2026-11-25, `docs/decisions.md`).
  - [ ] A copied truck and trailer checked in the game (owner).
  - [ ] Buy a new truck or trailer: any model the installed game has, as its dealer configures it.

- [ ] Cars: the player's car (`player.cars`, ATS's Road Trip car) in Fleet, with fuel, wear, repairs and plate.
- [ ] Create a new save: a fresh career in a new local profile, from the start screen (see "Starting fresh").
- [ ] Plates: layout and badges (the region's plate, badge images, text placement), from the game's own plate definitions.
- [ ] Accessories: a vehicle's add-on parts (lights, horns, grilles …), with the game's alternatives for each slot.
- [ ] Coach services: bus stations, the coach fleet's timetable and the coach offers.
- [ ] Freight market: edit an existing offer (cargo and trailer, destination, urgency, expiry, distance, ferry costs); the cargo market's offers per depot.
- [ ] GPS routes: copy a saved route (and the truck's position) between saves.
- [ ] Restore a damaged save's backup into a new profile; snapshots of the open save to go back to.
- [ ] Find the game's own versioned backup profiles too.
- [ ] Updates that download and install themselves (verified), not only a notice.
- [ ] A log of what the editor did, viewable in the app.
- [ ] French and Spanish (languages are the owner's call).

## Later

- [ ] American Truck Simulator: test on real ATS saves (the code is written for both from the start). Until then, ATS profiles and saves show a warning (2026-11-22). Still ETS2-only: the game's data (city names, ranks, engines, icons, the DLC check), distances in miles, and Settings (the game's folder, whether it's running, its config.cfg).
  - [x] A first real ATS save (owner, 2026-11-24, a new career; copy in `fixtures/ats/2`) opens and reads; `TSE_GAME=ats` opens it with `TSE_SLOT`.
  - [x] The game's data is read per game, from that game's install (2026-11-24): an ATS save gets ATS's map, cities, dealers, photo album, texts and DLC check; Settings keeps a folder for each game; Game settings acts on the current game. ATS's place names come from its definitions (its texts don't translate them).
  - [x] Coach career only for a game with coaches (`Game::has_coaches`): ATS saves carry empty coach fields.
  - [x] Distances read in miles for ATS (`units::distance`).
  - [x] Recruitment agencies for ATS, read off its map (2026-11-24): the agencies' prefab models from the game's definitions, each building placed in the city area it stands in (or one within 250 m). ETS2 keeps its list: the same rule gives 135 there, the list 138, the game 136.
  - [x] The odometer and a trailer's weights are edited in miles and pounds for ATS; hazardous cargo and states in ATS's words.
  - [ ] ATS (owner): what Career Progress says for recruitment agencies, to compare with the editor's 25.
  - [x] Recruitment agencies unlocked in the save stay shut in the game (black screen, no hiring online), in both games: the editor no longer unlocks them, only locks them, which with a drive past repairs one (2026-11-25, `docs/decisions.md`).
  - [x] A real check in ATS (owner, 2026-11-25): money, experience, a dealer unlocked and a garage upgraded by the editor, saved and loaded in the game, all as expected (agencies aside, see below).
  - [x] Quick travel was off for the ATS save because it has no truck of its own yet (quick jobs only); it now says so, and World map says there's no truck instead of putting one at the map's origin (2026-11-24).
  - [x] ATS's cargo names in full (the game's `local.override.sii` was not read), its BC3 company logos, Garages and Buy all without cities of map DLCs that aren't installed, the start screen's folders for the picked game (2026-11-24).
  - [ ] An ATS save with a truck of its own (owner): to check the truck panel, plates, paint, parts and trailers with ATS's data.
  - [ ] ATS (owner): whether the game's own map shows the regions east of the installed states (its `base_map.scs` holds their roads, which World map draws without names).
- [x] App icon (`scripts/make-icon.py`, embedded in the .exe) and a release workflow (version tag → draft release).
- [ ] A final product name (Truck Save Editor for now, in `APP_NAME`; never a game name, those are SCS trademarks), signed Windows builds (see `docs/releasing.md` → Code signing), the first release (the owner's call; the pipeline is ready, `docs/releasing.md`).
- [ ] Web version, on hold (the owner's call, 2026-11-21): the desktop app comes first. The core crates stay free of UI and disk access so it stays possible (GPUI's web target, or a page around the core compiled to wasm), but no work on it until it's picked up again.
