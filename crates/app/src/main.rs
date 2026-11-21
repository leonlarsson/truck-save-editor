//! Truck Save Editor: the desktop app. It finds the game's saves, opens one into a `Session`, and
//! shows it in the `Workspace`: the pages, the sidebar and the header with the pending changes.

mod assets;
mod format;
mod game_data;
mod i18n;
mod labels;
mod pages;
mod palette;
mod quick_start;
mod session;
#[cfg(test)]
mod tests;
mod theme;
mod ui;
mod workspace;

use gpui_kit::{AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use crate::workspace::Workspace;

/// The product name. Never hard-code it anywhere else.
pub const APP_NAME: &str = "Truck Save Editor";

/// Every keyboard shortcut, in one place.
pub fn bind_keys(cx: &mut gpui_kit::App) {
    use gpui_kit::KeyBinding;
    let workspace = Some(workspace::KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("ctrl-k", palette::OpenPalette, None),
        KeyBinding::new("cmd-k", palette::OpenPalette, None),
        KeyBinding::new("ctrl-z", workspace::Undo, workspace),
        KeyBinding::new("cmd-z", workspace::Undo, workspace),
        KeyBinding::new("ctrl-shift-z", workspace::Redo, workspace),
        KeyBinding::new("ctrl-y", workspace::Redo, workspace),
        KeyBinding::new("cmd-shift-z", workspace::Redo, workspace),
    ]);
}

fn main() {
    gpui_kit::application()
        .with_assets(assets::AppAssets)
        .run(|cx| {
            // GPUI Kit first, then our theme on top of it.
            gpui_kit::init(cx);
            i18n::init(cx);
            theme::init(cx);
            game_data::init(cx);
            bind_keys(cx);

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
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
