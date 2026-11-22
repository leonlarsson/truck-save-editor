//! The bar above a table that narrows it down: a search field, a few toggles for the questions
//! people ask of that table ("Needs repair", "Not visited"), and how many rows are shown of how
//! many. The page owns the rows; it asks the bar whether a row matches, and tells it the counts.
//!
//! Toggles combine (a row must match every one that's on), so each page offers toggles that can
//! all be on at once without ruling each other out.

use std::collections::BTreeSet;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
};
use gpui_kit::{
    AppContext as _, Context, Entity, EventEmitter, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div, rems,
};

use crate::format::thousands;
use crate::i18n::t;

/// The search or a toggle changed: the page filters its rows again.
pub struct FilterChanged;

pub struct FilterBar {
    search: Entity<InputState>,
    toggles: Vec<SharedString>,
    active: BTreeSet<usize>,
    shown: usize,
    total: usize,
    _subscription: Subscription,
}

impl EventEmitter<FilterChanged> for FilterBar {}

impl FilterBar {
    /// `placeholder` says what the search looks in ("Search by model, plate or garage");
    /// `toggles` are the toggle labels, in order.
    pub fn new(
        placeholder: SharedString,
        toggles: Vec<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        let _subscription = cx.subscribe_in(&search, window, |_, _, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                cx.emit(FilterChanged);
                cx.notify();
            }
        });
        Self {
            search,
            toggles,
            active: BTreeSet::new(),
            shown: 0,
            total: 0,
            _subscription,
        }
    }

    /// Whether toggle `index` is on.
    pub fn is_on(&self, index: usize) -> bool {
        self.active.contains(&index)
    }

    /// Whether a row matches the search: every word typed appears in one of the row's texts,
    /// ignoring case and accents (`malmo` finds Malmö).
    pub fn matches(&self, texts: &[&str], cx: &gpui_kit::App) -> bool {
        let query = fold(&self.search.read(cx).value());
        let haystack: Vec<String> = texts.iter().map(|text| fold(text)).collect();
        query
            .split_whitespace()
            .all(|word| haystack.iter().any(|text| text.contains(word)))
    }

    /// How many rows matched the last time the page filtered.
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// How many rows matched, of how many there are.
    pub fn set_counts(&mut self, shown: usize, total: usize, cx: &mut Context<Self>) {
        if (self.shown, self.total) != (shown, total) {
            self.shown = shown;
            self.total = total;
            cx.notify();
        }
    }

    /// Whether a search or a toggle is narrowing the list.
    pub fn is_filtering(&self, cx: &gpui_kit::App) -> bool {
        !self.active.is_empty() || !self.search.read(cx).value().trim().is_empty()
    }

    fn toggle(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.active.remove(&index) {
            self.active.insert(index);
        }
        cx.emit(FilterChanged);
        cx.notify();
    }

    /// Clears the search and every toggle.
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active.clear();
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        cx.emit(FilterChanged);
        cx.notify();
    }
}

impl Render for FilterBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toggles = self.toggles.iter().enumerate().map(|(index, label)| {
            let is_on = self.is_on(index);
            let button = Button::new(("filter-toggle", index))
                .small()
                .outline()
                .label(label.clone())
                .selected(is_on);
            // A check mark too, so "on" doesn't rest on a shade of grey alone.
            let button = if is_on {
                button.icon(Icon::new(IconName::Check).small())
            } else {
                button
            };
            button.on_click(cx.listener(move |this, _, _, cx| this.toggle(index, cx)))
        });
        let is_filtering = self.is_filtering(cx);
        let count = if is_filtering {
            t!(
                cx,
                "filter-count",
                shown = thousands(self.shown as i64),
                total = thousands(self.total as i64)
            )
        } else {
            SharedString::default()
        };

        h_flex()
            .gap_2()
            .flex_wrap()
            .items_center()
            .child(
                div().w(rems(20.)).child(
                    Input::new(&self.search)
                        .id("filter-search")
                        .small()
                        .cleanable(true)
                        .prefix(Icon::new(IconName::Search).small()),
                ),
            )
            .children(toggles)
            // Only while filtering: an empty element would still take a line when the bar wraps.
            .children(is_filtering.then(|| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(count)
            }))
            .children(is_filtering.then(|| {
                Button::new("clear-filters")
                    .small()
                    .ghost()
                    .label(t!(cx, "filter-clear"))
                    .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx)))
            }))
    }
}

/// Lowercase without accents, for matching what people type (`Örebro` → `orebro`).
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ą' | 'ă' => 'a',
            'ç' | 'ć' | 'č' => 'c',
            'ď' | 'đ' => 'd',
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
            'ł' | 'ľ' | 'ĺ' => 'l',
            'ñ' | 'ń' | 'ň' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => 'o',
            'ř' => 'r',
            'ś' | 'š' | 'ş' | 'ș' => 's',
            'ť' | 'ţ' | 'ț' => 't',
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' => 'u',
            'ý' | 'ÿ' => 'y',
            'ź' | 'ż' | 'ž' => 'z',
            other => other,
        })
        .collect()
}

/// The table, or what to say instead when the filters leave nothing: the empty state, with Clear
/// filters. `empty` is what to show when there's nothing even without filters (no drivers yet).
pub fn table_or_empty(
    filter: &gpui_kit::Entity<FilterBar>,
    table: gpui_kit::AnyElement,
    empty: Option<crate::ui::EmptyState>,
    cx: &gpui_kit::App,
) -> gpui_kit::AnyElement {
    let bar = filter.read(cx);
    if bar.shown() > 0 {
        return table;
    }
    if bar.is_filtering(cx) {
        let filter = filter.clone();
        return crate::ui::EmptyState::new(t!(cx, "filter-no-matches"))
            .description(t!(cx, "filter-no-matches-body"))
            .action(
                Button::new("empty-clear-filters")
                    .small()
                    .outline()
                    .label(t!(cx, "filter-clear"))
                    .on_click(move |_, window, cx| {
                        filter.update(cx, |filter, cx| filter.clear(window, cx));
                    }),
            )
            .into_any_element();
    }
    match empty {
        Some(empty) => empty.into_any_element(),
        None => table,
    }
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_case_and_accents() {
        assert_eq!(fold("Örebro"), "orebro");
        assert_eq!(fold("Łódź"), "lodz");
        assert_eq!(fold("A Coruña"), "a coruna");
    }
}
