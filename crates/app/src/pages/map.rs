//! Map: every city, how often the player has been there, and its truck dealer and recruitment
//! agency (if it has them) and whether they're unlocked. Visit every city or unlock every dealer
//! and agency in one go.

use std::collections::BTreeSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::Button,
    h_flex,
    table::{DataTable, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div,
};
use save::Place;

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{FilterBar, FilterChanged};
use crate::ui::{RowLook, TextColumn, TextRow, TextTable};

pub struct MapPage {
    session: Entity<Session>,
    table: Entity<TableState<TextTable>>,
    visited: usize,
    count: usize,
    dealers: PlaceCount,
    recruitments: PlaceCount,
    filter: Entity<FilterBar>,
    _subscriptions: Vec<Subscription>,
}

/// The Map filter's toggles, in order.
const FILTER_NOT_VISITED: usize = 0;
const FILTER_DEALER_LOCKED: usize = 1;
const FILTER_RECRUITMENT_LOCKED: usize = 2;

/// How many of a kind of place are unlocked, of how many there are (`None` without the game).
#[derive(Clone, Copy, Default)]
struct PlaceCount {
    unlocked: usize,
    total: Option<usize>,
}

impl PlaceCount {
    fn is_complete(self) -> bool {
        self.total.is_some_and(|total| self.unlocked >= total)
    }
}

impl MapPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let columns = vec![
            TextColumn {
                key: "city",
                title: t!(cx, "column-city"),
                width: 220.,
            },
            TextColumn {
                key: "visits",
                title: t!(cx, "column-visits"),
                width: 130.,
            },
            TextColumn {
                key: "dealer",
                title: t!(cx, "column-dealer"),
                width: 160.,
            },
            TextColumn {
                key: "recruitment",
                title: t!(cx, "column-recruitment"),
                width: 180.,
            },
        ];
        let table = cx.new(|cx| TableState::new(TextTable::new(columns), window, cx));
        let toggles = vec![
            t!(cx, "map-filter-not-visited"),
            t!(cx, "map-filter-dealer-locked"),
            t!(cx, "map-filter-recruitment-locked"),
        ];
        let placeholder = t!(cx, "map-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
        ];
        let mut page = Self {
            session,
            table,
            visited: 0,
            count: 0,
            dealers: PlaceCount::default(),
            recruitments: PlaceCount::default(),
            filter,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let unlocked_dealers = session.unlocked(Place::Dealer);
        let unlocked_recruitments = session.unlocked(Place::Recruitment);
        let dealer_cities = game_data::place_cities(cx, Place::Dealer);
        let recruitment_cities = game_data::place_cities(cx, Place::Recruitment);
        self.dealers = count(&unlocked_dealers, dealer_cities.as_ref());
        self.recruitments = count(&unlocked_recruitments, recruitment_cities.as_ref());

        let mut cities: Vec<(String, u32, SharedString, SharedString)> = session
            .cities()
            .into_iter()
            .map(|city| {
                let dealer = place_cell(&city.token, &unlocked_dealers, dealer_cities.as_ref(), cx);
                let recruitment = place_cell(
                    &city.token,
                    &unlocked_recruitments,
                    recruitment_cities.as_ref(),
                    cx,
                );
                (
                    game_data::city_name(cx, &city.token),
                    city.visits,
                    dealer,
                    recruitment,
                )
            })
            .collect();
        cities.sort_by(|a, b| a.0.cmp(&b.0));
        self.count = cities.len();
        self.visited = cities.iter().filter(|city| city.1 > 0).count();
        // "Not found yet" is what the cell says when the city has one that's still locked.
        let locked = t!(cx, "map-place-locked");
        let cities: Vec<_> = {
            let filter = self.filter.read(cx);
            cities
                .into_iter()
                .filter(|(name, visits, dealer, recruitment)| {
                    (!filter.is_on(FILTER_NOT_VISITED) || *visits == 0)
                        && (!filter.is_on(FILTER_DEALER_LOCKED) || *dealer == locked)
                        && (!filter.is_on(FILTER_RECRUITMENT_LOCKED) || *recruitment == locked)
                        && filter.matches(&[name], cx)
                })
                .collect()
        };
        let (shown, total) = (cities.len(), self.count);
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        let rows = cities
            .into_iter()
            .map(|(name, visits, dealer, recruitment)| TextRow {
                cells: vec![
                    name.into(),
                    t!(cx, "map-visits", count = visits),
                    dealer,
                    recruitment,
                ],
                look: if visits == 0 {
                    RowLook::Muted
                } else {
                    RowLook::Normal
                },
            })
            .collect();
        self.table.update(cx, |table, cx| {
            table.delegate_mut().rows = rows;
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    fn unlock(&mut self, place: Place, cx: &mut Context<Self>) {
        let Some(cities) = game_data::place_cities(cx, place) else {
            return;
        };
        self.session
            .update(cx, |session, cx| session.unlock_all(place, &cities, cx));
    }
}

/// Unlocked ones count only where the game says there's one (the save can hold leftovers from
/// older maps).
fn count(unlocked: &BTreeSet<String>, cities: Option<&BTreeSet<String>>) -> PlaceCount {
    match cities {
        Some(cities) => PlaceCount {
            unlocked: unlocked.intersection(cities).count(),
            total: Some(cities.len()),
        },
        None => PlaceCount {
            unlocked: unlocked.len(),
            total: None,
        },
    }
}

/// A city's dealer or agency: unlocked, not found yet, or nothing (the city has none, or without
/// the game, nobody knows).
fn place_cell(
    token: &str,
    unlocked: &BTreeSet<String>,
    cities: Option<&BTreeSet<String>>,
    cx: &App,
) -> SharedString {
    if unlocked.contains(token) {
        t!(cx, "map-place-unlocked")
    } else if cities.is_some_and(|cities| cities.contains(token)) {
        t!(cx, "map-place-locked")
    } else {
        SharedString::default()
    }
}

impl Render for MapPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_game_data = self.dealers.total.is_some() && self.recruitments.total.is_some();
        let mut summary = vec![t!(
            cx,
            "map-summary",
            visited = self.visited,
            count = self.count
        )];
        // Literal keys, so the check that every key has English text sees them.
        if let Some(total) = self.dealers.total {
            summary.push(t!(
                cx,
                "map-dealers-summary",
                unlocked = self.dealers.unlocked,
                count = total
            ));
        }
        if let Some(total) = self.recruitments.total {
            summary.push(t!(
                cx,
                "map-recruitments-summary",
                unlocked = self.recruitments.unlocked,
                count = total
            ));
        }
        // Each fact stays on one line; the line breaks only between them.
        let summary = summary
            .into_iter()
            .map(|part| div().whitespace_nowrap().child(part));

        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    // The summary wraps; the buttons keep their size.
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .flex_wrap()
                            .gap_x_4()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .children(summary),
                    )
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .gap_2()
                            .child(
                                Button::new("visit-all")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-visit-all"))
                                    .tooltip(t!(cx, "map-visit-all-tooltip"))
                                    .disabled(self.visited == self.count)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.session
                                            .update(cx, |session, cx| session.visit_all_cities(cx));
                                    })),
                            )
                            .child(
                                Button::new("unlock-dealers")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-unlock-dealers"))
                                    .tooltip(t!(cx, "map-unlock-dealers-tooltip"))
                                    .disabled(!has_game_data || self.dealers.is_complete())
                                    .on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.unlock(Place::Dealer, cx)
                                        }),
                                    ),
                            )
                            .child(
                                Button::new("unlock-recruitments")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-unlock-recruitments"))
                                    .tooltip(t!(cx, "map-unlock-recruitments-tooltip"))
                                    .disabled(!has_game_data || self.recruitments.is_complete())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.unlock(Place::Recruitment, cx)
                                    })),
                            ),
                    ),
            )
            .child(self.filter.clone())
            // Not hidden, and not just a disabled button: say why, and what makes it work.
            .children((!has_game_data).then(|| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "map-needs-game"))
            }))
            .child(
                div().flex_1().min_h_0().child(crate::ui::table_or_empty(
                    &self.filter,
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label(t!(cx, "map-table"))
                        .into_any_element(),
                    None,
                    cx,
                )),
            )
    }
}
