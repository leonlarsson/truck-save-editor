//! Quick start: a fresh career's usual changes in one go. Start a new profile in the game, open
//! its save here, pick "Rich start" or "Everything unlocked" (or tick your own), and apply. It's
//! one change: one Undo takes it all back, and nothing is written until you save.

use std::collections::BTreeSet;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Window, div, px, rems,
};
use save::Place;

use crate::game_data;
use crate::i18n::t;
use crate::session::{QuickStartPlan, Session};
use crate::ui::AmountField;

/// What a quick start can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Choice {
    Money,
    TruckSkills,
    Adr,
    CoachSkills,
    HqGarage,
    VisitCities,
    Dealers,
    Recruitments,
}

impl Choice {
    const ALL: [Choice; 8] = [
        Choice::Money,
        Choice::TruckSkills,
        Choice::Adr,
        Choice::CoachSkills,
        Choice::HqGarage,
        Choice::VisitCities,
        Choice::Dealers,
        Choice::Recruitments,
    ];
    const RICH_START: [Choice; 5] = [
        Choice::Money,
        Choice::TruckSkills,
        Choice::Adr,
        Choice::CoachSkills,
        Choice::HqGarage,
    ];
}

/// The money a rich start begins with.
const RICH_START_MONEY: i64 = 10_000_000;

pub struct QuickStart {
    session: Entity<Session>,
    money: Entity<AmountField>,
    chosen: BTreeSet<Choice>,
}

impl QuickStart {
    fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let label = t!(cx, "money");
        let money = cx.new(|cx| AmountField::new(RICH_START_MONEY, label, window, cx));
        Self {
            session,
            money,
            chosen: Choice::RICH_START.into_iter().collect(),
        }
    }

    /// Why a choice can't be made for this save, if it can't.
    fn unavailable(&self, choice: Choice, cx: &Context<Self>) -> Option<SharedString> {
        let session = self.session.read(cx);
        match choice {
            Choice::CoachSkills
                if !session.has_coach_career() || session.career().coach_skills.is_none() =>
            {
                Some(t!(cx, "quick-start-no-coaches"))
            }
            Choice::HqGarage => {
                let hq = &session.career().hq_city;
                let garage = session.garages().into_iter().find(|g| &g.city == hq);
                match garage {
                    None => Some(t!(cx, "quick-start-no-hq")),
                    Some(garage) if garage.is_owned && garage.is_large() => {
                        Some(t!(cx, "quick-start-hq-large"))
                    }
                    Some(_) => None,
                }
            }
            Choice::Dealers if game_data::place_cities(cx, Place::Dealer).is_none() => {
                Some(t!(cx, "quick-start-needs-game"))
            }
            Choice::Recruitments if game_data::place_cities(cx, Place::Recruitment).is_none() => {
                Some(t!(cx, "quick-start-needs-game"))
            }
            _ => None,
        }
    }

    fn label(&self, choice: Choice, cx: &Context<Self>) -> SharedString {
        match choice {
            Choice::Money => t!(cx, "quick-start-money"),
            Choice::TruckSkills => t!(cx, "palette-max-truck-skills"),
            Choice::Adr => t!(
                cx,
                "palette-all-adr",
                game = crate::labels::current_game_key(cx)
            ),
            Choice::CoachSkills => t!(cx, "palette-max-coach-skills"),
            Choice::HqGarage => {
                let hq = self.session.read(cx).career().hq_city.clone();
                t!(
                    cx,
                    "quick-start-hq-garage",
                    city = game_data::city_name(cx, &hq)
                )
            }
            Choice::VisitCities => t!(cx, "map-visit-all"),
            Choice::Dealers => t!(cx, "map-unlock-dealers"),
            Choice::Recruitments => t!(cx, "map-unlock-recruitments"),
        }
    }

    fn choose(&mut self, choices: &[Choice], cx: &mut Context<Self>) {
        self.chosen = choices.iter().copied().collect();
        cx.notify();
    }

    /// Makes the changes (as one) and says how many there are now.
    fn apply(&mut self, cx: &mut Context<Self>) {
        let available = |choice: Choice| {
            self.chosen.contains(&choice) && self.unavailable(choice, cx).is_none()
        };
        let plan = QuickStartPlan {
            money: available(Choice::Money).then(|| self.money.read(cx).value()),
            truck_skills: available(Choice::TruckSkills),
            adr: available(Choice::Adr),
            coach_skills: available(Choice::CoachSkills),
            hq_garage: available(Choice::HqGarage),
            visit_cities: available(Choice::VisitCities),
            dealers: available(Choice::Dealers)
                .then(|| self.session.read(cx).place_cities(Place::Dealer, cx))
                .flatten(),
            recruitments: available(Choice::Recruitments)
                .then(|| self.session.read(cx).place_cities(Place::Recruitment, cx))
                .flatten(),
        };
        self.session
            .update(cx, |session, cx| session.apply_quick_start(plan, cx));
    }
}

impl Render for QuickStart {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let all: Vec<Choice> = Choice::ALL.to_vec();
        let is_rich = self.chosen == Choice::RICH_START.into_iter().collect();
        let is_everything = self.chosen == all.iter().copied().collect();
        let presets = ButtonGroup::new("quick-start-presets")
            .outline()
            .small()
            .child(
                Button::new("rich-start")
                    .label(t!(cx, "quick-start-rich"))
                    .selected(is_rich),
            )
            .child(
                Button::new("everything")
                    .label(t!(cx, "quick-start-everything"))
                    .selected(is_everything),
            )
            .on_click(cx.listener(
                move |this, clicked: &Vec<usize>, _, cx| match clicked.first() {
                    Some(0) => this.choose(&Choice::RICH_START, cx),
                    Some(1) => this.choose(&Choice::ALL, cx),
                    _ => {}
                },
            ));

        let rows = Choice::ALL.into_iter().map(|choice| {
            let reason = self.unavailable(choice, cx);
            let is_chosen = self.chosen.contains(&choice) && reason.is_none();
            let checkbox = Checkbox::new(("quick-start", choice as usize))
                .label(self.label(choice, cx))
                .checked(is_chosen)
                .disabled(reason.is_some())
                .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                    if *checked {
                        this.chosen.insert(choice);
                    } else {
                        this.chosen.remove(&choice);
                    }
                    cx.notify();
                }));
            v_flex()
                .gap_1()
                .child(checkbox)
                // Not hidden: say why it can't be done here.
                .children(reason.map(|reason| {
                    div()
                        .pl(rems(1.5))
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(reason)
                }))
                .children(
                    (choice == Choice::Money && is_chosen)
                        .then(|| div().pl(rems(1.5)).w(rems(17.5)).child(self.money.clone())),
                )
        });

        v_flex()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "quick-start-description")),
            )
            .child(presets)
            .child(v_flex().gap_3().children(rows))
    }
}

/// Opens Quick start for the open save.
pub fn open(session: Entity<Session>, window: &mut Window, cx: &mut gpui_kit::App) {
    let view = cx.new(|cx| QuickStart::new(session, window, cx));
    window.open_dialog(cx, move |dialog, _, cx| {
        let apply = view.clone();
        dialog
            .title(t!(cx, "quick-start"))
            .width(px(520.))
            .content({
                let view = view.clone();
                move |content, _, _| content.child(view.clone())
            })
            .footer(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("quick-start-cancel")
                            .outline()
                            .label(t!(cx, "cancel"))
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("quick-start-apply")
                            .primary()
                            .label(t!(cx, "quick-start-apply"))
                            .on_click(move |_, window, cx| {
                                apply.update(cx, |view, cx| view.apply(cx));
                                window.close_dialog(cx);
                            }),
                    ),
            )
    });
}
