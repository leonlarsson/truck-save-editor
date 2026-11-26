//! Garages: every city's garage, whether it's owned, how big it is and how full. Buy them all or
//! upgrade the owned ones to large in one go.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, Subscription, Window, div,
};
use save::Garage;

use crate::game_data;
use crate::i18n::t;
use crate::pages::details;
use crate::session::Session;
use crate::ui::Tip as _;
use crate::ui::{FillsWidth, fill_widths};
use crate::ui::{FilterBar, FilterChanged};
use crate::workspace::OpenDetails;

pub struct GaragesPage {
    session: Entity<Session>,
    table: Entity<TableState<GarageTable>>,
    /// The headquarters: one of the cities with an owned garage.
    headquarters: Entity<SelectState<SearchableVec<CityOption>>>,
    filter: Entity<FilterBar>,
    /// How the last action on the picked rows went.
    _subscriptions: Vec<Subscription>,
}

/// The Garages filter's toggles, in order.
const FILTER_OWNED: usize = 0;
const FILTER_NOT_LARGE: usize = 1;

impl GaragesPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = GarageTable {
            table_width: None,
            garages: Vec::new(),
            session: session.clone(),
            sort: None,
            picks: crate::ui::Picks::default(),
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
            // A click on a column's heading sorts by it (the table itself would select the
            // column, which nothing here uses).
            cx.subscribe_in(&table, window, |this, table, event: &TableEvent, _, cx| {
                if let TableEvent::SelectColumn(col_ix) = event {
                    let col_ix = *col_ix;
                    let is_reset = table.update(cx, |table, cx| {
                        let is_reset = table.delegate_mut().sort_by_header(col_ix, cx);
                        table.clear_selection(cx);
                        table.refresh(cx);
                        cx.notify();
                        is_reset
                    });
                    // Sorting off: the rows go back to the save's own order.
                    if is_reset {
                        this.reload(cx);
                    }
                }
            }),
            cx.subscribe_in(
                &table,
                window,
                |this, table, event: &TableEvent, window, cx| {
                    if let TableEvent::DoubleClickedRow(row) = event
                        && let Some(garage) = table.read(cx).delegate().garages.get(*row).cloned()
                    {
                        details::open_garage(this.session.clone(), &garage, window, cx);
                    }
                },
            ),
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
        let mut _subscriptions = _subscriptions;
        _subscriptions.push(cx.observe(&table, |_, _, cx| cx.notify()));
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
        let existing: Vec<String> = all.iter().map(|garage| garage.id.clone()).collect();
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
            let delegate = table.delegate_mut();
            delegate.set_garages(garages);
            delegate
                .picks
                .keep_only(existing.iter().map(String::as_str));
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

    /// The bar above the table while rows are picked: what can be done to all of them.
    fn render_pick_bar(&self, cx: &mut Context<Self>) -> Option<crate::ui::PickBar> {
        let picks = self.table.read(cx).delegate().picks.clone();
        if picks.is_empty() {
            return None;
        }
        let ids = picks.ids();
        let table = self.table.clone();
        let large_ids = ids.clone();
        let remove_ids = ids;
        let bar = crate::ui::PickBar::new(picks.len(), move |_, cx| {
            table.update(cx, |table, cx| {
                table.delegate_mut().picks.clear();
                cx.notify();
            });
        })
        .action(
            Button::new("picks-large")
                .small()
                .outline()
                .label(t!(cx, "picks-large"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    let ids = large_ids.clone();
                    this.session
                        .update(cx, |session, cx| session.upgrade_garages_many(&ids, cx));
                    let title = t!(cx, "picks-done-large", count = ids.len());
                    crate::ui::picks_done(title, true, window, cx);
                })),
        )
        .action(
            Button::new("picks-remove")
                .small()
                .outline()
                .danger()
                .label(t!(cx, "picks-remove"))
                .tip(t!(cx, "remove-tooltip"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    let removals: Vec<save::Removal> = remove_ids
                        .iter()
                        .map(|id| save::Removal::Garage(id.clone()))
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
        Some(bar)
    }

    fn upgrade(&mut self, buy_all: bool, cx: &mut Context<Self>) {
        self.session
            .update(cx, |session, cx| session.upgrade_garages(buy_all, cx));
    }
}

impl Render for GaragesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pick_bar = self.render_pick_bar(cx);
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
                                .tip(t!(cx, "garages-upgrade-owned-tooltip"))
                                .disabled(owned == large)
                                .on_click(cx.listener(|this, _, _, cx| this.upgrade(false, cx))),
                        )
                        .child(
                            Button::new("buy-all")
                                .small()
                                .outline()
                                .label(t!(cx, "garages-buy-all"))
                                .tip(t!(cx, "garages-buy-all-tooltip"))
                                .disabled(large == count)
                                .on_click(cx.listener(|this, _, _, cx| this.upgrade(true, cx))),
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
                        if let Some(item) = table.read(cx).delegate().garages.get(row).cloned() {
                            details::open_garage(this.session.clone(), &item, window, cx);
                        }
                    }))
                    .child(crate::ui::table_or_empty(
                        &self.filter,
                        crate::ui::fill_table(
                            &self.table,
                            DataTable::new(&self.table)
                                .stripe(true)
                                .bordered(true)
                                .accessibility_label(t!(cx, "garages-table"))
                                .into_any_element(),
                        ),
                        None,
                        cx,
                    )),
            )
    }
}

struct GarageTable {
    /// The table's measured width, which its name columns grow to fill.
    table_width: Option<Pixels>,
    garages: Vec<Garage>,
    /// For the Details buttons.
    session: Entity<Session>,
    sort: Option<(GarageColumn, ColumnSort)>,
    /// The rows picked to act on together (by garage id).
    picks: crate::ui::Picks,
}

#[derive(Clone, Copy, PartialEq)]
enum GarageColumn {
    /// The checkbox that picks the row.
    Pick,
    City,
    Size,
    Trucks,
    Drivers,
    Details,
}

impl GarageColumn {
    const ALL: [GarageColumn; 6] = [
        GarageColumn::Pick,
        GarageColumn::City,
        GarageColumn::Size,
        GarageColumn::Trucks,
        GarageColumn::Drivers,
        GarageColumn::Details,
    ];
}

impl GarageTable {
    /// Each column as it is, before the sort arrow (see `column`).
    fn base_column(&self, col_ix: usize, cx: &App) -> Column {
        match GarageColumn::ALL.get(col_ix) {
            Some(GarageColumn::City) => Column::new("city", t!(cx, "column-city")).sortable(),
            Some(GarageColumn::Size) => {
                Column::new("size", t!(cx, "column-garage-size")).sortable()
            }
            Some(GarageColumn::Trucks) => Column::new("trucks", t!(cx, "column-trucks"))
                .text_right()
                .sortable(),
            Some(GarageColumn::Details) => Column::new("details", ""),
            Some(GarageColumn::Pick) => Column::new("pick", "").resizable(false).movable(false),
            Some(GarageColumn::Drivers) | None => Column::new("drivers", t!(cx, "column-drivers"))
                .text_right()
                .sortable(),
        }
    }

    /// A click on a column's heading: sort by it: ascending, then descending, then not at all. Columns
    /// that can't be sorted (Details) are left alone.
    fn sort_by_header(&mut self, col_ix: usize, cx: &App) -> bool {
        let Some(column) = GarageColumn::ALL.get(col_ix).copied() else {
            return false;
        };
        if self.base_column(col_ix, cx).sort.is_none() {
            return false;
        }
        let current = self
            .sort
            .filter(|(sorted, _)| *sorted == column)
            .map(|(_, order)| order);
        let order = crate::ui::next_sort(current);
        // The third click takes the sort off: the page then lists the rows in the save's order
        // again (this says so by answering `true`).
        self.sort = (order != ColumnSort::Default).then_some((column, order));
        self.sort_by(column, order);
        order == ColumnSort::Default
    }

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
                GarageColumn::Details | GarageColumn::Pick => std::cmp::Ordering::Equal,
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
            GarageColumn::Details | GarageColumn::Pick => SharedString::default(),
        }
    }
}

impl GarageColumn {
    /// The column's width in a table no wider than all of them together.
    fn base_width(self) -> f32 {
        match self {
            GarageColumn::City => 220.,
            GarageColumn::Size => 150.,
            GarageColumn::Trucks => 110.,
            GarageColumn::Details => 44.,
            GarageColumn::Pick => 40.,
            GarageColumn::Drivers => 110.,
        }
    }

    /// Columns of names grow into a wider table; numbers and short values keep their width.
    fn grows(self) -> bool {
        matches!(self, GarageColumn::City)
    }
}

impl FillsWidth for GarageTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TableDelegate for GarageTable {
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
        GarageColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.garages.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        // The sorted column shows its arrow; the others show they can be sorted.
        let columns: Vec<(f32, bool)> = GarageColumn::ALL
            .iter()
            .map(|column| (column.base_width(), column.grows()))
            .collect();
        let width = fill_widths(&columns, self.table_width)
            .get(col_ix)
            .copied()
            .unwrap_or(100.);
        let column = self.base_column(col_ix, cx).width(width);
        match (GarageColumn::ALL.get(col_ix), self.sort) {
            (Some(this), Some((sorted, order))) if *this == sorted => column.sort(order),
            _ => column,
        }
    }

    /// The kit calls this for a click on a heading's sort button. That click is also a click on
    /// the heading itself, which sorts (`sort_by_header`, from the page's `SelectColumn`
    /// subscription); sorting here too made the two undo each other, so the button seemed to
    /// do nothing. The heading's way is the one way.
    fn perform_sort(
        &mut self,
        _: usize,
        _: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
    }

    /// Headings as the kit draws them, but the pick column's: a checkbox for every row shown.
    fn render_th(
        &mut self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if GarageColumn::ALL.get(col_ix) != Some(&GarageColumn::Pick) {
            // One line, ending in "…" when a long heading (in German, say) doesn't fit.
            return div()
                .w_full()
                .truncate()
                .child(self.column(col_ix, cx).name)
                .into_any_element();
        }
        let shown: Vec<String> = self
            .garages
            .iter()
            .map(|garage| garage.id.clone())
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
        let column = GarageColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(GarageColumn::City);
        if column == GarageColumn::Pick {
            let Some(garage) = self.garages.get(row_ix) else {
                return div().into_any_element();
            };
            let id = garage.id.clone();
            let label = t!(
                cx,
                "picks-row",
                name = game_data::city_name(cx, &garage.city)
            );
            return h_flex()
                .w_full()
                .h_full()
                .items_center()
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
        if column == GarageColumn::Details {
            let Some(garage) = self.garages.get(row_ix).cloned() else {
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
                        .tip(t!(cx, "details-open"))
                        .on_click(move |_, window, cx| {
                            details::open_garage(session.clone(), &garage, window, cx)
                        }),
                )
                .into_any_element();
        }
        let is_muted = self.garages.get(row_ix).is_some_and(|g| !g.is_owned);
        let text = self.text(row_ix, column, cx);
        if is_muted {
            div()
                .text_color(cx.theme().muted_foreground)
                .child(crate::ui::cell_text(text))
                .into_any_element()
        } else {
            div().child(crate::ui::cell_text(text)).into_any_element()
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
