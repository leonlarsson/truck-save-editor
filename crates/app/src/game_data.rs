//! The installed game's own data (level tables, rank names, city names), loaded in the background
//! at startup. Until it arrives, or if the game isn't found, the app uses built-in fallbacks.

use std::sync::Arc;

use gamedata::GameData;
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
