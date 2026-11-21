//! One hired driver: face, where they're from and are now, experience, skills and ADR classes.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _, button::Button,
    checkbox::Checkbox, h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div, px, rems,
};
use save::{Driver, MAX_SKILL_LEVEL, Skills};

use super::{SHEET_WIDTH, fact, group};
use crate::game_data;
use crate::i18n::t;
use crate::labels::{ADR_CLASS_COUNT, adr_class, skill_name};
use crate::session::Session;
use crate::ui::{AmountChanged, AmountField, GameIconView, LevelPicker};

pub struct DriverDetail {
    session: Entity<Session>,
    id: String,
    experience: Entity<AmountField>,
    _subscriptions: Vec<Subscription>,
}

impl DriverDetail {
    fn new(
        session: Entity<Session>,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let xp = find(&session, &id, cx).map_or(0, |driver| driver.experience);
        let label = t!(cx, "column-experience");
        let experience = cx.new(|cx| {
            AmountField::new(xp, label, window, cx)
                .min(0)
                .suffix("XP")
                .compact()
        });
        let _subscriptions = vec![
            cx.subscribe(&experience, |this, _, AmountChanged(xp), cx| {
                let xp = (*xp).max(0).to_string();
                this.set(cx, "experience_points", xp);
            }),
            cx.observe_in(&session, window, |this, _, window, cx| {
                if let Some(driver) = find(&this.session, &this.id, cx)
                    && this.experience.read(cx).value() != driver.experience
                {
                    this.experience.update(cx, |field, cx| {
                        field.set_value(driver.experience, window, cx)
                    });
                }
                cx.notify();
            }),
        ];
        Self {
            session,
            id,
            experience,
            _subscriptions,
        }
    }

    fn set(&mut self, cx: &mut Context<Self>, field: &'static str, text: String) {
        let id = self.id.clone();
        self.session.update(cx, |session, cx| {
            session.set_field(&id, field, None, text, cx)
        });
    }
}

fn find(session: &Entity<Session>, id: &str, cx: &gpui_kit::App) -> Option<Driver> {
    session
        .read(cx)
        .drivers()
        .into_iter()
        .find(|driver| driver.id == id)
}

impl Render for DriverDetail {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(driver) = find(&self.session, &self.id, cx) else {
            return div().child(t!(cx, "detail-gone")).into_any_element();
        };
        let face = game_data::driver_face(cx, &driver.id);
        let skills = driver.skills;
        let rows = skills
            .levels()
            .into_iter()
            .enumerate()
            .map(|(index, (field, level))| {
                let name = skill_name(cx, index);
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_sm().child(name.clone()))
                    .child(
                        LevelPicker::new(("driver-skill", index), name)
                            .max(MAX_SKILL_LEVEL)
                            .value(level)
                            .on_change(cx.listener(move |this, level: &u8, _, cx| {
                                this.set(cx, field, level.to_string());
                            })),
                    )
            });
        let adr = (0..ADR_CLASS_COUNT).map(|bit| {
            let mask = 1u8 << bit;
            Checkbox::new(("driver-adr", bit))
                .label(adr_class(cx, bit))
                .checked(skills.adr & mask != 0)
                .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                    let Some(driver) = find(&this.session, &this.id, cx) else {
                        return;
                    };
                    let adr = if *checked {
                        driver.skills.adr | mask
                    } else {
                        driver.skills.adr & !mask
                    };
                    this.set(cx, "adr", adr.to_string());
                }))
        });
        let is_maxed = skills == Skills::maxed();
        let id = self.id.clone();

        v_flex()
            .gap_6()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .size(rems(3.))
                            .flex_shrink_0()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().muted)
                            .children(face.map(|face| {
                                GameIconView::new(face).size(3.).rounded(cx.theme().radius)
                            })),
                    )
                    .child(
                        v_flex()
                            .child(div().text_lg().child(t!(
                                cx,
                                "driver-from",
                                city = game_data::city_name(cx, &driver.hometown)
                            )))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!(
                                        cx,
                                        "driver-now-in",
                                        city = game_data::city_name(cx, &driver.current_city)
                                    )),
                            ),
                    ),
            )
            .child(group(
                t!(cx, "detail-about"),
                v_flex().gap_1().child(fact(
                    t!(cx, "column-experience"),
                    div().w(rems(11.)).child(self.experience.clone()),
                    cx,
                )),
            ))
            .child(group(
                t!(cx, "column-skills"),
                v_flex().gap_2().children(rows),
            ))
            .child(group(t!(cx, "adr"), v_flex().gap_2().children(adr)))
            .child(
                h_flex().child(
                    Button::new("max-driver")
                        .small()
                        .outline()
                        .label(t!(cx, "driver-max"))
                        .disabled(is_maxed)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let id = id.clone();
                            this.session
                                .update(cx, |session, cx| session.max_driver(&id, cx));
                        })),
                ),
            )
            .into_any_element()
    }
}

/// Opens a driver's panel.
pub fn open(
    session: Entity<Session>,
    driver: &Driver,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    let id = driver.id.clone();
    let view = cx.new(|cx| DriverDetail::new(session, id, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "detail-driver"))
            .size(px(SHEET_WIDTH))
            .child(view.clone())
    });
}
