//! A field for a whole-number amount (money, experience points).
//!
//! While you type it holds plain digits, so editing never fights a formatter; when you leave it (or
//! press Enter) it reads what you typed (`2.5m`, `+1m`, `-500k`, see `format::parse_amount`) and
//! shows the result formatted (`2,480,350`). Something that isn't an amount puts the old value back.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, EventEmitter, Focusable as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _,
};

use crate::format::{parse_amount, thousands};

/// Emitted when a new amount is confirmed.
pub struct AmountChanged(pub i64);

pub struct AmountField {
    input: Entity<InputState>,
    value: i64,
    min: Option<i64>,
    label: SharedString,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    hint: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<AmountChanged> for AmountField {}

impl AmountField {
    /// `label` names the field for screen readers ("Bank balance").
    pub fn new(
        value: i64,
        label: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).default_value(thousands(value)));
        let subscription =
            cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
                InputEvent::Focus => this.start_editing(window, cx),
                InputEvent::PressEnter { .. } => {
                    this.commit(window, cx);
                    this.start_editing(window, cx);
                }
                InputEvent::Blur => {
                    this.commit(window, cx);
                    this.show_formatted(window, cx);
                }
                InputEvent::Change => {}
            });
        Self {
            input,
            value,
            min: None,
            label: label.into(),
            prefix: None,
            suffix: None,
            hint: None,
            _subscriptions: vec![subscription],
        }
    }

    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    /// Shown before the number, like a currency sign.
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Shown after the number, like a unit.
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// A line under the field saying what can be typed.
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Sets the amount from elsewhere (a button, Discard). Doesn't emit `AmountChanged`.
    pub fn set_value(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.value = value;
        let is_editing = self.input.read(cx).focus_handle(cx).is_focused(window);
        if is_editing {
            self.start_editing(window, cx);
        } else {
            self.show_formatted(window, cx);
        }
    }

    /// Plain digits, all selected, so typing replaces the amount and arrows edit it.
    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let raw = self.value.to_string();
        self.input.update(cx, |state, cx| {
            state.set_value(raw, window, cx);
            state.select_all(window, cx);
        });
    }

    fn show_formatted(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let formatted = thousands(self.value);
        self.input
            .update(cx, |state, cx| state.set_value(formatted, window, cx));
    }

    fn commit(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value();
        let Some(mut value) = parse_amount(&text, self.value) else {
            return;
        };
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if value != self.value {
            self.value = value;
            cx.emit(AmountChanged(value));
            cx.notify();
        }
    }
}

impl Render for AmountField {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        v_flex()
            .gap_1p5()
            .child(
                Input::new(&self.input)
                    .large()
                    .aria_label(self.label.clone())
                    .when_some(self.prefix.clone(), |input, prefix| {
                        input.prefix(div().text_color(muted).child(prefix))
                    })
                    .when_some(self.suffix.clone(), |input, suffix| {
                        input.suffix(div().text_color(muted).child(suffix))
                    }),
            )
            .when_some(self.hint.clone(), |this, hint| {
                this.child(div().text_xs().text_color(muted).child(hint))
            })
    }
}
