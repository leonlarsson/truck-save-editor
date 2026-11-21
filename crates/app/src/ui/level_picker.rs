//! Picks a level from 0 to a maximum, drawn as a meter: one segment per level, filled up to the
//! current one. The level reads at a glance and every level is one click away. Clicking the
//! current top segment again lowers the level by one, which is how a skill goes back to 0.

use std::rc::Rc;

use crate::i18n::t;

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
};
use gpui_kit::{App, ElementId, IntoElement, RenderOnce, SharedString, Styled as _, Window, rems};

type OnChange = Rc<dyn Fn(&u8, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct LevelPicker {
    id: ElementId,
    value: u8,
    max: u8,
    label: SharedString,
    on_change: Option<OnChange>,
}

impl LevelPicker {
    /// `label` names what the level is for, for screen readers ("Long distance").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: 0,
            max: 6,
            label: label.into(),
            on_change: None,
        }
    }

    pub fn value(mut self, value: u8) -> Self {
        self.value = value;
        self
    }

    pub fn max(mut self, max: u8) -> Self {
        self.max = max;
        self
    }

    /// Asks for a new level. The owner stores it; the picker shows whatever `value` it's given.
    pub fn on_change(mut self, handler: impl Fn(&u8, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for LevelPicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = self.value;
        let max = self.max;
        let segments = (1..=max).map(|level| {
            let button = Button::new(("level", usize::from(level)))
                .label(level.to_string())
                // Every segment the same width, so meters line up from row to row.
                .w(rems(1.75))
                .accessibility_label(t!(
                    cx,
                    "skill-level-of",
                    skill = &self.label,
                    level = level,
                    max = max
                ));
            if level <= value {
                button.primary()
            } else {
                button.outline()
            }
        });

        let on_change = self.on_change;
        ButtonGroup::new(self.id)
            .small()
            .children(segments)
            .on_click(move |clicked: &Vec<usize>, window, cx| {
                let (Some(handler), Some(&index)) = (on_change.as_ref(), clicked.first()) else {
                    return;
                };
                // Segment `index` is level `index + 1`.
                let Ok(clicked_level) = u8::try_from(index + 1) else {
                    return;
                };
                let level = if clicked_level == value {
                    clicked_level - 1
                } else {
                    clicked_level
                };
                handler(&level, window, cx);
            })
    }
}
