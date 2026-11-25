//! The frame every page shares: a header (title, a line of context, actions) and a scrolling body
//! with the same insets, so pages line up with each other.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _, h_flex, tag::Tag, v_flex,
};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};

#[derive(IntoElement)]
pub struct PageHeader {
    title: SharedString,
    /// A word beside the title about the page as a whole (Experimental).
    tag: Option<SharedString>,
    subtitle: Option<SharedString>,
    actions: Vec<AnyElement>,
}

impl PageHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            tag: None,
            subtitle: None,
            actions: Vec::new(),
        }
    }

    pub fn tag(mut self, tag: impl Into<SharedString>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for PageHeader {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        h_flex()
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .gap_4()
            .px_8()
            .py_4()
            .border_b_1()
            .border_color(theme.border)
            .child(
                v_flex()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(div().text_xl().font_semibold().child(self.title))
                            .when_some(self.tag, |this, tag| {
                                this.child(Tag::secondary().outline().small().child(tag))
                            }),
                    )
                    .when_some(self.subtitle, |this, subtitle| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(subtitle),
                        )
                    }),
            )
            .child(h_flex().gap_2().items_center().children(self.actions))
    }
}

/// The page's content area, inset to line up with the header.
pub fn page_body() -> gpui_kit::Div {
    div().px_8().py_6()
}
