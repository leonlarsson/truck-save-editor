//! Cities: every city, how often the player has been there, and its truck dealer and recruitment
//! agency (if it has them) and whether they're unlocked. Visit every city or unlock every dealer
//! and agency in one go, or one city at a time in its panel (whose Show on map opens World map).

use std::collections::BTreeSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::Button,
    h_flex,
    table::{ColumnSort, DataTable, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div,
};
use save::Place;

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::Tip as _;
use crate::ui::{FilterBar, FilterChanged};
use crate::ui::{RowLook, TextColumn, TextRow, TextTable};
use crate::workspace::OpenDetails;

pub struct CitiesPage {
    session: Entity<Session>,
    table: Entity<TableState<TextTable>>,
    /// The cities' tokens, row by row as the table shows them, to open a row's panel.
    tokens: Vec<String>,
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

impl CitiesPage {
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
        // Each row's Details button opens its city, as double-click and Enter do.
        let page = cx.entity().downgrade();
        let details = t!(cx, "details-open");
        let table = cx.new(|cx| {
            let rows = TextTable::new(columns)
                .with_picks()
                .with_sorting()
                .with_details(details, move |row, window, cx| {
                    let _ = page.update(cx, |page, cx| page.open_city(row, window, cx));
                });
            TableState::new(rows, window, cx)
        });
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
            // ATS's recruitment agencies are found on its map, read after the rest.
            game_data::observe_map_in(window, cx, |this, _, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            // Picking a row shows the bar of what can be done to the picked cities.
            cx.observe(&table, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &table,
                window,
                |this, table, event: &TableEvent, window, cx| {
                    match event {
                        TableEvent::DoubleClickedRow(row) => this.open_city(*row, window, cx),
                        // A click on a column's heading sorts by it (the table itself would select
                        // the column, which nothing here uses).
                        TableEvent::SelectColumn(col_ix) => {
                            let col_ix = *col_ix;
                            table.update(cx, |table, cx| {
                                table.delegate_mut().sort_by_header(col_ix);
                                table.clear_selection(cx);
                            });
                            this.reload(cx);
                        }
                        _ => {}
                    }
                },
            ),
        ];
        let mut page = Self {
            session,
            table,
            tokens: Vec::new(),
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
        let dealer_cities = session.place_cities(Place::Dealer, cx);
        let recruitment_cities = session.place_cities(Place::Recruitment, cx);
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
        // By name; then by the column whose heading was clicked, if one was (cities that are
        // equal there stay in name order).
        cities.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some((column, order)) = self.table.read(cx).delegate().sort {
            cities.sort_by(|a, b| {
                let ordering = match column {
                    0 => a.0.cmp(&b.0),
                    // Visits as a number, not as the text the cell shows.
                    1 => a.1.cmp(&b.1),
                    2 => a.2.cmp(&b.2),
                    _ => a.3.cmp(&b.3),
                };
                if order == ColumnSort::Descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
        }
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
        // A row is picked by its city, so picks follow the cities through search and filters.
        let pick_ids: Vec<Option<String>> = self.tokens.iter().cloned().map(Some).collect();
        self.table.update(cx, |table, cx| {
            table.delegate_mut().pick_ids = pick_ids;
            table.delegate_mut().rows = rows;
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    /// Ticks the first rows' checkboxes (for `TSE_SCRIPT`'s `pick` step, to look at the bar).
    pub(crate) fn pick_first(&mut self, count: usize, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            let ids: Vec<String> = delegate
                .pick_ids
                .iter()
                .flatten()
                .take(count)
                .cloned()
                .collect();
            for id in &ids {
                if !delegate.picks.contains(id) {
                    delegate.picks.toggle(id);
                }
            }
            cx.notify();
        });
    }

    /// The bar above the table while cities are picked: what can be done to all of them. Each
    /// is one change (one step for Undo), however many cities it touches.
    fn render_pick_bar(&self, cx: &mut Context<Self>) -> Option<crate::ui::PickBar> {
        let picks = self.table.read(cx).delegate().picks.clone();
        if picks.is_empty() {
            return None;
        }
        let tokens = picks.ids();
        let table = self.table.clone();
        let mut bar = crate::ui::PickBar::new(picks.len(), move |_, cx| {
            table.update(cx, |table, cx| {
                table.delegate_mut().picks.clear();
                cx.notify();
            });
        });

        // One button: does its work on the picked cities, then says how many it changed.
        let action = |id: &'static str,
                      label: SharedString,
                      tip: SharedString,
                      cities: Vec<String>,
                      work: fn(&mut Session, &[String], &mut Context<Session>) -> usize,
                      cx: &mut Context<Self>| {
            Button::new(id)
                .small()
                .outline()
                .label(label)
                .tip(tip)
                .on_click(cx.listener(move |this, _, window, cx| {
                    let changed = this
                        .session
                        .update(cx, |session, cx| work(session, &cities, cx));
                    crate::ui::picks_changed(changed, window, cx);
                }))
        };
        bar = bar
            .action(action(
                "picks-visit",
                t!(cx, "city-visit"),
                t!(cx, "city-visit-tooltip"),
                tokens.clone(),
                |session, cities, cx| session.visit_cities_many(cities, cx),
                cx,
            ))
            .action(action(
                "picks-unvisit",
                t!(cx, "city-unvisit"),
                t!(cx, "city-unvisit-tooltip"),
                tokens.clone(),
                |session, cities, cx| session.unvisit_cities_many(cities, cx),
                cx,
            ));

        // Dealers and agencies: only when the game's data says where they are, and only for the
        // picked cities that have one.
        let session = self.session.read(cx);
        let with_place = |place: Place| -> Option<Vec<String>> {
            let cities = session.place_cities(place, cx)?;
            Some(
                tokens
                    .iter()
                    .filter(|token| cities.contains(*token))
                    .cloned()
                    .collect(),
            )
        };
        let (dealers, agencies) = (with_place(Place::Dealer), with_place(Place::Recruitment));
        if let Some(dealers) = dealers.filter(|cities| !cities.is_empty()) {
            bar = bar
                .action(action(
                    "picks-unlock-dealers",
                    t!(cx, "picks-unlock-dealers"),
                    t!(cx, "city-unlock-tooltip"),
                    dealers.clone(),
                    |session, cities, cx| session.unlock_many(Place::Dealer, cities, cx),
                    cx,
                ))
                .action(action(
                    "picks-lock-dealers",
                    t!(cx, "picks-lock-dealers"),
                    t!(cx, "city-lock-tooltip"),
                    dealers,
                    |session, cities, cx| session.lock_many(Place::Dealer, cities, cx),
                    cx,
                ));
        }
        if let Some(agencies) = agencies.filter(|cities| !cities.is_empty()) {
            bar = bar
                .action(action(
                    "picks-unlock-agencies",
                    t!(cx, "picks-unlock-agencies"),
                    t!(cx, "city-unlock-tooltip"),
                    agencies.clone(),
                    |session, cities, cx| session.unlock_many(Place::Recruitment, cities, cx),
                    cx,
                ))
                .action(action(
                    "picks-lock-agencies",
                    t!(cx, "picks-lock-agencies"),
                    t!(cx, "city-lock-tooltip"),
                    agencies,
                    |session, cities, cx| session.lock_many(Place::Recruitment, cities, cx),
                    cx,
                ));
        }
        Some(bar)
    }

    /// Opens the panel of the city on this row.
    fn open_city(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(token) = self.tokens.get(row) {
            let show_on_map: crate::pages::details::ShowOnMap =
                std::rc::Rc::new(move |token, _, cx| {
                    crate::workspace::show_on_world_map(
                        crate::pages::world_map::MapSpot::City(token.to_string()),
                        cx,
                    );
                });
            crate::pages::details::open_city(
                self.session.clone(),
                token,
                Some(show_on_map),
                window,
                cx,
            );
        }
    }

    fn unlock(&mut self, place: Place, cx: &mut Context<Self>) {
        let Some(cities) = self.session.read(cx).place_cities(place, cx) else {
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

impl Render for CitiesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Without the game's data nobody knows where dealers and agencies are; with it, ATS's
        // agencies wait for its map (read a moment later).
        let has_game_data = self.dealers.total.is_some();
        let has_agencies = self.recruitments.total.is_some();
        let agencies_unknown = has_game_data && !has_agencies;
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

        let pick_bar = self.render_pick_bar(cx);
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            // While cities are picked, the bar of what can be done to them takes the toolbar's
            // place.
            .child(match pick_bar {
                Some(bar) => bar.into_any_element(),
                None => h_flex()
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
                                Button::new("visit-all")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "map-visit-all"))
                                    .tip(t!(cx, "map-visit-all-tooltip"))
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
                                    .tip(t!(cx, "map-unlock-dealers-tooltip"))
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
                                    .tip(if agencies_unknown {
                                        t!(cx, "map-agencies-reading")
                                    } else {
                                        t!(cx, "map-unlock-recruitments-tooltip")
                                    })
                                    .disabled(!has_agencies || self.recruitments.is_complete())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.unlock(Place::Recruitment, cx)
                                    })),
                            ),
                    )
                    .into_any_element(),
            })
            .child(self.filter.clone())
            // Not hidden, and not just a disabled button: say why, and what makes it work.
            .children(
                match (has_game_data, agencies_unknown) {
                    (false, _) => Some(t!(cx, "map-needs-game")),
                    (true, true) => Some(t!(cx, "map-agencies-reading")),
                    (true, false) => None,
                }
                .map(|note| {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(note)
                }),
            )
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
    }
}
