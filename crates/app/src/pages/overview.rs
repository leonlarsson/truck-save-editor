//! Overview: the big levers first. Money, experience, skills and ADR, each one click from maxed.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _, button::Button, checkbox::Checkbox, h_flex,
    progress::Progress, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div, rems,
};

use crate::format::compact;
use crate::sample::{self, ADR_ALL, ADR_CLASSES, Career, MAX_SKILL_LEVEL, SKILLS};
use crate::ui::{AmountChanged, AmountField, LevelPicker, Section, StepButtons, page_body};

/// What typing in an amount field can do, said once under each field.
const AMOUNT_HINT: &str = "Try 2.5m, 10k, +1m or -500k.";

pub struct OverviewPage {
    /// The career as loaded; edits are compared against it to count changes.
    original: Career,
    career: Career,
    money: Entity<AmountField>,
    experience: Entity<AmountField>,
    _subscriptions: Vec<Subscription>,
}

impl OverviewPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let career = Career::sample();
        let money = cx.new(|cx| {
            AmountField::new(career.money, "Bank balance in euros", window, cx)
                .prefix("€")
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
                this.career.money = *value;
                cx.notify();
            }),
            cx.subscribe(&experience, |this, _, AmountChanged(value), cx| {
                this.career.experience = *value;
                cx.notify();
            }),
        ];

        Self {
            original: career.clone(),
            career,
            money,
            experience,
            _subscriptions,
        }
    }

    /// How many values differ from the loaded save.
    pub fn changes(&self) -> usize {
        self.career.changes_from(&self.original)
    }

    pub fn discard_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.career = self.original.clone();
        self.sync_fields(window, cx);
        cx.notify();
    }

    fn set_money(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.career.money = value;
        self.sync_fields(window, cx);
        cx.notify();
    }

    fn set_experience(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.career.experience = value.max(0);
        self.sync_fields(window, cx);
        cx.notify();
    }

    /// Puts the career's values into the fields after a change made elsewhere (a button, Discard).
    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (money, experience) = (self.career.money, self.career.experience);
        self.money
            .update(cx, |field, cx| field.set_value(money, window, cx));
        self.experience
            .update(cx, |field, cx| field.set_value(experience, window, cx));
    }

    fn render_money(&self, cx: &mut Context<Self>) -> Section {
        let steps: [i64; 3] = [100_000, 1_000_000, 10_000_000];
        Section::new("Money")
            .description("Your company's bank balance.")
            .child(self.money.clone())
            .child(h_flex().gap_2().flex_wrap().children(steps.map(|step| {
                StepButtons::new(("money-step", step as usize), compact(step)).on_step(cx.listener(
                    move |this, direction: &i64, window, cx| {
                        let value = this.career.money.saturating_add(direction * step);
                        this.set_money(value, window, cx);
                    },
                ))
            })))
    }

    fn render_experience(&self, cx: &mut Context<Self>) -> Section {
        let (level, progress) = sample::level_for(self.career.experience);
        // The experience at which a level starts. PLACEHOLDER curve, like `sample::level_for`.
        let level_start = |level: i64| level.max(0) * level.max(0) * 400;
        Section::new("Experience")
            .description("Experience points decide your level.")
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_end()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_semibold()
                                    .child(format!("Level {level}")),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{}% to level {}",
                                        (progress * 100.0).floor(),
                                        level + 1
                                    )),
                            ),
                    )
                    .child(
                        Progress::new("level-progress")
                            .value(progress * 100.0)
                            .accessibility_label("Progress to the next level"),
                    ),
            )
            .child(self.experience.clone())
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children([(1, "1 level"), (10, "10 levels")].map(|(levels, label)| {
                        StepButtons::new(("level-step", levels as usize), label).on_step(
                            cx.listener(move |this, direction: &i64, window, cx| {
                                let target = level + direction * levels;
                                this.set_experience(level_start(target), window, cx);
                            }),
                        )
                    })),
            )
    }

    fn render_skills(&self, cx: &mut Context<Self>) -> Section {
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
                        .value(self.career.skills[index])
                        .on_change(cx.listener(move |this, level: &u8, _, cx| {
                            this.career.skills[index] = *level;
                            cx.notify();
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
                        this.career.skills = [MAX_SKILL_LEVEL; 5];
                        cx.notify();
                    })),
            )
            .children(rows)
    }

    fn render_adr(&self, cx: &mut Context<Self>) -> Section {
        let classes = ADR_CLASSES.iter().enumerate().map(|(bit, name)| {
            let mask = 1u8 << bit;
            div().w(rems(13.)).child(
                Checkbox::new(("adr", bit))
                    .label(*name)
                    .checked(self.career.adr & mask != 0)
                    .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                        if *checked {
                            this.career.adr |= mask;
                        } else {
                            this.career.adr &= !mask;
                        }
                        cx.notify();
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
                        this.career.adr = ADR_ALL;
                        cx.notify();
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
