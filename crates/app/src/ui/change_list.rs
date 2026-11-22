//! The changes waiting to be saved, as the save dialog lists them: grouped under what they belong
//! to (the career, a truck, a driver, the map …), each one a row of what changed and its value
//! before and after; a removal is one line.

use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div, rems,
};

use crate::session::Change;

#[derive(IntoElement)]
pub struct ChangeList {
    changes: Vec<Change>,
}

impl ChangeList {
    pub fn new(changes: Vec<Change>) -> Self {
        Self { changes }
    }
}

/// The changes by what they belong to, in the order each first comes up.
fn grouped(changes: Vec<Change>) -> Vec<(SharedString, Vec<Change>)> {
    let mut groups: Vec<(SharedString, Vec<Change>)> = Vec::new();
    for change in changes {
        match groups
            .iter_mut()
            .find(|(subject, _)| *subject == change.subject)
        {
            Some((_, group)) => group.push(change),
            None => groups.push((change.subject.clone(), vec![change])),
        }
    }
    groups
}

impl RenderOnce for ChangeList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        // Nothing to list (changes that undo each other): say so rather than show an empty frame.
        if self.changes.is_empty() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(crate::i18n::t!(cx, "changes-none"))
                .into_any_element();
        }
        let muted = cx.theme().muted_foreground;
        let foreground = cx.theme().foreground;
        let danger = cx.theme().danger;
        // Framed, apart from the dialog's own words around it, with every colour set here: a
        // dialog's description is muted throughout, and the new values should stand out.
        v_flex()
            .gap_4()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .children(grouped(self.changes).into_iter().map(|(subject, changes)| {
                v_flex()
                    .gap_1()
                    .text_sm()
                    .min_w_0()
                    .child(
                        div()
                            .font_weight(gpui_kit::FontWeight::MEDIUM)
                            .text_color(foreground)
                            .child(subject),
                    )
                    .children(changes.into_iter().map(|change| {
                        if change.is_removal() {
                            return div()
                                .text_color(danger)
                                .child(change.after)
                                .into_any_element();
                        }
                        // What changed in a column of its own, so the values line up; the values
                        // wrap rather than leave the dialog (`min_w_0`).
                        h_flex()
                            .items_start()
                            .gap_3()
                            .min_w_0()
                            .child(
                                div()
                                    .w(rems(9.))
                                    .flex_shrink_0()
                                    .text_color(muted)
                                    .child(change.what),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .flex_wrap()
                                    .gap_x_1p5()
                                    .min_w_0()
                                    .child(div().min_w_0().text_color(muted).child(change.before))
                                    .child(div().text_color(muted).child("→"))
                                    .child(
                                        div().min_w_0().text_color(foreground).child(change.after),
                                    ),
                            )
                            .into_any_element()
                    }))
            }))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(subject: &str, what: &str) -> Change {
        Change {
            subject: subject.to_string().into(),
            what: what.to_string().into(),
            before: "a".into(),
            after: "b".into(),
        }
    }

    #[test]
    fn groups_by_subject_in_first_seen_order() {
        let groups = grouped(vec![
            change("Truck", "Plate"),
            change("Career", "Money"),
            change("Truck", "Engine"),
        ]);
        let shape: Vec<(String, usize)> = groups
            .iter()
            .map(|(subject, changes)| (subject.to_string(), changes.len()))
            .collect();
        assert_eq!(shape, [("Truck".to_string(), 2), ("Career".to_string(), 1)]);
    }
}
