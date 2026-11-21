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

use crate::format::{compact, thousands};
use crate::labels::{ADR_CLASSES, SKILLS};
use crate::session::Session;
use crate::ui::{AmountChanged, AmountField, LevelPicker, Section, StepButtons, page_body};

/// What typing in an amount field can do, said once under each field.
const AMOUNT_HINT: &str = "Try 2.5m, 10k, +1m or -500k.";

pub struct OverviewPage {
    session: Entity<Session>,
    money: Entity<AmountField>,
    experience: Entity<AmountField>,
    _subscriptions: Vec<Subscription>,
}

impl OverviewPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (career, currency) = {
            let session = session.read(cx);
            (session.career().clone(), session.game().currency_symbol())
        };
        let money = cx.new(|cx| {
            AmountField::new(career.money, "Bank balance", window, cx)
                .prefix(currency)
                .hint(AMOUNT_HINT)
        });
        let experience = cx.new(|cx| {
            AmountField::new(career.experience, "Experience points", window, cx)
                .min(0)
                .suffix("XP")
                .hint(AMOUNT_HINT)
        });

        let _subscriptions = vec![
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

        Self {
            session,
            money,
            experience,
            _subscriptions,
        }
    }

    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let career = self.session.read(cx).career().clone();
        for (field, value) in [
            (&self.money, career.money),
            (&self.experience, career.experience),
        ] {
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
        Section::new("Money")
            .description("Your company's bank balance.")
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
        let bus = self.session.read(cx).career().bus_experience;
        let mut section = Section::new("Experience")
            .description("Experience points decide your level.")
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
            })));
        if let Some(bus) = bus.filter(|bus| *bus > 0) {
            section = section.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Bus driving: {} XP", thousands(bus))),
            );
        }
        section
    }

    fn render_skills(&self, cx: &mut Context<Self>) -> Section {
        let skills = self.session.read(cx).career().skills;
        let rows = SKILLS.iter().enumerate().map(|(index, skill)| {
            h_flex()
                .justify_between()
                .gap_4()
                .py_1()
                .child(
                    v_flex()
                        .min_w_0()
                        .child(div().text_sm().font_medium().child(skill.name))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(skill.description),
                        ),
                )
                .child(
                    LevelPicker::new(("skill", index), skill.name)
                        .max(MAX_SKILL_LEVEL)
                        .value(skills.levels()[index].1)
                        .on_change(cx.listener(move |this, level: &u8, _, cx| {
                            let level = *level;
                            this.edit(cx, |career| career.skills.set_level(index, level));
                        })),
                )
        });

        Section::new("Skills")
            .description(
                "Click a level to set it. Click the highest filled level again to lower it.",
            )
            .action(
                Button::new("max-skills")
                    .small()
                    .outline()
                    .label("Max all skills")
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
        let classes = ADR_CLASSES.iter().enumerate().map(|(bit, name)| {
            let mask = 1u8 << bit;
            div().w(rems(13.)).child(
                Checkbox::new(("adr", bit))
                    .label(*name)
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

        Section::new("ADR")
            .description("Dangerous goods your driver may haul.")
            .action(
                Button::new("all-adr")
                    .small()
                    .outline()
                    .label("All classes")
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
                .child(
                    // The two cards sit side by side at the same height.
                    h_flex()
                        .items_stretch()
                        .gap_6()
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(self.render_money(cx).flex_1()),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(self.render_experience(cx).flex_1()),
                        ),
                )
                .child(self.render_skills(cx))
                .child(self.render_adr(cx)),
        )
    }
}
