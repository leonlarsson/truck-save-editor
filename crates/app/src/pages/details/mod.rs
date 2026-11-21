//! Detail panels: one truck, trailer, driver or garage, opened from its row (double-click, or the
//! row's Details button), in a sheet at the side. Every edit goes through the session like any
//! other change: it shows on every page, counts in the header, and Undo takes it back.

mod driver;
mod garage;
mod garage_picker;
mod vehicle;

pub use driver::open as open_driver;
pub use garage::open as open_garage;
pub use vehicle::open as open_vehicle;

use gpui_kit::component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::{App, IntoElement, ParentElement as _, SharedString, Styled as _, div, rems};

/// The width every detail panel opens at.
pub const SHEET_WIDTH: f32 = 440.;

/// A labelled fact: the label on the left, what it is on the right.
pub fn fact(label: SharedString, value: impl IntoElement, cx: &App) -> impl IntoElement {
    h_flex()
        .gap_4()
        .justify_between()
        .items_center()
        .min_h(rems(2.))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(div().text_sm().child(value))
}

/// A titled group of facts or controls.
pub fn group(title: SharedString, body: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap_2()
        .child(div().text_sm().font_medium().child(title))
        .child(body)
}
