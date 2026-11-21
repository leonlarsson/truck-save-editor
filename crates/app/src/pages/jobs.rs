//! Jobs: the cargo market, every offer every company has, sortable and searchable. One action:
//! keep every offer for a week, so the job you want is still there when you get to it.

use std::collections::HashSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Pixels,
    Render, SharedString, Styled as _, Subscription, Window, div, prelude::FluentBuilder as _,
};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::session::{KEEP_OFFERS_MINUTES, Session};
use crate::ui::{FillsWidth, fill_widths};
use crate::ui::{FilterBar, FilterChanged, GameIconView, Section};

pub struct JobsPage {
    session: Entity<Session>,
    table: Entity<TableState<JobTable>>,
    filter: Entity<FilterBar>,
    /// Every offer, before the filter (for the summary).
    total: usize,
    /// Whether every offer already lasts a week.
    all_kept: bool,
    _subscriptions: Vec<Subscription>,
}

/// The Jobs filter's toggles, in order.
const FILTER_MY_GARAGES: usize = 0;
const FILTER_URGENT: usize = 1;
const FILTER_NO_FERRY: usize = 2;

/// An offer with its text made once (the game's names), and what its columns sort by.
struct Row {
    from: SharedString,
    to: SharedString,
    cargo: SharedString,
    /// Tokens and city names, for the logos and icons drawn beside them.
    from_company: String,
    from_city: SharedString,
    to_company: String,
    to_city: SharedString,
    cargo_token: String,
    distance: SharedString,
    expires: SharedString,
    urgency_text: SharedString,
    distance_km: i64,
    expires_in: i64,
    urgency: u32,
}

impl JobsPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| {
            TableState::new(
                JobTable {
                    table_width: None,
                    rows: Vec::new(),
                    sort: None,
                },
                window,
                cx,
            )
            .sortable(true)
        });
        let toggles = vec![
            t!(cx, "jobs-filter-my-garages"),
            t!(cx, "jobs-filter-urgent"),
            t!(cx, "jobs-filter-no-ferry"),
        ];
        let placeholder = t!(cx, "jobs-search");
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
        ];
        let mut page = Self {
            session,
            table,
            filter,
            total: 0,
            all_kept: false,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let (jobs, now) = session.jobs();
        let my_cities: HashSet<String> = session
            .garages()
            .into_iter()
            .filter(|garage| garage.is_owned)
            .map(|garage| garage.city)
            .collect();
        self.total = jobs.len();
        self.all_kept = jobs
            .iter()
            .all(|job| job.expires_at >= now + KEEP_OFFERS_MINUTES);

        let rows: Vec<Row> = {
            let filter = self.filter.read(cx);
            jobs.into_iter()
                .filter(|job| {
                    (!filter.is_on(FILTER_MY_GARAGES) || my_cities.contains(&job.from_city))
                        && (!filter.is_on(FILTER_URGENT) || job.urgency >= 2)
                        && (!filter.is_on(FILTER_NO_FERRY) || !job.has_ferry)
                })
                .map(|job| {
                    let place = |company: &str, city: &str| {
                        t!(
                            cx,
                            "job-place",
                            company = game_data::company_name(cx, company),
                            city = game_data::city_name(cx, city)
                        )
                    };
                    let distance = t!(cx, "distance-km", amount = thousands(job.distance_km));
                    Row {
                        from: place(&job.from_company, &job.from_city),
                        to: place(&job.to_company, &job.to_city),
                        from_company: job.from_company.clone(),
                        from_city: game_data::city_name(cx, &job.from_city).into(),
                        to_company: job.to_company.clone(),
                        to_city: game_data::city_name(cx, &job.to_city).into(),
                        cargo_token: job.cargo.clone(),
                        cargo: game_data::cargo_name(cx, &job.cargo).into(),
                        distance: if job.has_ferry {
                            t!(cx, "job-ferry", distance = distance)
                        } else {
                            distance
                        },
                        expires: expires_text(job.expires_at - now, cx),
                        urgency_text: match job.urgency {
                            0 => t!(cx, "job-urgency-standard"),
                            1 => t!(cx, "job-urgency-important"),
                            _ => t!(cx, "job-urgency-urgent"),
                        },
                        distance_km: job.distance_km,
                        expires_in: job.expires_at - now,
                        urgency: job.urgency,
                    }
                })
                .filter(|row| filter.matches(&[&row.from, &row.to, &row.cargo], cx))
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

impl JobsPage {
    /// The job being driven, if any: where from and to, the cargo, the deadline, and Cancel and
    /// Remove the deadline. A cancelled job stays shown (from the save on disk) until saving, so
    /// it's clear what Undo would bring back.
    fn render_current_job(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let session = self.session.read(cx);
        let (saved, is_cancelled) = session.saved_job()?;
        let job = session.current_job().unwrap_or(saved);
        let now = session.jobs().1;

        let place = |company: &str, city: &str, cx: &mut Context<Self>| {
            let logo = game_data::company_logo(cx, company);
            h_flex()
                .gap_2()
                .items_center()
                .children(logo.map(|logo| GameIconView::new(logo).size(1.1)))
                .child(game_data::city_name(cx, city))
        };
        let from = place(&job.from_company, &job.from_city, cx);
        let to = place(&job.to_company, &job.to_city, cx);
        let cargo_icon = game_data::cargo_icon(cx, &job.cargo);
        let cargo = h_flex()
            .gap_2()
            .items_center()
            .children(cargo_icon.map(|icon| GameIconView::new(icon).size(1.1)))
            .child(game_data::cargo_name(cx, &job.cargo));
        let deadline: SharedString = match job.deadline {
            None => t!(cx, "current-job-no-deadline"),
            Some(deadline) if deadline <= now => t!(cx, "current-job-late"),
            Some(deadline) => expires_text(deadline - now, cx),
        };
        let urgency = match job.urgency {
            0 => t!(cx, "job-urgency-standard"),
            1 => t!(cx, "job-urgency-important"),
            _ => t!(cx, "job-urgency-urgent"),
        };
        let facts = [
            (
                t!(cx, "column-distance"),
                format!("{} km", thousands(job.planned_distance_km)).into(),
            ),
            (t!(cx, "current-job-deadline"), deadline),
            (t!(cx, "column-urgency"), urgency),
        ];
        let is_urgent = job.urgency >= 2;
        let fact = |label: SharedString, value: SharedString, cx: &Context<Self>| {
            // Urgent in the danger colour, as in the table, with the word saying it too.
            let is_danger = is_urgent && label == t!(cx, "column-urgency");
            v_flex()
                .gap_0p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(
                    div()
                        .text_sm()
                        .when(is_danger, |this| this.text_color(cx.theme().danger))
                        .child(value),
                )
        };

        let description = if is_cancelled {
            t!(cx, "current-job-cancelled-note")
        } else {
            match job.kind {
                save::JobKind::FreightMarket => t!(cx, "current-job-freight-market"),
                save::JobKind::ExternalContract => t!(cx, "current-job-external"),
                save::JobKind::QuickJob => t!(cx, "current-job-quick-job"),
                save::JobKind::OwnTrailer => t!(cx, "current-job-own-trailer"),
                save::JobKind::SpecialTransport => t!(cx, "current-job-special"),
            }
        };
        let section = Section::new(t!(cx, "current-job"))
            .description(description)
            .when(!is_cancelled, |section| {
                section.action(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("remove-deadline")
                                .small()
                                .outline()
                                .label(t!(cx, "current-job-remove-deadline"))
                                .disabled(job.deadline.is_none())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.session
                                        .update(cx, |session, cx| session.remove_job_deadline(cx));
                                })),
                        )
                        .child(
                            Button::new("cancel-job")
                                .small()
                                .outline()
                                .danger()
                                .label(t!(cx, "current-job-cancel"))
                                .disabled(!job.kind.can_cancel())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.session
                                        .update(cx, |session, cx| session.cancel_job(cx));
                                })),
                        ),
                )
            })
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_x_8()
                    .gap_y_3()
                    .items_end()
                    .child(
                        v_flex().gap_1().child(cargo.text_sm().font_medium()).child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .text_sm()
                                .child(from)
                                .child(div().text_color(cx.theme().muted_foreground).child("→"))
                                .child(to),
                        ),
                    )
                    .children(
                        facts
                            .into_iter()
                            .map(|(label, value)| fact(label, value, cx)),
                    ),
            )
            // A cancelled job reads as set aside, not gone.
            .when(is_cancelled, |section| section.opacity(0.6));
        Some(section.into_any_element())
    }
}

/// `in 40 minutes`, `in 5 hours`, `in 3 days`, or `Expired`.
fn expires_text(minutes: i64, cx: &App) -> SharedString {
    if minutes <= 0 {
        t!(cx, "job-expired")
    } else if minutes < 60 {
        t!(cx, "job-expires-in", minutes = minutes)
    } else if minutes < 48 * 60 {
        t!(cx, "job-expires-in-hours", hours = minutes / 60)
    } else {
        t!(cx, "job-expires-in-days", days = minutes / (24 * 60))
    }
}

impl Render for JobsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = t!(
            cx,
            "jobs-summary",
            count = self.total,
            amount = thousands(self.total as i64)
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
                        Button::new("keep-offers")
                            .small()
                            .outline()
                            .label(t!(cx, "jobs-keep"))
                            .tooltip(t!(cx, "jobs-keep-tooltip"))
                            .disabled(self.all_kept || self.total == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.keep_offers(cx));
                            })),
                    ),
            )
            .children(self.render_current_job(cx))
            .child(self.filter.clone())
            .child(
                div().flex_1().min_h_0().child(crate::ui::table_or_empty(
                    &self.filter,
                    crate::ui::fill_table(
                        &self.table,
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "jobs-table"))
                            .into_any_element(),
                    ),
                    Some(
                        crate::ui::EmptyState::new(t!(cx, "jobs-none"))
                            .description(t!(cx, "jobs-none-body")),
                    ),
                    cx,
                )),
            )
    }
}

struct JobTable {
    /// The table's measured width, which its name columns grow to fill.
    table_width: Option<Pixels>,
    rows: Vec<Row>,
    sort: Option<(JobColumn, ColumnSort)>,
}

#[derive(Clone, Copy, PartialEq)]
enum JobColumn {
    From,
    To,
    Cargo,
    Distance,
    Expires,
    Urgency,
}

impl JobColumn {
    const ALL: [JobColumn; 6] = [
        JobColumn::From,
        JobColumn::To,
        JobColumn::Cargo,
        JobColumn::Distance,
        JobColumn::Expires,
        JobColumn::Urgency,
    ];
}

impl JobTable {
    /// Each column as it is, before the sort arrow (see `column`).
    fn base_column(&self, col_ix: usize, cx: &App) -> Column {
        match JobColumn::ALL.get(col_ix) {
            Some(JobColumn::From) => Column::new("from", t!(cx, "column-from")).sortable(),
            Some(JobColumn::To) => Column::new("to", t!(cx, "column-to")).sortable(),
            Some(JobColumn::Cargo) => Column::new("cargo", t!(cx, "column-cargo")).sortable(),
            Some(JobColumn::Distance) => Column::new("distance", t!(cx, "column-distance"))
                .text_right()
                .sortable(),
            Some(JobColumn::Expires) => Column::new("expires", t!(cx, "column-expires")).sortable(),
            Some(JobColumn::Urgency) | None => {
                Column::new("urgency", t!(cx, "column-urgency")).sortable()
            }
        }
    }

    /// A click on a column's heading: sort by it, ascending first, then the other way. Columns
    /// that can't be sorted (Details) are left alone.
    fn sort_by_header(&mut self, col_ix: usize, cx: &App) {
        let Some(column) = JobColumn::ALL.get(col_ix).copied() else {
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

    fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        if let Some((column, sort)) = self.sort {
            self.sort_by(column, sort);
        }
    }

    /// Sorts by a column; equal values keep the save's order (the sort is stable).
    fn sort_by(&mut self, column: JobColumn, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            return;
        }
        self.rows.sort_by(|a, b| {
            let ordering = match column {
                JobColumn::From => a.from.cmp(&b.from),
                JobColumn::To => a.to.cmp(&b.to),
                JobColumn::Cargo => a.cargo.cmp(&b.cargo),
                JobColumn::Distance => a.distance_km.cmp(&b.distance_km),
                JobColumn::Expires => a.expires_in.cmp(&b.expires_in),
                JobColumn::Urgency => a.urgency.cmp(&b.urgency),
            };
            if sort == ColumnSort::Descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn text(&self, row: usize, column: JobColumn) -> SharedString {
        let Some(row) = self.rows.get(row) else {
            return SharedString::default();
        };
        match column {
            JobColumn::From => row.from.clone(),
            JobColumn::To => row.to.clone(),
            JobColumn::Cargo => row.cargo.clone(),
            JobColumn::Distance => row.distance.clone(),
            JobColumn::Expires => row.expires.clone(),
            JobColumn::Urgency => row.urgency_text.clone(),
        }
    }
}

impl JobColumn {
    /// The column's width in a table no wider than all of them together.
    fn base_width(self) -> f32 {
        match self {
            JobColumn::From => 190.,
            JobColumn::To => 190.,
            JobColumn::Cargo => 140.,
            JobColumn::Distance => 120.,
            JobColumn::Expires => 105.,
            JobColumn::Urgency => 100.,
        }
    }

    /// Columns of names grow into a wider table; numbers and short values keep their width.
    fn grows(self) -> bool {
        matches!(self, JobColumn::From | JobColumn::To | JobColumn::Cargo)
    }
}

impl FillsWidth for JobTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TableDelegate for JobTable {
    fn columns_count(&self, _: &App) -> usize {
        JobColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        // The sorted column shows its arrow; the others show they can be sorted.
        let columns: Vec<(f32, bool)> = JobColumn::ALL
            .iter()
            .map(|column| (column.base_width(), column.grows()))
            .collect();
        let width = fill_widths(&columns, self.table_width)
            .get(col_ix)
            .copied()
            .unwrap_or(100.);
        let column = self.base_column(col_ix, cx).width(width);
        match (JobColumn::ALL.get(col_ix), self.sort) {
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
        let Some(column) = JobColumn::ALL.get(col_ix).copied() else {
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
        let column = JobColumn::ALL
            .get(col_ix)
            .copied()
            .unwrap_or(JobColumn::From);
        let text = self.text(row_ix, column);
        let cell = h_flex().w_full().items_center().gap_2();
        // The company's logo, then the city (the logo says who); the full text while a logo
        // loads or when the game has none.
        let place =
            |logo: Option<std::sync::Arc<gamedata::GameIcon>>, city: SharedString| match logo {
                Some(logo) => h_flex()
                    .w_full()
                    .items_center()
                    .gap_2()
                    .child(GameIconView::new(logo).size(1.1))
                    .child(city)
                    .into_any_element(),
                None => h_flex()
                    .w_full()
                    .items_center()
                    .child(crate::ui::cell_text(text.clone()))
                    .into_any_element(),
            };
        let Some(row) = self.rows.get(row_ix) else {
            return cell.into_any_element();
        };
        match column {
            JobColumn::From => {
                let (company, city) = (row.from_company.clone(), row.from_city.clone());
                return place(game_data::company_logo(cx, &company), city);
            }
            JobColumn::To => {
                let (company, city) = (row.to_company.clone(), row.to_city.clone());
                return place(game_data::company_logo(cx, &company), city);
            }
            JobColumn::Cargo => {
                let cargo = row.cargo_token.clone();
                let icon = game_data::cargo_icon(cx, &cargo);
                return cell
                    .child(
                        div()
                            .w(gpui_kit::rems(1.4))
                            .flex_shrink_0()
                            .children(icon.map(|icon| GameIconView::new(icon).size(1.1))),
                    )
                    .child(crate::ui::cell_text(text))
                    .into_any_element();
            }
            _ => {}
        }
        match column {
            JobColumn::Distance => cell.justify_end().child(crate::ui::cell_text(text)),
            // Urgent offers stand out in the theme's danger colour (readable on both themes), with
            // the word saying it too.
            JobColumn::Urgency if self.rows.get(row_ix).is_some_and(|row| row.urgency >= 2) => cell
                .text_color(cx.theme().danger)
                .child(crate::ui::cell_text(text)),
            _ => cell.child(crate::ui::cell_text(text)),
        }
        .into_any_element()
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, _: &App) -> Option<SharedString> {
        let column = JobColumn::ALL.get(col_ix).copied()?;
        Some(self.text(row_ix, column))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        JobColumn::ALL
            .get(col_ix)
            .map(|column| self.text(row_ix, *column).to_string())
            .unwrap_or_default()
    }
}
