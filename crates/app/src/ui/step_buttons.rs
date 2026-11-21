//! A pair of buttons that take away or add one step ("−1M" / "+1M"), joined as one control so the
//! two directions always sit together.

use std::rc::Rc;

use gpui_kit::component::{
    Sizable as _,
    button::{Button, ButtonGroup},
};
use gpui_kit::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};

type OnStep = Rc<dyn Fn(&i64, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct StepButtons {
    id: ElementId,
    label: SharedString,
    on_step: Option<OnStep>,
}

impl StepButtons {
    /// `label` is the step without a sign: "1M", "1 level".
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_step: None,
        }
    }

    /// Called with -1 for the minus button and +1 for the plus button.
    pub fn on_step(mut self, handler: impl Fn(&i64, &mut Window, &mut App) + 'static) -> Self {
        self.on_step = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for StepButtons {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let on_step = self.on_step;
        ButtonGroup::new(self.id)
            .small()
            .child(
                Button::new("minus")
                    .outline()
                    .label(format!("−{}", self.label)),
            )
            .child(
                Button::new("plus")
                    .outline()
                    .label(format!("+{}", self.label)),
            )
            .on_click(move |clicked: &Vec<usize>, window, cx| {
                let (Some(handler), Some(&index)) = (on_step.as_ref(), clicked.first()) else {
                    return;
                };
                let direction = if index == 0 { -1 } else { 1 };
                handler(&direction, window, cx);
            })
    }
}
