//! Each game's own icon (the one Steam shows), stored in the repository: taken once from the
//! game's program (`eurotrucks2.exe`, `amtrucks.exe`, their 256-pixel icon). Shown beside the open
//! save's name, so it's clear at a glance which game a save belongs to.

use std::sync::{Arc, OnceLock};

use gpui_kit::{Image, ImageFormat};
use save::Game;

const ETS2: &[u8] = include_bytes!("../assets/games/ets2.png");
const ATS: &[u8] = include_bytes!("../assets/games/ats.png");

/// A game's icon, ready to draw (`img(…)`). Made once per game and kept: the window draws it on
/// every frame.
pub fn icon(game: Game) -> Arc<Image> {
    static ETS2_ICON: OnceLock<Arc<Image>> = OnceLock::new();
    static ATS_ICON: OnceLock<Arc<Image>> = OnceLock::new();
    let (cell, bytes) = match game {
        Game::Ets2 => (&ETS2_ICON, ETS2),
        Game::Ats => (&ATS_ICON, ATS),
    };
    cell.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec())))
        .clone()
}
