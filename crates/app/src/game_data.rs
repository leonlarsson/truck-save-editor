//! The installed game's own data (level tables, rank names, city names, icons), loaded in the background
//! at startup. Until it arrives, or if the game isn't found, the app uses built-in fallbacks.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use gamedata::{GameData, GameIcon};
use gpui_kit::{App, AppContext as _, Context, Global, Subscription, Window};
use save::{Game, LevelTable};

#[derive(Default)]
struct Loaded {
    data: Option<Arc<GameData>>,
    /// The DLCs and mods installed (`None` without the game).
    installed: Option<Arc<gamedata::Installed>>,
    /// A load is under way (at startup, or after the language or the game's folder changed).
    is_loading: bool,
}

/// Every road and junction on the game's map (for the road map and Discover every road), read
/// after the rest of the game's data so it never holds that up. `None` until read, or without the
/// game.
#[derive(Default)]
struct MapRoads(Option<RoadsRead>);

/// The map's roads as read: with where they are (for the road map), and their ids alone (for
/// Discover every road, kept apart so it isn't rebuilt from the roads each time).
struct RoadsRead {
    roads: Arc<Vec<gamedata::MapRoad>>,
    ids: Arc<Vec<u64>>,
}

impl Global for MapRoads {}

/// The map's roads and junctions by id, once read (`None` before, or without the game).
pub fn road_ids(cx: &App) -> Option<Arc<Vec<u64>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.ids.clone()))
}

/// The map's roads and junctions with where they are, once read.
pub fn map_roads(cx: &App) -> Option<Arc<Vec<gamedata::MapRoad>>> {
    cx.try_global::<MapRoads>()
        .and_then(|read| read.0.as_ref().map(|read| read.roads.clone()))
}

impl Global for Loaded {}

/// Starts reading the game's data in the background; windows redraw when it arrives.
pub fn init(cx: &mut App) {
    cx.set_global(Loaded::default());
    reload(cx);
}

/// Where the game is: the folder picked in the settings, else the one Steam knows.
pub fn install_dir(cx: &App) -> Option<std::path::PathBuf> {
    crate::settings::get(cx)
        .game_dir
        .or_else(|| gamedata::find_install(Game::Ets2))
}

/// Reads the game's data again (after the language or the game's folder changed). What's shown
/// until it arrives is the data read before.
pub fn reload(cx: &mut App) {
    if !cx.has_global::<Loaded>() {
        cx.set_global(Loaded::default());
    }
    cx.global_mut::<Loaded>().is_loading = true;
    let dir = install_dir(cx);
    let roads_dir = dir.clone();
    let language = crate::settings::get(cx).language.game_code();
    cx.spawn(async move |cx| {
        let (data, installed) = cx
            .background_spawn(async move {
                let game = Game::Ets2;
                let Some(dir) = dir else {
                    return (None, None);
                };
                // The game's text in the app's language, else English (`load` falls back).
                let data = GameData::load(game, &dir, language).ok();
                let installed = data.is_some().then(|| gamedata::installed(game, &dir));
                (data, installed)
            })
            .await;
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
                let roads = gamedata::map_roads(Game::Ets2, roads_dir.as_deref()?);
                let ids: Vec<u64> = roads.iter().map(|road| road.id).collect();
                (!roads.is_empty()).then(|| RoadsRead {
                    roads: Arc::new(roads),
                    ids: Arc::new(ids),
                })
            })
            .await;
        cx.update(|cx| {
            cx.set_global(MapRoads(roads));
            cx.refresh_windows();
        });
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
/// when the game (or its city definitions) wasn't found: then nobody knows where they are.
pub fn place_cities(cx: &App, place: save::Place) -> Option<BTreeSet<String>> {
    let data = data(cx)?;
    let cities = match place {
        save::Place::Dealer => data.dealer_cities(),
        save::Place::Recruitment => data.recruitment_cities(),
    };
    (!cities.is_empty()).then(|| cities.clone())
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
