//! Coach career: everything about driving coaches, in one place (the sidebar shows it only for a
//! save with a coach career). Coach experience and skills, the coach services as the game's
//! statistics show them (passengers and distance to edit, every service perfect, every bus stop
//! found), and every service in the game's coach job log, newest first. The coaches themselves
//! are in Fleet, beside the trucks and trailers.

use gpui_kit::component::{
    Disableable as _, Sizable as _,
    button::Button,
    h_flex,
    table::{DataTable, TableState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, rems,
};
use save::{CoachSkills, MAX_COACH_SKILL_LEVEL};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::labels::{coach_skill_description, coach_skill_name};
use crate::pages::overview::{Career, skill_icon, skill_label, with_level};
use crate::session::Session;
use crate::ui::Tip as _;
use crate::ui::{
    AmountChanged, AmountField, FilterBar, FilterChanged, LevelPicker, RowLook, Section, Stat,
    TextColumn, TextRow, TextTable, page_body,
};

pub struct CoachCareerPage {
    session: Entity<Session>,
    experience: Entity<AmountField>,
    /// The services' passengers and the coach distance, as the game's statistics add them up.
    passengers: Entity<AmountField>,
    distance: Entity<AmountField>,
    /// Every service in the coach job log, with a search.
    table: Entity<TableState<TextTable>>,
    filter: Entity<FilterBar>,
    _subscriptions: Vec<Subscription>,
}

impl CoachCareerPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // A save without a coach career never shows this page; zeros keep it simple then.
        let experience_value = session.read(cx).career().coach_experience.unwrap_or(0);
        let stats = session.read(cx).coach_stats().unwrap_or_default();
        let experience = cx.new(|cx| {
            AmountField::new(
                experience_value,
                t!(cx, "coach-experience-label"),
                window,
                cx,
            )
            .min(0)
            .suffix(t!(cx, "experience-unit"))
        });
        let passengers = cx.new(|cx| {
            AmountField::new(stats.passengers, t!(cx, "coach-passengers"), window, cx)
                .id("coach-passengers-input")
                .min(0)
                .compact()
        });
        let distance = cx.new(|cx| {
            AmountField::new(stats.distance_km, t!(cx, "coach-distance"), window, cx)
                .id("coach-distance-input")
                .min(0)
                .suffix(crate::units::Unit::Kilometre.symbol(cx))
                .compact()
        });
        let columns = vec![
            TextColumn {
                key: "when",
                title: t!(cx, "column-when"),
                width: 100.,
            },
            TextColumn {
                key: "from",
                title: t!(cx, "column-from"),
                width: 140.,
            },
            TextColumn {
                key: "to",
                title: t!(cx, "column-to"),
                width: 140.,
            },
            TextColumn {
                key: "passengers",
                title: t!(cx, "column-passengers"),
                width: 100.,
            },
            TextColumn {
                key: "satisfaction",
                title: t!(cx, "column-satisfaction"),
                width: 100.,
            },
            TextColumn {
                key: "distance",
                title: t!(cx, "column-distance"),
                width: 90.,
            },
            TextColumn {
                key: "income",
                title: t!(cx, "column-income"),
                width: 100.,
            },
            TextColumn {
                key: "xp",
                title: t!(cx, "column-xp"),
                width: 70.,
            },
        ];
        let table = cx.new(|cx| TableState::new(TextTable::new(columns), window, cx));
        let placeholder = t!(cx, "coach-log-search");
        let filter = cx.new(|cx| FilterBar::new(placeholder, Vec::new(), window, cx));
        let _subscriptions = vec![
            cx.subscribe(&experience, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.edit(cx, |career| career.coach_experience = Some(value));
            }),
            cx.subscribe(&passengers, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.session
                    .update(cx, |session, cx| session.set_coach_passengers(value, cx));
            }),
            cx.subscribe(&distance, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.session
                    .update(cx, |session, cx| session.set_coach_distance(value, cx));
            }),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            // Changes made elsewhere (step buttons, Undo, Discard, saving) show in the fields too.
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.sync_fields(window, cx);
                this.reload(cx);
            }),
        ];
        let mut page = Self {
            session,
            experience,
            passengers,
            distance,
            table,
            filter,
            _subscriptions,
        };
        page.reload(cx);
        page
    }

    fn edit(&self, cx: &mut Context<Self>, edit: impl FnOnce(&mut save::Career)) {
        self.session
            .update(cx, |session, cx| session.edit_career(cx, edit));
    }

    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let experience = session.career().coach_experience.unwrap_or(0);
        let stats = session.coach_stats().unwrap_or_default();
        for (field, value) in [
            (self.experience.clone(), experience),
            (self.passengers.clone(), stats.passengers),
            (self.distance.clone(), stats.distance_km),
        ] {
            if field.read(cx).value() != value {
                field.update(cx, |field, cx| field.set_value(value, window, cx));
            }
        }
    }

    /// The log's rows again (the save, the search or the game's city names changed).
    fn reload(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let document = session.preview();
        let game = session.game();
        let currency = game.currency_symbol();
        let today = save::totals(&document).game_days;
        let services = save::coach_services(&document);
        let total = services.len();
        let rows: Vec<TextRow> = {
            let filter = self.filter.read(cx);
            services
                .iter()
                // Newest first.
                .rev()
                .map(|service| {
                    vec![
                        t!(cx, "stats-when", days = (today - service.day).max(0)),
                        game_data::city_name(cx, &service.from_city).into(),
                        game_data::city_name(cx, &service.to_city).into(),
                        SharedString::from(thousands(service.passengers)),
                        service
                            .satisfaction
                            .map(|share| t!(cx, "percent", amount = (share * 100.0).round() as i64))
                            .unwrap_or_default(),
                        crate::units::distance(cx, game, service.distance_km),
                        format!("{currency} {}", thousands(service.reward)).into(),
                        SharedString::from(thousands(service.xp)),
                    ]
                })
                .filter(|cells| filter.matches(&[&cells[1], &cells[2]], cx))
                .map(|cells| TextRow {
                    cells,
                    look: RowLook::Normal,
                })
                .collect()
        };
        let shown = rows.len();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(shown, total, cx));
        self.table.update(cx, |table, cx| {
            table.delegate_mut().rows = rows;
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    fn render_experience(&self, cx: &mut Context<Self>) -> Section {
        let experience = self.session.read(cx).career().coach_experience.unwrap_or(0);
        let section = Section::new(t!(cx, "coach-experience"))
            .description(t!(cx, "coach-experience-description"));
        with_level(
            section,
            experience,
            &self.experience,
            Career::Coach,
            &self.session,
            cx,
        )
    }

    fn render_skills(&self, cx: &mut Context<Self>) -> Option<Section> {
        let skills = self.session.read(cx).career().coach_skills?;
        let rows = (0..skills.levels().len()).map(|index| {
            let name = coach_skill_name(cx, index);
            let (field, level) = skills.levels()[index];
            h_flex()
                .justify_between()
                .gap_4()
                .py_1()
                .child(skill_label(
                    skill_icon(field, level, cx),
                    name.clone(),
                    coach_skill_description(cx, index),
                    cx,
                ))
                .child(
                    LevelPicker::new(("coach-skill", index), name)
                        .max(MAX_COACH_SKILL_LEVEL)
                        .value(level)
                        .on_change(cx.listener(move |this, level: &u8, _, cx| {
                            let level = *level;
                            this.edit(cx, |career| {
                                if let Some(skills) = &mut career.coach_skills {
                                    skills.set_level(index, level);
                                }
                            });
                        })),
                )
        });
        Some(
            Section::new(t!(cx, "coach-skills"))
                .description(t!(cx, "coach-skills-description"))
                .action(
                    Button::new("max-coach-skills")
                        .small()
                        .outline()
                        .label(t!(cx, "skills-max"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.edit(cx, |career| {
                                if career.coach_skills.is_some() {
                                    career.coach_skills = Some(CoachSkills::maxed());
                                }
                            });
                        })),
                )
                .children(rows),
        )
    }

    /// The coach services as the game's statistics show them (worked out from the services it
    /// logged), the passengers and distance to edit, and making every service perfect or finding
    /// every bus stop.
    fn render_services(&self, cx: &mut Context<Self>) -> Section {
        let session = self.session.read(cx);
        let stats = session.coach_stats().unwrap_or_default();
        let currency = session.game().currency_symbol();
        let earned: i64 = save::coach_services(&session.preview())
            .iter()
            .map(|service| service.reward)
            .sum();
        let percent = |satisfaction: Option<f32>, cx: &mut Context<Self>| -> SharedString {
            match satisfaction {
                Some(satisfaction) => t!(
                    cx,
                    "percent",
                    amount = (satisfaction * 100.0).round() as i64
                ),
                None => "–".into(),
            }
        };
        let numbers = h_flex()
            .flex_wrap()
            .gap_x_10()
            .gap_y_4()
            .child(Stat::new(
                t!(cx, "coach-service-count"),
                SharedString::from(thousands(stats.services as i64)),
            ))
            .child(Stat::new(
                t!(cx, "coach-average-satisfaction"),
                percent(stats.average_satisfaction, cx),
            ))
            .child(Stat::new(
                t!(cx, "coach-best-satisfaction"),
                percent(stats.best_satisfaction, cx),
            ))
            .child(Stat::new(
                t!(cx, "coach-earned"),
                SharedString::from(format!("{currency} {}", thousands(earned))),
            ))
            .child(Stat::new(
                t!(cx, "coach-stops"),
                t!(
                    cx,
                    "coach-stops-found",
                    found = stats.stops_found,
                    count = stats.stops_total
                ),
            ));
        let field = |label: SharedString, field: Entity<AmountField>| {
            v_flex()
                .gap_1p5()
                .w(rems(16.))
                .child(div().text_sm().child(label))
                .child(field)
        };
        let is_perfect = stats
            .average_satisfaction
            .is_none_or(|average| average >= 1.0);
        let has_found_all = stats.stops_found >= stats.stops_total;
        Section::new(t!(cx, "coach-services"))
            .description(t!(cx, "coach-services-description"))
            .child(numbers)
            .child(
                h_flex()
                    .gap_4()
                    .flex_wrap()
                    .child(field(t!(cx, "coach-passengers"), self.passengers.clone()))
                    .child(field(t!(cx, "coach-distance"), self.distance.clone())),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        Button::new("perfect-satisfaction")
                            .small()
                            .outline()
                            .label(t!(cx, "coach-perfect-satisfaction"))
                            .tip(t!(cx, "coach-perfect-satisfaction-tooltip"))
                            .disabled(is_perfect)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session.update(cx, |session, cx| {
                                    session.perfect_coach_satisfaction(cx)
                                });
                            })),
                    )
                    .child(
                        Button::new("discover-bus-stops")
                            .small()
                            .outline()
                            .label(t!(cx, "coach-discover-stops"))
                            .tip(t!(cx, "coach-discover-stops-tooltip"))
                            .disabled(has_found_all)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.discover_bus_stops(cx));
                            })),
                    ),
            )
    }

    /// Every service in the coach job log: read-only, as the game keeps it.
    fn render_log(&self, cx: &mut Context<Self>) -> Section {
        Section::new(t!(cx, "coach-log-table"))
            .description(t!(cx, "coach-log-note"))
            .child(self.filter.clone())
            .child(
                // The page scrolls, so the table gets a height of its own.
                div().h(rems(24.)).child(crate::ui::table_or_empty(
                    &self.filter,
                    crate::ui::fill_table(
                        &self.table,
                        DataTable::new(&self.table)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "coach-log-table"))
                            .into_any_element(),
                    ),
                    Some(
                        crate::ui::EmptyState::new(t!(cx, "coach-log-none"))
                            .description(t!(cx, "coach-log-none-body")),
                    ),
                    cx,
                )),
            )
    }
}

impl Render for CoachCareerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        page_body().child(
            v_flex()
                .gap_6()
                .max_w(rems(60.))
                .child(self.render_experience(cx))
                .children(self.render_skills(cx))
                .child(self.render_services(cx))
                .child(self.render_log(cx)),
        )
    }
}
