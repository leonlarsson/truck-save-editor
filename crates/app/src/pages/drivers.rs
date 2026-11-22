//! Drivers: everyone the company has hired, with their skills and experience, and one click to max
//! them all out.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, Subscription, Window, div, rems,
};
use save::{Driver, Skills};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::labels::ADR_CLASS_COUNT;
use crate::pages::details;
use crate::session::Session;
use crate::ui::{FillsWidth, fill_widths};
use crate::ui::{FilterBar, FilterChanged, GameIconView};
use crate::workspace::OpenDetails;

pub struct DriversPage {
    session: Entity<Session>,
    table: Entity<TableState<DriverTable>>,
    filter: Entity<FilterBar>,
    /// The company's garages, to move the picked drivers to.
    garages: Entity<SelectState<SearchableVec<crate::ui::Choice>>>,
    /// How the last action on the picked rows went.
    pick_status: Option<SharedString>,
    garages_stale: bool,
    _subscriptions: Vec<Subscription>,
}

/// The Drivers filter's toggles, in order.
const FILTER_NOT_MAXED: usize = 0;

impl DriversPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = DriverTable {
            table_width: None,
            drivers: Vec::new(),
            session: session.clone(),
            sort: None,
            picks: crate::ui::Picks::default(),
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
        let garages = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
        });
        let mut _subscriptions = _subscriptions;
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
                    this.pick_status = Some(crate::labels::picks_outcome(
                        cx,
                        crate::labels::PickAction::Move,
                        moved,
                        ids.len(),
                    ));
                    this.garages
                        .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                    cx.notify();
                }
            },
        ));
        let mut page = Self {
            session,
            table,
            filter,
            garages,
            pick_status: None,
            garages_stale: true,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    /// Puts the drivers that match the filter into the table.
    fn reload(&mut self, cx: &mut Context<Self>) {
        let all = self.session.read(cx).drivers();
        let total = all.len();
        let existing: Vec<String> = all.iter().map(|driver| driver.id.clone()).collect();
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
            let delegate = table.delegate_mut();
            delegate.set_drivers(drivers);
            delegate
                .picks
                .keep_only(existing.iter().map(String::as_str));
            table.refresh(cx);
            cx.notify();
        });
        self.garages_stale = true;
        cx.notify();
    }

    /// The garage list, filled again when the garages may have changed (it needs the window).
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

    /// The bar above the table while rows are picked: what can be done to all of them.
    fn render_pick_bar(&self, cx: &mut Context<Self>) -> Option<crate::ui::PickBar> {
        let picks = self.table.read(cx).delegate().picks.clone();
        if picks.is_empty() {
            return None;
        }
        let ids = picks.ids();
        let table = self.table.clone();
        let max_ids = ids.clone();
        let remove_ids = ids;
        let mut bar = crate::ui::PickBar::new(picks.len(), move |_, cx| {
            table.update(cx, |table, cx| {
                table.delegate_mut().picks.clear();
                cx.notify();
            });
        })
        .action(
            Button::new("picks-max")
                .xsmall()
                .outline()
                .label(t!(cx, "picks-max"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    let ids = max_ids.clone();
                    this.session
                        .update(cx, |session, cx| session.max_drivers_many(&ids, cx));
                })),
        )
        .action(
            div().w(rems(13.)).child(
                Select::new(&self.garages)
                    .xsmall()
                    .placeholder(t!(cx, "picks-move"))
                    .accessibility_label(t!(cx, "picks-move")),
            ),
        )
        .action(
            Button::new("picks-remove")
                .xsmall()
                .outline()
                .danger()
                .label(t!(cx, "picks-remove"))
                .tooltip(t!(cx, "remove-tooltip"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    let removals: Vec<save::Removal> = remove_ids
                        .iter()
                        .map(|id| save::Removal::Driver(id.clone()))
                        .collect();
                    let count = removals.len();
                    let removed = this
                        .session
                        .update(cx, |session, cx| session.remove_many(removals, cx));
                    this.pick_status = Some(crate::labels::picks_outcome(
                        cx,
                        crate::labels::PickAction::Remove,
                        removed,
                        count,
                    ));
                    cx.notify();
                })),
        );
        if let Some(status) = self.pick_status.clone() {
            bar = bar.action(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(status),
            );
        }
        Some(bar)
    }
}

impl Render for DriversPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_garages(window, cx);
        let pick_bar = self.render_pick_bar(cx);
        // The summary counts every driver, whatever the filter shows.
        let drivers = self.session.read(cx).drivers();
        let count = drivers.len();
        let maxed = drivers
            .iter()
            .filter(|d| d.skills == Skills::maxed())
            .count();
        let summary = t!(cx, "drivers-summary", count = count, maxed = maxed);

        // While rows are picked, what can be done to them takes the toolbar's place.
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
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(summary),
                )
                .child(
                    Button::new("max-drivers")
                        .small()
                        .outline()
                        .label(t!(cx, "drivers-max-all"))
                        .tooltip(t!(cx, "drivers-max-all-tooltip"))
                        .disabled(maxed == count)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.session
                                .update(cx, |session, cx| session.max_all_drivers(cx));
                        })),
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
                        if let Some(item) = table.read(cx).delegate().drivers.get(row).cloned() {
                            details::open_driver(this.session.clone(), &item, window, cx);
                        }
                    }))
                    .child(crate::ui::table_or_empty(
                        &self.filter,
                        crate::ui::fill_table(
                            &self.table,
                            DataTable::new(&self.table)
                                .stripe(true)
                                .bordered(true)
                                .accessibility_label(t!(cx, "drivers-table"))
                                .into_any_element(),
                        ),
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
    /// The table's measured width, which its name columns grow to fill.
    table_width: Option<Pixels>,
    drivers: Vec<Driver>,
    /// For the Details buttons.
    session: Entity<Session>,
    sort: Option<(DriverColumn, ColumnSort)>,
    /// The rows picked to act on together (by driver id).
    picks: crate::ui::Picks,
}

#[derive(Clone, Copy, PartialEq)]
enum DriverColumn {
    /// The checkbox that picks the row.
    Pick,
    Hometown,
    Location,
    Experience,
    Skills,
    Adr,
    Details,
}

impl DriverColumn {
    const ALL: [DriverColumn; 7] = [
        DriverColumn::Pick,
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
            Some(DriverColumn::Hometown) => {
                Column::new("hometown", t!(cx, "column-hometown")).sortable()
            }
            Some(DriverColumn::Location) => {
                Column::new("location", t!(cx, "column-location")).sortable()
            }
            Some(DriverColumn::Experience) => {
                Column::new("experience", t!(cx, "column-experience"))
                    .text_right()
                    .sortable()
            }
            Some(DriverColumn::Skills) => Column::new("skills", t!(cx, "column-skills")).sortable(),
            Some(DriverColumn::Details) => Column::new("details", ""),
            Some(DriverColumn::Pick) => Column::new("pick", "").resizable(false).movable(false),
            Some(DriverColumn::Adr) | None => Column::new("adr", t!(cx, "column-adr")).sortable(),
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
                DriverColumn::Details | DriverColumn::Pick => std::cmp::Ordering::Equal,
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
            DriverColumn::Details | DriverColumn::Pick => SharedString::default(),
            DriverColumn::Adr => t!(
                cx,
                "adr-count",
                count = adr_count(&driver.skills),
                total = ADR_CLASS_COUNT
            ),
        }
    }
}

impl DriverColumn {
    /// The column's width in a table no wider than all of them together.
    fn base_width(self) -> f32 {
        match self {
            DriverColumn::Hometown => 200.,
            DriverColumn::Location => 170.,
            DriverColumn::Experience => 140.,
            DriverColumn::Skills => 150.,
            DriverColumn::Details => 44.,
            DriverColumn::Pick => 40.,
            DriverColumn::Adr => 90.,
        }
    }

    /// Columns of names grow into a wider table; numbers and short values keep their width.
    fn grows(self) -> bool {
        matches!(self, DriverColumn::Hometown | DriverColumn::Location)
    }
}

impl FillsWidth for DriverTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TableDelegate for DriverTable {
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
        DriverColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.drivers.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        // The sorted column shows its arrow; the others show they can be sorted.
        let columns: Vec<(f32, bool)> = DriverColumn::ALL
            .iter()
            .map(|column| (column.base_width(), column.grows()))
            .collect();
        let width = fill_widths(&columns, self.table_width)
            .get(col_ix)
            .copied()
            .unwrap_or(100.);
        let column = self.base_column(col_ix, cx).width(width);
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

    /// Headings as the kit draws them, but the pick column's: a checkbox for every row shown.
    fn render_th(
        &mut self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if DriverColumn::ALL.get(col_ix) != Some(&DriverColumn::Pick) {
            // One line, ending in "…" when a long heading (in German, say) doesn't fit.
            return div()
                .w_full()
                .truncate()
                .child(self.column(col_ix, cx).name)
                .into_any_element();
        }
        let shown: Vec<String> = self
            .drivers
            .iter()
            .map(|driver| driver.id.clone())
            .collect();
        let is_all = self.picks.all_of(shown.iter().map(String::as_str));
        div()
            .w_full()
            .flex()
            .justify_center()
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
        let column = DriverColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(DriverColumn::Hometown);
        let text = self.text(row_ix, column, cx);
        if column == DriverColumn::Pick {
            let Some(driver) = self.drivers.get(row_ix) else {
                return div().into_any_element();
            };
            let id = driver.id.clone();
            let label = t!(
                cx,
                "picks-row",
                name = t!(
                    cx,
                    "driver-from",
                    city = game_data::city_name(cx, &driver.hometown)
                )
            );
            return h_flex()
                .w_full()
                .h_full()
                .items_center()
                .justify_center()
                .child(
                    crate::ui::pick_box(("pick", row_ix), self.picks.contains(&id), label)
                        .on_click(cx.listener(move |table, _: &bool, _, cx| {
                            table.delegate_mut().picks.toggle(&id);
                            cx.notify();
                        })),
                )
                .into_any_element();
        }
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
                        // Quiet: every row has one, and rows also open by double-click and
                        // Enter. Named for screen readers, explained by its tooltip.
                        .icon(Icon::new(IconName::ChevronRight))
                        .accessibility_label(t!(cx, "details-open"))
                        .tooltip(t!(cx, "details-open"))
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
