//! Overview: the big levers first. Money, experience, skills and ADR, each one click from maxed.

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _, StyledExt as _,
    button::Button,
    checkbox::Checkbox,
    h_flex,
    input::{Input, InputEvent, InputState, MaskPattern},
    progress::Progress,
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div, rems,
};

use crate::format::{parse_whole_number, thousands};
use crate::sample::{self, ADR_ALL, ADR_CLASSES, Career, MAX_SKILL_LEVEL, SKILLS};
use crate::ui::{LevelPicker, Section, page_body};

pub struct OverviewPage {
    /// The career as loaded; edits are compared against it to count changes.
    original: Career,
    career: Career,
    money_input: Entity<InputState>,
    experience_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl OverviewPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let career = Career::sample();
        let money_input = number_input(career.money, window, cx);
        let experience_input = number_input(career.experience, window, cx);

        // Typing in a field updates the career as you type; a value that isn't a number is ignored
        // until it becomes one.
        let _subscriptions = vec![
            cx.subscribe_in(&money_input, window, |this, state, event, _, cx| {
                if let InputEvent::Change = event
                    && let Some(value) = parse_whole_number(&state.read(cx).value())
                {
                    this.career.money = value;
                    cx.notify();
                }
            }),
            cx.subscribe_in(&experience_input, window, |this, state, event, _, cx| {
                if let InputEvent::Change = event
                    && let Some(value) = parse_whole_number(&state.read(cx).value())
                {
                    this.career.experience = value.max(0);
                    cx.notify();
                }
            }),
        ];

        Self {
            original: career.clone(),
            career,
            money_input,
            experience_input,
            _subscriptions,
        }
    }

    /// How many values differ from the loaded save.
    pub fn changes(&self) -> usize {
        self.career.changes_from(&self.original)
    }

    pub fn discard_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.career = self.original.clone();
        self.sync_inputs(window, cx);
        cx.notify();
    }

    fn set_money(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.career.money = value;
        self.sync_inputs(window, cx);
        cx.notify();
    }

    fn set_experience(&mut self, value: i64, window: &mut Window, cx: &mut Context<Self>) {
        self.career.experience = value.max(0);
        self.sync_inputs(window, cx);
        cx.notify();
    }

    /// Puts the career's values back into the text fields after a change made elsewhere.
    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let money = thousands(self.career.money);
        let experience = thousands(self.career.experience);
        self.money_input
            .update(cx, |state, cx| state.set_value(money, window, cx));
        self.experience_input
            .update(cx, |state, cx| state.set_value(experience, window, cx));
    }

    fn render_money(&self, cx: &mut Context<Self>) -> Section {
        let amounts: [i64; 3] = [100_000, 1_000_000, 10_000_000];
        Section::new("Money")
            .description("Your company's bank balance.")
            .child(
                Input::new(&self.money_input)
                    .large()
                    .prefix(div().text_color(cx.theme().muted_foreground).child("€"))
                    .aria_label("Bank balance in euros"),
            )
            .child(h_flex().gap_2().flex_wrap().children(amounts.map(|amount| {
                Button::new(("add-money", amount as usize))
                    .small()
                    .outline()
                    .label(format!("+ {}", thousands(amount)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.set_money(this.career.money.saturating_add(amount), window, cx);
                    }))
            })))
    }

    fn render_experience(&self, cx: &mut Context<Self>) -> Section {
        let (level, progress) = sample::level_for(self.career.experience);
        let next_level_xp = move |levels_up: i64| {
            let target = level + levels_up;
            target * target * 400
        };
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
            .child(
                Input::new(&self.experience_input)
                    .large()
                    .suffix(div().text_color(cx.theme().muted_foreground).child("XP"))
                    .aria_label("Experience points"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("level-up")
                            .small()
                            .outline()
                            .label("+1 level")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.set_experience(next_level_xp(1), window, cx);
                            })),
                    )
                    .child(
                        Button::new("level-up-ten")
                            .small()
                            .outline()
                            .label("+10 levels")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.set_experience(next_level_xp(10), window, cx);
                            })),
                    ),
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

/// A whole-number field that groups thousands as you type.
fn number_input(
    value: i64,
    window: &mut Window,
    cx: &mut Context<OverviewPage>,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .mask_pattern(MaskPattern::Number {
                separator: Some(','),
                fraction: None,
            })
            .default_value(thousands(value))
    })
}
