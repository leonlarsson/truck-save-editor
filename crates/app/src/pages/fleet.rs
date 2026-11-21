//! Fleet: every truck the company owns, in a sortable table that stays smooth with thousands of rows.

use std::collections::HashMap;

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::Button,
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Window, div,
};

use crate::format::thousands;
use crate::sample::{self, Truck};

/// The number of made-up trucks: well beyond any real fleet, to prove the table stays smooth.
const SAMPLE_TRUCKS: u32 = 2_000;

pub struct FleetPage {
    table: Entity<TableState<TruckTable>>,
}

impl FleetPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let trucks = sample::trucks(SAMPLE_TRUCKS);
        let original_wear = trucks
            .iter()
            .map(|truck| (truck.id, truck.wear_percent))
            .collect();
        let delegate = TruckTable {
            trucks,
            original_wear,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        cx.observe(&table, |_, _, cx| cx.notify()).detach();
        Self { table }
    }

    /// How many trucks differ from the loaded save.
    pub fn changes(&self, cx: &App) -> usize {
        self.table.read(cx).delegate().changed_trucks()
    }

    pub fn discard_changes(&mut self, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            for truck in &mut delegate.trucks {
                if let Some(wear) = delegate.original_wear.get(&truck.id) {
                    truck.wear_percent = *wear;
                }
            }
            table.refresh(cx);
            cx.notify();
        });
    }

    fn repair_all(&mut self, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            for truck in &mut table.delegate_mut().trucks {
                truck.wear_percent = 0;
            }
            table.refresh(cx);
            cx.notify();
        });
    }
}

impl Render for FleetPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let delegate = self.table.read(cx).delegate();
        let truck_count = delegate.trucks.len();
        let worn_count = delegate
            .trucks
            .iter()
            .filter(|truck| truck.wear_percent > 0)
            .count();

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
                            .child(format!(
                                "{} trucks · {} need repairs",
                                thousands(truck_count as i64),
                                thousands(worn_count as i64)
                            )),
                    )
                    .child(
                        Button::new("repair-all")
                            .small()
                            .outline()
                            .label("Repair all")
                            .on_click(cx.listener(|this, _, _, cx| this.repair_all(cx))),
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
    /// Wear as loaded, by truck id, to count changes and undo them.
    original_wear: HashMap<u32, u8>,
}

#[derive(Clone, Copy)]
enum TruckColumn {
    Truck,
    Plate,
    Driver,
    Garage,
    Odometer,
    Wear,
}

impl TruckColumn {
    const ALL: [TruckColumn; 6] = [
        TruckColumn::Truck,
        TruckColumn::Plate,
        TruckColumn::Driver,
        TruckColumn::Garage,
        TruckColumn::Odometer,
        TruckColumn::Wear,
    ];
}

impl TruckTable {
    fn changed_trucks(&self) -> usize {
        self.trucks
            .iter()
            .filter(|truck| self.original_wear.get(&truck.id) != Some(&truck.wear_percent))
            .count()
    }

    fn text(&self, row: usize, column: TruckColumn) -> SharedString {
        let Some(truck) = self.trucks.get(row) else {
            return SharedString::default();
        };
        match column {
            TruckColumn::Truck => truck.name.clone().into(),
            TruckColumn::Plate => truck.plate.clone().into(),
            TruckColumn::Driver => truck
                .driver
                .clone()
                .unwrap_or_else(|| "No driver".into())
                .into(),
            TruckColumn::Garage => truck.garage.into(),
            TruckColumn::Odometer => format!("{} km", thousands(truck.odometer_km)).into(),
            TruckColumn::Wear => format!("{} %", truck.wear_percent).into(),
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
            Some(TruckColumn::Truck) => Column::new("truck", "Truck").width(200.).sortable(),
            Some(TruckColumn::Plate) => Column::new("plate", "Plate").width(100.),
            Some(TruckColumn::Driver) => Column::new("driver", "Driver").width(170.).sortable(),
            Some(TruckColumn::Garage) => Column::new("garage", "Garage").width(130.).sortable(),
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
        // Sort by the column, keeping a stable order (by id) for equal values.
        self.trucks.sort_by(|a, b| {
            let ordering = match column {
                TruckColumn::Truck => a.name.cmp(&b.name),
                TruckColumn::Plate => a.plate.cmp(&b.plate),
                TruckColumn::Driver => a.driver.cmp(&b.driver),
                TruckColumn::Garage => a.garage.cmp(b.garage),
                TruckColumn::Odometer => a.odometer_km.cmp(&b.odometer_km),
                TruckColumn::Wear => a.wear_percent.cmp(&b.wear_percent),
            };
            match sort {
                ColumnSort::Ascending => ordering.then(a.id.cmp(&b.id)),
                ColumnSort::Descending => ordering.reverse().then(a.id.cmp(&b.id)),
                ColumnSort::Default => a.id.cmp(&b.id),
            }
        });
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
        let is_missing_driver = matches!(column, TruckColumn::Driver)
            && self
                .trucks
                .get(row_ix)
                .is_some_and(|truck| truck.driver.is_none());
        let text = self.text(row_ix, column);
        if is_missing_driver {
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
