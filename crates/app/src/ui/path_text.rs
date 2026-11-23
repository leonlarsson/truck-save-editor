//! A folder's path as the settings pages show it: in the code font, quiet, wrapping where it must.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Div, ParentElement as _, Styled as _, div};

/// The path, or nothing when there's none (a folder not found).
pub fn path_text(path: Option<&std::path::Path>, cx: &App) -> Div {
    div()
        .min_w_0()
        .text_sm()
        .font_family(cx.theme().mono_font_family.clone())
        .text_color(cx.theme().muted_foreground)
        .child(
            path.map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )
}
