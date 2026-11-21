//! The installed game's own data (level tables, rank names, city names, icons), loaded in the background
//! at startup. Until it arrives, or if the game isn't found, the app uses built-in fallbacks.

use std::sync::Arc;

use gamedata::{GameData, GameIcon};
use gpui_kit::{App, AppContext as _, Global};
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
