//! Detail panels: one truck, trailer, driver, garage or city, opened from its row (double-click,
//! Enter, or the row's Details button) or from the map, in a sheet at the side. Every edit goes
//! through the session like any other change: it shows on every page, counts in the header, and
//! Undo takes it back.

mod city;
mod driver;
mod garage;
mod garage_picker;
mod vehicle;

pub use city::{ShowOnMap, open as open_city};
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

/// A panel's closing row: its own action, then removing the panel's driver, truck, trailer or
/// garage (a quiet danger button), and below them, when it can't be removed, why. Removing closes
/// the panel (what it showed is gone); Undo brings it back.
pub fn actions_row(
    action: impl IntoElement,
    session: &gpui_kit::Entity<crate::session::Session>,
    removal: Option<(save::Removal, SharedString)>,
    cx: &App,
) -> impl IntoElement {
    let Some((removal, label)) = removal else {
        return v_flex().child(h_flex().child(action));
    };
    use gpui_kit::component::{
        Disableable as _, Sizable as _, WindowExt as _,
        button::{Button, ButtonVariants as _},
    };
    use save::{Removal, RemoveProblem};

    let problem = session.read(cx).can_remove(&removal).err();
    let why = problem.and_then(|problem| {
        Some(match (problem, &removal) {
            (RemoveProblem::NotFound, _) => return None,
            (RemoveProblem::IsPlayers, Removal::Trailer(_)) => {
                crate::i18n::t!(cx, "remove-why-your-trailer")
            }
            (RemoveProblem::IsPlayers, _) => crate::i18n::t!(cx, "remove-why-yours"),
            (RemoveProblem::HasDriver, _) => crate::i18n::t!(cx, "remove-why-has-driver"),
            (RemoveProblem::InUse, _) => crate::i18n::t!(cx, "remove-why-in-use"),
            (RemoveProblem::IsHeadquarters, _) => crate::i18n::t!(cx, "remove-why-headquarters"),
            (RemoveProblem::NotEmpty, _) => crate::i18n::t!(cx, "remove-why-not-empty"),
            (RemoveProblem::UnknownShape, _) => crate::i18n::t!(cx, "remove-why-unknown"),
        })
    });
    let session = session.clone();
    v_flex()
        .gap_2()
        .child(
            h_flex().gap_2().flex_wrap().child(action).child(
                Button::new("remove")
                    .small()
                    .outline()
                    .danger()
                    .label(label)
                    .tooltip(crate::i18n::t!(cx, "remove-tooltip"))
                    .disabled(problem.is_some())
                    .on_click(move |_, window, cx| {
                        let removal = removal.clone();
                        session.update(cx, |session, cx| session.remove(removal, cx));
                        window.close_sheet(cx);
                    }),
            ),
        )
        .children(why.map(|why| {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(why)
        }))
}
