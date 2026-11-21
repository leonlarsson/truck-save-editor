//! Overview: the big levers first. Money, experience, skills and ADR, each one click from maxed.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _, button::Button, checkbox::Checkbox, h_flex,
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div, rems,
};
use save::{ADR_ALL, MAX_SKILL_LEVEL, Skills};

use crate::format::compact;
use crate::i18n::t;
use crate::labels::{ADR_CLASS_COUNT, adr_class, skill_description, skill_name};
use crate::session::Session;
use crate::ui::{AmountChanged, AmountField, LevelPicker, Section, StepButtons, page_body};

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
        let steps: [i64; 3] = [10_000, 100_000, 1_000_000];
        Section::new(t!(cx, "experience"))
            .description(t!(cx, "experience-description"))
            .child(self.experience.clone())
            .child(h_flex().gap_2().flex_wrap().children(steps.map(|step| {
                StepButtons::new(("xp-step", step as usize), compact(step)).on_step(cx.listener(
                    move |this, direction: &i64, _, cx| {
                        let direction = *direction;
                        this.edit(cx, |career| {
                            career.experience = (career.experience + direction * step).max(0);
                        });
                    },
                ))
            })))
    }

    fn render_coach_experience(
        &self,
        field: &Entity<AmountField>,
        cx: &mut Context<Self>,
    ) -> Section {
        let steps: [i64; 3] = [1_000, 10_000, 100_000];
        Section::new(t!(cx, "coach-experience"))
            .description(t!(cx, "coach-experience-description"))
            .child(field.clone())
            .child(h_flex().gap_2().flex_wrap().children(steps.map(|step| {
                StepButtons::new(("coach-xp-step", step as usize), compact(step)).on_step(
                    cx.listener(move |this, direction: &i64, _, cx| {
                        let direction = *direction;
                        this.edit(cx, |career| {
                            if let Some(value) = &mut career.coach_experience {
                                *value = (*value + direction * step).max(0);
                            }
                        });
                    }),
                )
            })))
    }

    fn render_skills(&self, cx: &mut Context<Self>) -> Section {
        let skills = self.session.read(cx).career().skills;
        let rows = (0..skills.levels().len()).map(|index| {
            let name = skill_name(cx, index);
            h_flex()
                .justify_between()
                .gap_4()
                .py_1()
                .child(
                    v_flex()
                        .min_w_0()
                        .child(div().text_sm().font_medium().child(name.clone()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(skill_description(cx, index)),
                        ),
                )
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

    fn render_adr(&self, cx: &mut Context<Self>) -> Section {
        let adr = self.session.read(cx).career().skills.adr;
        let classes = (0..ADR_CLASS_COUNT).map(|bit| {
            let mask = 1u8 << bit;
            div().w(rems(13.)).child(
                Checkbox::new(("adr", bit))
                    .label(adr_class(cx, bit))
                    .checked(adr & mask != 0)
                    .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                        let checked = *checked;
                        this.edit(cx, |career| {
                            if checked {
                                career.skills.adr |= mask;
                            } else {
                                career.skills.adr &= !mask;
                            }
                        });
                    })),
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
            .child(h_flex().flex_wrap().gap_y_3().children(classes))
    }
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
                .child(self.render_adr(cx)),
        )
    }
}
