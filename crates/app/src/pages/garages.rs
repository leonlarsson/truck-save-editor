//! Garages: every city's garage, whether it's owned, how big it is and how full. Buy them all or
//! upgrade the owned ones to large in one go.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div,
};
use save::Garage;

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{FilterBar, FilterChanged};

pub struct GaragesPage {
    session: Entity<Session>,
    table: Entity<TableState<GarageTable>>,
    /// The headquarters: one of the cities with an owned garage.
    headquarters: Entity<SelectState<SearchableVec<CityOption>>>,
    filter: Entity<FilterBar>,
    _subscriptions: Vec<Subscription>,
}

/// The Garages filter's toggles, in order.
const FILTER_OWNED: usize = 0;
const FILTER_NOT_LARGE: usize = 1;

impl GaragesPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = GarageTable {
            garages: Vec::new(),
            sort: None,
        };
        let toggles = vec![
            t!(cx, "garages-filter-owned"),
            t!(cx, "garages-filter-not-large"),
        ];
        let placeholder = t!(cx, "garages-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let headquarters = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let _subscriptions = vec![
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.reload(cx);
                this.sync_headquarters(window, cx);
            }),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            game_data::observe_in(window, cx, |this, window, cx| {
                this.reload(cx);
                this.sync_headquarters(window, cx);
            }),
            cx.subscribe_in(
                &headquarters,
                window,
                |this, _, event: &SelectEvent<SearchableVec<CityOption>>, _, cx| {
                    if let SelectEvent::Confirm(Some(city)) = event {
                        let city = city.to_string();
                        this.session.update(cx, |session, cx| {
                            session.edit_career(cx, |career| career.hq_city = city);
                        });
                    }
                },
            ),
        ];
        let mut page = Self {
            session,
            table,
            headquarters,
            filter,
            _subscriptions,
        };
        page.reload(cx);
        page.sync_headquarters(window, cx);
        page
    }

    /// Puts the garages that match the filter into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let all = self.session.read(cx).garages();
        let total = all.len();
        let garages: Vec<Garage> = {
            let filter = self.filter.read(cx);
            all.into_iter()
                .filter(|garage| {
                    let city = game_data::city_name(cx, &garage.city);
                    (!filter.is_on(FILTER_OWNED) || garage.is_owned)
                        && (!filter.is_on(FILTER_NOT_LARGE) || !garage.is_large())
                        && filter.matches(&[&city], cx)
                })
                .collect()
        };
        let shown = garages.len();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_garages(garages);
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    /// Puts the owned garages' cities in the HQ picker, the current HQ selected.
    fn sync_headquarters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let hq: SharedString = session.career().hq_city.clone().into();
        let mut cities: Vec<CityOption> = session
            .garages()
            .into_iter()
            .filter(|garage| garage.is_owned)
            .map(|garage| CityOption {
                name: game_data::city_name(cx, &garage.city).into(),
                token: garage.city.into(),
            })
            .collect();
        cities.sort_by(|a, b| a.name.cmp(&b.name));
        self.headquarters.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(cities), window, cx);
            select.set_selected_value(&hq, window, cx);
        });
    }

    fn upgrade(&mut self, buy_all: bool, cx: &mut Context<Self>) {
        self.session
            .update(cx, |session, cx| session.upgrade_garages(buy_all, cx));
    }
}

impl Render for GaragesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The summary counts every garage, whatever the filter shows.
        let garages = self.session.read(cx).garages();
        let count = garages.len();
        let owned = garages.iter().filter(|g| g.is_owned).count();
        let large = garages.iter().filter(|g| g.is_large()).count();
        let summary = t!(
            cx,
            "garages-summary",
            count = count,
            owned = owned,
            large = large
        );

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
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().text_sm().child(t!(cx, "headquarters")))
                                    .child(
                                        div().w(gpui_kit::rems(13.)).child(
                                            Select::new(&self.headquarters)
                                                .small()
                                                .placeholder(t!(cx, "headquarters-pick")),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(summary),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("upgrade-owned")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "garages-upgrade-owned"))
                                    .tooltip(t!(cx, "garages-upgrade-owned-tooltip"))
                                    .disabled(owned == large)
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.upgrade(false, cx)),
                                    ),
                            )
                            .child(
                                Button::new("buy-all")
                                    .small()
                                    .primary()
                                    .label(t!(cx, "garages-buy-all"))
                                    .tooltip(t!(cx, "garages-buy-all-tooltip"))
                                    .disabled(large == count)
                                    .on_click(cx.listener(|this, _, _, cx| this.upgrade(true, cx))),
                            ),
                    ),
            )
            .child(self.filter.clone())
            .child(
                div().flex_1().min_h_0().child(
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label(t!(cx, "garages-table")),
                ),
            )
    }
}

struct GarageTable {
    garages: Vec<Garage>,
    sort: Option<(GarageColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum GarageColumn {
    City,
    Size,
    Trucks,
    Drivers,
}

impl GarageColumn {
    const ALL: [GarageColumn; 4] = [
        GarageColumn::City,
        GarageColumn::Size,
        GarageColumn::Trucks,
        GarageColumn::Drivers,
    ];
}

impl GarageTable {
    fn set_garages(&mut self, garages: Vec<Garage>) {
        self.garages = garages;
        if let Some((column, sort)) = self.sort {
            self.sort_by(column, sort);
        }
    }

    /// Sorts by a column; equal values keep the save's order (the sort is stable).
    fn sort_by(&mut self, column: GarageColumn, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            return;
        }
        self.garages.sort_by(|a, b| {
            let ordering = match column {
                GarageColumn::City => a.city.cmp(&b.city),
                GarageColumn::Size => (a.is_owned, a.slots).cmp(&(b.is_owned, b.slots)),
                GarageColumn::Trucks => a.trucks.cmp(&b.trucks),
                GarageColumn::Drivers => a.drivers.cmp(&b.drivers),
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: GarageColumn, cx: &App) -> SharedString {
        let Some(garage) = self.garages.get(row) else {
            return SharedString::default();
        };
        let filled = |filled: usize| {
            if garage.is_owned {
                t!(cx, "garage-filled", filled = filled, slots = garage.slots)
            } else {
                SharedString::default()
            }
        };
        match column {
            GarageColumn::City => game_data::city_name(cx, &garage.city).into(),
            GarageColumn::Size if !garage.is_owned => t!(cx, "garage-not-owned"),
            GarageColumn::Size if garage.is_large() => t!(cx, "garage-large"),
            GarageColumn::Size => t!(cx, "garage-slots", count = garage.slots),
            GarageColumn::Trucks => filled(garage.trucks),
            GarageColumn::Drivers => filled(garage.drivers),
        }
    }
}

impl TableDelegate for GarageTable {
    fn columns_count(&self, _: &App) -> usize {
        GarageColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.garages.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        match GarageColumn::ALL.get(col_ix) {
            Some(GarageColumn::City) => Column::new("city", t!(cx, "column-city"))
                .width(220.)
                .sortable(),
            Some(GarageColumn::Size) => Column::new("size", t!(cx, "column-garage-size"))
                .width(150.)
                .sortable(),
            Some(GarageColumn::Trucks) => Column::new("trucks", t!(cx, "column-trucks"))
                .width(110.)
                .text_right()
                .sortable(),
            Some(GarageColumn::Drivers) | None => Column::new("drivers", t!(cx, "column-drivers"))
                .width(110.)
                .text_right()
                .sortable(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        let Some(column) = GarageColumn::ALL.get(col_ix).copied() else {
            return;
        };
        self.sort = (sort != ColumnSort::Default).then_some((column, sort));
        self.sort_by(column, sort);
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let column = GarageColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(GarageColumn::City);
        let is_muted = self.garages.get(row_ix).is_some_and(|g| !g.is_owned);
        let text = self.text(row_ix, column, cx);
        if is_muted {
            div().text_color(cx.theme().muted_foreground).child(text)
        } else {
            div().child(text)
        }
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, cx: &App) -> Option<SharedString> {
        let column = GarageColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column, cx))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, cx: &App) -> String {
        GarageColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column, cx).to_string())
            .unwrap_or_default()
    }
}

/// A city in the HQ picker: shown by its name, chosen by its token.
#[derive(Clone)]
struct CityOption {
    token: SharedString,
    name: SharedString,
}

impl SelectItem for CityOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.token
    }
}
