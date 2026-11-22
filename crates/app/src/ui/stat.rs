//! One number with its name, for a row of them at the top of a page (12,480 deliveries).

use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div,
};

#[derive(IntoElement)]
pub struct Stat {
    label: SharedString,
    value: SharedString,
}

impl Stat {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

impl RenderOnce for Stat {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .gap_1()
            .min_w(gpui_kit::rems(9.))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(self.label),
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(self.value),
            )
    }
}
