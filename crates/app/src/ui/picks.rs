//! Picking several rows of a table to act on them together: a checkbox at the start of each row,
//! one in the heading for every row shown, and a bar above the table while anything is picked,
//! with what can be done to all of them at once.
//!
//! The picks are ids (not row numbers), so they survive sorting, filtering and changes; a pick
//! whose row the filter hides stays picked, and the bar counts it.

use std::collections::BTreeSet;

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    select::SelectItem,
};
use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString,
    Styled as _, Window, div,
};

use crate::i18n::t;

/// The ids picked in one table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picks(BTreeSet<String>);

impl Picks {
    pub fn contains(&self, id: &str) -> bool {
        self.0.contains(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Picks it, or leaves it out if it was picked.
    pub fn toggle(&mut self, id: &str) {
        if !self.0.remove(id) {
            self.0.insert(id.to_string());
        }
    }

    /// Every one of `shown`, or none of them if they're all picked already.
    pub fn toggle_all<'a>(&mut self, shown: impl IntoIterator<Item = &'a str> + Clone) {
        if self.all_of(shown.clone()) {
            for id in shown {
                self.0.remove(id);
            }
        } else {
            self.0.extend(shown.into_iter().map(str::to_string));
        }
    }

    /// Whether every one of `shown` is picked (and there's at least one).
    pub fn all_of<'a>(&self, shown: impl IntoIterator<Item = &'a str>) -> bool {
        let mut any = false;
        for id in shown {
            any = true;
            if !self.0.contains(id) {
                return false;
            }
        }
        any
    }

    /// Only the picks that still exist (after a removal, say).
    pub fn keep_only<'a>(&mut self, existing: impl IntoIterator<Item = &'a str>) {
        let existing: BTreeSet<&str> = existing.into_iter().collect();
        self.0.retain(|id| existing.contains(id.as_str()));
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn ids(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}

/// A row's checkbox, named for screen readers by what the row is.
pub fn pick_box(id: impl Into<ElementId>, is_picked: bool, label: SharedString) -> Checkbox {
    Checkbox::new(id)
        .checked(is_picked)
        .accessibility_label(label)
}

/// What Clear does (unpicks every row).
type OnClear = Box<dyn Fn(&mut Window, &mut App)>;

/// The bar above a table while rows are picked: how many, what can be done to them, and Clear.
#[derive(IntoElement)]
pub struct PickBar {
    count: usize,
    actions: Vec<AnyElement>,
    on_clear: OnClear,
}

impl PickBar {
    pub fn new(count: usize, on_clear: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        Self {
            count,
            actions: Vec::new(),
            on_clear: Box::new(on_clear),
        }
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for PickBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_clear = self.on_clear;
        h_flex()
            .gap_2()
            .flex_wrap()
            .items_center()
            .px_3()
            .min_h(gpui_kit::rems(1.75))
            .rounded(cx.theme().radius)
            .bg(cx.theme().secondary)
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .mr_2()
                    .child(t!(cx, "picks-count", count = self.count)),
            )
            .children(self.actions)
            .child(div().flex_1())
            .child(
                Button::new("clear-picks")
                    .xsmall()
                    .ghost()
                    .label(t!(cx, "picks-clear"))
                    .on_click(move |_, window, cx| on_clear(window, cx)),
            )
    }
}

/// An item in a list to choose from (a garage to move picked rows to): shown by its name, chosen
/// by its value (an id).
#[derive(Clone)]
pub struct Choice {
    pub value: SharedString,
    pub name: SharedString,
}

impl SelectItem for Choice {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// The company's own garages, by city name, to move picked trucks or drivers to.
pub fn owned_garages(session: &crate::session::Session, cx: &App) -> Vec<Choice> {
    let mut garages: Vec<Choice> = session
        .garages()
        .into_iter()
        .filter(|garage| garage.is_owned)
        .map(|garage| Choice {
            name: crate::game_data::city_name(cx, &garage.city).into(),
            value: garage.id.into(),
        })
        .collect();
    garages.sort_by(|a, b| a.name.cmp(&b.name));
    garages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_toggle_and_follow_the_rows_that_exist() {
        let mut picks = Picks::default();
        picks.toggle("a");
        picks.toggle("b");
        picks.toggle("a");
        assert_eq!(picks.ids(), ["b"]);
        let shown = ["a", "b", "c"];
        picks.toggle_all(shown.iter().copied());
        assert!(picks.all_of(shown.iter().copied()));
        picks.toggle_all(shown.iter().copied());
        assert!(picks.is_empty());
        picks.toggle_all(shown.iter().copied());
        picks.keep_only(["a", "c"].iter().copied());
        assert_eq!(picks.ids(), ["a", "c"]);
    }
}
