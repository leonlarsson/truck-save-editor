//! Fleet: every truck the company owns, in a sortable table that stays smooth with thousands of rows.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    tag::Tag,
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, Subscription, Window, div, rems,
};
use save::{Vehicle, VehicleKind};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::pages::details;
use crate::session::Session;
use crate::ui::{FillsWidth, fill_widths};
use crate::ui::{FilterBar, FilterChanged, GameIconView};
use crate::workspace::OpenDetails;

pub struct FleetPage {
    session: Entity<Session>,
    /// Which vehicles the table shows: trucks or coaches.
    kind: VehicleKind,
    table: Entity<TableState<TruckTable>>,
    filter: Entity<FilterBar>,
    /// The company's garages, to move the picked trucks to.
    garages: Entity<SelectState<SearchableVec<crate::ui::Choice>>>,
    /// How the last action on the picked rows went (some may not have fit).
    /// Whether the garage list needs filling again.
    garages_stale: bool,
    _subscriptions: Vec<Subscription>,
}

/// The Fleet filter's toggles, in order.
const FILTER_WORN: usize = 0;
const FILTER_NO_GARAGE: usize = 1;
const FILTER_YOURS: usize = 2;

impl FleetPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let kind = VehicleKind::Truck;
        let delegate = TruckTable {
            table_width: None,
            trucks: Vec::new(),
            columns: TruckColumn::ALL.to_vec(),
            session: session.clone(),
            sort: None,
            picks: crate::ui::Picks::default(),
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let toggles = vec![
            t!(cx, "fleet-filter-worn"),
            t!(cx, "fleet-filter-no-garage"),
            t!(cx, "fleet-filter-yours"),
        ];
        let placeholder = t!(cx, "fleet-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            // A click on a column's heading sorts by it (the table itself would select the
            // column, which nothing here uses).
            cx.subscribe_in(&table, window, |_, table, event: &TableEvent, _, cx| {
                if let TableEvent::SelectColumn(col_ix) = event {
                    let col_ix = *col_ix;
                    table.update(cx, |table, cx| {
                        table.delegate_mut().sort_by_header(col_ix, cx);
                        table.clear_selection(cx);
                        table.refresh(cx);
                        cx.notify();
                    });
                }
            }),
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
        let garages = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let mut _subscriptions = _subscriptions;
        // Picking a row redraws the bar above the table.
        _subscriptions.push(cx.observe(&table, |_, _, cx| cx.notify()));
        _subscriptions.push(cx.subscribe_in(
            &garages,
            window,
            |this, _, event: &SelectEvent<SearchableVec<crate::ui::Choice>>, window, cx| {
                if let SelectEvent::Confirm(Some(garage)) = event {
                    let garage = garage.to_string();
                    let ids = this.table.read(cx).delegate().picks.ids();
                    let moved = this
                        .session
                        .update(cx, |session, cx| session.move_many(&ids, &garage, cx));
                    let outcome = crate::labels::picks_outcome(
                        cx,
                        crate::labels::PickAction::Move,
                        moved,
                        ids.len(),
                    );
                    crate::ui::picks_done(outcome, moved == ids.len(), window, cx);
                    this.garages
                        .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                    cx.notify();
                }
            },
        ));
        let mut page = Self {
            session,
            kind,
            table,
            filter,
            garages,
            garages_stale: true,
            _subscriptions,
        };
        page.reload(cx);
        page
    }
}

impl FleetPage {
    /// Puts the session's current vehicles of the shown kind into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        // The truck the player drives and the trailer they pull.
        let yours: Vec<String> = {
            let session = self.session.read(cx);
            [session.own_truck(), save::own_trailer(&session.preview())]
                .into_iter()
                .flatten()
                .collect()
        };
        let mut all = rows(
            of_kind(self.session.read(cx).vehicles(), self.kind),
            &yours,
            cx,
        );
        // Yours first, until a column's heading sorts the table.
        all.sort_by_key(|row| !row.is_yours);
        let total = all.len();
        let existing: Vec<String> = all.iter().map(|row| row.vehicle.id.clone()).collect();
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
                        && (!filter.is_on(FILTER_YOURS) || row.is_yours)
                        && filter.matches(&[&row.name, &truck.plate, &garage], cx)
                })
                .collect()
        };
        let shown = vehicles.len();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.set_trucks(vehicles);
            delegate
                .picks
                .keep_only(existing.iter().map(String::as_str));
            table.refresh(cx);
            cx.notify();
        });
        self.garages_stale = true;
        cx.notify();
    }

    /// The company's garages in the move list, by city name (when they may have changed; filling
    /// a list needs the window, which a redraw has).
    fn sync_garages(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.garages_stale {
            return;
        }
        self.garages_stale = false;
        let garages = crate::ui::owned_garages(self.session.read(cx), cx);
        self.garages.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(garages), window, cx)
        });
    }

    fn show(&mut self, kind: VehicleKind, cx: &mut Context<Self>) {
        self.kind = kind;
        self.table
            .update(cx, |table, _| table.delegate_mut().picks.clear());
        let columns: Vec<TruckColumn> = TruckColumn::ALL
            .into_iter()
            .filter(|column| kind != VehicleKind::Trailer || *column != TruckColumn::Fuel)
            .collect();
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            delegate.columns = columns;
            // A sort by a column that's gone is dropped.
            if delegate
                .sort
                .is_some_and(|(column, _)| !delegate.columns.contains(&column))
            {
                delegate.sort = None;
            }
            table.refresh(cx);
        });
        self.reload(cx);
    }
}

impl FleetPage {
    /// The bar above the table while rows are picked: what can be done to all of them.
    fn render_pick_bar(&self, cx: &mut Context<Self>) -> Option<crate::ui::PickBar> {
        let picks = self.table.read(cx).delegate().picks.clone();
        if picks.is_empty() {
            return None;
        }
        let ids = picks.ids();
        let table = self.table.clone();
        let mut bar = crate::ui::PickBar::new(picks.len(), move |_, cx| {
            table.update(cx, |table, cx| {
                table.delegate_mut().picks.clear();
                cx.notify();
            });
        });
        let action =
            |id: &'static str, label: SharedString| Button::new(id).small().outline().label(label);
        let repair_ids = ids.clone();
        bar = bar.action(
            action("picks-repair", t!(cx, "repair")).on_click(cx.listener(
                move |this, _, window, cx| {
                    let ids = repair_ids.clone();
                    this.session
                        .update(cx, |session, cx| session.repair_many(&ids, cx));
                    let title = t!(cx, "picks-done-repair", count = ids.len());
                    crate::ui::picks_done(title, true, window, cx);
                },
            )),
        );
        if self.kind != VehicleKind::Trailer {
            let fuel_ids = ids.clone();
            bar = bar.action(
                action("picks-refuel", t!(cx, "picks-refuel")).on_click(cx.listener(
                    move |this, _, window, cx| {
                        let ids = fuel_ids.clone();
                        this.session
                            .update(cx, |session, cx| session.refuel_many(&ids, cx));
                        let title = t!(cx, "picks-done-refuel", count = ids.len());
                        crate::ui::picks_done(title, true, window, cx);
                    },
                )),
            );
        }
        if self.kind == VehicleKind::Truck {
            bar = bar.action(
                div().w(rems(13.)).child(
                    Select::new(&self.garages)
                        .small()
                        .placeholder(t!(cx, "picks-move"))
                        .accessibility_label(t!(cx, "picks-move")),
                ),
            );
        }
        if self.kind != VehicleKind::Coach {
            let kind = self.kind;
            let remove_ids = ids.clone();
            bar = bar.action(
                action("picks-remove", t!(cx, "picks-remove"))
                    .danger()
                    .tooltip(t!(cx, "remove-tooltip"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let removals: Vec<save::Removal> = remove_ids
                            .iter()
                            .map(|id| match kind {
                                VehicleKind::Trailer => save::Removal::Trailer(id.clone()),
                                _ => save::Removal::Truck(id.clone()),
                            })
                            .collect();
                        let count = removals.len();
                        let removed = this
                            .session
                            .update(cx, |session, cx| session.remove_many(removals, cx));
                        let outcome = crate::labels::picks_outcome(
                            cx,
                            crate::labels::PickAction::Remove,
                            removed,
                            count,
                        );
                        crate::ui::picks_done(outcome, removed == count, window, cx);
                        cx.notify();
                    })),
            );
        }
        Some(bar)
    }
}

fn of_kind(vehicles: Vec<Vehicle>, kind: VehicleKind) -> Vec<Vehicle> {
    vehicles.into_iter().filter(|v| v.kind == kind).collect()
}

fn rows(vehicles: Vec<Vehicle>, yours: &[String], cx: &App) -> Vec<Row> {
    vehicles
        .into_iter()
        .map(|vehicle| Row {
            is_yours: yours.contains(&vehicle.id),
            name: game_data::model_name(cx, &vehicle)
                .map(SharedString::from)
                .unwrap_or_else(|| t!(cx, "unknown-truck")),
            logo: game_data::brand_logo(cx, &vehicle),
            vehicle,
        })
        .collect()
}

impl Render for FleetPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_garages(window, cx);
        let pick_bar = self.render_pick_bar(cx);
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

        // While rows are picked, what can be done to them takes the toolbar's place (the same
        // height, so the table doesn't jump under the pointer).
        let toolbar = match pick_bar {
            Some(bar) => bar.into_any_element(),
            None => h_flex()
                .justify_between()
                .items_center()
                .gap_4()
                // On a narrow window the buttons move to a line of their own.
                .flex_wrap()
                .gap_y_2()
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
                )
                .into_any_element(),
        };
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(toolbar)
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
                    .child(crate::ui::table_or_empty(
                        &self.filter,
                        crate::ui::fill_table(
                            &self.table,
                            DataTable::new(&self.table)
                                .stripe(true)
                                .bordered(true)
                                .accessibility_label(t!(cx, "trucks-table"))
                                .into_any_element(),
                        ),
                        None,
                        cx,
                    )),
            )
    }
}

/// A vehicle with what the table shows of it that needs the game's data (made once per reload,
/// so sorting doesn't look names up again).
struct Row {
    vehicle: Vehicle,
    /// The truck the player drives, or the trailer they pull.
    is_yours: bool,
    name: SharedString,
    logo: Option<std::sync::Arc<gamedata::GameIcon>>,
}

/// Feeds the trucks to GPUI Kit's table. Only the visible rows are ever rendered.
struct TruckTable {
    /// The table's measured width, which its name columns grow to fill.
    table_width: Option<Pixels>,
    trucks: Vec<Row>,
    /// The columns shown: every one, less Fuel for trailers (they have no tank).
    columns: Vec<TruckColumn>,
    /// Asks for one vehicle's repair (by id); the page passes it to the session.
    session: Entity<Session>,
    /// The sort in effect, kept so new data (after a repair) stays in the same order.
    sort: Option<(TruckColumn, ColumnSort)>,
    /// The rows picked to act on together (by vehicle id).
    picks: crate::ui::Picks,
}

#[derive(Clone, Copy, PartialEq)]
enum TruckColumn {
    /// The checkbox that picks the row.
    Pick,
    Truck,
    Plate,
    Garage,
    Odometer,
    Fuel,
    Wear,
    Details,
}

impl TruckColumn {
    const ALL: [TruckColumn; 8] = [
        TruckColumn::Pick,
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
    /// Each column as it is, before the sort arrow (see `column`).
    fn base_column(&self, col_ix: usize, cx: &App) -> Column {
        match self.columns.get(col_ix) {
            Some(TruckColumn::Truck) => Column::new("truck", t!(cx, "column-truck")).sortable(),
            Some(TruckColumn::Plate) => Column::new("plate", t!(cx, "column-plate")).sortable(),
            Some(TruckColumn::Garage) => Column::new("garage", t!(cx, "column-garage")).sortable(),
            Some(TruckColumn::Odometer) => Column::new("odometer", t!(cx, "column-odometer"))
                .text_right()
                .sortable(),
            Some(TruckColumn::Fuel) => Column::new("fuel", t!(cx, "column-fuel"))
                .text_right()
                .sortable(),
            Some(TruckColumn::Details) => Column::new("details", ""),
            Some(TruckColumn::Pick) => Column::new("pick", "").resizable(false).movable(false),
            Some(TruckColumn::Wear) | None => Column::new("wear", t!(cx, "column-wear"))
                .text_right()
                .sortable(),
        }
    }

    /// A click on a column's heading: sort by it, ascending first, then the other way. Columns
    /// that can't be sorted (Details) are left alone.
    fn sort_by_header(&mut self, col_ix: usize, cx: &App) {
        let Some(column) = self.columns.get(col_ix).copied() else {
            return;
        };
        if self.base_column(col_ix, cx).sort.is_none() {
            return;
        }
        let current = self
            .sort
            .filter(|(sorted, _)| *sorted == column)
            .map(|(_, order)| order);
        let order = crate::ui::next_sort(current);
        self.sort = Some((column, order));
        self.sort_by(column, order);
    }

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
                TruckColumn::Details | TruckColumn::Pick => std::cmp::Ordering::Equal,
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
            TruckColumn::Details | TruckColumn::Pick => SharedString::default(),
            // Rounded up, so any wear at all shows (and matches the "need repairs" count).
            TruckColumn::Wear => t!(
                cx,
                "percent",
                amount = (truck.wear.worst() * 100.0).ceil() as i64
            ),
        }
    }
}

impl TruckColumn {
    /// The column's width in a table no wider than all of them together.
    fn base_width(self) -> f32 {
        match self {
            TruckColumn::Truck => 210.,
            TruckColumn::Plate => 100.,
            TruckColumn::Garage => 130.,
            TruckColumn::Odometer => 120.,
            TruckColumn::Fuel => 70.,
            TruckColumn::Details => 44.,
            TruckColumn::Pick => 40.,
            TruckColumn::Wear => 140.,
        }
    }

    /// Columns of names grow into a wider table; numbers and short values keep their width.
    fn grows(self) -> bool {
        matches!(self, TruckColumn::Truck | TruckColumn::Garage)
    }
}

impl FillsWidth for TruckTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TableDelegate for TruckTable {
    /// No empty column after the last one: the columns fill the table (`ui::fill_table`), and the
    /// kit's spare column showed as an odd extra heading.
    fn render_last_empty_col(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        gpui_kit::div()
    }

    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.trucks.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        // The sorted column shows its arrow; the others show they can be sorted.
        let columns: Vec<(f32, bool)> = self
            .columns
            .iter()
            .map(|column| (column.base_width(), column.grows()))
            .collect();
        let width = fill_widths(&columns, self.table_width)
            .get(col_ix)
            .copied()
            .unwrap_or(100.);
        let column = self.base_column(col_ix, cx).width(width);
        match (self.columns.get(col_ix), self.sort) {
            (Some(this), Some((sorted, order))) if *this == sorted => column.sort(order),
            _ => column,
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        let Some(column) = self.columns.get(col_ix).copied() else {
            return;
        };
        self.sort = (sort != ColumnSort::Default).then_some((column, sort));
        self.sort_by(column, sort);
    }

    /// Headings as the kit draws them, but the pick column's: a checkbox for every row shown.
    fn render_th(
        &mut self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if self.columns.get(col_ix) != Some(&TruckColumn::Pick) {
            // One line, ending in "…" when a long heading (in German, say) doesn't fit.
            return div()
                .w_full()
                .truncate()
                .child(self.column(col_ix, cx).name)
                .into_any_element();
        }
        let shown: Vec<String> = self
            .trucks
            .iter()
            .map(|row| row.vehicle.id.clone())
            .collect();
        let is_all = self.picks.all_of(shown.iter().map(String::as_str));
        div()
            .w_full()
            .flex()
            .justify_center()
            .on_mouse_down(gpui_kit::MouseButton::Left, crate::ui::keep_clicks)
            .child(
                crate::ui::pick_box("pick-all", is_all, t!(cx, "picks-all")).on_click(cx.listener(
                    move |table, _: &bool, _, cx| {
                        table
                            .delegate_mut()
                            .picks
                            .toggle_all(shown.iter().map(String::as_str));
                        cx.notify();
                    },
                )),
            )
            .into_any_element()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let column = self
            .columns
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
            TruckColumn::Pick => {
                let Some(truck) = truck else {
                    return cell.into_any_element();
                };
                let id = truck.id.clone();
                let label = t!(cx, "picks-row", name = truck.plate.clone());
                return cell
                    .justify_center()
                    .on_mouse_down(gpui_kit::MouseButton::Left, crate::ui::keep_clicks)
                    .child(
                        crate::ui::pick_box(("pick", row_ix), self.picks.contains(&id), label)
                            .on_click(cx.listener(move |table, _: &bool, _, cx| {
                                table.delegate_mut().picks.toggle(&id);
                                cx.notify();
                            })),
                    )
                    .into_any_element();
            }
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
                .child(crate::ui::cell_text(text))
                // Said in words, not only a colour.
                .children(row.filter(|row| row.is_yours).map(|_| {
                    Tag::primary()
                        .outline()
                        .xsmall()
                        .flex_shrink_0()
                        .child(t!(cx, "fleet-yours"))
                })),
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
                cell.justify_end()
                    .children(repair)
                    .child(crate::ui::cell_text(text))
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
                            // Quiet: every row has one, and rows also open by double-click
                            // and Enter. Named for screen readers, explained by its tooltip.
                            .icon(Icon::new(IconName::ChevronRight))
                            .accessibility_label(t!(cx, "details-open"))
                            .tooltip(t!(cx, "details-open"))
                            .on_click(move |_, window, cx| {
                                details::open_vehicle(session.clone(), &vehicle, window, cx)
                            }),
                    )
                }
                None => cell,
            },
            // Numbers line up on the right, like their headings.
            TruckColumn::Odometer | TruckColumn::Fuel => {
                cell.justify_end().child(crate::ui::cell_text(text))
            }
            _ => cell.child(crate::ui::cell_text(text)),
        }
        .into_any_element()
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, cx: &App) -> Option<SharedString> {
        let column = self.columns.get(col_ix).copied()?;
        Some(self.text(row_ix, column, cx))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, cx: &App) -> String {
        self.columns
            .get(col_ix)
            .map(|column| self.text(row_ix, *column, cx).to_string())
            .unwrap_or_default()
    }
}
