//! Fleet: every truck the company owns, in a sortable table that stays smooth with thousands of rows.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup},
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div,
};
use save::{Vehicle, VehicleKind};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::session::Session;

pub struct FleetPage {
    session: Entity<Session>,
    /// Which vehicles the table shows: trucks or coaches.
    kind: VehicleKind,
    table: Entity<TableState<TruckTable>>,
    _subscriptions: Vec<Subscription>,
}

impl FleetPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let kind = VehicleKind::Truck;
        let delegate = TruckTable {
            trucks: of_kind(session.read(cx).vehicles(), kind),
            sort: None,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
        ];
        Self {
            session,
            kind,
            table,
            _subscriptions,
        }
    }
}

impl FleetPage {
    /// Puts the session's current vehicles of the shown kind into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let vehicles = of_kind(self.session.read(cx).vehicles(), self.kind);
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_trucks(vehicles);
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    fn show(&mut self, kind: VehicleKind, cx: &mut Context<Self>) {
        self.kind = kind;
        self.reload(cx);
    }
}

fn of_kind(vehicles: Vec<Vehicle>, kind: VehicleKind) -> Vec<Vehicle> {
    vehicles.into_iter().filter(|v| v.kind == kind).collect()
}

impl Render for FleetPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trucks = &self.table.read(cx).delegate().trucks;
        let truck_count = trucks.len();
        let worn_count = trucks
            .iter()
            .filter(|truck| truck.wear.worst() > 0.0)
            .count();
        let count = thousands(truck_count as i64);
        let vehicles = match self.kind {
            VehicleKind::Truck => t!(cx, "fleet-trucks", count = truck_count, amount = count),
            VehicleKind::Coach => t!(cx, "fleet-coaches", count = truck_count, amount = count),
        };
        let worn = t!(
            cx,
            "fleet-worn",
            count = worn_count,
            amount = thousands(worn_count as i64)
        );
        let summary = t!(cx, "fleet-summary", vehicles = vehicles, worn = worn);

        // Trucks and coaches each get their own list; the switch only shows when there are coaches.
        let all = self.session.read(cx).vehicles();
        let coach_count = all.iter().filter(|v| v.kind == VehicleKind::Coach).count();
        let all_truck_count = all.len() - coach_count;
        let switch = (coach_count > 0).then(|| {
            ButtonGroup::new("vehicle-kind")
                .outline()
                .small()
                .child(
                    Button::new("trucks")
                        .label(t!(cx, "fleet-tab-trucks", count = all_truck_count))
                        .selected(self.kind == VehicleKind::Truck),
                )
                .child(
                    Button::new("coaches")
                        .label(t!(cx, "fleet-tab-coaches", count = coach_count))
                        .selected(self.kind == VehicleKind::Coach),
                )
                .on_click(
                    cx.listener(|this, clicked: &Vec<usize>, _, cx| match clicked.first() {
                        Some(0) => this.show(VehicleKind::Truck, cx),
                        Some(1) => this.show(VehicleKind::Coach, cx),
                        _ => {}
                    }),
                )
        });

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
                        h_flex().gap_4().items_center().children(switch).child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(summary),
                        ),
                    )
                    .child(
                        Button::new("repair-all")
                            .small()
                            .outline()
                            .label(t!(cx, "repair-all"))
                            .disabled(worn_count == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                let kind = this.kind;
                                this.session
                                    .update(cx, |session, cx| session.repair_all(kind, cx));
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label(t!(cx, "trucks-table")),
                ),
            )
    }
}

/// Feeds the trucks to GPUI Kit's table. Only the visible rows are ever rendered.
struct TruckTable {
    trucks: Vec<Vehicle>,
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
    fn set_trucks(&mut self, trucks: Vec<Vehicle>) {
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

    fn text(&self, row: usize, column: TruckColumn, cx: &App) -> SharedString {
        let Some(truck) = self.trucks.get(row) else {
            return SharedString::default();
        };
        match column {
            TruckColumn::Truck => truck
                .display_name()
                .map(SharedString::from)
                .unwrap_or_else(|| t!(cx, "unknown-truck")),
            TruckColumn::Plate => truck.plate.clone().into(),
            TruckColumn::Garage => truck
                .garage_city
                .as_deref()
                .map(|city| SharedString::from(game_data::city_name(cx, city)))
                .unwrap_or_else(|| t!(cx, "no-garage")),
            TruckColumn::Odometer => t!(cx, "distance-km", amount = thousands(truck.odometer_km)),
            // Rounded up, so any wear at all shows (and matches the "need repairs" count).
            TruckColumn::Wear => t!(
                cx,
                "percent",
                amount = (truck.wear.worst() * 100.0).ceil() as i64
            ),
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

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        match TruckColumn::ALL.get(col_ix) {
            Some(TruckColumn::Truck) => Column::new("truck", t!(cx, "column-truck"))
                .width(220.)
                .sortable(),
            Some(TruckColumn::Plate) => Column::new("plate", t!(cx, "column-plate"))
                .width(110.)
                .sortable(),
            Some(TruckColumn::Garage) => Column::new("garage", t!(cx, "column-garage"))
                .width(150.)
                .sortable(),
            Some(TruckColumn::Odometer) => Column::new("odometer", t!(cx, "column-odometer"))
                .width(130.)
                .text_right()
                .sortable(),
            Some(TruckColumn::Wear) | None => Column::new("wear", t!(cx, "column-wear"))
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
        let text = self.text(row_ix, column, cx);
        if is_muted {
            div().text_color(cx.theme().muted_foreground).child(text)
        } else {
            div().child(text)
        }
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, cx: &App) -> Option<SharedString> {
        let column = TruckColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column, cx))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, cx: &App) -> String {
        TruckColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column, cx).to_string())
            .unwrap_or_default()
    }
}
