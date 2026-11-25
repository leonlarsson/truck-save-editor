//! A named amount with a bar: how full a tank is, how worn a part (Fuel · 612 l, with the bar
//! under it; or Engine, a bar, 3 %, on one line).

use gpui_kit::component::{ActiveTheme as _, h_flex, progress::Progress, v_flex};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window,
    div, rems,
};

#[derive(IntoElement)]
pub struct Meter {
    id: ElementId,
    label: SharedString,
    value: SharedString,
    /// How full the bar is, 0 to 1.
    share: f32,
    is_on_one_line: bool,
}

impl Meter {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        value: impl Into<SharedString>,
        share: f32,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            share: share.clamp(0.0, 1.0),
            is_on_one_line: false,
        }
    }

    /// The name, the bar and the amount on one line (for a list of them), not the bar beneath.
    pub fn on_one_line(mut self) -> Self {
        self.is_on_one_line = true;
        self
    }
}

impl RenderOnce for Meter {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let name = format!("{}: {}", self.label, self.value);
        let bar = Progress::new(self.id)
            .value(self.share * 100.0)
            .color(theme.foreground)
            .accessibility_label(name);
        let label = div().text_color(theme.muted_foreground).child(self.label);
        if self.is_on_one_line {
            h_flex()
                .gap_3()
                .items_center()
                .text_xs()
                .child(label.w(rems(5.5)).flex_shrink_0())
                .child(div().flex_1().child(bar))
                .child(
                    div()
                        .w(rems(2.5))
                        .flex_shrink_0()
                        .text_right()
                        .child(self.value),
                )
                .into_any_element()
        } else {
            v_flex()
                .gap_1p5()
                .text_xs()
                .child(
                    h_flex()
                        .justify_between()
                        .gap_3()
                        .child(label)
                        .child(self.value),
                )
                .child(bar)
                .into_any_element()
        }
    }
}
