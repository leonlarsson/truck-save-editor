//! The installed game's own data (level tables, rank names, city names, icons), loaded in the background
//! at startup. Until it arrives, or if the game isn't found, the app uses built-in fallbacks.
//!
//! It's one game's at a time: the current game's (`current_game`), the open save's or the one
//! picked on the start screen (`set_game`). Each game's data, once read, is kept while the other's
//! is shown, so switching back is instant. Everything here reads the current game's.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use gamedata::{GameData, GameIcon};
use gpui_kit::{App, AppContext as _, Context, Global, Subscription, Window};
use save::{Game, LevelTable};

#[derive(Clone, Default)]
struct Loaded {
    data: Option<Arc<GameData>>,
    /// The DLCs and mods installed (`None` without the game).
    installed: Option<Arc<gamedata::Installed>>,
    /// A load is under way (at startup, or after the language or the game's folder changed).
    is_loading: bool,
}

/// Every road and junction on the game's map (for the map view and Discover every road), read
/// after the rest of the game's data so it never holds that up. `None` until read, or without the
/// game.
#[derive(Clone, Default)]
struct MapRoads(Option<RoadsRead>);

/// The map's roads as read: with where they are (for the map view), and their ids alone (for
/// Discover every road, kept apart so it isn't rebuilt from the roads each time).
#[derive(Clone)]
struct RoadsRead {
    roads: Arc<Vec<gamedata::MapRoad>>,
    /// How the roads run (their lines on the map), for the map view to draw.
    lines: Arc<Vec<gamedata::MapLine>>,
    ids: Arc<Vec<u64>>,
    /// The cities' areas and the garages, for names and garages on the map view.
    places: Arc<Vec<gamedata::MapPlace>>,
    /// Where the viewpoints and photo trophies are.
    markers: Arc<Vec<gamedata::MapMarker>>,
    /// The cities with a recruitment agency, read off the map (for a game without a list of
    /// them, ATS: see `place_cities`).
    agency_cities: Arc<BTreeSet<String>>,
}

impl Global for MapRoads {}

/// The map's roads and junctions by id, once read (`None` before, or without the game).
pub fn road_ids(cx: &App) -> Option<Arc<Vec<u64>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.ids.clone()))
}

/// The map's city areas and garages, once read.
pub fn map_places(cx: &App) -> Option<Arc<Vec<gamedata::MapPlace>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.places.clone()))
}

/// Every city in the installed game, by token, with the map DLC it comes with (`None` for the
/// base game's): see `gamedata::GameData::cities`.
pub fn cities(cx: &App) -> Option<Vec<(String, Option<String>)>> {
    data(cx).map(|data| {
        data.cities()
            .map(|(token, dlc)| (token.to_string(), dlc.map(str::to_string)))
            .collect()
    })
}

/// Calls back when the map's roads and places have been read (after the rest of the data).
pub fn observe_map_in<V: 'static>(
    window: &mut gpui_kit::Window,
    cx: &mut gpui_kit::Context<V>,
    on_read: impl Fn(&mut V, &mut gpui_kit::Window, &mut gpui_kit::Context<V>) + 'static,
) -> gpui_kit::Subscription {
    cx.observe_global_in::<MapRoads>(window, on_read)
}

/// Whether the game's map has been read (or found missing): the last of what loads in the
/// background after startup.
pub fn is_map_read(cx: &App) -> bool {
    cx.try_global::<MapRoads>().is_some()
}

/// Where the map's viewpoints and photo trophies are, once read.
pub fn map_markers(cx: &App) -> Option<Arc<Vec<gamedata::MapMarker>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.markers.clone()))
}

/// One of the game's world map icons (see `gamedata::MAP_ICONS`), if the game was found.
pub fn map_icon(cx: &App, name: &str) -> Option<Arc<gamedata::GameIcon>> {
    data(cx)?.map_icon(name)
}

/// The game's photo album (every viewpoint and photo trophy), once its data is read.
pub fn photo_album(cx: &App) -> Option<gamedata::PhotoAlbum> {
    data(cx).map(|data| data.photo_album.clone())
}

/// A viewpoint's or photo trophy's name in the game's words (see `GameData::album_item_name`).
pub fn album_item_name(cx: &App, item: &gamedata::AlbumItem) -> String {
    match data(cx) {
        Some(data) => data.album_item_name(item),
        None => item.id.clone(),
    }
}

/// Text that may be a key (`@@landmark_prague@@`) in the game's words, else as it is.
pub fn resolve(cx: &App, raw: &str) -> String {
    match data(cx) {
        Some(data) => data.resolve(raw),
        None => raw.trim_matches('@').to_string(),
    }
}

/// How the map's roads and junctions run, once read: the lines the map view draws.
pub fn map_lines(cx: &App) -> Option<Arc<Vec<gamedata::MapLine>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.lines.clone()))
}

/// The map's roads and junctions with where they are, once read.
pub fn map_roads(cx: &App) -> Option<Arc<Vec<gamedata::MapRoad>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.roads.clone()))
}

impl Global for Loaded {}

/// The game whose data is loaded (see the module's comment).
struct CurrentGame(Game);

impl Global for CurrentGame {}

/// Each other game's data, once read: its data and its map.
#[derive(Default)]
struct Kept(HashMap<Game, (Loaded, Option<MapRoads>)>);

impl Global for Kept {}

/// The game whose data is loaded: Euro Truck Simulator 2 until another is picked.
pub fn current_game(cx: &App) -> Game {
    cx.try_global::<CurrentGame>()
        .map(|current| current.0)
        .unwrap_or(Game::Ets2)
}

/// Calls back when the current game changes (see `set_game`).
pub fn observe_current_game_in<V: 'static>(
    window: &mut Window,
    cx: &mut Context<V>,
    on_change: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
) -> Subscription {
    cx.observe_global_in::<CurrentGame>(window, on_change)
}

/// Shows another game's data: what was read for it before, else read now. The current game's is
/// kept for when it's picked again.
pub fn set_game(cx: &mut App, game: Game) {
    let previous = current_game(cx);
    if previous == game {
        return;
    }
    if !cx.has_global::<Kept>() {
        cx.set_global(Kept::default());
    }
    // Kept only when fully read: a half-read game is read again when picked.
    let loaded = cx.try_global::<Loaded>().cloned();
    let map = cx.try_global::<MapRoads>().cloned();
    if let Some(loaded) = loaded.filter(|loaded| !loaded.is_loading)
        && map.is_some()
    {
        cx.global_mut::<Kept>().0.insert(previous, (loaded, map));
    }
    cx.set_global(CurrentGame(game));
    forget_read_on_demand(cx);
    match cx.global_mut::<Kept>().0.remove(&game) {
        Some((loaded, map)) => {
            cx.set_global(loaded);
            match map {
                Some(map) => cx.set_global(map),
                None => forget_map(cx),
            }
            cx.refresh_windows();
        }
        None => {
            forget_map(cx);
            // The other game's data goes at once, not when this one's has been read: until then
            // nothing is known (and nothing is said), where the other game's answers would be
            // wrong ones: its DLCs held against this game's saves, its names for this game's
            // cities.
            cx.set_global(Loaded::default());
            load(cx);
        }
    }
}

/// The map isn't read (for the game being switched to): what's shown waits for it.
fn forget_map(cx: &mut App) {
    if cx.has_global::<MapRoads>() {
        cx.remove_global::<MapRoads>();
    }
}

/// What's read the first time it's asked for (a model's engines, its paint jobs, the companies'
/// cargo, drivers' faces) is the current game's: read again for another.
fn forget_read_on_demand(cx: &mut App) {
    cx.set_global(Parts::default());
    cx.set_global(PaintJobs::default());
    cx.set_global(CompanyCargoState::default());
    cx.set_global(DealerTrucks::default());
    cx.set_global(DealerTrailers::default());
    cx.set_global(Pictures::default());
}

/// Each installed game's logo (the start screen's game picker shows them).
#[derive(Default)]
struct Logos(Vec<(Game, Arc<gamedata::GameIcon>)>);

impl Global for Logos {}

/// Starts reading the game's data in the background; windows redraw when it arrives.
pub fn init(cx: &mut App) {
    cx.set_global(Loaded::default());
    reload(cx);
    load_logos(cx);
}

/// Reads each installed game's logo from its own files, in the background.
fn load_logos(cx: &mut App) {
    cx.set_global(Logos::default());
    let dirs: Vec<(Game, Option<std::path::PathBuf>)> = Game::ALL
        .into_iter()
        .map(|game| (game, install_dir_of(cx, game)))
        .collect();
    cx.spawn(async move |cx| {
        let logos = cx
            .background_spawn(async move {
                dirs.into_iter()
                    .filter_map(|(game, dir)| {
                        let mut logo = gamedata::game_logo(&dir?)?;
                        // Both games' logos have the same path, and drawn icons are kept by
                        // name (`ui::GameIconView`): each gets its own.
                        logo.name = format!("{}{}", game.name(), logo.name);
                        Some((game, Arc::new(logo)))
                    })
                    .collect::<Vec<_>>()
            })
            .await;
        cx.update(|cx| {
            cx.set_global(Logos(logos));
            cx.refresh_windows();
        });
    })
    .detach();
}

/// A game's logo, from its install, once read.
pub fn game_logo(cx: &App, game: Game) -> Option<Arc<gamedata::GameIcon>> {
    cx.try_global::<Logos>()?
        .0
        .iter()
        .find(|(of, _)| *of == game)
        .map(|(_, logo)| logo.clone())
}

/// Where the current game is: the folder picked in the settings, else the one Steam knows.
pub fn install_dir(cx: &App) -> Option<std::path::PathBuf> {
    install_dir_of(cx, current_game(cx))
}

/// Where a game is: the folder picked in the settings, else the one Steam knows.
pub fn install_dir_of(cx: &App, game: Game) -> Option<std::path::PathBuf> {
    let picked = crate::settings::get(cx).game_dir(game).cloned();
    // Tests never read the games installed on the machine running them (as they never read its
    // settings file): only a folder a test picks.
    if cfg!(test) {
        return picked;
    }
    picked.or_else(|| gamedata::find_install(game))
}

/// Reads the current game's data again (after the language or a game's folder changed), and
/// forgets what was read for the others. What's shown until it arrives is the data read before.
pub fn reload(cx: &mut App) {
    cx.set_global(Kept::default());
    forget_read_on_demand(cx);
    load(cx);
}

/// Reads the current game's data, then its map, in the background.
fn load(cx: &mut App) {
    if !cx.has_global::<Loaded>() {
        cx.set_global(Loaded::default());
    }
    cx.global_mut::<Loaded>().is_loading = true;
    let game = current_game(cx);
    let dir = install_dir_of(cx, game);
    let roads_dir = dir.clone();
    let language = crate::settings::get(cx).language.game_code();
    cx.spawn(async move |cx| {
        let (data, installed) = cx
            .background_spawn(async move {
                let Some(dir) = dir else {
                    return (None, None);
                };
                // The game's text in the app's language, else English (`load` falls back).
                let data = GameData::load(game, &dir, language).ok();
                let installed = data.is_some().then(|| gamedata::installed(game, &dir));
                (data, installed)
            })
            .await;
        // Which buildings are recruitment agencies, for finding them on the map below, and which
        // cities the installed game has (the map's roads of other map DLCs are left out).
        let agency_models = data
            .as_ref()
            .map(|data| data.agency_models.clone())
            .unwrap_or_default();
        let installed_cities: std::collections::HashSet<String> = data
            .as_ref()
            .map(|data| data.cities().map(|(city, _)| city.to_string()).collect())
            .unwrap_or_default();
        // Another game picked meanwhile: this one's read isn't shown (it's read again if picked).
        let is_current = cx.update(|cx| current_game(cx) == game);
        if !is_current {
            return;
        }
        cx.update(|cx| {
            cx.set_global(Loaded {
                data: data.map(Arc::new),
                installed: installed.map(Arc::new),
                is_loading: false,
            });
            cx.refresh_windows();
        });
        // Then the map's roads: a second or two of reading, which only Discover every road needs.
        let roads = cx
            .background_spawn(async move {
                let mut map = gamedata::map_contents(game, roads_dir.as_deref()?);
                gamedata::keep_installed_roads(&mut map, &installed_cities);
                let ids: Vec<u64> = map.roads.iter().map(|road| road.id).collect();
                let agency_cities = gamedata::agency_cities(&map, &agency_models);
                (!map.roads.is_empty()).then(|| RoadsRead {
                    roads: Arc::new(map.roads),
                    lines: Arc::new(map.lines),
                    ids: Arc::new(ids),
                    places: Arc::new(map.places),
                    markers: Arc::new(map.markers),
                    agency_cities: Arc::new(agency_cities),
                })
            })
            .await;
        let is_current = cx.update(|cx| current_game(cx) == game);
        if !is_current {
            return;
        }
        cx.update(|cx| {
            cx.set_global(MapRoads(roads));
            cx.refresh_windows();
        });
        // Development builds only: `TSE_READY_FILE=<path>` is written once the game's data and
        // its map are read, the last of what loads in the background, so a script taking
        // screenshots (`scripts/screenshots.sh`) knows when to start instead of guessing.
        if cfg!(debug_assertions)
            && let Some(path) = std::env::var_os("TSE_READY_FILE")
        {
            let _ = std::fs::write(path, "ready");
        }
    })
    .detach();
}

fn data(cx: &App) -> Option<Arc<GameData>> {
    cx.try_global::<Loaded>()
        .and_then(|loaded| loaded.data.clone())
}

/// The DLCs and mods installed, once the game's data is read (`None` without the game).
pub fn installed(cx: &App) -> Option<Arc<gamedata::Installed>> {
    cx.try_global::<Loaded>()
        .and_then(|loaded| loaded.installed.clone())
}

/// Whether the game's data has been read (the game was found).
pub fn has_data(cx: &App) -> bool {
    data(cx).is_some()
}

/// Whether the game's data is being read right now.
pub fn is_loading(cx: &App) -> bool {
    cx.try_global::<Loaded>()
        .is_some_and(|loaded| loaded.is_loading)
}

pub fn truck_levels(cx: &App) -> LevelTable {
    data(cx).map(|d| d.truck_levels.clone()).unwrap_or_default()
}

pub fn coach_levels(cx: &App) -> LevelTable {
    data(cx).map(|d| d.coach_levels.clone()).unwrap_or_default()
}

pub fn truck_rank(cx: &App, level: i64) -> Option<String> {
    data(cx)?.truck_rank(level)
}

pub fn coach_rank(cx: &App, level: i64) -> Option<String> {
    data(cx)?.coach_rank(level)
}

/// A country's name in the app's language, by its token (`italy` → `Italy`); made from the token
/// when the game's data isn't there.
pub fn country_name(cx: &App, token: &str) -> String {
    data(cx)
        .and_then(|d| d.country_name(token))
        .unwrap_or_else(|| save::city_display_name(token))
}

/// A city's name: the game's own (`Malmö`), else made from its token (`Malmo`), with its
/// country's short code as the game shows a city in jobs and the delivery log: `Verona (I)`,
/// `Maribor (SLO)` (when the game's data is there).
pub fn city_name(cx: &App, token: &str) -> String {
    let data = data(cx);
    let name = data
        .as_ref()
        .and_then(|d| d.city_name(token))
        .unwrap_or_else(|| save::city_display_name(token));
    match data.as_ref().and_then(|d| d.city_country_code(token)) {
        Some(code) => format!("{name} ({code})"),
        None => name,
    }
}

/// A skill's icon from the game, by its save field name (`long_dist`, `long_duty` …).
pub fn skill_icon(cx: &App, field: &str) -> Option<Arc<GameIcon>> {
    data(cx)?.skill_icon(field)
}

/// An ADR class's placard from the game, by its bit in the save.
pub fn adr_icon(cx: &App, bit: usize) -> Option<Arc<GameIcon>> {
    data(cx)?.adr_icon(bit)
}

/// The game's own text for a key, if the game was found and has it.
pub fn text(cx: &App, key: &str) -> Option<String> {
    data(cx)?.text(key).filter(|text| !text.is_empty())
}

/// The cities with a truck dealer, or with a recruitment agency, in the installed game. `None`
/// when they aren't known: the game (or its city definitions) wasn't found, or the editor doesn't
/// know them for this game (recruitment agencies for ATS).
pub fn place_cities(cx: &App, place: save::Place) -> Option<BTreeSet<String>> {
    let data = data(cx)?;
    let cities = match place {
        save::Place::Dealer => data.dealer_cities().clone(),
        // ETS2's list, else the agencies found on the map once it's read.
        save::Place::Recruitment => match data.recruitment_cities() {
            Some(cities) => cities.clone(),
            None => map_agency_cities(cx)?.as_ref().clone(),
        },
    };
    (!cities.is_empty()).then_some(cities)
}

/// The cities with a recruitment agency found on the map, once it's read.
fn map_agency_cities(cx: &App) -> Option<Arc<BTreeSet<String>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.agency_cities.clone()))
}

/// Calls `on_loaded` when the game's data arrives, for views that keep text made from it (city
/// names, which cities have dealers) and must remake it then.
pub fn observe<V: 'static>(
    cx: &mut Context<V>,
    on_loaded: impl Fn(&mut V, &mut Context<V>) + 'static,
) -> Subscription {
    cx.observe_global::<Loaded>(on_loaded)
}

/// A vehicle's model name: the game's own (`Volvo FH4`), else made from its id.
pub fn model_name(cx: &App, vehicle: &save::Vehicle) -> Option<String> {
    let model_id = vehicle.model_id.as_deref()?;
    // SCS's own trailers are known by their body (`Refrigerated`); their series names (`STD`)
    // say little. Branded trailers (Krone, Schmitz …) keep their model name.
    if vehicle.kind == save::VehicleKind::Trailer
        && model_id.starts_with("scs.")
        && let Some(body) = vehicle.body_type.as_deref()
    {
        return Some(text(cx, body).unwrap_or_else(|| save::model_display_name(body)));
    }
    data(cx)
        .and_then(|d| d.model_name(model_id))
        .or_else(|| vehicle.display_name())
}

/// A vehicle's brand logo from the game, if there is one.
pub fn brand_logo(cx: &App, vehicle: &save::Vehicle) -> Option<Arc<GameIcon>> {
    let brand = vehicle.model_id.as_deref()?.split('.').next()?;
    data(cx)?.brand_logo(brand, vehicle.kind == save::VehicleKind::Coach)
}

/// Like `observe`, for views that need the window to remake what they show (a picker's items).
pub fn observe_in<V: 'static>(
    window: &mut Window,
    cx: &mut Context<V>,
    on_loaded: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
) -> Subscription {
    cx.observe_global_in::<Loaded>(window, on_loaded)
}

/// Pictures from the game (drivers' faces, company logos, cargo icons), loaded one by one as
/// they're first shown (there are hundreds of each, and a page shows a few dozen), each shrunk to
/// a small copy so a big save doesn't cost much memory.
#[derive(Default)]
struct Pictures {
    /// `base.scs` and every DLC archive (some pictures, like a DLC's cargo icons, live only
    /// there), opened once on first use, shared with the background loads.
    archives: Arc<Mutex<Option<Vec<hashfs::Archive>>>>,
    /// By material path: the picture, or `None` while loading or if the game has none.
    loaded: HashMap<String, Option<Arc<GameIcon>>>,
}

impl Global for Pictures {}

/// The size pictures are kept at, in pixels on their longer side: sharp at 2× for what the
/// tables show.
const PICTURE_PIXELS: u32 = 96;

/// One of the game's pictures, by its material (`/material/ui/driver`, `59`). The first ask
/// starts loading it in the background and returns `None`; the window redraws when it's there.
fn picture(cx: &mut App, folder: &'static str, name: String) -> Option<Arc<GameIcon>> {
    let key = format!("{folder}/{name}");
    if !cx.has_global::<Pictures>() {
        cx.set_global(Pictures::default());
    }
    if let Some(picture) = cx.global::<Pictures>().loaded.get(&key) {
        return picture.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    let pictures = cx.global_mut::<Pictures>();
    pictures.loaded.insert(key.clone(), None);
    let archives = pictures.archives.clone();
    cx.spawn(async move |cx| {
        let picture = cx
            .background_spawn(async move {
                // A poisoned lock only means an earlier load panicked; the archives are fine.
                let mut archives = archives
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let archives = archives.get_or_insert_with(|| {
                    gamedata::content_archives(&install_dir)
                        .iter()
                        .filter_map(|path| hashfs::Archive::open(path).ok())
                        .collect()
                });
                // The first archive that has the picture's material, the base game's first.
                let material = format!("{folder}/{name}.mat");
                let archive = archives
                    .iter_mut()
                    .find(|archive| archive.contains(&material))?;
                GameIcon::load(archive, folder, &name)
                    .map(|picture| Arc::new(picture.shrunk(PICTURE_PIXELS)))
            })
            .await;
        cx.update(|cx| {
            cx.global_mut::<Pictures>().loaded.insert(key, picture);
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// A hired driver's face (`driver.59` → the game's face 59).
pub fn driver_face(cx: &mut App, driver_id: &str) -> Option<Arc<GameIcon>> {
    let number: u32 = driver_id.strip_prefix("driver.")?.parse().ok()?;
    picture(cx, "/material/ui/driver", number.to_string())
}

/// A company's small logo (`renar`), as the game's job lists show it.
pub fn company_logo(cx: &mut App, company: &str) -> Option<Arc<GameIcon>> {
    picture(cx, "/material/ui/company/small", company.to_string())
}

/// A cargo's icon (`bricks`).
pub fn cargo_icon(cx: &mut App, cargo: &str) -> Option<Arc<GameIcon>> {
    picture(cx, "/material/ui/cargo_icons", cargo.to_string())
}

/// A company's name (`renar` → `Renar Logistik`), else made from its token.
pub fn company_name(cx: &App, token: &str) -> String {
    data(cx)
        .and_then(|d| d.company_name(token))
        .unwrap_or_else(|| save::city_display_name(token))
}

/// A cargo's name (`battery` → `Batteries`), else made from its token.
pub fn cargo_name(cx: &App, token: &str) -> String {
    data(cx)
        .and_then(|d| d.cargo_name(token))
        .unwrap_or_else(|| save::city_display_name(token))
}

/// One engine or transmission a truck can have, in the app's language.
#[derive(Clone, Debug, PartialEq)]
pub struct PartChoice {
    /// Its definition, what the truck's accessory points at.
    pub path: String,
    pub name: String,
    /// What it's like (`750 hp (551 kW)`, `3,550 Nm`).
    pub details: String,
    pub price: i64,
    pub unlock: i64,
    /// The DLC it comes with, `None` for the base game's (see `Session::has_dlc`).
    pub dlc: Option<String>,
}

/// The engines and transmissions of truck models, loaded when a truck's panel first asks.
#[derive(Default)]
struct Parts(HashMap<(String, gamedata::PartKind), Option<Arc<Vec<gamedata::Part>>>>);

impl Global for Parts {}

/// The engines or transmissions a truck model can have (`volvo.fh16_2012`). The first ask starts
/// reading them in the background and returns `None`; the window redraws when they're there.
pub fn model_parts(cx: &mut App, model: &str, kind: gamedata::PartKind) -> Option<Vec<PartChoice>> {
    if !cx.has_global::<Parts>() {
        cx.set_global(Parts::default());
    }
    let key = (model.to_string(), kind);
    if let Some(parts) = cx.global::<Parts>().0.get(&key) {
        let data = data(cx)?;
        return parts.as_ref().map(|parts| {
            parts
                .iter()
                .map(|part| PartChoice {
                    path: part.path.clone(),
                    name: data.resolve(&part.name),
                    details: part
                        .info
                        .iter()
                        .map(|line| data.resolve(line))
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · "),
                    price: part.price,
                    unlock: part.unlock,
                    dlc: part.dlc.clone(),
                })
                .collect()
        });
    }
    let install_dir = data(cx)?.install_dir.clone();
    cx.global_mut::<Parts>().0.insert(key.clone(), None);
    let model = model.to_string();
    cx.spawn(async move |cx| {
        let parts = cx
            .background_spawn(async move { gamedata::model_parts(&install_dir, &model, kind) })
            .await;
        cx.update(|cx| {
            cx.global_mut::<Parts>()
                .0
                .insert(key, Some(Arc::new(parts)));
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// What every company ships and takes, once read (`None` inside while reading).
#[derive(Default)]
struct CompanyCargoState(Option<Option<Arc<gamedata::CompanyCargo>>>);

impl Global for CompanyCargoState {}

/// What every company ships and takes (`gamedata::company_cargo`). The first ask starts reading
/// it in the background and returns `None`; the window redraws when it's there.
pub fn company_cargo(cx: &mut App) -> Option<Arc<gamedata::CompanyCargo>> {
    if !cx.has_global::<CompanyCargoState>() {
        cx.set_global(CompanyCargoState::default());
    }
    if let Some(state) = &cx.global::<CompanyCargoState>().0 {
        return state.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    cx.global_mut::<CompanyCargoState>().0 = Some(None);
    cx.spawn(async move |cx| {
        let cargo = cx
            .background_spawn(async move { gamedata::company_cargo(&install_dir) })
            .await;
        cx.update(|cx| {
            cx.global_mut::<CompanyCargoState>().0 = Some(Some(Arc::new(cargo)));
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// The trucks the dealers sell, once read (`None` inside while reading).
#[derive(Default)]
struct DealerTrucks(Option<Option<Arc<Vec<gamedata::DealerTruck>>>>);

impl Global for DealerTrucks {}

/// The trucks the game's dealers sell ready-made (`gamedata::dealer_trucks`). The first ask starts
/// reading them in the background and returns `None`; the window redraws when they're there.
pub fn dealer_trucks(cx: &mut App) -> Option<Arc<Vec<gamedata::DealerTruck>>> {
    if !cx.has_global::<DealerTrucks>() {
        cx.set_global(DealerTrucks::default());
    }
    if let Some(state) = &cx.global::<DealerTrucks>().0 {
        return state.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    cx.global_mut::<DealerTrucks>().0 = Some(None);
    cx.spawn(async move |cx| {
        let trucks = cx
            .background_spawn(async move { gamedata::dealer_trucks(&install_dir) })
            .await;
        cx.update(|cx| {
            cx.global_mut::<DealerTrucks>().0 = Some(Some(Arc::new(trucks)));
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// The trailers the dealers sell, once read (`None` inside while reading).
#[derive(Default)]
struct DealerTrailers(Option<Option<Arc<Vec<gamedata::DealerTrailer>>>>);

impl Global for DealerTrailers {}

/// The trailers the game's dealers sell ready-made (`gamedata::dealer_trailers`), read on first
/// ask like `dealer_trucks`.
pub fn dealer_trailers(cx: &mut App) -> Option<Arc<Vec<gamedata::DealerTrailer>>> {
    if !cx.has_global::<DealerTrailers>() {
        cx.set_global(DealerTrailers::default());
    }
    if let Some(state) = &cx.global::<DealerTrailers>().0 {
        return state.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    cx.global_mut::<DealerTrailers>().0 = Some(None);
    cx.spawn(async move |cx| {
        let trailers = cx
            .background_spawn(async move { gamedata::dealer_trailers(&install_dir) })
            .await;
        cx.update(|cx| {
            cx.global_mut::<DealerTrailers>().0 = Some(Some(Arc::new(trailers)));
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// A trailer's name as lists show it: SCS's own by their body (`Refrigerated`), as `model_name`
/// does; branded ones (Krone …) by their model.
pub fn trailer_name(cx: &App, model_id: &str, body_type: &str) -> String {
    if model_id.starts_with("scs.") && !body_type.is_empty() {
        return text(cx, body_type).unwrap_or_else(|| save::model_display_name(body_type));
    }
    model_name_by_id(cx, model_id)
}

/// `count` new plates for vehicles of `kind` (`truck`, `trailer`) kept in a garage
/// (`garage.san_francisc`), made the game's way for its country (`gamedata::make_plate`): a
/// truck or trailer the editor adds gets one, and each trailer of a road train. Different each
/// time (seeded by the clock); the change keeps them, so the preview and the save agree. Empty
/// without the game's data, or for a country without plate formats.
pub fn new_plates(cx: &App, garage: &str, kind: &str, count: usize) -> Vec<String> {
    let Some(data) = data(cx) else {
        return Vec::new();
    };
    let city = garage.strip_prefix("garage.").unwrap_or(garage);
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|time| time.as_nanos() as u64)
        .unwrap_or(0);
    (0..count as u64)
        .filter_map(|index| data.new_plate(city, kind, seed.wrapping_add(index * 7_919)))
        .collect()
}

/// A model's name by its id (`volvo.vnl2018` → `Volvo VNL 2018`), else made from the id.
pub fn model_name_by_id(cx: &App, model_id: &str) -> String {
    data(cx)
        .and_then(|d| d.model_name(model_id))
        .unwrap_or_else(|| save::model_display_name(model_id))
}

/// The paint jobs read so far, by model.
#[derive(Default)]
struct PaintJobs(HashMap<String, Option<Arc<Vec<gamedata::PaintJob>>>>);

impl Global for PaintJobs {}

/// The paint jobs a model can have (`volvo.fh16`), as the game defines them (texts still keys:
/// see `paint_job_name`). The first ask starts reading them in the background and returns `None`;
/// the window redraws when they're there.
pub fn model_paint_jobs(cx: &mut App, model: &str) -> Option<Arc<Vec<gamedata::PaintJob>>> {
    if !cx.has_global::<PaintJobs>() {
        cx.set_global(PaintJobs::default());
    }
    if let Some(jobs) = cx.global::<PaintJobs>().0.get(model) {
        return jobs.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    cx.global_mut::<PaintJobs>()
        .0
        .insert(model.to_string(), None);
    let model = model.to_string();
    cx.spawn(async move |cx| {
        let jobs = cx
            .background_spawn({
                let model = model.clone();
                async move { gamedata::model_paint_jobs(&install_dir, &model) }
            })
            .await;
        cx.update(|cx| {
            cx.global_mut::<PaintJobs>()
                .0
                .insert(model, Some(Arc::new(jobs)));
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// A paint job's name, in the game's words (else from the file's name: `style1.sii` → `style1`).
pub fn paint_job_name(cx: &App, path: &str) -> String {
    let known = cx.try_global::<PaintJobs>().and_then(|jobs| {
        jobs.0.values().flatten().find_map(|list| {
            let job = list.iter().find(|job| job.path == path)?;
            Some(data(cx)?.resolve(&job.name))
        })
    });
    known.filter(|name| !name.is_empty()).unwrap_or_else(|| {
        std::path::Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
}

/// An engine's or transmission's name by its definition, from those already read (else from the
/// file's name: `d16g750.sii` → `D16G750`).
pub fn part_name(cx: &App, path: &str) -> String {
    let known = cx.try_global::<Parts>().and_then(|parts| {
        parts.0.values().flatten().find_map(|list| {
            let part = list.iter().find(|part| part.path == path)?;
            Some(data(cx)?.resolve(&part.name))
        })
    });
    known.filter(|name| !name.is_empty()).unwrap_or_else(|| {
        std::path::Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_uppercase())
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::TestAppContext;

    #[gpui_kit::test]
    fn another_game_s_installed_content_is_forgotten_as_soon_as_the_game_changes(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| {
            crate::settings::init(cx);
            // Euro Truck Simulator 2's data is read, with a DLC installed.
            cx.set_global(CurrentGame(Game::Ets2));
            let mut installed = gamedata::Installed::default();
            installed.dlcs.insert("eut2_north".to_string());
            cx.set_global(Loaded {
                data: None,
                installed: Some(Arc::new(installed)),
                is_loading: false,
            });
            assert!(installed_now(cx));

            // American Truck Simulator is picked, and its data isn't read yet: what ETS2 has
            // installed says nothing about an ATS save, so nothing is known for now.
            set_game(cx, Game::Ats);
            assert!(!installed_now(cx));
            assert!(is_loading(cx));

            // Back to ETS2: its own is shown again at once (it was kept only if its map was
            // read too; here it's read again).
            set_game(cx, Game::Ets2);
            assert_eq!(current_game(cx), Game::Ets2);
        });
    }

    fn installed_now(cx: &App) -> bool {
        installed(cx).is_some()
    }
}
