//! A titled region of a page: a title, an optional line saying what it's for, optional actions on
//! the trailing edge, and its content. It has no box around it: regions are told apart by their
//! titles, by space, and by the hairlines `Divided` puts between them. See `Look` for the three
//! ways one sits on a page.

use gpui_kit::component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};

/// How a section sits on the page.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    /// Nothing around it: a title, and its content under it. What most sections are; a page
    /// tells them apart with space and hairlines (`Divided::sections`).
    Open,
    /// On a surface a shade off the page, with a hairline around it: for the one or two things a
    /// page leads with
    /// (Overview's money and level). Its title is a small label, since what it holds is the
    /// headline.
    Panel,
    /// The title and description on the leading side, the content beside them: for settings,
    /// where each section is a short explanation and a control or two. Narrow windows put the
    /// content under the title.
    Aside,
}

#[derive(IntoElement)]
pub struct Section {
    title: SharedString,
    description: Option<SharedString>,
    actions: Vec<AnyElement>,
    children: Vec<AnyElement>,
    look: Look,
    style: StyleRefinement,
}

impl Section {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            actions: Vec::new(),
            children: Vec::new(),
            look: Look::Open,
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

    /// On a filled surface: for what the page leads with (see `Look::Panel`).
    pub fn panel(mut self) -> Self {
        self.look = Look::Panel;
        self
    }

    /// Title beside the content: for settings (see `Look::Aside`).
    pub fn aside(mut self) -> Self {
        self.look = Look::Aside;
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
        let is_panel = self.look == Look::Panel;
        let heading = v_flex()
            .gap_1()
            .min_w_0()
            .child(if is_panel {
                // Quieter than an open section's title: the panel's content is the headline.
                div().text_sm().font_semibold().child(self.title)
            } else {
                div().text_base().font_semibold().child(self.title)
            })
            .when_some(self.description, |this, description| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(description),
                )
            });
        let actions = (!self.actions.is_empty())
            .then(|| h_flex().gap_2().flex_shrink_0().children(self.actions));
        let content = v_flex().gap_3().min_w_0().children(self.children);

        match self.look {
            Look::Aside => h_flex()
                .items_start()
                .flex_wrap()
                .gap_x_8()
                .gap_y_4()
                .refine_style(&self.style)
                .child(
                    v_flex()
                        .gap_3()
                        .w(rems(17.))
                        .flex_shrink_0()
                        .child(heading)
                        .children(actions),
                )
                .child(content.flex_1().min_w(rems(18.))),
            Look::Open | Look::Panel => v_flex()
                .gap_4()
                .when(is_panel, |this| {
                    // A fill only a shade off the page, held by a hairline: the fill alone read
                    // as a bright slab, the line alone as a wireframe.
                    this.p_5()
                        .bg(theme.group_box)
                        .border_1()
                        .border_color(theme.border)
                        .rounded(theme.radius_lg)
                })
                .refine_style(&self.style)
                .child(
                    h_flex()
                        .items_start()
                        .justify_between()
                        .gap_4()
                        .child(heading)
                        .children(actions),
                )
                .child(content),
        }
    }
}
