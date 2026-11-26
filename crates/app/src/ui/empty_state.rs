//! What a list shows when there's nothing in it: a short title, a line on why, and the action that
//! changes that (Clear filters), instead of an empty grid.

use gpui_kit::component::{ActiveTheme as _, StyledExt as _, v_flex};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _,
    Window, div,
};

#[derive(IntoElement)]
pub struct EmptyState {
    title: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
}

impl EmptyState {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            action: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .child(div().font_medium().child(self.title))
            .children(self.description.map(|description| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(description)
            }))
            .children(self.action.map(|action| div().pt_2().child(action)))
    }
}
