//! Garages: every city's garage, whether it's owned, how big it is and how full. Buy them all or
//! upgrade the owned ones to large in one go.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
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

pub struct GaragesPage {
    session: Entity<Session>,
    table: Entity<TableState<GarageTable>>,
    _subscriptions: Vec<Subscription>,
}

impl GaragesPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = GarageTable {
            garages: session.read(cx).garages(),
            sort: None,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let _subscriptions = vec![cx.observe(&session, |this, session, cx| {
            let garages = session.read(cx).garages();
            this.table.update(cx, |table, cx| {
                table.delegate_mut().set_garages(garages);
                table.refresh(cx);
                cx.notify();
            });
            cx.notify();
        })];
        Self {
            session,
            table,
            _subscriptions,
        }
    }

    fn upgrade(&mut self, buy_all: bool, cx: &mut Context<Self>) {
        self.session
            .update(cx, |session, cx| session.upgrade_garages(buy_all, cx));
    }
}

impl Render for GaragesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let garages = &self.table.read(cx).delegate().garages;
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
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(summary),
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
