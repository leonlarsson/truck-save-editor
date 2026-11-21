//! Jobs: the cargo market, every offer every company has, sortable and searchable. One action:
//! keep every offer for a week, so the job you want is still there when you get to it.

use std::collections::HashSet;

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

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::session::{KEEP_OFFERS_MINUTES, Session};
use crate::ui::{FilterBar, FilterChanged, GameIconView};

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
            .child(self.filter.clone())
            .child(
                div().flex_1().min_h_0().child(
                    DataTable::new(&self.table)
                        .stripe(true)
                        .bordered(true)
                        .accessibility_label(t!(cx, "jobs-table")),
                ),
            )
    }
}

struct JobTable {
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

impl TableDelegate for JobTable {
    fn columns_count(&self, _: &App) -> usize {
        JobColumn::ALL.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, cx: &App) -> Column {
        match JobColumn::ALL.get(col_ix) {
            Some(JobColumn::From) => Column::new("from", t!(cx, "column-from"))
                .width(190.)
                .sortable(),
            Some(JobColumn::To) => Column::new("to", t!(cx, "column-to"))
                .width(190.)
                .sortable(),
            Some(JobColumn::Cargo) => Column::new("cargo", t!(cx, "column-cargo"))
                .width(140.)
                .sortable(),
            Some(JobColumn::Distance) => Column::new("distance", t!(cx, "column-distance"))
                .width(120.)
                .text_right()
                .sortable(),
            Some(JobColumn::Expires) => Column::new("expires", t!(cx, "column-expires"))
                .width(105.)
                .sortable(),
            Some(JobColumn::Urgency) | None => Column::new("urgency", t!(cx, "column-urgency"))
                .width(100.)
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
                    .child(text.clone())
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
                    .child(text)
                    .into_any_element();
            }
            _ => {}
        }
        match column {
            JobColumn::Distance => cell.justify_end().child(text),
            // Urgent offers stand out in the theme's danger colour (readable on both themes), with
            // the word saying it too.
            JobColumn::Urgency if self.rows.get(row_ix).is_some_and(|row| row.urgency >= 2) => {
                cell.text_color(cx.theme().danger).child(text)
            }
            _ => cell.child(text),
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
