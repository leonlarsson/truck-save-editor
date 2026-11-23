//! Statistics: the career in numbers (deliveries, what they earned, how far the company has
//! driven, and the coach career's services) and every delivery in the game's delivery log, newest
//! first. Read-only.

use gpui_kit::component::{
    ActiveTheme as _, h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Pixels, Render,
    SharedString, Styled as _, Subscription, Window, div,
};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{FillsWidth, FilterBar, FilterChanged, GameIconView, Stat, fill_widths};

pub struct StatisticsPage {
    session: Entity<Session>,
    table: Entity<TableState<DeliveryTable>>,
    filter: Entity<FilterBar>,
    /// The numbers at the top, made when the save changes.
    stats: Vec<(SharedString, SharedString)>,
    total: usize,
    _subscriptions: Vec<Subscription>,
}

/// The filter's toggles, in order.
const FILTER_DAMAGED: usize = 0;

/// A delivery with its text made once (the game's names), and what its columns sort by.
struct Row {
    when: SharedString,
    cargo: SharedString,
    cargo_token: String,
    from: SharedString,
    to: SharedString,
    distance: SharedString,
    income: SharedString,
    xp: SharedString,
    damage: SharedString,
    day: i64,
    distance_km: i64,
    income_amount: i64,
    xp_amount: i64,
    damage_amount: f32,
}

impl StatisticsPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| {
            TableState::new(
                DeliveryTable {
                    table_width: None,
                    rows: Vec::new(),
                    sort: None,
                },
                window,
                cx,
            )
            .sortable(true)
        });
        let toggles = vec![t!(cx, "stats-filter-damaged")];
        let placeholder = t!(cx, "stats-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            // A click on a column's heading sorts by it.
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
        ];
        let mut page = Self {
            session,
            table,
            filter,
            stats: Vec::new(),
            total: 0,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let document = session.preview();
        let currency = session.game().currency_symbol();
        let deliveries = save::deliveries(&document);
        let totals = save::totals(&document);
        let money =
            |amount: i64| -> SharedString { format!("{currency} {}", thousands(amount)).into() };

        let income: i64 = deliveries.iter().map(|delivery| delivery.income).sum();
        let xp: i64 = deliveries.iter().map(|delivery| delivery.xp).sum();
        self.total = deliveries.len();
        let mut stats = vec![
            (
                t!(cx, "stats-deliveries"),
                SharedString::from(thousands(deliveries.len() as i64)),
            ),
            (t!(cx, "stats-income"), money(income)),
            (
                t!(cx, "stats-xp"),
                t!(cx, "experience-amount", amount = thousands(xp)),
            ),
            (
                t!(cx, "stats-driven"),
                t!(cx, "distance-km", amount = thousands(totals.distance_km)),
            ),
            (
                t!(cx, "stats-days"),
                SharedString::from(thousands(totals.game_days)),
            ),
        ];
        if !deliveries.is_empty() {
            let average = income / deliveries.len() as i64;
            stats.push((t!(cx, "stats-average"), money(average)));
        }
        // The cargo hauled most often.
        let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
        for delivery in &deliveries {
            *counts.entry(delivery.cargo.as_str()).or_default() += 1;
        }
        if let Some((cargo, count)) = counts.iter().max_by_key(|(_, count)| **count) {
            stats.push((
                t!(cx, "stats-favourite"),
                t!(
                    cx,
                    "stats-favourite-value",
                    cargo = game_data::cargo_name(cx, cargo),
                    count = *count
                ),
            ));
        }
        stats.push((
            t!(cx, "stats-cancelled"),
            SharedString::from(thousands(totals.cancelled_jobs)),
        ));
        // The coach career's, when there is one (edited on Overview).
        if let Some(coach) = save::coach_stats(&document).filter(|coach| coach.services > 0) {
            stats.push((
                t!(cx, "coach-service-count"),
                SharedString::from(thousands(coach.services as i64)),
            ));
            stats.push((
                t!(cx, "coach-passengers"),
                SharedString::from(thousands(coach.passengers)),
            ));
            if let Some(average) = coach.average_satisfaction {
                stats.push((
                    t!(cx, "coach-average-satisfaction"),
                    t!(cx, "percent", amount = (average * 100.0).round() as i64),
                ));
            }
        }
        self.stats = stats;

        let today = totals.game_days;
        let rows: Vec<Row> = {
            let filter = self.filter.read(cx);
            deliveries
                .into_iter()
                // Newest first.
                .rev()
                .filter(|delivery| !filter.is_on(FILTER_DAMAGED) || delivery.cargo_damage > 0.0)
                .map(|delivery| {
                    let damage_percent = (delivery.cargo_damage * 100.0).round() as i64;
                    Row {
                        when: t!(cx, "stats-when", days = (today - delivery.day).max(0)),
                        cargo: game_data::cargo_name(cx, &delivery.cargo).into(),
                        cargo_token: delivery.cargo.clone(),
                        from: game_data::city_name(cx, &delivery.from_city).into(),
                        to: game_data::city_name(cx, &delivery.to_city).into(),
                        distance: t!(cx, "distance-km", amount = thousands(delivery.distance_km)),
                        income: money(delivery.income),
                        xp: SharedString::from(thousands(delivery.xp)),
                        damage: t!(cx, "percent", amount = damage_percent),
                        day: delivery.day,
                        distance_km: delivery.distance_km,
                        income_amount: delivery.income,
                        xp_amount: delivery.xp,
                        damage_amount: delivery.cargo_damage,
                    }
                })
                .filter(|row| filter.matches(&[&row.cargo, &row.from, &row.to], cx))
                .collect()
        };
        let shown = rows.len();
        let total = self.total;
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_rows(rows);
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for StatisticsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(
                h_flex().flex_wrap().gap_x_10().gap_y_4().children(
                    self.stats
                        .iter()
                        .map(|(label, value)| Stat::new(label.clone(), value.clone())),
                ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "stats-note")),
            )
            .child(self.filter.clone())
            .child(
                div().flex_1().min_h_0().child(crate::ui::table_or_empty(
                    &self.filter,
                    crate::ui::fill_table(
                        &self.table,
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "stats-table"))
                            .into_any_element(),
                    ),
                    Some(
                        crate::ui::EmptyState::new(t!(cx, "stats-none"))
                            .description(t!(cx, "stats-none-body")),
                    ),
                    cx,
                )),
            )
    }
}

struct DeliveryTable {
    /// The table's measured width, which its name columns grow to fill.
    table_width: Option<Pixels>,
    rows: Vec<Row>,
    sort: Option<(DeliveryColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum DeliveryColumn {
    When,
    Cargo,
    From,
    To,
    Distance,
    Income,
    Xp,
    Damage,
}

impl DeliveryColumn {
    const ALL: [DeliveryColumn; 8] = [
        DeliveryColumn::When,
        DeliveryColumn::Cargo,
        DeliveryColumn::From,
        DeliveryColumn::To,
        DeliveryColumn::Distance,
        DeliveryColumn::Income,
        DeliveryColumn::Xp,
        DeliveryColumn::Damage,
    ];

    /// The column's width in a table no wider than all of them together.
    fn base_width(self) -> f32 {
        match self {
            DeliveryColumn::When => 100.,
            DeliveryColumn::Cargo => 140.,
            DeliveryColumn::From => 140.,
            DeliveryColumn::To => 140.,
            DeliveryColumn::Distance => 90.,
            DeliveryColumn::Income => 100.,
            DeliveryColumn::Xp => 65.,
            DeliveryColumn::Damage => 75.,
        }
    }

    /// Columns of names grow into a wider table; numbers keep their width.
    fn grows(self) -> bool {
        matches!(
            self,
            DeliveryColumn::Cargo | DeliveryColumn::From | DeliveryColumn::To
        )
    }

    fn is_number(self) -> bool {
        matches!(
            self,
            DeliveryColumn::Distance
                | DeliveryColumn::Income
                | DeliveryColumn::Xp
                | DeliveryColumn::Damage
        )
    }
}

impl DeliveryTable {
    fn base_column(&self, col_ix: usize, cx: &App) -> Column {
        let column = DeliveryColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(DeliveryColumn::When);
        let (key, name) = match column {
            DeliveryColumn::When => ("when", t!(cx, "column-when")),
            DeliveryColumn::Cargo => ("cargo", t!(cx, "column-cargo")),
            DeliveryColumn::From => ("from", t!(cx, "column-from")),
            DeliveryColumn::To => ("to", t!(cx, "column-to")),
            DeliveryColumn::Distance => ("distance", t!(cx, "column-distance")),
            DeliveryColumn::Income => ("income", t!(cx, "column-income")),
            DeliveryColumn::Xp => ("xp", t!(cx, "column-xp")),
            DeliveryColumn::Damage => ("damage", t!(cx, "column-damage")),
        };
        let base = Column::new(key, name).sortable();
        if column.is_number() {
            base.text_right()
        } else {
            base
        }
    }

    /// A click on a column's heading: sort by it, ascending first, then the other way.
    fn sort_by_header(&mut self, col_ix: usize, _: &App) {
        let Some(column) = DeliveryColumn::ALL.get(col_ix).copied() else {
            return;
        };
        let current = self
            .sort
            .filter(|(sorted, _)| *sorted == column)
            .map(|(_, order)| order);
        let order = crate::ui::next_sort(current);
        self.sort = Some((column, order));
        self.sort_by(column, order);
    }

    fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        if let Some((column, sort)) = self.sort {
            self.sort_by(column, sort);
        }
    }

    /// Sorts by a column; equal values keep their order (the sort is stable).
    fn sort_by(&mut self, column: DeliveryColumn, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            return;
        }
        self.rows.sort_by(|a, b| {
            let ordering = match column {
                // Ascending is the oldest first, like the other columns' smallest first.
                DeliveryColumn::When => a.day.cmp(&b.day),
                DeliveryColumn::Cargo => a.cargo.cmp(&b.cargo),
                DeliveryColumn::From => a.from.cmp(&b.from),
                DeliveryColumn::To => a.to.cmp(&b.to),
                DeliveryColumn::Distance => a.distance_km.cmp(&b.distance_km),
                DeliveryColumn::Income => a.income_amount.cmp(&b.income_amount),
                DeliveryColumn::Xp => a.xp_amount.cmp(&b.xp_amount),
                DeliveryColumn::Damage => a.damage_amount.total_cmp(&b.damage_amount),
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: DeliveryColumn) -> SharedString {
        let Some(row) = self.rows.get(row) else {
            return SharedString::default();
        };
        match column {
            DeliveryColumn::When => row.when.clone(),
            DeliveryColumn::Cargo => row.cargo.clone(),
            DeliveryColumn::From => row.from.clone(),
            DeliveryColumn::To => row.to.clone(),
            DeliveryColumn::Distance => row.distance.clone(),
            DeliveryColumn::Income => row.income.clone(),
            DeliveryColumn::Xp => row.xp.clone(),
            DeliveryColumn::Damage => row.damage.clone(),
        }
    }
}

impl FillsWidth for DeliveryTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TableDelegate for DeliveryTable {
    /// No empty column after the last one: the columns fill the table (`ui::fill_table`).
    fn render_last_empty_col(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        gpui_kit::div()
    }

    fn columns_count(&self, _: &App) -> usize {
        DeliveryColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        let columns: Vec<(f32, bool)> = DeliveryColumn::ALL
            .iter()
            .map(|column| (column.base_width(), column.grows()))
            .collect();
        let width = fill_widths(&columns, self.table_width)
            .get(col_ix)
            .copied()
            .unwrap_or(100.);
        let column = self.base_column(col_ix, cx).width(width);
        match (DeliveryColumn::ALL.get(col_ix), self.sort) {
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
        let Some(column) = DeliveryColumn::ALL.get(col_ix).copied() else {
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
        let column = DeliveryColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(DeliveryColumn::When);
        let text = self.text(row_ix, column);
        let cell = h_flex().w_full().items_center().gap_2();
        let Some(row) = self.rows.get(row_ix) else {
            return cell.into_any_element();
        };
        // The cargo's icon before its name, in a fixed-width slot so the names line up while
        // icons load or when the game has none. (No company logos: they're wide, and the city
        // names need the room.)
        let picture = (column == DeliveryColumn::Cargo).then(|| {
            let token = row.cargo_token.clone();
            game_data::cargo_icon(cx, &token)
        });
        let is_damaged = self
            .rows
            .get(row_ix)
            .is_some_and(|row| row.damage_amount > 0.0);
        match picture {
            Some(icon) => cell
                .child(
                    div()
                        .w(gpui_kit::rems(1.4))
                        .flex_shrink_0()
                        .children(icon.map(|icon| GameIconView::new(icon).size(1.1))),
                )
                .child(crate::ui::cell_text(text)),
            None if column.is_number() => {
                let cell = cell.justify_end();
                // Undamaged cargo recedes; damage stands out (the number says it too).
                if column == DeliveryColumn::Damage && !is_damaged {
                    cell.text_color(cx.theme().muted_foreground)
                } else {
                    cell
                }
                .child(crate::ui::cell_text(text))
            }
            None => cell.child(crate::ui::cell_text(text)),
        }
        .into_any_element()
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, _: &App) -> Option<SharedString> {
        let column = DeliveryColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        DeliveryColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column).to_string())
            .unwrap_or_default()
    }
}
