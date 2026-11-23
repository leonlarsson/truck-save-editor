//! Overview: the big levers first. Money, experience, skills and ADR, each one click from maxed.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _,
    button::Button,
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState},
    progress::Progress,
    v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, rems,
};
use save::{ADR_ALL, CoachSkills, LevelTable, MAX_COACH_SKILL_LEVEL, MAX_SKILL_LEVEL, Skills};

use crate::format::{compact, thousands};
use crate::game_data;
use crate::i18n::t;
use crate::labels::{
    ADR_CLASS_COUNT, adr_class, coach_skill_description, coach_skill_name, skill_description,
    skill_name,
};
use crate::session::Session;
use crate::ui::{
    AmountChanged, AmountField, GameIconView, LevelPicker, Section, Stat, StepButtons, page_body,
};

pub struct OverviewPage {
    session: Entity<Session>,
    money: Entity<AmountField>,
    experience: Entity<AmountField>,
    /// Coach experience (Coaches DLC); `None` for saves from before coaches.
    coach_experience: Option<Entity<AmountField>>,
    /// The coach services' passengers and the coach distance; `None` without a coach career.
    coach_passengers: Option<Entity<AmountField>>,
    coach_distance: Option<Entity<AmountField>>,
    /// The company's name (the profile's); `None` when the profile file couldn't be read.
    company: Option<Entity<InputState>>,
    /// The profile's name (in the profile file, like the company's).
    profile: Option<Entity<InputState>>,
    _subscriptions: Vec<Subscription>,
}

impl OverviewPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (career, currency) = {
            let session = session.read(cx);
            (session.career().clone(), session.game().currency_symbol())
        };
        let money = cx.new(|cx| {
            AmountField::new(career.money, t!(cx, "money-label"), window, cx)
                .id("money-input")
                .prefix(currency)
                .hint(t!(cx, "amount-hint"))
        });
        let experience = cx.new(|cx| {
            AmountField::new(career.experience, t!(cx, "experience-label"), window, cx)
                .min(0)
                .suffix(t!(cx, "experience-unit"))
        });
        let coach_experience = career.coach_experience.map(|value| {
            cx.new(|cx| {
                AmountField::new(value, t!(cx, "coach-experience-label"), window, cx)
                    .min(0)
                    .suffix(t!(cx, "experience-unit"))
            })
        });

        let coach = session.read(cx).coach_stats();
        let coach_passengers = coach.as_ref().map(|stats| {
            cx.new(|cx| {
                AmountField::new(stats.passengers, t!(cx, "coach-passengers"), window, cx)
                    .id("coach-passengers-input")
                    .min(0)
                    .compact()
            })
        });
        let coach_distance = coach.as_ref().map(|stats| {
            cx.new(|cx| {
                AmountField::new(stats.distance_km, t!(cx, "coach-distance"), window, cx)
                    .id("coach-distance-input")
                    .min(0)
                    .suffix("km")
                    .compact()
            })
        });

        let mut _subscriptions = vec![
            cx.subscribe(&money, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.edit(cx, |career| career.money = value);
            }),
            cx.subscribe(&experience, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.edit(cx, |career| career.experience = value);
            }),
            // Changes made elsewhere (step buttons, Discard, saving) show in the fields too.
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.sync_fields(window, cx);
                cx.notify();
            }),
        ];
        if let Some(field) = &coach_experience {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.edit(cx, |career| career.coach_experience = Some(value));
            }));
        }
        if let Some(field) = &coach_passengers {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.session
                    .update(cx, |session, cx| session.set_coach_passengers(value, cx));
            }));
        }
        if let Some(field) = &coach_distance {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(value), cx| {
                let value = *value;
                this.session
                    .update(cx, |session, cx| session.set_coach_distance(value, cx));
            }));
        }
        let company = session
            .read(cx)
            .company_name()
            .map(|name| cx.new(|cx| InputState::new(window, cx).default_value(name)));
        if let Some(input) = &company {
            _subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, input, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        let name = input.read(cx).value().to_string();
                        this.session
                            .update(cx, |session, cx| session.set_company_name(name, cx));
                    }
                },
            ));
        }

        let profile = session
            .read(cx)
            .profile_name()
            .map(|name| cx.new(|cx| InputState::new(window, cx).default_value(name)));
        if let Some(input) = &profile {
            _subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, input, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        let name = input.read(cx).value().to_string();
                        this.session
                            .update(cx, |session, cx| session.set_profile_name(name, cx));
                    }
                },
            ));
        }

        Self {
            session,
            money,
            experience,
            coach_experience,
            coach_passengers,
            coach_distance,
            company,
            profile,
            _subscriptions,
        }
    }

    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let career = self.session.read(cx).career().clone();
        let mut fields = vec![
            (self.money.clone(), career.money),
            (self.experience.clone(), career.experience),
        ];
        if let (Some(field), Some(value)) = (&self.coach_experience, career.coach_experience) {
            fields.push((field.clone(), value));
        }
        if let Some(stats) = self.session.read(cx).coach_stats() {
            if let Some(field) = &self.coach_passengers {
                fields.push((field.clone(), stats.passengers));
            }
            if let Some(field) = &self.coach_distance {
                fields.push((field.clone(), stats.distance_km));
            }
        }
        for (field, value) in fields {
            if field.read(cx).value() != value {
                field.update(cx, |field, cx| field.set_value(value, window, cx));
            }
        }
        if let (Some(input), Some(name)) = (&self.company, self.session.read(cx).company_name())
            && input.read(cx).value().as_ref() != name
        {
            input.update(cx, |input, cx| input.set_value(name, window, cx));
        }
        if let (Some(input), Some(name)) = (&self.profile, self.session.read(cx).profile_name())
            && input.read(cx).value().as_ref() != name
        {
            input.update(cx, |input, cx| input.set_value(name, window, cx));
        }
    }

    fn quick_start_button(&self, cx: &mut Context<Self>) -> Button {
        Button::new("quick-start")
            .small()
            .outline()
            .icon(Icon::new(IconName::Rocket))
            .label(t!(cx, "quick-start-open"))
            .on_click(cx.listener(|this, _, window, cx| {
                crate::quick_start::open(this.session.clone(), window, cx)
            }))
    }

    fn edit(&self, cx: &mut Context<Self>, edit: impl FnOnce(&mut save::Career)) {
        self.session
            .update(cx, |session, cx| session.edit_career(cx, edit));
    }

    fn render_money(&self, cx: &mut Context<Self>) -> Section {
        let steps: [i64; 3] = [100_000, 1_000_000, 10_000_000];
        Section::new(t!(cx, "money"))
            .description(t!(cx, "money-description"))
            .child(self.money.clone())
            .child(h_flex().gap_2().flex_wrap().children(steps.map(|step| {
                StepButtons::new(("money-step", step as usize), compact(step)).on_step(cx.listener(
                    move |this, direction: &i64, _, cx| {
                        let direction = *direction;
                        this.edit(cx, |career| {
                            career.money = career.money.saturating_add(direction * step);
                        });
                    },
                ))
            })))
    }

    fn render_experience(&self, cx: &mut Context<Self>) -> Section {
        let experience = self.session.read(cx).career().experience;
        let section =
            Section::new(t!(cx, "experience")).description(t!(cx, "experience-description"));
        self.with_level(
            section,
            experience,
            &self.experience.clone(),
            Career::Truck,
            cx,
        )
    }

    fn render_coach_experience(
        &self,
        field: &Entity<AmountField>,
        cx: &mut Context<Self>,
    ) -> Section {
        let experience = self.session.read(cx).career().coach_experience.unwrap_or(0);
        let section = Section::new(t!(cx, "coach-experience"))
            .description(t!(cx, "coach-experience-description"));
        self.with_level(section, experience, field, Career::Coach, cx)
    }

    /// The level (big), how far into it, the XP field, and buttons to move whole levels.
    fn with_level(
        &self,
        section: Section,
        experience: i64,
        field: &Entity<AmountField>,
        career: Career,
        cx: &mut Context<Self>,
    ) -> Section {
        let table = career.levels(cx);
        let level = table.level(experience);
        let rank = career.rank(cx, level.number);
        let percent = (level.fraction() * 100.0).floor() as i64;
        section
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        // A long rank name (in German, say) pushes the progress onto a line of
                        // its own rather than into it.
                        h_flex()
                            .justify_between()
                            .items_end()
                            .flex_wrap()
                            .gap_x_4()
                            .child(
                                v_flex()
                                    .child(div().text_2xl().font_semibold().child(t!(
                                        cx,
                                        "level-number",
                                        level = level.number
                                    )))
                                    .children(rank.map(|rank| {
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(rank)
                                    })),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .whitespace_nowrap()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!(
                                        cx,
                                        "level-progress",
                                        percent = percent,
                                        next = level.number + 1
                                    )),
                            ),
                    )
                    .child(
                        Progress::new(career.progress_id())
                            .value(level.fraction() * 100.0)
                            .accessibility_label(t!(cx, "level-progress-label")),
                    ),
            )
            .child(field.clone())
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children([1i64, 10].map(|levels| {
                        StepButtons::new(
                            (career.step_id(), levels as usize),
                            t!(cx, "level-step", count = levels),
                        )
                        .on_step(cx.listener(
                            move |this, direction: &i64, _, cx| {
                                let direction = *direction;
                                let table = career.levels(cx);
                                this.edit(cx, |c| {
                                    let current = career.experience(c);
                                    let target =
                                        (table.level(current).number + direction * levels).max(0);
                                    career.set_experience(c, table.experience_for(target));
                                });
                            },
                        ))
                    })),
            )
    }

    fn render_skills(&self, cx: &mut Context<Self>) -> Section {
        let skills = self.session.read(cx).career().skills;
        let rows = (0..skills.levels().len()).map(|index| {
            let name = skill_name(cx, index);
            let (field, level) = skills.levels()[index];
            h_flex()
                .justify_between()
                .gap_4()
                .py_1()
                .child(skill_label(
                    skill_icon(field, level, cx),
                    name.clone(),
                    skill_description(cx, index),
                    cx,
                ))
                .child(
                    LevelPicker::new(("skill", index), name)
                        .max(MAX_SKILL_LEVEL)
                        .value(skills.levels()[index].1)
                        .on_change(cx.listener(move |this, level: &u8, _, cx| {
                            let level = *level;
                            this.edit(cx, |career| career.skills.set_level(index, level));
                        })),
                )
        });

        Section::new(t!(cx, "skills"))
            .description(t!(cx, "skills-description"))
            .action(
                Button::new("max-skills")
                    .small()
                    .outline()
                    .label(t!(cx, "skills-max"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.edit(cx, |career| {
                            career.skills = Skills {
                                adr: career.skills.adr,
                                ..Skills::maxed()
                            };
                        });
                    })),
            )
            .children(rows)
    }

    fn render_coach_skills(&self, cx: &mut Context<Self>) -> Option<Section> {
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
                        .value(skills.levels()[index].1)
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

    /// The coach career's services as the game's statistics show them (worked out from the
    /// services it logged), the passengers and distance to edit, and making every service perfect
    /// or finding every bus stop.
    fn render_coach_services(&self, cx: &mut Context<Self>) -> Option<Section> {
        let stats = self.session.read(cx).coach_stats()?;
        let (passengers, distance) = (self.coach_passengers.clone()?, self.coach_distance.clone()?);
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
        Some(
            Section::new(t!(cx, "coach-services"))
                .description(t!(cx, "coach-services-description"))
                .child(numbers)
                .child(
                    h_flex()
                        .gap_4()
                        .flex_wrap()
                        .child(field(t!(cx, "coach-passengers"), passengers))
                        .child(field(t!(cx, "coach-distance"), distance)),
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
                                .tooltip(t!(cx, "coach-perfect-satisfaction-tooltip"))
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
                                .tooltip(t!(cx, "coach-discover-stops-tooltip"))
                                .disabled(has_found_all)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.session
                                        .update(cx, |session, cx| session.discover_bus_stops(cx));
                                })),
                        ),
                ),
        )
    }

    /// The player's day: tiredness, the police record and the clock, each with what resets it.
    /// What the save was made with (DLCs and mods), and whether this PC has them: a save that
    /// needs something missing doesn't load in the game.
    fn render_requirements(&self, cx: &mut Context<Self>) -> Option<Section> {
        use save::RequirementKind;
        let requirements = self.session.read(cx).requirements();
        if requirements.is_empty() {
            return None;
        }
        let installed = game_data::installed(cx);
        let dlcs = requirements
            .iter()
            .filter(|r| r.kind != RequirementKind::Mod)
            .count();
        let mods: Vec<&save::Requirement> = requirements
            .iter()
            .filter(|r| r.kind == RequirementKind::Mod)
            .collect();
        let theme = cx.theme();
        let (muted, danger, success) = (theme.muted_foreground, theme.danger, theme.success);
        let line = |text: SharedString, icon: IconName, color: gpui_kit::Hsla| {
            h_flex()
                .gap_2()
                .items_start()
                .text_sm()
                .child(
                    div()
                        .pt(rems(0.15))
                        .child(Icon::new(icon).small().text_color(color)),
                )
                .child(text)
        };
        let names = |missing: &[&save::Requirement]| -> SharedString {
            missing
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
                .into()
        };
        let mut section = Section::new(t!(cx, "needs")).description(t!(
            cx,
            "needs-description",
            dlcs = dlcs,
            mods = mods.len()
        ));
        let Some(installed) = installed else {
            return Some(
                section.child(
                    div()
                        .text_sm()
                        .text_color(muted)
                        .child(t!(cx, "needs-unknown")),
                ),
            );
        };
        let missing: Vec<&save::Requirement> = requirements
            .iter()
            .filter(|r| r.kind != RequirementKind::RemovableDlc && !installed.has(r))
            .collect();
        let missing_extras: Vec<&save::Requirement> = requirements
            .iter()
            .filter(|r| r.kind == RequirementKind::RemovableDlc && !installed.has(r))
            .collect();
        let mut lines: Vec<AnyElement> = Vec::new();
        if missing.is_empty() && missing_extras.is_empty() {
            lines.push(
                line(t!(cx, "needs-all-here"), IconName::CircleCheck, success).into_any_element(),
            );
        }
        if !missing.is_empty() {
            lines.push(
                v_flex()
                    .gap_1()
                    .child(line(
                        t!(cx, "needs-missing", count = missing.len()),
                        IconName::TriangleAlert,
                        danger,
                    ))
                    .child(
                        div()
                            .pl(rems(1.5))
                            .text_sm()
                            .text_color(muted)
                            .child(names(&missing)),
                    )
                    .into_any_element(),
            );
        }
        if !missing_extras.is_empty() {
            lines.push(
                v_flex()
                    .gap_1()
                    .child(line(
                        t!(cx, "needs-missing-extras", count = missing_extras.len()),
                        IconName::Info,
                        muted,
                    ))
                    .child(
                        div()
                            .pl(rems(1.5))
                            .text_sm()
                            .text_color(muted)
                            .child(names(&missing_extras)),
                    )
                    .into_any_element(),
            );
        }
        // Mods by name, each saying whether it's here (in words, not only by colour).
        if !mods.is_empty() {
            lines.push(
                v_flex()
                    .gap_1()
                    .child(div().text_sm().font_medium().child(t!(cx, "needs-mods")))
                    .children(mods.iter().map(|r| {
                        let is_here = installed.has(r);
                        h_flex().gap_2().text_sm().child(r.name.clone()).child(
                            div()
                                .text_color(if is_here { muted } else { danger })
                                .child(if is_here {
                                    t!(cx, "needs-installed")
                                } else {
                                    t!(cx, "needs-not-installed")
                                }),
                        )
                    }))
                    .into_any_element(),
            );
        }
        section = section.child(v_flex().gap_3().children(lines));
        Some(section)
    }

    fn render_day(&self, cx: &mut Context<Self>) -> Section {
        let preview = self.session.read(cx).preview();
        let muted = cx.theme().muted_foreground;
        let since_break = save::minutes_since_break(&preview);
        let (offences, fines) = save::offences(&preview);
        let clock = save::time_of_day(&preview);
        let on_job = save::is_on_job(&preview);
        let row =
            |label: SharedString, value: SharedString, action: AnyElement, cx: &Context<Self>| {
                h_flex()
                    .gap_4()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .min_h(rems(2.25))
                    .child(
                        v_flex().child(div().text_sm().child(label)).child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(value),
                        ),
                    )
                    .child(action)
            };
        let mut rows: Vec<AnyElement> = Vec::new();
        if let Some(minutes) = since_break {
            rows.push(
                row(
                    t!(cx, "day-since-break"),
                    crate::labels::duration(cx, minutes),
                    Button::new("rest")
                        .small()
                        .outline()
                        .label(t!(cx, "day-rest"))
                        .tooltip(t!(cx, "day-rest-tooltip"))
                        .disabled(minutes == 0)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.session.update(cx, |session, cx| {
                                session.edit_player(save::PlayerEdit::Rest, cx)
                            });
                        }))
                        .into_any_element(),
                    cx,
                )
                .into_any_element(),
            );
        }
        rows.push(
            row(
                t!(cx, "day-record"),
                t!(
                    cx,
                    "day-record-summary",
                    count = offences,
                    // The game's own currency (€ or $), written the way the app writes money.
                    amount = format!(
                        "{} {}",
                        self.session.read(cx).game().currency_symbol(),
                        crate::format::thousands(fines)
                    )
                ),
                Button::new("clear-record")
                    .small()
                    .outline()
                    .label(t!(cx, "day-record-clear-button"))
                    .tooltip(t!(cx, "day-record-tooltip"))
                    .disabled(offences == 0)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.session.update(cx, |session, cx| {
                            session.edit_player(save::PlayerEdit::ClearOffences, cx)
                        });
                    }))
                    .into_any_element(),
                cx,
            )
            .into_any_element(),
        );
        if let Some((day, minutes)) = clock {
            let skip =
                |id: &'static str, hour: i64, label: SharedString, cx: &mut Context<Self>| {
                    Button::new(id)
                        .small()
                        .outline()
                        .label(label)
                        .disabled(on_job)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.session.update(cx, |session, cx| {
                                session.edit_player(save::PlayerEdit::SkipTo { hour }, cx)
                            });
                        }))
                };
            let buttons = h_flex()
                .gap_2()
                .flex_wrap()
                .child(skip("skip-morning", 8, t!(cx, "day-skip-morning"), cx))
                .child(skip("skip-noon", 12, t!(cx, "day-skip-noon"), cx))
                .child(skip("skip-evening", 18, t!(cx, "day-skip-evening"), cx))
                .child(skip("skip-night", 22, t!(cx, "day-skip-night"), cx));
            rows.push(
                row(
                    t!(cx, "day-time"),
                    crate::labels::day_time(cx, day, minutes),
                    v_flex()
                        .items_end()
                        .gap_1()
                        .child(buttons)
                        .children(on_job.then(|| {
                            div()
                                .text_xs()
                                .text_color(muted)
                                .child(t!(cx, "day-time-on-job"))
                        }))
                        .into_any_element(),
                    cx,
                )
                .into_any_element(),
            );
        }
        Section::new(t!(cx, "day-section"))
            .description(t!(cx, "day-description"))
            .child(v_flex().gap_2().children(rows))
    }

    fn render_adr(&self, cx: &mut Context<Self>) -> Section {
        let adr = self.session.read(cx).career().skills.adr;
        let classes = (0..ADR_CLASS_COUNT).map(|bit| {
            let mask = 1u8 << bit;
            let is_on = adr & mask != 0;
            let set = move |this: &mut Self, on: bool, cx: &mut Context<Self>| {
                this.edit(cx, |career| {
                    if on {
                        career.skills.adr |= mask;
                    } else {
                        career.skills.adr &= !mask;
                    }
                });
            };
            // The game's placard, when it's there: on when the class is, faded when it isn't
            // (the checkbox says which too, so it's never colour alone). Clicking it toggles.
            let placard = game_data::adr_icon(cx, bit).map(|icon| {
                div()
                    .id(("adr-placard", bit))
                    .cursor_pointer()
                    .child(
                        GameIconView::new(icon)
                            .size(2.5)
                            .backdrop(crate::theme::PLACARD_BACKDROP)
                            .opacity(if is_on { 1. } else { 0.35 }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| set(this, !is_on, cx)))
            });
            h_flex().min_w_0().gap_2().children(placard).child(
                Checkbox::new(("adr", bit))
                    .flex_1()
                    .min_w_0()
                    .label(adr_class(cx, bit))
                    .checked(is_on)
                    .on_change(
                        cx.listener(move |this, checked: &bool, _, cx| set(this, *checked, cx)),
                    ),
            )
        });

        Section::new(t!(cx, "adr"))
            .description(t!(cx, "adr-description"))
            .action(
                Button::new("all-adr")
                    .small()
                    .outline()
                    .label(t!(cx, "adr-all"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.edit(cx, |career| career.skills.adr = ADR_ALL);
                    })),
            )
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_x_4()
                    .gap_y_3()
                    .children(classes),
            )
    }
}

/// A skill's icon from the game: the highlight in the theme's accent once the skill is learned,
/// all quiet while it's at level 0.
fn skill_icon(field: &str, level: u8, cx: &App) -> Option<GameIconView> {
    let icon = game_data::skill_icon(cx, field)?;
    let base = cx.theme().muted_foreground;
    let highlight = if level > 0 {
        cx.theme().primary
    } else {
        cx.theme().muted_foreground.opacity(0.6)
    };
    Some(
        GameIconView::new(icon)
            .size(2.)
            .channel(gamedata::RED, base)
            .channel(gamedata::GREEN, highlight)
            .channel(gamedata::BLUE, base),
    )
}

/// A skill's icon, name and what it does.
fn skill_label(
    icon: Option<GameIconView>,
    name: SharedString,
    description: SharedString,
    cx: &App,
) -> impl IntoElement {
    h_flex().gap_3().min_w_0().children(icon).child(
        v_flex()
            .min_w_0()
            .child(div().text_sm().font_medium().child(name))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            ),
    )
}

impl Render for OverviewPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        page_body().child(
            v_flex()
                .gap_6()
                .max_w(rems(60.))
                // The company, with Quick start beside it (a fresh career's changes, for this
                // company). Without a readable profile file there's no name to edit, and Quick
                // start keeps a row of its own.
                .child(match self.company.clone() {
                    Some(input) => Section::new(t!(cx, "company-section"))
                        .description(t!(cx, "company-name-description"))
                        .action(self.quick_start_button(cx))
                        .child(
                            h_flex()
                                .gap_4()
                                .flex_wrap()
                                .child(
                                    v_flex()
                                        .gap_1p5()
                                        .w(rems(20.))
                                        .child(div().text_sm().child(t!(cx, "company-name")))
                                        .child(Input::new(&input)),
                                )
                                .children(self.profile.clone().map(|profile| {
                                    v_flex()
                                        .gap_1p5()
                                        .w(rems(20.))
                                        .child(div().text_sm().child(t!(cx, "profile-name")))
                                        .child(Input::new(&profile))
                                })),
                        )
                        .into_any_element(),
                    None => h_flex()
                        .justify_end()
                        .child(self.quick_start_button(cx))
                        .into_any_element(),
                })
                .child({
                    // The big levers side by side, at the same height; they wrap on narrow windows.
                    let mut cards = vec![self.render_money(cx), self.render_experience(cx)];
                    if let Some(field) = self.coach_experience.clone() {
                        cards.push(self.render_coach_experience(&field, cx));
                    }
                    h_flex().flex_wrap().items_stretch().gap_6().children(
                        cards
                            .into_iter()
                            .map(|card| v_flex().flex_1().min_w(rems(14.)).child(card.flex_1())),
                    )
                })
                .child(self.render_skills(cx))
                .child(self.render_adr(cx))
                .children(self.render_coach_skills(cx))
                .children(self.render_coach_services(cx))
                .child(self.render_day(cx))
                .children(self.render_requirements(cx)),
        )
    }
}

/// Which career an experience card edits.
#[derive(Clone, Copy)]
enum Career {
    Truck,
    Coach,
}

impl Career {
    fn experience(self, career: &save::Career) -> i64 {
        match self {
            Career::Truck => career.experience,
            Career::Coach => career.coach_experience.unwrap_or(0),
        }
    }

    fn set_experience(self, career: &mut save::Career, value: i64) {
        match self {
            Career::Truck => career.experience = value,
            Career::Coach => {
                if career.coach_experience.is_some() {
                    career.coach_experience = Some(value);
                }
            }
        }
    }

    /// How much XP each level needs: the game's own table when it was read.
    fn levels(self, cx: &App) -> LevelTable {
        match self {
            Career::Truck => game_data::truck_levels(cx),
            Career::Coach => game_data::coach_levels(cx),
        }
    }

    /// The game's rank name for a level (`Divine Champion`), when the game's data was read.
    fn rank(self, cx: &App, level: i64) -> Option<String> {
        match self {
            Career::Truck => game_data::truck_rank(cx, level),
            Career::Coach => game_data::coach_rank(cx, level),
        }
    }

    fn step_id(self) -> &'static str {
        match self {
            Career::Truck => "truck-level-step",
            Career::Coach => "coach-level-step",
        }
    }

    fn progress_id(self) -> &'static str {
        match self {
            Career::Truck => "truck-level-progress",
            Career::Coach => "coach-level-progress",
        }
    }
}
