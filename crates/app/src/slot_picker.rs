//! "Which save?": a dialog listing a profile's saves to pick one from, the player's own first,
//! then the game's (which it overwrites as it goes), each with what tells them apart: its slot,
//! its money, when it was saved. Save over another and Duplicate (on Steam Cloud) use it.

use std::rc::Rc;

use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, px, rems,
};

use crate::i18n::t;
use crate::labels;

/// What to do with the picked save: it and its name as the list shows it.
pub type OnPick = Rc<dyn Fn(save::Slot, SharedString, &mut Window, &mut App)>;

/// A slot with its place in the list it came from (for its button's id).
type NumberedSlot<'a> = (usize, &'a save::Slot);

/// Opens the dialog. `slots` are the saves to choose from, `body` says what happens to the one
/// picked; picking closes the dialog and calls `on_pick`.
pub fn pick(
    slots: Vec<save::Slot>,
    currency: &'static str,
    title: SharedString,
    body: SharedString,
    on_pick: OnPick,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let (own, game_made): (Vec<NumberedSlot>, Vec<NumberedSlot>) = slots
            .iter()
            .enumerate()
            .partition(|(_, slot)| slot.kind.is_manual());
        let muted = cx.theme().muted_foreground;
        let row = |index: usize, slot: &save::Slot, cx: &mut App| {
            let name = labels::slot_name(cx, slot);
            let mut detail: Vec<String> = Vec::new();
            if let Some(money) = slot.info.money {
                detail.push(format!("{currency} {}", crate::format::thousands(money)));
            }
            if let Some(modified) = slot.modified {
                detail.push(labels::ago(cx, modified).to_string());
            }
            let number = match slot.kind {
                save::SlotKind::Manual(number) => {
                    Some(t!(cx, "start-slot-number", number = number))
                }
                _ => None,
            };
            let slot = slot.clone();
            let on_pick = on_pick.clone();
            Button::new(("pick-slot", index))
                .ghost()
                .w_full()
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .gap_4()
                        .child(
                            h_flex()
                                .min_w_0()
                                .gap_2()
                                .child(div().truncate().child(name.clone()))
                                .children(number.map(|number| {
                                    div().flex_shrink_0().text_color(muted).child(number)
                                })),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_color(muted)
                                .child(detail.join(" · ")),
                        ),
                )
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    on_pick(slot.clone(), name.clone(), window, cx);
                })
                .into_any_element()
        };
        let heading = |title: SharedString| {
            div()
                .px_3()
                .pt_2()
                .text_xs()
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_color(muted)
                .child(title)
                .into_any_element()
        };
        let mut rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if !own.is_empty() {
            rows.push(heading(t!(cx, "start-your-saves")));
            for (index, slot) in own {
                rows.push(row(index, slot, cx));
            }
        }
        if !game_made.is_empty() {
            rows.push(heading(t!(cx, "start-game-saves")));
            for (index, slot) in game_made {
                rows.push(row(index, slot, cx));
            }
        }
        alert
            .title(title.clone())
            .description(
                v_flex()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(body.clone()),
                    )
                    .child(
                        v_flex()
                            .id("pick-slots")
                            .max_h(rems(18.))
                            .overflow_y_scroll()
                            .gap_1()
                            .children(rows),
                    ),
            )
            .width(px(560.))
            .footer(
                h_flex().w_full().justify_end().child(
                    Button::new("cancel-pick-slot")
                        .outline()
                        .label(t!(cx, "cancel"))
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                ),
            )
    });
}

/// A profile's saves other than `open`, newest first: the ones a save can be written over.
pub fn other_slots(profile_dir: &std::path::Path, open: &std::path::Path) -> Vec<save::Slot> {
    let mut slots: Vec<save::Slot> = save::profile_slots(profile_dir)
        .into_iter()
        .filter(|slot| slot.dir != open)
        .collect();
    slots.sort_by_key(|slot| std::cmp::Reverse(slot.modified));
    slots
}
