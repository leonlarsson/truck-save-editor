//! One city's garage: owned or not, how big, and the trucks and drivers it holds, each opening
//! its own panel.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window, div, px, rems,
};
use save::Garage;

use super::{SHEET_WIDTH, fact, group};
use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::GameIconView;
use crate::ui::Tip as _;

pub struct GarageDetail {
    session: Entity<Session>,
    id: String,
    _subscriptions: Vec<Subscription>,
}

impl GarageDetail {
    fn new(session: Entity<Session>, id: String, cx: &mut Context<Self>) -> Self {
        let _subscriptions = vec![cx.observe(&session, |_, _, cx| cx.notify())];
        Self {
            session,
            id,
            _subscriptions,
        }
    }
}

/// A truck or driver in the garage: the whole row is a quiet button that opens their panel.
fn open_row(
    id: impl Into<ElementId>,
    content: impl IntoElement,
    tooltip: SharedString,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    // The button's padding would push the row in from the group's edges: the row is widened by
    // that much on each side, so its text lines up with the facts above.
    div()
        .mx(px(-8.))
        .child(
            Button::new(id)
                .ghost()
                .small()
                .w_full()
                .tip(tooltip)
                .child(div().flex_1().min_w_0().child(content))
                .on_click(move |_, window, cx| on_click(window, cx)),
        )
        .into_any_element()
}

impl Render for GarageDetail {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.session.read(cx);
        let Some(garage) = session
            .garages()
            .into_iter()
            .find(|garage| garage.id == self.id)
        else {
            return div().child(t!(cx, "detail-gone")).into_any_element();
        };
        let unit = session.unit(&self.id);
        let ids = |field: &str| -> Vec<String> {
            unit.as_ref()
                .map(|unit| {
                    unit.items(field)
                        .iter()
                        .filter(|id| *id != "null")
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        };
        let vehicle_ids = ids("vehicles");
        let driver_ids = ids("drivers");
        let vehicles: Vec<_> = session
            .vehicles()
            .into_iter()
            .filter(|vehicle| vehicle_ids.contains(&vehicle.id))
            .collect();
        let drivers: Vec<_> = session
            .drivers()
            .into_iter()
            .filter(|driver| driver_ids.contains(&driver.id))
            .collect();

        let state: SharedString = if !garage.is_owned {
            t!(cx, "garage-not-owned")
        } else if garage.is_large() {
            t!(cx, "garage-large")
        } else {
            t!(cx, "garage-slots", count = garage.slots)
        };
        // Built now, not lazily: each row borrows `cx`, and loading a face needs it mutably.
        let vehicle_rows: Vec<_> = vehicles
            .iter()
            .map(|vehicle| {
                let content = h_flex()
                    .justify_between()
                    .text_sm()
                    .child(game_data::model_name(cx, vehicle).unwrap_or_default())
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(vehicle.plate.clone()),
                    );
                let (session, vehicle) = (self.session.clone(), vehicle.clone());
                open_row(
                    SharedString::from(format!("truck-{}", vehicle.id)),
                    content,
                    t!(cx, "driver-truck-open"),
                    move |window, cx| {
                        super::open_vehicle(session.clone(), &vehicle, window, cx);
                    },
                )
            })
            .collect();
        let driver_rows: Vec<_> = drivers
            .iter()
            .map(|driver| {
                (
                    game_data::city_name(cx, &driver.hometown),
                    driver.id.clone(),
                )
            })
            .collect();
        let mut faces = Vec::new();
        for (_, id) in &driver_rows {
            faces.push(game_data::driver_face(cx, id));
        }
        let driver_rows: Vec<_> = driver_rows
            .into_iter()
            .zip(faces)
            .zip(drivers.iter().cloned())
            .map(|(((hometown, _), face), driver)| {
                let content = h_flex()
                    .gap_2()
                    .items_center()
                    .text_sm()
                    .child(
                        div()
                            .size(rems(1.4))
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().muted)
                            .children(face.map(|face| {
                                GameIconView::new(face).size(1.4).rounded(cx.theme().radius)
                            })),
                    )
                    .child(t!(cx, "driver-from", city = hometown));
                let session = self.session.clone();
                open_row(
                    SharedString::from(format!("driver-{}", driver.id)),
                    content,
                    t!(cx, "garage-driver-open"),
                    move |window, cx| {
                        super::open_driver(session.clone(), &driver, window, cx);
                    },
                )
            })
            .collect();
        let empty = |cx: &Context<Self>| {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(t!(cx, "garage-empty"))
        };
        let id = self.id.clone();
        let label = if garage.is_owned {
            t!(cx, "garage-make-large")
        } else {
            t!(cx, "garage-buy-large")
        };

        crate::ui::Divided::groups()
            .pt_2()
            .child(group(
                t!(cx, "detail-about"),
                v_flex()
                    .gap_1()
                    .child(fact(t!(cx, "column-garage-size"), state, cx))
                    .child(fact(
                        t!(cx, "column-trucks"),
                        t!(
                            cx,
                            "garage-filled",
                            filled = garage.trucks,
                            slots = garage.slots
                        ),
                        cx,
                    ))
                    .child(fact(
                        t!(cx, "column-drivers"),
                        t!(
                            cx,
                            "garage-filled",
                            filled = garage.drivers,
                            slots = garage.slots
                        ),
                        cx,
                    )),
            ))
            .child(group(
                t!(cx, "column-trucks"),
                if vehicles.is_empty() {
                    empty(cx).into_any_element()
                } else {
                    v_flex().gap_1().children(vehicle_rows).into_any_element()
                },
            ))
            .child(group(
                t!(cx, "column-drivers"),
                if drivers.is_empty() {
                    empty(cx).into_any_element()
                } else {
                    v_flex().gap_1().children(driver_rows).into_any_element()
                },
            ))
            .child(super::actions_row(
                Button::new("upgrade-garage")
                    .small()
                    .outline()
                    .label(label)
                    .disabled(garage.is_owned && garage.is_large())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let id = id.clone();
                        this.session
                            .update(cx, |session, cx| session.upgrade_garage(&id, cx));
                    })),
                &self.session,
                garage.is_owned.then(|| {
                    (
                        save::Removal::Garage(self.id.clone()),
                        t!(cx, "remove-garage"),
                    )
                }),
                cx,
            ))
            .into_any_element()
    }
}

/// Opens a garage's panel, titled with its city.
pub fn open(
    session: Entity<Session>,
    garage: &Garage,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    let id = garage.id.clone();
    let title: SharedString = t!(
        cx,
        "detail-garage-in",
        city = game_data::city_name(cx, &garage.city)
    );
    let view = cx.new(|cx| GarageDetail::new(session, id, cx));
    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(title.clone())
            .size(px(SHEET_WIDTH))
            .child(view.clone())
    });
}
