//! The colours saved in the game's paint screen (32 slots), in a side panel: each a colour picker,
//! and the lot copied as text to share, or pasted from text someone shared. Changes go through
//! the session like any other, and show as quick picks in every vehicle's paint colours.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    h_flex,
    notification::{Notification, NotificationType},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, ClipboardItem, Context, Entity, IntoElement, ParentElement as _, Render,
    Styled as _, Subscription, Window, div, px,
};

use crate::i18n::t;
use crate::session::Session;
use crate::ui::{to_hsla, to_rgb8};

pub struct SavedColours {
    session: Entity<Session>,
    slots: Vec<Entity<ColorPickerState>>,
    _subscriptions: Vec<Subscription>,
}

impl SavedColours {
    fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let colours = session.read(cx).saved_colours();
        let slots: Vec<Entity<ColorPickerState>> = colours
            .iter()
            .map(|colour| {
                cx.new(|cx| {
                    let state = ColorPickerState::new(window, cx);
                    match colour {
                        Some(colour) => state.default_value(to_hsla(*colour)),
                        None => state,
                    }
                })
            })
            .collect();
        let mut _subscriptions = vec![cx.observe_in(&session, window, |this, _, window, cx| {
            this.sync(window, cx)
        })];
        for (slot, state) in slots.iter().enumerate() {
            _subscriptions.push(cx.subscribe(
                state,
                move |this, _, event: &ColorPickerEvent, cx| {
                    let ColorPickerEvent::Change(colour) = event;
                    let mut colours = this.session.read(cx).saved_colours();
                    if let Some(entry) = colours.get_mut(slot) {
                        *entry = colour.map(to_rgb8);
                    }
                    this.session
                        .update(cx, |session, cx| session.set_saved_colours(colours, cx));
                },
            ));
        }
        Self {
            session,
            slots,
            _subscriptions,
        }
    }

    /// Shows the colours as they'll be after saving (Undo, Paste, Discard change them).
    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let colours = self.session.read(cx).saved_colours();
        for (state, colour) in self.slots.iter().zip(colours) {
            let shown = state.read(cx).value().map(to_rgb8);
            if shown == colour {
                continue;
            }
            state.update(cx, |state, cx| match colour {
                Some(colour) => state.set_value(to_hsla(colour), window, cx),
                None => state.clear_value(window, cx),
            });
        }
        cx.notify();
    }

    fn copy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = save::colours_to_text(&self.session.read(cx).saved_colours());
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Success)
                .title(t!(cx, "saved-colours-copied")),
            cx,
        );
    }

    /// Takes the colours from text on the clipboard, slot by slot (the slots after them are left
    /// as they are).
    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pasted = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .and_then(|text| save::colours_from_text(&text));
        let Some(pasted) = pasted else {
            window.push_notification(
                Notification::new()
                    .with_type(NotificationType::Error)
                    .title(t!(cx, "saved-colours-not-colours")),
                cx,
            );
            return;
        };
        let mut colours = self.session.read(cx).saved_colours();
        for (slot, colour) in pasted.into_iter().enumerate() {
            if let Some(entry) = colours.get_mut(slot) {
                *entry = colour;
            }
        }
        self.session
            .update(cx, |session, cx| session.set_saved_colours(colours, cx));
    }
}

impl Render for SavedColours {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Eight to a row, as the game's paint screen lays them out.
        let rows: Vec<_> = self
            .slots
            .chunks(8)
            .enumerate()
            .map(|(row, slots)| {
                h_flex()
                    .gap_2()
                    .children(slots.iter().enumerate().map(|(at, state)| {
                        ColorPicker::new(state).small().accessibility_label(t!(
                            cx,
                            "saved-colours-slot",
                            slot = row * 8 + at + 1
                        ))
                    }))
            })
            .collect();
        v_flex()
            .pt_2()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "saved-colours-description")),
            )
            .child(v_flex().gap_2().children(rows))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("copy-colours")
                            .small()
                            .outline()
                            .label(t!(cx, "saved-colours-copy"))
                            .tooltip(t!(cx, "saved-colours-copy-tooltip"))
                            .on_click(cx.listener(|this, _, window, cx| this.copy(window, cx))),
                    )
                    .child(
                        Button::new("paste-colours")
                            .small()
                            .ghost()
                            .label(t!(cx, "saved-colours-paste"))
                            .tooltip(t!(cx, "saved-colours-paste-tooltip"))
                            .on_click(cx.listener(|this, _, window, cx| this.paste(window, cx))),
                    ),
            )
    }
}

/// Opens the saved colours in a side panel.
pub fn open(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let view = cx.new(|cx| SavedColours::new(session, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "saved-colours"))
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}
