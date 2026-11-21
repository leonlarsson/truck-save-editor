//! The installed game's own data (level tables, rank names, city names, icons), loaded in the background
//! at startup. Until it arrives, or if the game isn't found, the app uses built-in fallbacks.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use gamedata::{GameData, GameIcon};
use gpui_kit::{App, AppContext as _, Context, Global, Subscription, Window};
use save::{Game, LevelTable};

#[derive(Default)]
struct Loaded(Option<Arc<GameData>>);

impl Global for Loaded {}

/// Starts reading the game's data in the background; windows redraw when it arrives.
pub fn init(cx: &mut App) {
    cx.set_global(Loaded::default());
    cx.spawn(async move |cx| {
        let data = cx
            .background_spawn(async move {
                let game = Game::Ets2;
                let dir = gamedata::find_install(game)?;
                // The game's text in English for now; this follows the app's language later.
                GameData::load(game, &dir, "en_us").ok()
            })
            .await;
        cx.update(|cx| {
            cx.set_global(Loaded(data.map(Arc::new)));
            cx.refresh_windows();
        });
    })
    .detach();
}

fn data(cx: &App) -> Option<Arc<GameData>> {
    cx.try_global::<Loaded>()
        .and_then(|loaded| loaded.0.clone())
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

/// A city's name: the game's own (`Malmö`), else made from its token (`Malmo`).
pub fn city_name(cx: &App, token: &str) -> String {
    data(cx)
        .and_then(|d| d.city_name(token))
        .unwrap_or_else(|| save::city_display_name(token))
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

/// Drivers' faces, loaded one by one as they're first shown (there are hundreds, and a page shows
/// a few dozen), each shrunk to a small copy so a big company doesn't cost much memory.
#[derive(Default)]
struct Faces {
    /// `base.scs`, opened once on first use, shared with the background loads.
    archive: Arc<Mutex<Option<hashfs::Archive>>>,
    /// By face number: the face, or `None` while loading or if the game has none.
    loaded: HashMap<u32, Option<Arc<GameIcon>>>,
}

impl Global for Faces {}

/// The pixel size faces are kept at: sharp at 2× for the table's 1.6rem.
const FACE_PIXELS: u32 = 64;

/// A hired driver's face (`driver.59` → the game's face 59). The first ask starts loading it in
/// the background and returns `None`; the window redraws when it's there.
pub fn driver_face(cx: &mut App, driver_id: &str) -> Option<Arc<GameIcon>> {
    let number: u32 = driver_id.strip_prefix("driver.")?.parse().ok()?;
    if !cx.has_global::<Faces>() {
        cx.set_global(Faces::default());
    }
    if let Some(face) = cx.global::<Faces>().loaded.get(&number) {
        return face.clone();
    }
    let install_dir = data(cx)?.install_dir.clone();
    let faces = cx.global_mut::<Faces>();
    faces.loaded.insert(number, None);
    let archive = faces.archive.clone();
    cx.spawn(async move |cx| {
        let face = cx
            .background_spawn(async move {
                // A poisoned lock only means an earlier load panicked; the archive is still fine.
                let mut archive = archive
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if archive.is_none() {
                    *archive = hashfs::Archive::open(install_dir.join("base.scs")).ok();
                }
                let archive = archive.as_mut()?;
                let name = number.to_string();
                GameIcon::load(archive, "/material/ui/driver", &name)
                    .map(|face| Arc::new(face.shrunk(FACE_PIXELS)))
            })
            .await;
        cx.update(|cx| {
            cx.global_mut::<Faces>().loaded.insert(number, face);
            cx.refresh_windows();
        });
    })
    .detach();
    None
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
