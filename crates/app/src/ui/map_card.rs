//! A card laid over a map: a panel with the page's background, a border and a soft shadow, that
//! takes the pointer for itself (the map under it isn't dragged or zoomed through it).

use gpui_kit::component::{ActiveTheme as _, StyledExt as _, v_flex};
use gpui_kit::{
    AnyElement, App, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window,
};

#[derive(IntoElement, Default)]
pub struct MapCard {
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl MapCard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for MapCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Lets a page size and lay out the card (a width, a row of items); its look stays the kit's.
impl Styled for MapCard {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for MapCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .gap_2()
            .px_4()
            .py_3()
            // A little see-through, so the map reads as going on beneath.
            .bg(theme.background.opacity(0.94))
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_lg)
            .shadow_md()
            .occlude()
            .refine_style(&self.style)
            .children(self.children)
    }
}
