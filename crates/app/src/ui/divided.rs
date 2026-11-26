//! Things one under another with a hairline between each two: the sections of a page, or the rows
//! of a list. The line sits between neighbours only, never above the first or under the last, so
//! a region is told apart by lines and space rather than by a box around it.

use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, div,
    px,
};

/// How far apart the things are: what they are to each other.
#[derive(Clone, Copy)]
enum Rhythm {
    /// A page's sections.
    Sections,
    /// A side panel's groups.
    Groups,
    /// One section's rows.
    Rows,
}

#[derive(IntoElement)]
pub struct Divided {
    rhythm: Rhythm,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Divided {
    /// A page's sections: wide apart.
    pub fn sections() -> Self {
        Self {
            rhythm: Rhythm::Sections,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// A side panel's groups: nearer than a page's sections, since a panel is narrow.
    pub fn groups() -> Self {
        Self {
            rhythm: Rhythm::Groups,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// A section's rows: close together.
    pub fn rows() -> Self {
        Self {
            rhythm: Rhythm::Rows,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }
}

impl ParentElement for Divided {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Lets a page size it (a width to keep to); its look stays the kit's.
impl Styled for Divided {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Divided {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui_kit::component::StyledExt as _;
        let line = cx.theme().border;
        let mut stack = match self.rhythm {
            Rhythm::Sections => v_flex().gap_8(),
            Rhythm::Groups => v_flex().gap_5(),
            Rhythm::Rows => v_flex().gap_3(),
        }
        .refine_style(&self.style);
        for (index, child) in self.children.into_iter().enumerate() {
            if index > 0 {
                // One device pixel: a hairline is a physical boundary, not a spacing step.
                stack = stack.child(div().flex_shrink_0().h(px(1.)).bg(line));
            }
            stack = stack.child(child);
        }
        stack
    }
}
