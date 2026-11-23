//! Map: every city, how often the player has been there, and its truck dealer and recruitment
//! agency (if it has them) and whether they're unlocked, as a list or on the map. Visit every city
//! or unlock every dealer and agency in one go, or one city at a time in its panel.

use std::collections::BTreeSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonGroup},
    h_flex,
    table::{DataTable, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _,
};
use save::Place;

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{FilterBar, FilterChanged};
use crate::ui::{RowLook, TextColumn, TextRow, TextTable};
use crate::workspace::OpenDetails;

pub struct MapPage {
    session: Entity<Session>,
    table: Entity<TableState<TextTable>>,
    /// The cities' tokens, row by row as the table shows them, to open a row's panel.
    tokens: Vec<String>,
    visited: usize,
    count: usize,
    dealers: PlaceCount,
    recruitments: PlaceCount,
    filter: Entity<FilterBar>,
    /// Moving the player's truck to another save's spot.
    position: Entity<crate::pages::position::PositionPicker>,
    /// The roads as a picture, shown instead of the cities when chosen.
    map_view: Entity<crate::pages::map_view::MapView>,
    shows_map: bool,
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
            cx.subscribe_in(&table, window, |this, _, event: &TableEvent, window, cx| {
                if let TableEvent::DoubleClickedRow(row) = event {
                    this.open_city(*row, window, cx);
                }
            }),
        ];
        let position = {
            let session = session.clone();
            cx.new(|cx| crate::pages::position::PositionPicker::new(session, window, cx))
        };
        let map_view = {
            let session = session.clone();
            cx.new(|cx| crate::pages::map_view::MapView::new(session, cx))
        };
        let mut page = Self {
            map_view,
            shows_map: false,
            session,
            table,
            tokens: Vec::new(),
            visited: 0,
            count: 0,
            dealers: PlaceCount::default(),
            recruitments: PlaceCount::default(),
            filter,
            position,
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

        let mut cities: Vec<(String, u32, SharedString, SharedString, String)> = session
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
                    city.token,
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
                .filter(|(name, visits, dealer, recruitment, _)| {
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
        self.tokens = cities.iter().map(|city| city.4.clone()).collect();
        let rows = cities
            .into_iter()
            .map(|(name, visits, dealer, recruitment, _)| TextRow {
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

    /// Opens the panel of the city on this row.
    fn open_city(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(token) = self.tokens.get(row) {
            crate::pages::details::open_city(self.session.clone(), token, window, cx);
        }
    }

    fn unlock(&mut self, place: Place, cx: &mut Context<Self>) {
        let Some(cities) = game_data::place_cities(cx, place) else {
            return;
        };
        self.session
            .update(cx, |session, cx| session.unlock_all(place, &cities, cx));
    }
}

/// Asks before marking every road as driven: it may count toward the game's road achievements, and
/// nobody knows for sure whether it does.
fn confirm_discover_roads(
    session: Entity<Session>,
    ids: std::sync::Arc<Vec<u64>>,
    window: &mut Window,
    cx: &mut App,
) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let session = session.clone();
        let ids = ids.clone();
        alert
            .title(t!(cx, "map-discover-roads-confirm-title"))
            .description(t!(cx, "map-discover-roads-confirm-body"))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "map-discover-roads"))
            .on_ok(move |_, _, cx| {
                let ids = ids.clone();
                session.update(cx, |session, cx| session.discover_roads(ids, cx));
                true
            })
    });
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
                    // On a narrow window the buttons move to a line of their own.
                    .flex_wrap()
                    .gap_y_2()
                    // The summary wraps; the buttons keep their size.
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w(gpui_kit::rems(18.))
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
                                Button::new("forget-roads")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-forget-roads"))
                                    .tooltip(t!(cx, "map-forget-roads-tooltip"))
                                    .disabled(self.session.read(cx).discovered_roads().1 == 0)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.session
                                            .update(cx, |session, cx| session.forget_roads(cx));
                                    })),
                            )
                            .child({
                                // Needs the game's map, read in the background after startup.
                                let roads = game_data::road_ids(cx);
                                let (_, after) = self.session.read(cx).discovered_roads();
                                let is_all = roads.as_ref().is_some_and(|ids| after >= ids.len());
                                Button::new("discover-roads")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-discover-roads"))
                                    .tooltip(if roads.is_some() {
                                        t!(cx, "map-discover-roads-tooltip")
                                    } else {
                                        t!(cx, "map-discover-roads-needs-game")
                                    })
                                    .disabled(roads.is_none() || is_all)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if let Some(ids) = roads.clone() {
                                            confirm_discover_roads(
                                                this.session.clone(),
                                                ids,
                                                window,
                                                cx,
                                            );
                                        }
                                    }))
                            })
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
            .child({
                // Where the player's truck stands, and moving it to another save's spot.
                let place = save::position(&self.session.read(cx).preview())
                    .and_then(|position| position.city)
                    .map(|city| game_data::city_name(cx, &city))
                    .unwrap_or_else(|| t!(cx, "player-position-somewhere").to_string());
                // A column, so a note under the buttons doesn't move them.
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        div()
                            .text_sm()
                            .child(t!(cx, "player-position-now", place = place)),
                    )
                    .child(self.position.clone())
            })
            // The cities as a list, or the roads as a map.
            .child(
                ButtonGroup::new("map-view")
                    .outline()
                    .small()
                    .child(
                        Button::new("view-cities")
                            .label(t!(cx, "map-view-list"))
                            .selected(!self.shows_map),
                    )
                    .child(
                        Button::new("view-roads")
                            .label(t!(cx, "map-view-map"))
                            .selected(self.shows_map),
                    )
                    .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                        this.shows_map = clicked.first() == Some(&1);
                        cx.notify();
                    })),
            )
            .map(|this| {
                if self.shows_map {
                    return this.child(div().flex_1().min_h_0().child(self.map_view.clone()));
                }
                this.child(self.filter.clone())
                    // Not hidden, and not just a disabled button: say why, and what makes it work.
                    .children((!has_game_data).then(|| {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "map-needs-game"))
                    }))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            // Enter on the selected row opens it, like a double-click.
                            .on_action(cx.listener(|this, _: &OpenDetails, window, cx| {
                                if let Some(row) = this.table.read(cx).selected_row() {
                                    this.open_city(row, window, cx);
                                }
                            }))
                            .child(crate::ui::table_or_empty(
                                &self.filter,
                                crate::ui::fill_table(
                                    &self.table,
                                    DataTable::new(&self.table)
                                        .stripe(true)
                                        .bordered(true)
                                        .accessibility_label(t!(cx, "map-table"))
                                        .into_any_element(),
                                ),
                                None,
                                cx,
                            )),
                    )
            })
    }
}
