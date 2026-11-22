//! Truck Save Editor: the desktop app. It finds the game's saves, opens one into a `Session`, and
//! shows it in the `Workspace`: the pages, the sidebar and the header with the pending changes.

// A Windows app, not a console program: release builds open without a terminal window. Debug
// builds keep theirs, for log output while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod format;
mod game_data;
mod i18n;
mod labels;
mod pages;
mod palette;
mod quick_start;
mod report;
mod session;
mod settings;
#[cfg(test)]
mod tests;
mod theme;
mod ui;
mod updates;
mod workspace;

use gpui_kit::{AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use crate::workspace::Workspace;

/// The product name, from `build.rs` (which also puts it in the Windows .exe's details). Never
/// hard-code it anywhere else.
pub const APP_NAME: &str = env!("APP_NAME");

/// Every keyboard shortcut, in one place, and the app-wide handlers for the ones that work
/// everywhere (Undo, Redo).
pub fn bind_keys(cx: &mut gpui_kit::App) {
    use gpui_kit::KeyBinding;
    workspace::init(cx);
    // No context: anywhere in the window, panels and dialogs included.
    let workspace = None;
    cx.bind_keys([
        KeyBinding::new("ctrl-k", palette::OpenPalette, None),
        KeyBinding::new("cmd-k", palette::OpenPalette, None),
        KeyBinding::new("ctrl-s", workspace::SaveChanges, None),
        KeyBinding::new("cmd-s", workspace::SaveChanges, None),
        KeyBinding::new("ctrl-z", workspace::Undo, workspace),
        KeyBinding::new("cmd-z", workspace::Undo, workspace),
        KeyBinding::new("ctrl-shift-z", workspace::Redo, workspace),
        KeyBinding::new("ctrl-y", workspace::Redo, workspace),
        KeyBinding::new("cmd-shift-z", workspace::Redo, workspace),
        // Enter on a table's selected row opens its detail panel.
        KeyBinding::new("enter", workspace::OpenDetails, Some("DataTable")),
        KeyBinding::new("ctrl-1", workspace::GoToPage1, None),
        KeyBinding::new("ctrl-2", workspace::GoToPage2, None),
        KeyBinding::new("ctrl-3", workspace::GoToPage3, None),
        KeyBinding::new("ctrl-4", workspace::GoToPage4, None),
        KeyBinding::new("ctrl-5", workspace::GoToPage5, None),
        KeyBinding::new("ctrl-6", workspace::GoToPage6, None),
        KeyBinding::new("ctrl-7", workspace::GoToPage7, None),
        KeyBinding::new("ctrl-8", workspace::GoToPage8, None),
        KeyBinding::new("ctrl-9", workspace::GoToPage9, None),
    ]);
}

/// Development builds only: `TSE_WINDOW=880x560` opens the window at that size, to check layouts
/// at the smallest size the window allows.
fn dev_window_size() -> Option<gpui_kit::Size<gpui_kit::Pixels>> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let text = std::env::var("TSE_WINDOW").ok()?;
    let (width, height) = text.split_once('x')?;
    Some(size(px(width.parse().ok()?), px(height.parse().ok()?)))
}

fn main() {
    gpui_kit::application()
        .with_assets(assets::AppAssets)
        .run(|cx| {
            // GPUI Kit first, then our theme on top of it.
            gpui_kit::init(cx);
            settings::init(cx);
            i18n::init(cx);
            updates::init(cx);
            theme::init(cx);
            game_data::init(cx);
            bind_keys(cx);

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    dev_window_size().unwrap_or(size(px(1200.), px(800.))),
                    cx,
                ))),
                window_min_size: Some(size(px(880.), px(560.))),
                titlebar: Some(TitlebarOptions {
                    title: Some(APP_NAME.into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            // `open_window` wraps our view in GPUI Kit's `Root`, which draws dialogs,
            // notifications and tooltips above it.
            let opened = gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            });
            if let Err(error) = opened {
                eprintln!("{APP_NAME} could not open its window: {error}");
                cx.quit();
            }
        });
}
