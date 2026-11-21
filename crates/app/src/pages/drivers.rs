//! Drivers: everyone the company has hired, with their skills and experience, and one click to max
//! them all out.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div, rems,
};
use save::{Driver, Skills};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::labels::ADR_CLASS_COUNT;
use crate::pages::details;
use crate::session::Session;
use crate::ui::{FilterBar, FilterChanged, GameIconView};
use crate::workspace::OpenDetails;

pub struct DriversPage {
    session: Entity<Session>,
    table: Entity<TableState<DriverTable>>,
    filter: Entity<FilterBar>,
    _subscriptions: Vec<Subscription>,
}

/// The Drivers filter's toggles, in order.
const FILTER_NOT_MAXED: usize = 0;

impl DriversPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = DriverTable {
            drivers: Vec::new(),
            session: session.clone(),
            sort: None,
        };
        let table = cx.new(|cx| TableState::new(delegate, window, cx).sortable(true));
        let toggles = vec![t!(cx, "drivers-filter-not-maxed")];
        let placeholder = t!(cx, "drivers-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
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
                        && let Some(driver) = table.read(cx).delegate().drivers.get(*row).cloned()
                    {
                        details::open_driver(this.session.clone(), &driver, window, cx);
                    }
                },
            ),
            game_data::observe(cx, |this, cx| this.reload(cx)),
        ];
        let mut page = Self {
            session,
            table,
            filter,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    /// Puts the drivers that match the filter into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let all = self.session.read(cx).drivers();
        let total = all.len();
        let drivers: Vec<Driver> = {
            let filter = self.filter.read(cx);
            all.into_iter()
                .filter(|driver| {
                    let hometown = game_data::city_name(cx, &driver.hometown);
                    let location = game_data::city_name(cx, &driver.current_city);
                    (!filter.is_on(FILTER_NOT_MAXED) || driver.skills != Skills::maxed())
                        && filter.matches(&[&hometown, &location], cx)
                })
                .collect()
        };
        let shown = drivers.len();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_drivers(drivers);
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for DriversPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The summary counts every driver, whatever the filter shows.
        let drivers = self.session.read(cx).drivers();
        let count = drivers.len();
        let maxed = drivers
            .iter()
            .filter(|d| d.skills == Skills::maxed())
            .count();
        let summary = t!(cx, "drivers-summary", count = count, maxed = maxed);

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
                        Button::new("max-drivers")
                            .small()
                            .primary()
                            .label(t!(cx, "drivers-max-all"))
                            .tooltip(t!(cx, "drivers-max-all-tooltip"))
                            .disabled(maxed == count)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.max_all_drivers(cx));
                            })),
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
                        if let Some(item) = table.read(cx).delegate().drivers.get(row).cloned() {
                            details::open_driver(this.session.clone(), &item, window, cx);
                        }
                    }))
                    .child(crate::ui::table_or_empty(
                        &self.filter,
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "drivers-table"))
                            .into_any_element(),
                        Some(
                            crate::ui::EmptyState::new(t!(cx, "drivers-none"))
                                .description(t!(cx, "drivers-none-body")),
                        ),
                        cx,
                    )),
            )
    }
}

struct DriverTable {
    drivers: Vec<Driver>,
    /// For the Details buttons.
    session: Entity<Session>,
    sort: Option<(DriverColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum DriverColumn {
    Hometown,
    Location,
    Experience,
    Skills,
    Adr,
    Details,
}

impl DriverColumn {
    const ALL: [DriverColumn; 6] = [
        DriverColumn::Hometown,
        DriverColumn::Location,
        DriverColumn::Experience,
        DriverColumn::Skills,
        DriverColumn::Adr,
        DriverColumn::Details,
    ];
}

fn skill_total(skills: &Skills) -> u32 {
    skills.levels().iter().map(|(_, l)| u32::from(*l)).sum()
}

fn adr_count(skills: &Skills) -> u32 {
    skills.adr.count_ones()
}

impl DriverTable {
    /// Each column as it is, before the sort arrow (see `column`).
    fn base_column(&self, col_ix: usize, cx: &App) -> Column {
        match DriverColumn::ALL.get(col_ix) {
            Some(DriverColumn::Hometown) => Column::new("hometown", t!(cx, "column-hometown"))
                .width(200.)
                .sortable(),
            Some(DriverColumn::Location) => Column::new("location", t!(cx, "column-location"))
                .width(170.)
                .sortable(),
            Some(DriverColumn::Experience) => {
                Column::new("experience", t!(cx, "column-experience"))
                    .width(140.)
                    .text_right()
                    .sortable()
            }
            Some(DriverColumn::Skills) => Column::new("skills", t!(cx, "column-skills"))
                .width(150.)
                .sortable(),
            Some(DriverColumn::Details) => Column::new("details", "").width(80.),
            Some(DriverColumn::Adr) | None => Column::new("adr", t!(cx, "column-adr"))
                .width(90.)
                .sortable(),
        }
    }

    /// A click on a column's heading: sort by it, ascending first, then the other way. Columns
    /// that can't be sorted (Details) are left alone.
    fn sort_by_header(&mut self, col_ix: usize, cx: &App) {
        let Some(column) = DriverColumn::ALL.get(col_ix).copied() else {
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

    fn set_drivers(&mut self, drivers: Vec<Driver>) {
        self.drivers = drivers;
        if let Some((column, sort)) = self.sort {
            self.sort_by(column, sort);
        }
    }

    /// Sorts by a column; equal values keep the save's order (the sort is stable).
    fn sort_by(&mut self, column: DriverColumn, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            return;
        }
        self.drivers.sort_by(|a, b| {
            let ordering = match column {
                DriverColumn::Hometown => a.hometown.cmp(&b.hometown),
                DriverColumn::Location => a.current_city.cmp(&b.current_city),
                DriverColumn::Experience => a.experience.cmp(&b.experience),
                DriverColumn::Skills => skill_total(&a.skills).cmp(&skill_total(&b.skills)),
                DriverColumn::Adr => adr_count(&a.skills).cmp(&adr_count(&b.skills)),
                DriverColumn::Details => std::cmp::Ordering::Equal,
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: DriverColumn, cx: &App) -> SharedString {
        let Some(driver) = self.drivers.get(row) else {
            return SharedString::default();
        };
        let levels = driver.skills.levels();
        match column {
            DriverColumn::Hometown => game_data::city_name(cx, &driver.hometown).into(),
            DriverColumn::Location => game_data::city_name(cx, &driver.current_city).into(),
            DriverColumn::Experience => t!(
                cx,
                "experience-amount",
                amount = thousands(driver.experience)
            ),
            DriverColumn::Skills => t!(
                cx,
                "skills-short",
                long = levels[0].1,
                value = levels[1].1,
                fragile = levels[2].1,
                urgent = levels[3].1,
                eco = levels[4].1
            ),
            DriverColumn::Details => SharedString::default(),
            DriverColumn::Adr => t!(
                cx,
                "adr-count",
                count = adr_count(&driver.skills),
                total = ADR_CLASS_COUNT
            ),
        }
    }
}

impl TableDelegate for DriverTable {
    fn columns_count(&self, _: &App) -> usize {
        DriverColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.drivers.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        // The sorted column shows its arrow; the others show they can be sorted.
        let column = self.base_column(col_ix, cx);
        match (DriverColumn::ALL.get(col_ix), self.sort) {
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
        let Some(column) = DriverColumn::ALL.get(col_ix).copied() else {
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
        let column = DriverColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(DriverColumn::Hometown);
        let text = self.text(row_ix, column, cx);
        if column == DriverColumn::Details {
            let Some(driver) = self.drivers.get(row_ix).cloned() else {
                return div().into_any_element();
            };
            let session = self.session.clone();
            return h_flex()
                .w_full()
                .h_full()
                .items_center()
                .justify_end()
                .child(
                    Button::new(("details", row_ix))
                        .xsmall()
                        .ghost()
                        .label(t!(cx, "details"))
                        .on_click(move |_, window, cx| {
                            details::open_driver(session.clone(), &driver, window, cx)
                        }),
                )
                .into_any_element();
        }
        if column != DriverColumn::Hometown {
            return div().child(crate::ui::cell_text(text)).into_any_element();
        }
        // The driver's face from the game, in a slot of its own so
        // rows line up while faces load.
        let face = self
            .drivers
            .get(row_ix)
            .and_then(|driver| game_data::driver_face(cx, &driver.id));
        h_flex()
            .gap_2()
            .items_center()
            .child(
                div()
                    .size(rems(1.6))
                    .flex_shrink_0()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().muted)
                    .children(
                        face.map(|face| {
                            GameIconView::new(face).size(1.6).rounded(cx.theme().radius)
                        }),
                    ),
            )
            .child(crate::ui::cell_text(text))
            .into_any_element()
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, cx: &App) -> Option<SharedString> {
        let column = DriverColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column, cx))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, cx: &App) -> String {
        DriverColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column, cx).to_string())
            .unwrap_or_default()
    }
}
