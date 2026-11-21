//! A titled panel on a page: a title, an optional line saying what it's for, optional actions on
//! the trailing edge, and its content below.

use gpui_kit::component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _,
};

#[derive(IntoElement)]
pub struct Section {
    title: SharedString,
    description: Option<SharedString>,
    actions: Vec<AnyElement>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Section {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            actions: Vec::new(),
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// A command shown at the trailing edge of the title row.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl ParentElement for Section {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Lets a page size the section (stretch it, give it a width); its look stays the kit's.
impl Styled for Section {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Section {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .gap_4()
            .p_5()
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_lg)
            .refine_style(&self.style)
            .child(
                h_flex()
                    .items_start()
                    .justify_between()
                    .gap_4()
                    .child(
                        v_flex()
                            .gap_1()
                            .min_w_0()
                            .child(div().text_base().font_semibold().child(self.title))
                            .when_some(self.description, |this, description| {
                                this.child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.muted_foreground)
                                        .child(description),
                                )
                            }),
                    )
                    .when(!self.actions.is_empty(), |this| {
                        this.child(h_flex().gap_2().flex_shrink_0().children(self.actions))
                    }),
            )
            .child(v_flex().gap_3().children(self.children))
    }
}
