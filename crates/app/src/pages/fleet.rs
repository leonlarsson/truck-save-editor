//! Fleet: every truck the company owns, in a sortable table that stays smooth with thousands of rows.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::Button,
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div,
};
use save::Truck;

use crate::format::thousands;
use crate::session::Session;

pub struct FleetPage {
    session: Entity<Session>,
    table: Entity<TableState<TruckTable>>,
    _subscriptions: Vec<Subscription>,
}

impl FleetPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = TruckTable {
            trucks: session.read(cx).trucks(),
            sort: None,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let _subscriptions = vec![cx.observe(&session, |this, session, cx| {
            let trucks = session.read(cx).trucks();
            this.table.update(cx, |table, cx| {
                table.delegate_mut().set_trucks(trucks);
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
}

impl Render for FleetPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trucks = &self.table.read(cx).delegate().trucks;
        let truck_count = trucks.len();
        let worn_count = trucks
            .iter()
            .filter(|truck| truck.wear.worst() > 0.0)
            .count();
        let summary = match worn_count {
            0 => format!(
                "{} trucks · all in good shape",
                thousands(truck_count as i64)
            ),
            1 => format!("{} trucks · 1 needs repairs", thousands(truck_count as i64)),
            n => format!(
                "{} trucks · {} need repairs",
                thousands(truck_count as i64),
                thousands(n as i64)
            ),
        };

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
                        Button::new("repair-all")
                            .small()
                            .outline()
                            .label("Repair all")
                            .disabled(worn_count == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.repair_all(cx));
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label("Trucks"),
                ),
            )
    }
}

/// Feeds the trucks to GPUI Kit's table. Only the visible rows are ever rendered.
struct TruckTable {
    trucks: Vec<Truck>,
    /// The sort in effect, kept so new data (after a repair) stays in the same order.
    sort: Option<(TruckColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum TruckColumn {
    Truck,
    Plate,
    Garage,
    Odometer,
    Wear,
}

impl TruckColumn {
    const ALL: [TruckColumn; 5] = [
        TruckColumn::Truck,
        TruckColumn::Plate,
        TruckColumn::Garage,
        TruckColumn::Odometer,
        TruckColumn::Wear,
    ];
}

impl TruckTable {
    fn set_trucks(&mut self, trucks: Vec<Truck>) {
        self.trucks = trucks;
        if let Some((column, sort)) = self.sort {
            self.sort_by(column, sort);
        }
    }

    /// Sorts by a column; equal values keep the order the save lists them in (the sort is stable).
    fn sort_by(&mut self, column: TruckColumn, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            return;
        }
        self.trucks.sort_by(|a, b| {
            let ordering = match column {
                TruckColumn::Truck => a.display_name().cmp(&b.display_name()),
                TruckColumn::Plate => a.plate.cmp(&b.plate),
                TruckColumn::Garage => a.garage_city.cmp(&b.garage_city),
                TruckColumn::Odometer => a.odometer_km.cmp(&b.odometer_km),
                TruckColumn::Wear => a.wear.worst().total_cmp(&b.wear.worst()),
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: TruckColumn) -> SharedString {
        let Some(truck) = self.trucks.get(row) else {
            return SharedString::default();
        };
        match column {
            TruckColumn::Truck => truck.display_name().into(),
            TruckColumn::Plate => truck.plate.clone().into(),
            TruckColumn::Garage => truck
                .garage_city
                .as_deref()
                .map(save::city_display_name)
                .unwrap_or_else(|| "No garage".to_string())
                .into(),
            TruckColumn::Odometer => format!("{} km", thousands(truck.odometer_km)).into(),
            // Rounded up, so any wear at all shows (and matches the "need repairs" count).
            TruckColumn::Wear => format!("{}%", (truck.wear.worst() * 100.0).ceil()).into(),
        }
    }
}

impl TableDelegate for TruckTable {
    fn columns_count(&self, _: &App) -> usize {
        TruckColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.trucks.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        match TruckColumn::ALL.get(col_ix) {
            Some(TruckColumn::Truck) => Column::new("truck", "Truck").width(220.).sortable(),
            Some(TruckColumn::Plate) => Column::new("plate", "Plate").width(110.).sortable(),
            Some(TruckColumn::Garage) => Column::new("garage", "Garage").width(150.).sortable(),
            Some(TruckColumn::Odometer) => Column::new("odometer", "Odometer")
                .width(130.)
                .text_right()
                .sortable(),
            Some(TruckColumn::Wear) | None => Column::new("wear", "Wear")
                .width(80.)
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
        let Some(column) = TruckColumn::ALL.get(col_ix).copied() else {
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
        let column = TruckColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(TruckColumn::Truck);
        let truck = self.trucks.get(row_ix);
        let is_muted = match column {
            TruckColumn::Garage => truck.is_some_and(|truck| truck.garage_city.is_none()),
            TruckColumn::Wear => truck.is_some_and(|truck| truck.wear.worst() == 0.0),
            _ => false,
        };
        let text = self.text(row_ix, column);
        if is_muted {
            div().text_color(cx.theme().muted_foreground).child(text)
        } else {
            div().child(text)
        }
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, _: &App) -> Option<SharedString> {
        let column = TruckColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        TruckColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column).to_string())
            .unwrap_or_default()
    }
}
