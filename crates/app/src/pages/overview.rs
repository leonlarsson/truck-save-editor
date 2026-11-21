//! Overview: the big levers first. Money, experience, skills and ADR, each one click from maxed.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _, button::Button, checkbox::Checkbox, h_flex,
    progress::Progress, v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, rems,
};
use save::{ADR_ALL, CoachSkills, LevelTable, MAX_COACH_SKILL_LEVEL, MAX_SKILL_LEVEL, Skills};

use crate::format::compact;
use crate::game_data;
use crate::i18n::t;
use crate::labels::{
    ADR_CLASS_COUNT, adr_class, coach_skill_description, coach_skill_name, skill_description,
    skill_name,
};
use crate::session::Session;
use crate::ui::{
    AmountChanged, AmountField, GameIconView, LevelPicker, Section, StepButtons, page_body,
};

pub struct OverviewPage {
    session: Entity<Session>,
    money: Entity<AmountField>,
    experience: Entity<AmountField>,
    /// Coach experience (Coaches DLC); `None` for saves from before coaches.
    coach_experience: Option<Entity<AmountField>>,
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

        Self {
            session,
            money,
            experience,
            coach_experience,
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
        for (field, value) in fields {
            if field.read(cx).value() != value {
                field.update(cx, |field, cx| field.set_value(value, window, cx));
            }
        }
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
                        h_flex()
                            .justify_between()
                            .items_end()
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
                            .backdrop(PLACARD_BACKDROP)
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

/// The dark of the game's own panels, which its placards are drawn over (the black half of the
/// corrosive placard is that panel showing through). The same in both themes: it's the sign's.
const PLACARD_BACKDROP: Hsla = Hsla {
    h: 0.,
    s: 0.,
    l: 0.07,
    a: 1.,
};

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
                .children(self.render_coach_skills(cx)),
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
