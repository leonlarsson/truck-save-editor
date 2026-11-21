//! Fleet: every truck the company owns, in a sortable table that stays smooth with thousands of rows.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div, rems,
};
use save::{Vehicle, VehicleKind};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::pages::details;
use crate::session::Session;
use crate::ui::{FilterBar, FilterChanged, GameIconView};
use crate::workspace::OpenDetails;

pub struct FleetPage {
    session: Entity<Session>,
    /// Which vehicles the table shows: trucks or coaches.
    kind: VehicleKind,
    table: Entity<TableState<TruckTable>>,
    filter: Entity<FilterBar>,
    _subscriptions: Vec<Subscription>,
}

/// The Fleet filter's toggles, in order.
const FILTER_WORN: usize = 0;
const FILTER_NO_GARAGE: usize = 1;

impl FleetPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let kind = VehicleKind::Truck;
        let delegate = TruckTable {
            trucks: Vec::new(),
            session: session.clone(),
            sort: None,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let toggles = vec![
            t!(cx, "fleet-filter-worn"),
            t!(cx, "fleet-filter-no-garage"),
        ];
        let placeholder = t!(cx, "fleet-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            cx.subscribe_in(
                &table,
                window,
                |this, table, event: &TableEvent, window, cx| {
                    if let TableEvent::DoubleClickedRow(row) = event
                        && let Some(row) = table.read(cx).delegate().trucks.get(*row)
                    {
                        let vehicle = row.vehicle.clone();
                        details::open_vehicle(this.session.clone(), &vehicle, window, cx);
                    }
                },
            ),
        ];
        let mut page = Self {
            session,
            kind,
            table,
            filter,
            _subscriptions,
        };
        page.reload(cx);
        page
    }
}

impl FleetPage {
    /// Puts the session's current vehicles of the shown kind into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let all = rows(of_kind(self.session.read(cx).vehicles(), self.kind), cx);
        let total = all.len();
        let vehicles: Vec<Row> = {
            let filter = self.filter.read(cx);
            all.into_iter()
                .filter(|row| {
                    let truck = &row.vehicle;
                    let garage = truck
                        .garage_city
                        .as_deref()
                        .map(|city| game_data::city_name(cx, city))
                        .unwrap_or_default();
                    (!filter.is_on(FILTER_WORN) || truck.wear.worst() > 0.0)
                        && (!filter.is_on(FILTER_NO_GARAGE) || truck.garage_city.is_none())
                        && filter.matches(&[&row.name, &truck.plate, &garage], cx)
                })
                .collect()
        };
        let shown = vehicles.len();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
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

fn rows(vehicles: Vec<Vehicle>, cx: &App) -> Vec<Row> {
    vehicles
        .into_iter()
        .map(|vehicle| Row {
            name: game_data::model_name(cx, &vehicle)
                .map(SharedString::from)
                .unwrap_or_else(|| t!(cx, "unknown-truck")),
            logo: game_data::brand_logo(cx, &vehicle),
            vehicle,
        })
        .collect()
}

impl Render for FleetPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The summary counts every vehicle of the kind, whatever the filter shows.
        let trucks = of_kind(self.session.read(cx).vehicles(), self.kind);
        let truck_count = trucks.len();
        let worn_count = trucks
            .iter()
            .filter(|truck| truck.wear.worst() > 0.0)
            .count();
        let count = thousands(truck_count as i64);
        let vehicles = match self.kind {
            VehicleKind::Truck => t!(cx, "fleet-trucks", count = truck_count, amount = count),
            VehicleKind::Coach => t!(cx, "fleet-coaches", count = truck_count, amount = count),
            VehicleKind::Trailer => t!(cx, "fleet-trailers", count = truck_count, amount = count),
        };
        let worn = t!(
            cx,
            "fleet-worn",
            count = worn_count,
            amount = thousands(worn_count as i64)
        );
        let summary = t!(cx, "fleet-summary", vehicles = vehicles, worn = worn);

        // Trucks, coaches and trailers each get their own list; the switch shows the kinds the
        // save has (trucks always).
        let all = self.session.read(cx).vehicles();
        let count_of = |kind: VehicleKind| all.iter().filter(|v| v.kind == kind).count();
        let kinds: Vec<VehicleKind> =
            [VehicleKind::Truck, VehicleKind::Coach, VehicleKind::Trailer]
                .into_iter()
                .filter(|kind| *kind == VehicleKind::Truck || count_of(*kind) > 0)
                .collect();
        let switch = (kinds.len() > 1).then(|| {
            let buttons = kinds.iter().map(|kind| {
                let count = count_of(*kind);
                let label = match kind {
                    VehicleKind::Truck => t!(cx, "fleet-tab-trucks", count = count),
                    VehicleKind::Coach => t!(cx, "fleet-tab-coaches", count = count),
                    VehicleKind::Trailer => t!(cx, "fleet-tab-trailers", count = count),
                };
                Button::new(("vehicle-kind", *kind as usize))
                    .label(label)
                    .selected(self.kind == *kind)
            });
            let kinds = kinds.clone();
            ButtonGroup::new("vehicle-kind")
                .outline()
                .small()
                .children(buttons)
                .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                    if let Some(kind) = clicked.first().and_then(|index| kinds.get(*index)) {
                        this.show(*kind, cx);
                    }
                }))
        });
        let has_fuel = self.kind != VehicleKind::Trailer;
        let empty_tanks = trucks
            .iter()
            .filter(|truck| truck.fuel.is_some_and(|fuel| fuel < 1.0))
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
                        h_flex().gap_4().items_center().children(switch).child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(summary),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .children(has_fuel.then(|| {
                                Button::new("refuel-all")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "refuel-all"))
                                    .tooltip(t!(cx, "refuel-all-tooltip"))
                                    .disabled(empty_tanks == 0)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let kind = this.kind;
                                        this.session
                                            .update(cx, |session, cx| session.refuel_all(kind, cx));
                                    }))
                            }))
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
                    ),
            )
            .child(self.filter.clone())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    // Enter on the selected row opens it, like a double-click.
                    .on_action(cx.listener(|this, _: &OpenDetails, window, cx| {
                        let table = this.table.clone();
                        let Some(row) = table.read(cx).selected_row() else {
                            return;
                        };
                        if let Some(item) = table
                            .read(cx)
                            .delegate()
                            .trucks
                            .get(row)
                            .map(|row| row.vehicle.clone())
                        {
                            details::open_vehicle(this.session.clone(), &item, window, cx);
                        }
                    }))
                    .child(
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "trucks-table")),
                    ),
            )
    }
}

/// A vehicle with what the table shows of it that needs the game's data (made once per reload,
/// so sorting doesn't look names up again).
struct Row {
    vehicle: Vehicle,
    name: SharedString,
    logo: Option<std::sync::Arc<gamedata::GameIcon>>,
}

/// Feeds the trucks to GPUI Kit's table. Only the visible rows are ever rendered.
struct TruckTable {
    trucks: Vec<Row>,
    /// Asks for one vehicle's repair (by id); the page passes it to the session.
    session: Entity<Session>,
    /// The sort in effect, kept so new data (after a repair) stays in the same order.
    sort: Option<(TruckColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum TruckColumn {
    Truck,
    Plate,
    Garage,
    Odometer,
    Fuel,
    Wear,
    Details,
}

impl TruckColumn {
    const ALL: [TruckColumn; 7] = [
        TruckColumn::Truck,
        TruckColumn::Plate,
        TruckColumn::Garage,
        TruckColumn::Odometer,
        TruckColumn::Fuel,
        TruckColumn::Wear,
        TruckColumn::Details,
    ];
}

impl TruckTable {
    fn set_trucks(&mut self, trucks: Vec<Row>) {
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
            let (name_a, name_b) = (&a.name, &b.name);
            let (a, b) = (&a.vehicle, &b.vehicle);
            let ordering = match column {
                TruckColumn::Truck => name_a.cmp(name_b),
                TruckColumn::Plate => a.plate.cmp(&b.plate),
                TruckColumn::Garage => a.garage_city.cmp(&b.garage_city),
                TruckColumn::Odometer => a.odometer_km.cmp(&b.odometer_km),
                TruckColumn::Fuel => a.fuel.unwrap_or(0.0).total_cmp(&b.fuel.unwrap_or(0.0)),
                TruckColumn::Wear => a.wear.worst().total_cmp(&b.wear.worst()),
                TruckColumn::Details => std::cmp::Ordering::Equal,
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: TruckColumn, cx: &App) -> SharedString {
        let Some(Row {
            vehicle: truck,
            name,
            ..
        }) = self.trucks.get(row)
        else {
            return SharedString::default();
        };
        match column {
            TruckColumn::Truck => name.clone(),
            TruckColumn::Plate => truck.plate.clone().into(),
            // Coaches have no garage: they're driven, or parked in a city.
            TruckColumn::Garage => {
                match (&truck.garage_city, truck.is_in_use, &truck.parked_city) {
                    (Some(city), _, _) => game_data::city_name(cx, city).into(),
                    (None, true, _) => t!(cx, "vehicle-in-use"),
                    (None, false, Some(city)) => {
                        t!(cx, "vehicle-parked", city = game_data::city_name(cx, city))
                    }
                    (None, false, None) => t!(cx, "no-garage"),
                }
            }
            TruckColumn::Odometer => t!(cx, "distance-km", amount = thousands(truck.odometer_km)),
            // Rounded down, so only a full tank says 100%. Trailers have none.
            TruckColumn::Fuel => truck
                .fuel
                .map(|fuel| t!(cx, "percent", amount = (fuel * 100.0).floor() as i64))
                .unwrap_or_default(),
            TruckColumn::Details => SharedString::default(),
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
                .width(210.)
                .sortable(),
            Some(TruckColumn::Plate) => Column::new("plate", t!(cx, "column-plate"))
                .width(100.)
                .sortable(),
            Some(TruckColumn::Garage) => Column::new("garage", t!(cx, "column-garage"))
                .width(130.)
                .sortable(),
            Some(TruckColumn::Odometer) => Column::new("odometer", t!(cx, "column-odometer"))
                .width(120.)
                .text_right()
                .sortable(),
            Some(TruckColumn::Fuel) => Column::new("fuel", t!(cx, "column-fuel"))
                .width(70.)
                .text_right()
                .sortable(),
            Some(TruckColumn::Details) => Column::new("details", "").width(80.),
            Some(TruckColumn::Wear) | None => Column::new("wear", t!(cx, "column-wear"))
                .width(140.)
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
        let row = self.trucks.get(row_ix);
        let truck = row.map(|row| &row.vehicle);
        let is_muted = match column {
            TruckColumn::Garage => truck.is_some_and(|truck| {
                truck.garage_city.is_none() && !truck.is_in_use && truck.parked_city.is_none()
            }),
            TruckColumn::Wear => truck.is_some_and(|truck| truck.wear.worst() == 0.0),
            _ => false,
        };
        let text = self.text(row_ix, column, cx);
        let cell = h_flex().gap_2().w_full().items_center();
        let cell = if is_muted {
            cell.text_color(cx.theme().muted_foreground)
        } else {
            cell
        };
        match column {
            // The brand's logo from the game, in the text colour, then the model's name.
            // The brand's logo from the game, in a slot of its own so names line up even where
            // there's no logo, then the model's name.
            TruckColumn::Truck => cell
                .child(
                    div()
                        .w(rems(2.2))
                        .flex_shrink_0()
                        .flex()
                        .justify_center()
                        .children(row.and_then(|row| row.logo.clone()).map(|logo| {
                            GameIconView::new(logo)
                                .size(1.1)
                                .channel(gamedata::RED, cx.theme().muted_foreground)
                        })),
                )
                .child(text),
            // The wear, and Repair for a worn vehicle: always shown, not only on hover.
            TruckColumn::Wear => {
                let repair = truck.filter(|truck| truck.wear.worst() > 0.0).map(|truck| {
                    let id = truck.id.clone();
                    let session = self.session.clone();
                    Button::new(("repair", row_ix))
                        .xsmall()
                        .ghost()
                        .label(t!(cx, "repair"))
                        .on_click(move |_, _, cx| {
                            session.update(cx, |session, cx| session.repair(&id, cx));
                        })
                });
                cell.justify_end().children(repair).child(text)
            }
            // Opens the vehicle's panel: always shown, so it's found without hovering, and
            // reachable with Tab.
            TruckColumn::Details => match truck.cloned() {
                Some(vehicle) => {
                    let session = self.session.clone();
                    cell.justify_end().child(
                        Button::new(("details", row_ix))
                            .xsmall()
                            .ghost()
                            .label(t!(cx, "details"))
                            .on_click(move |_, window, cx| {
                                details::open_vehicle(session.clone(), &vehicle, window, cx)
                            }),
                    )
                }
                None => cell,
            },
            // Numbers line up on the right, like their headings.
            TruckColumn::Odometer | TruckColumn::Fuel => cell.justify_end().child(text),
            _ => cell.child(text),
        }
        .into_any_element()
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
