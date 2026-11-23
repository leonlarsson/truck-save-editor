//! The colours saved in the game's paint screen, in a side panel. The game keeps 8, each a set of
//! up to four colours (it saves a paint job's colours together; a single colour uses the first),
//! so each row is one saved colour with its four colour pickers. The lot copies as text to share,
//! or pastes from text someone shared. Changes go through the session like any other, and show as
//! quick picks in every vehicle's paint colours.

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
    Styled as _, Subscription, Window, div, px, rems,
};

use crate::i18n::t;
use crate::session::Session;
use crate::ui::{to_hsla, to_rgb8};

pub struct SavedColours {
    session: Entity<Session>,
    /// One picker per colour: saved colour by saved colour, four each.
    pickers: Vec<[Entity<ColorPickerState>; save::COLOURS_PER_SAVED]>,
    _subscriptions: Vec<Subscription>,
}

impl SavedColours {
    fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let colours = session.read(cx).saved_colours();
        let pickers: Vec<[Entity<ColorPickerState>; save::COLOURS_PER_SAVED]> = colours
            .iter()
            .map(|saved| {
                std::array::from_fn(|at| {
                    let colour = saved[at];
                    cx.new(|cx| {
                        let state = ColorPickerState::new(window, cx);
                        match colour {
                            Some(colour) => state.default_value(to_hsla(colour)),
                            None => state,
                        }
                    })
                })
            })
            .collect();
        let mut _subscriptions = vec![cx.observe_in(&session, window, |this, _, window, cx| {
            this.sync(window, cx)
        })];
        for (saved, row) in pickers.iter().enumerate() {
            for (at, state) in row.iter().enumerate() {
                _subscriptions.push(cx.subscribe(
                    state,
                    move |this, _, event: &ColorPickerEvent, cx| {
                        let ColorPickerEvent::Change(colour) = event;
                        let mut colours = this.session.read(cx).saved_colours();
                        if let Some(set) = colours.get_mut(saved) {
                            set[at] = colour.map(to_rgb8);
                        }
                        this.session
                            .update(cx, |session, cx| session.set_saved_colours(colours, cx));
                    },
                ));
            }
        }
        Self {
            session,
            pickers,
            _subscriptions,
        }
    }

    /// Shows the colours as they'll be after saving (Undo, Paste, Discard change them).
    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let colours = self.session.read(cx).saved_colours();
        for (row, saved) in self.pickers.iter().zip(colours) {
            for (state, colour) in row.iter().zip(saved) {
                let shown = state.read(cx).value().map(to_rgb8);
                if shown == colour {
                    continue;
                }
                state.update(cx, |state, cx| match colour {
                    Some(colour) => state.set_value(to_hsla(colour), window, cx),
                    None => state.clear_value(window, cx),
                });
            }
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

    /// Takes the colours from text on the clipboard, from the first saved colour on (the ones
    /// after them are left as they are).
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
        for (at, saved) in pasted.into_iter().enumerate() {
            if let Some(entry) = colours.get_mut(at) {
                *entry = saved;
            }
        }
        self.session
            .update(cx, |session, cx| session.set_saved_colours(colours, cx));
    }
}

impl Render for SavedColours {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<_> =
            self.pickers
                .iter()
                .enumerate()
                .map(|(saved, row)| {
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .w(rems(7.))
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(t!(cx, "saved-colours-slot", slot = saved + 1)),
                        )
                        .child(h_flex().gap_2().children(row.iter().enumerate().map(
                            |(at, state)| {
                                ColorPicker::new(state).small().accessibility_label(t!(
                                    cx,
                                    "saved-colours-colour",
                                    slot = saved + 1,
                                    colour = at + 1
                                ))
                            },
                        )))
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
