//! One truck, coach or trailer: its plate, odometer, fuel and the wear of each part.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    progress::Progress,
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px,
};
use save::{Vehicle, VehicleKind};

use super::{SHEET_WIDTH, fact, group};
use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::{AmountChanged, AmountField, GameIconView};

/// The wear fields a vehicle keeps on itself (the current format), with their labels' keys.
const PARTS: [(&str, &str); 6] = [
    ("engine_wear", "part-engine"),
    ("transmission_wear", "part-transmission"),
    ("cabin_wear", "part-cabin"),
    ("chassis_wear", "part-chassis"),
    ("trailer_body_wear", "part-body"),
    ("wheels_wear", "part-wheels"),
];

pub struct VehicleDetail {
    session: Entity<Session>,
    id: String,
    odometer: Entity<AmountField>,
    plate: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl VehicleDetail {
    fn new(
        session: Entity<Session>,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let vehicle = find(&session, &id, cx);
        let odometer_km = vehicle.as_ref().map_or(0, |v| v.odometer_km);
        let label = t!(cx, "column-odometer");
        let odometer = cx.new(|cx| {
            AmountField::new(odometer_km, label, window, cx)
                .min(0)
                .suffix("km")
        });
        let plate_text = vehicle
            .as_ref()
            .map(|v| v.plate.clone())
            .unwrap_or_default();
        let plate = cx.new(|cx| InputState::new(window, cx).default_value(plate_text));
        let _subscriptions = vec![
            cx.subscribe(&odometer, |this, _, AmountChanged(km), cx| {
                let id = this.id.clone();
                let km = (*km).max(0).to_string();
                this.session.update(cx, |session, cx| {
                    session.set_field(&id, "odometer", None, km, cx)
                });
            }),
            cx.subscribe_in(&plate, window, |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                    this.write_plate(cx);
                }
            }),
            // Undo, Discard and other pages change what this shows.
            cx.observe_in(&session, window, |this, _, window, cx| {
                this.sync(window, cx)
            }),
        ];
        Self {
            session,
            id,
            odometer,
            plate,
            _subscriptions,
        }
    }

    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return;
        };
        if self.odometer.read(cx).value() != vehicle.odometer_km {
            self.odometer.update(cx, |field, cx| {
                field.set_value(vehicle.odometer_km, window, cx)
            });
        }
        if self.plate.read(cx).value().as_ref() != vehicle.plate {
            self.plate.update(cx, |input, cx| {
                input.set_value(vehicle.plate.clone(), window, cx)
            });
        }
        cx.notify();
    }

    /// Writes the plate, keeping its country (`AB 123|germany`). Formatting tags the game put
    /// in the original plate (spacing) are dropped with a change; the game places plain text fine.
    fn write_plate(&mut self, cx: &mut Context<Self>) {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return;
        };
        let text: String = self
            .plate
            .read(cx)
            .value()
            .chars()
            .filter(|c| *c != '"' && *c != '|' && *c != '\\')
            .collect();
        let text = text.trim().to_string();
        if text == vehicle.plate || text.is_empty() {
            return;
        }
        let value = match &vehicle.plate_country {
            Some(country) => format!("\"{text}|{country}\""),
            None => format!("\"{text}\""),
        };
        let id = self.id.clone();
        self.session.update(cx, |session, cx| {
            session.set_field(&id, "license_plate", None, value, cx)
        });
    }

    /// Repairs one part (current-format vehicles keep each part's wear on themselves).
    fn repair_part(&mut self, field: &'static str, cx: &mut Context<Self>) {
        let Some(unit) = self.session.read(cx).unit(&self.id) else {
            return;
        };
        let id = self.id.clone();
        let count = match unit.get(field) {
            Some(sii::Value::Array(items)) => Some(items.len()),
            Some(sii::Value::Single(_)) => None,
            None => return,
        };
        self.session.update(cx, |session, cx| match count {
            Some(count) => {
                for index in 0..count {
                    session.set_field(&id, field, Some(index), "0".into(), cx);
                }
            }
            None => session.set_field(&id, field, None, "0".into(), cx),
        });
    }
}

fn find(session: &Entity<Session>, id: &str, cx: &gpui_kit::App) -> Option<Vehicle> {
    session
        .read(cx)
        .vehicles()
        .into_iter()
        .find(|vehicle| vehicle.id == id)
}

/// A part's wear from 0 to 1: the field's value, or the most worn item of an array.
fn part_wear(unit: &sii::Unit, field: &str) -> Option<f32> {
    match unit.get(field)? {
        sii::Value::Single(text) => sii::value::parse_f32(text),
        sii::Value::Array(items) => Some(
            items
                .iter()
                .filter_map(|text| sii::value::parse_f32(text))
                .fold(0.0, f32::max),
        ),
    }
}

impl Render for VehicleDetail {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return div().child(t!(cx, "detail-gone")).into_any_element();
        };
        let unit = self.session.read(cx).unit(&self.id);
        let where_kept: SharedString = match (
            &vehicle.garage_city,
            vehicle.is_in_use,
            &vehicle.parked_city,
        ) {
            (Some(city), _, _) => game_data::city_name(cx, city).into(),
            (None, true, _) => t!(cx, "vehicle-in-use"),
            (None, false, Some(city)) => {
                t!(cx, "vehicle-parked", city = game_data::city_name(cx, city))
            }
            (None, false, None) => t!(cx, "no-garage"),
        };

        let parts = unit.as_ref().map(|unit| {
            PARTS
                .iter()
                .filter_map(|(field, label)| Some((*field, *label, part_wear(unit, field)?)))
                .collect::<Vec<_>>()
        });
        let wear: gpui_kit::AnyElement =
            match parts.filter(|parts| !parts.is_empty()) {
                Some(parts) => {
                    v_flex()
                        .gap_2()
                        .children(parts.into_iter().enumerate().map(
                            |(index, (field, label, wear))| {
                                let percent = (wear * 100.0).ceil() as i64;
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .child(div().w(px(110.)).text_sm().child(part_label(label, cx)))
                                    .child(div().flex_1().child(
                                        Progress::new(("wear", index)).value(percent as f32),
                                    ))
                                    .child(div().w(px(40.)).text_sm().text_right().child(t!(
                                        cx,
                                        "percent",
                                        amount = percent
                                    )))
                                    .child(
                                        Button::new(("repair-part", index))
                                            .xsmall()
                                            .ghost()
                                            .label(t!(cx, "repair"))
                                            .disabled(percent == 0)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.repair_part(field, cx)
                                            })),
                                    )
                            },
                        ))
                        .into_any_element()
                }
                // Older saves keep wear on each part's accessory: repair as a whole.
                None => fact(
                    t!(cx, "column-wear"),
                    t!(
                        cx,
                        "percent",
                        amount = (vehicle.wear.worst() * 100.0).ceil() as i64
                    ),
                    cx,
                )
                .into_any_element(),
            };

        let id = self.id.clone();
        let refuel_id = self.id.clone();
        v_flex()
            .gap_6()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .children(game_data::brand_logo(cx, &vehicle).map(|logo| {
                        GameIconView::new(logo)
                            .size(1.4)
                            .channel(gamedata::RED, cx.theme().muted_foreground)
                    }))
                    .child(
                        div()
                            .text_lg()
                            .child(game_data::model_name(cx, &vehicle).unwrap_or_default()),
                    ),
            )
            .child(group(
                t!(cx, "detail-about"),
                v_flex()
                    .gap_1()
                    .child(fact(
                        t!(cx, "column-plate"),
                        div().w(px(180.)).child(Input::new(&self.plate).small()),
                        cx,
                    ))
                    .child(fact(t!(cx, "column-garage"), where_kept, cx))
                    .child(fact(
                        t!(cx, "column-odometer"),
                        div().w(px(180.)).child(self.odometer.clone()),
                        cx,
                    ))
                    .children(vehicle.fuel.map(|fuel| {
                        let percent = (fuel * 100.0).floor() as i64;
                        fact(
                            t!(cx, "column-fuel"),
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(t!(cx, "percent", amount = percent))
                                .child(
                                    Button::new("fill-tank")
                                        .xsmall()
                                        .ghost()
                                        .label(t!(cx, "fill-tank"))
                                        .disabled(percent >= 100)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let id = refuel_id.clone();
                                            this.session
                                                .update(cx, |session, cx| session.refuel(&id, cx));
                                        })),
                                ),
                            cx,
                        )
                    })),
            ))
            .child(group(t!(cx, "column-wear"), wear))
            .child(
                h_flex().child(
                    Button::new("repair-vehicle")
                        .small()
                        .outline()
                        .label(t!(cx, "repair-all-parts"))
                        .disabled(vehicle.wear.worst() == 0.0)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let id = id.clone();
                            this.session
                                .update(cx, |session, cx| session.repair(&id, cx));
                        })),
                ),
            )
            .into_any_element()
    }
}

fn part_label(key: &str, cx: &gpui_kit::App) -> SharedString {
    match key {
        "part-engine" => t!(cx, "part-engine"),
        "part-transmission" => t!(cx, "part-transmission"),
        "part-cabin" => t!(cx, "part-cabin"),
        "part-chassis" => t!(cx, "part-chassis"),
        "part-body" => t!(cx, "part-body"),
        _ => t!(cx, "part-wheels"),
    }
}

/// Opens a vehicle's panel.
pub fn open(
    session: Entity<Session>,
    vehicle: &Vehicle,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    let id = vehicle.id.clone();
    let title = match vehicle.kind {
        VehicleKind::Truck => t!(cx, "detail-truck"),
        VehicleKind::Coach => t!(cx, "detail-coach"),
        VehicleKind::Trailer => t!(cx, "detail-trailer"),
    };
    let view = cx.new(|cx| VehicleDetail::new(session, id, window, cx));
    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(title.clone())
            .size(px(SHEET_WIDTH))
            .child(view.clone())
    });
}
