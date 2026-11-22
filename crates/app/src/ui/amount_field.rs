//! A field for a whole-number amount (money, experience points).
//!
//! While you type it holds plain digits, so editing never fights a formatter; when you leave it (or
//! press Enter, or click elsewhere) it reads what you typed (`2.5m`, `+1m`, `-500k`, see `format::parse_amount`) and
//! shows the result formatted (`2,480,350`). Something that isn't an amount puts the old value back,
//! and so does Esc.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    input::{self, Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable as _,
    InteractiveElement as _, IntoElement, MouseDownEvent, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, prelude::FluentBuilder as _,
};

use crate::format::{parse_amount, thousands};

/// Emitted when a new amount is confirmed.
pub struct AmountChanged(pub i64);

pub struct AmountField {
    /// Small, to sit in a row of other small fields (detail panels); large otherwise.
    is_compact: bool,
    /// The input's element id (`amount-input` unless named), so a panel with several fields can
    /// tell them apart (tests find fields by it).
    element_id: SharedString,
    /// The field's own focus, outside the text box: where focus goes when it leaves editing, so it
    /// stays inside whatever holds the field (a panel still hears Esc to close).
    focus_handle: FocusHandle,
    input: Entity<InputState>,
    value: i64,
    min: Option<i64>,
    label: SharedString,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    hint: Option<SharedString>,
    /// Set while Esc is leaving the field, so leaving puts the old amount back instead of using
    /// what was typed.
    is_cancelling: bool,
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
                    if !std::mem::take(&mut this.is_cancelling) {
                        this.commit(window, cx);
                    }
                    this.show_formatted(window, cx);
                }
                InputEvent::Change => {}
            });
        Self {
            input,
            focus_handle: cx.focus_handle(),
            is_compact: false,
            element_id: "amount-input".into(),
            value,
            min: None,
            label: label.into(),
            prefix: None,
            suffix: None,
            hint: None,
            is_cancelling: false,
            _subscriptions: vec![subscription],
        }
    }

    /// The small size, matching the other fields in a detail panel.
    pub fn compact(mut self) -> Self {
        self.is_compact = true;
        self
    }

    /// Names the input element (see `element_id`).
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.element_id = id.into();
        self
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

    /// The amount, as last confirmed.
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Sets the amount from elsewhere (a button, Discard). Doesn't emit `AmountChanged`.
    pub fn set_value(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.value = value;
        if self.is_editing(window, cx) {
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

    fn is_editing(&self, window: &Window, cx: &Context<Self>) -> bool {
        self.input.read(cx).focus_handle(cx).is_focused(window)
    }

    /// Esc: leave the field and put the amount back as it was.
    fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_editing(window, cx) {
            self.is_cancelling = true;
            self.focus_handle.focus(window, cx);
        }
    }

    /// A click anywhere else: leave the field, keeping what was typed. GPUI doesn't take focus
    /// away from a field when empty space is clicked, so the field does it itself.
    fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_editing(window, cx) {
            self.focus_handle.focus(window, cx);
        }
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
            .track_focus(&self.focus_handle)
            // While editing, Esc is the field's: it leaves the field, and doesn't also close the
            // panel or dialog the field is in (the text box passes Esc on when it has nothing of
            // its own to cancel). The next Esc does that. Caught before the text box sees it.
            .capture_action(cx.listener(|this, _: &input::Escape, window, cx| {
                if this.is_editing(window, cx) {
                    this.cancel(window, cx);
                    cx.stop_propagation();
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| {
                this.leave(window, cx);
            }))
            .child(
                Input::new(&self.input)
                    .id(self.element_id.clone())
                    .when(self.is_compact, |input| input.small())
                    .when(!self.is_compact, |input| input.large())
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

#[cfg(test)]
mod tests {
    use gpui_kit::test::{TestSupportExt as _, TestWindowExt as _};
    use gpui_kit::{
        AnyWindowHandle, App, AppContext as _, Bounds, Context, Entity, InteractiveElement as _,
        IntoElement, ParentElement as _, Render, Styled as _, TestAppContext, Window, WindowBounds,
        WindowOptions, div, px, size,
    };

    use super::{AmountChanged, AmountField};

    /// A window with the field and some empty space beside it to click on.
    struct Host {
        field: Entity<AmountField>,
        confirmed: Vec<i64>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .p_4()
                .child(div().w(px(300.)).child(self.field.clone()))
                .child(div().id("outside").test_support().size(px(100.)))
        }
    }

    fn open(cx: &mut TestAppContext, value: i64) -> (AnyWindowHandle, Entity<Host>) {
        cx.update(gpui_kit::init);
        cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                        Default::default(),
                        size(px(640.), px(480.)),
                    ))),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        let field = cx.new(|cx| AmountField::new(value, "Amount", window, cx));
                        cx.subscribe(&field, |this: &mut Host, _, AmountChanged(value), _| {
                            this.confirmed.push(*value);
                        })
                        .detach();
                        Host {
                            field,
                            confirmed: Vec::new(),
                        }
                    })
                },
            )
        })
        .unwrap_or_else(|error| panic!("couldn't open the test window: {error}"))
    }

    /// Runs one user action in the window. Each gets its own update, like real input, so the
    /// events it causes are delivered before the next action.
    fn act(
        cx: &mut TestAppContext,
        handle: AnyWindowHandle,
        action: impl FnOnce(&mut Window, &mut App),
    ) {
        cx.update_window(handle, |_, window, cx| {
            action(window, cx);
            window.render_frame(cx);
        })
        .unwrap_or_else(|error| panic!("{error}"));
        cx.run_until_parked();
    }

    fn field_text(
        cx: &mut TestAppContext,
        handle: AnyWindowHandle,
    ) -> (Option<String>, Option<bool>) {
        let mut result = (None, None);
        act(cx, handle, |window, _| {
            let field = window.find("amount-input");
            result = (field.value().map(str::to_string), field.focused());
        });
        result
    }

    #[gpui_kit::test]
    fn clicking_elsewhere_confirms_and_leaves(cx: &mut TestAppContext) {
        let (handle, host) = open(cx, 2_480_350);
        // A headless window starts inactive, and an inactive window gets no focus events.
        act(cx, handle, |window, cx| {
            window.activate_window();
            window.render_frame(cx);
        });
        assert_eq!(
            field_text(cx, handle),
            (Some("2,480,350".into()), Some(false))
        );

        act(cx, handle, |window, cx| window.click("amount-input", cx));
        // Editing shows plain digits, all selected, so typing replaces them.
        assert_eq!(field_text(cx, handle), (Some("2480350".into()), Some(true)));

        act(cx, handle, |window, cx| window.input("2.5m", cx));
        act(cx, handle, |window, cx| window.click("outside", cx));
        assert_eq!(
            field_text(cx, handle),
            (Some("2,500,000".into()), Some(false))
        );
        cx.update(|cx| assert_eq!(host.read(cx).confirmed, vec![2_500_000]));
    }

    #[gpui_kit::test]
    fn escape_puts_the_old_amount_back_and_leaves(cx: &mut TestAppContext) {
        let (handle, host) = open(cx, 1_000);
        act(cx, handle, |window, cx| {
            window.activate_window();
            window.render_frame(cx);
        });
        act(cx, handle, |window, cx| window.click("amount-input", cx));
        act(cx, handle, |window, cx| window.input("999", cx));
        act(cx, handle, |window, cx| window.press("escape", cx));
        assert_eq!(field_text(cx, handle), (Some("1,000".into()), Some(false)));
        cx.update(|cx| assert!(host.read(cx).confirmed.is_empty()));
    }

    #[gpui_kit::test]
    fn enter_confirms_and_keeps_editing(cx: &mut TestAppContext) {
        let (handle, host) = open(cx, 1_000);
        act(cx, handle, |window, cx| {
            window.activate_window();
            window.render_frame(cx);
        });
        act(cx, handle, |window, cx| window.click("amount-input", cx));
        act(cx, handle, |window, cx| window.input("+1k", cx));
        act(cx, handle, |window, cx| window.press("enter", cx));
        assert_eq!(field_text(cx, handle), (Some("2000".into()), Some(true)));
        cx.update(|cx| assert_eq!(host.read(cx).confirmed, vec![2_000]));
    }

    #[gpui_kit::test]
    fn something_that_isnt_an_amount_is_ignored(cx: &mut TestAppContext) {
        let (handle, host) = open(cx, 1_000);
        act(cx, handle, |window, cx| {
            window.activate_window();
            window.render_frame(cx);
        });
        act(cx, handle, |window, cx| window.click("amount-input", cx));
        act(cx, handle, |window, cx| window.input("lots", cx));
        act(cx, handle, |window, cx| window.click("outside", cx));
        assert_eq!(field_text(cx, handle), (Some("1,000".into()), Some(false)));
        cx.update(|cx| assert!(host.read(cx).confirmed.is_empty()));
    }
}
