//! One truck, coach or trailer: its plate (and its colours), odometer, fuel, badge, a trailer's
//! cargo, weight and the countries it may enter, and the wear of each part.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    progress::Progress,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px, rems,
};
use save::{Vehicle, VehicleKind};

use super::garage_picker::GaragePicker;
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
    /// The garage, as a picker that moves the truck (trucks in a garage only).
    garage: Option<Entity<GaragePicker>>,
    /// The fuel level in percent (trucks and coaches): more than 100 is more than the tank holds.
    fuel: Option<Entity<AmountField>>,
    /// A trailer's cargo and its own weight, in kilograms (all its trailers together).
    cargo: Option<Entity<AmountField>>,
    empty_weight: Option<Entity<AmountField>>,
    /// Colour schemes for the plate.
    plate_colours: Entity<SelectState<SearchableVec<PlateColours>>>,
    /// The engine and transmission (trucks and coaches with a known model), filled in once the
    /// model's parts are read from the game.
    parts: Vec<PartPicker>,
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
                .id("odometer-input")
                .min(0)
                .suffix("km")
                .compact()
        });
        let plate_text = vehicle
            .as_ref()
            .map(|v| v.plate.clone())
            .unwrap_or_default();
        let plate = cx.new(|cx| InputState::new(window, cx).default_value(plate_text));
        let is_garaged_truck = vehicle
            .as_ref()
            .is_some_and(|v| v.kind == VehicleKind::Truck && v.garage_city.is_some());
        let garage = is_garaged_truck.then(|| {
            let (session, id) = (session.clone(), id.clone());
            cx.new(|cx| GaragePicker::new(session, id, true, window, cx))
        });
        let fuel = vehicle.as_ref().and_then(|v| v.fuel).map(|fuel| {
            let percent = (f64::from(fuel) * 100.0).round() as i64;
            let label = t!(cx, "column-fuel");
            cx.new(|cx| {
                AmountField::new(percent, label, window, cx)
                    .id("fuel-input")
                    .min(0)
                    .suffix("%")
                    .compact()
            })
        });
        let load = vehicle
            .as_ref()
            .filter(|v| v.kind == VehicleKind::Trailer)
            .and_then(|_| session.read(cx).trailer_load(&id));
        let cargo = load.as_ref().map(|load| {
            let label = t!(cx, "trailer-cargo");
            let kg = load.cargo_kg.round() as i64;
            cx.new(|cx| {
                AmountField::new(kg, label, window, cx)
                    .id("cargo-input")
                    .min(0)
                    .suffix("kg")
                    .compact()
            })
        });
        let empty_weight = load.as_ref().map(|load| {
            let label = t!(cx, "trailer-empty-weight");
            let kg = load.empty_kg.round() as i64;
            cx.new(|cx| {
                AmountField::new(kg, label, window, cx)
                    .id("empty-weight-input")
                    .min(0)
                    .suffix("kg")
                    .compact()
            })
        });
        let plate_colours = cx.new(|cx| {
            SelectState::new(SearchableVec::new(PlateColours::all(cx)), None, window, cx)
        });
        let model = vehicle
            .as_ref()
            .filter(|v| matches!(v.kind, VehicleKind::Truck | VehicleKind::Coach))
            .and_then(|v| v.model_id.clone());
        let mut parts = Vec::new();
        if let Some(model) = model {
            for (kind, part) in [
                (gamedata::PartKind::Engine, "engine"),
                (gamedata::PartKind::Transmission, "transmission"),
            ] {
                let select = cx.new(|cx| {
                    SelectState::new(SearchableVec::new(Vec::new()), None, window, cx)
                        .searchable(true)
                });
                parts.push(PartPicker {
                    kind,
                    part,
                    model: model.clone(),
                    select,
                    choices: Vec::new(),
                    is_filled: false,
                });
            }
        }
        let mut _subscriptions = vec![
            cx.subscribe_in(
                &plate_colours,
                window,
                |this, _, event: &SelectEvent<SearchableVec<PlateColours>>, window, cx| {
                    if let SelectEvent::Confirm(Some(key)) = event {
                        this.write_plate_colours(key, cx);
                        // An action, not a setting: the list goes back to its prompt.
                        this.plate_colours
                            .update(cx, |select, cx| select.set_selected_index(None, window, cx));
                    }
                },
            ),
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
        for (index, picker) in parts.iter().enumerate() {
            _subscriptions.push(cx.subscribe_in(
                &picker.select,
                window,
                move |this, _, event: &SelectEvent<SearchableVec<PartOption>>, _, cx| {
                    let SelectEvent::Confirm(Some(path)) = event else {
                        return;
                    };
                    let Some(picker) = this.parts.get(index) else {
                        return;
                    };
                    let Some(choice) = picker.choices.iter().find(|c| c.path == path.as_ref())
                    else {
                        return;
                    };
                    let edit = save::VehicleEdit::Part {
                        vehicle: this.id.clone(),
                        part: picker.part.to_string(),
                        data_path: choice.path.clone(),
                        price: choice.price,
                    };
                    this.session
                        .update(cx, |session, cx| session.edit_vehicle(edit, cx));
                },
            ));
        }
        if let Some(field) = &fuel {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(percent), cx| {
                let edit = save::VehicleEdit::Fuel {
                    vehicle: this.id.clone(),
                    ratio: (*percent).max(0) as f64 / 100.0,
                };
                this.session
                    .update(cx, |session, cx| session.edit_vehicle(edit, cx));
            }));
        }
        if let Some(field) = &cargo {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(kg), cx| {
                let edit = save::VehicleEdit::CargoMass {
                    trailer: this.id.clone(),
                    kg: (*kg).max(0) as f64,
                };
                this.session
                    .update(cx, |session, cx| session.edit_vehicle(edit, cx));
            }));
        }
        if let Some(field) = &empty_weight {
            _subscriptions.push(cx.subscribe(field, |this, _, AmountChanged(kg), cx| {
                let edit = save::VehicleEdit::EmptyMass {
                    trailer: this.id.clone(),
                    kg: (*kg).max(0) as f64,
                };
                this.session
                    .update(cx, |session, cx| session.edit_vehicle(edit, cx));
            }));
        }
        Self {
            session,
            id,
            odometer,
            plate,
            garage,
            fuel,
            cargo,
            empty_weight,
            plate_colours,
            parts,
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
        if let (Some(field), Some(fuel)) = (&self.fuel, vehicle.fuel) {
            let percent = (f64::from(fuel) * 100.0).round() as i64;
            if field.read(cx).value() != percent {
                field.update(cx, |field, cx| field.set_value(percent, window, cx));
            }
        }
        if let Some(load) = self.session.read(cx).trailer_load(&self.id) {
            for (field, kg) in [
                (&self.cargo, load.cargo_kg),
                (&self.empty_weight, load.empty_kg),
            ] {
                let kg = kg.round() as i64;
                if let Some(field) = field
                    && field.read(cx).value() != kg
                {
                    field.update(cx, |field, cx| field.set_value(kg, window, cx));
                }
            }
        }
        cx.notify();
    }

    /// Writes the plate in a colour scheme (or plain again), keeping its text and country.
    fn write_plate_colours(&mut self, key: &str, cx: &mut Context<Self>) {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return;
        };
        let country = vehicle.plate_country.clone().unwrap_or_default();
        let plate = match PlateColours::colours(key) {
            Some((background, letters)) => {
                save::coloured_plate(&vehicle.plate, &country, background, letters)
            }
            None if country.is_empty() => vehicle.plate.clone(),
            None => format!("{}|{country}", vehicle.plate),
        };
        let value = format!("\"{plate}\"");
        let id = self.id.clone();
        self.session.update(cx, |session, cx| {
            session.set_field(&id, "license_plate", None, value, cx)
        });
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

impl VehicleDetail {
    /// Fills the engine and transmission lists once the model's parts are read (the first ask
    /// starts reading; the window redraws when they're there), with the truck's own selected.
    fn fill_parts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let preview = self.session.read(cx).preview();
        for picker in &mut self.parts {
            if picker.is_filled {
                continue;
            }
            let Some(choices) = game_data::model_parts(cx, &picker.model, picker.kind) else {
                continue;
            };
            let current = save::vehicle_part(&preview, &self.id, picker.part).map(|(_, path)| path);
            let items: Vec<PartOption> = choices
                .iter()
                .map(|choice| PartOption {
                    path: choice.path.clone().into(),
                    title: if choice.details.is_empty() {
                        choice.name.clone().into()
                    } else {
                        // The first detail says the most (an engine's power).
                        let first = choice.details.split(" · ").next().unwrap_or_default();
                        format!("{} · {first}", choice.name).into()
                    },
                })
                .collect();
            picker.select.update(cx, |select, cx| {
                select.set_items(SearchableVec::new(items), window, cx);
                if let Some(current) = &current {
                    select.set_selected_value(&SharedString::from(current.clone()), window, cx);
                }
            });
            picker.choices = choices;
            picker.is_filled = true;
        }
    }
}

impl Render for VehicleDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.fill_parts(window, cx);
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
                                    .child(div().w(rems(7.)).text_sm().child(part_label(label, cx)))
                                    .child(div().flex_1().child(
                                        Progress::new(("wear", index)).value(percent as f32),
                                    ))
                                    .child(div().w(rems(2.5)).text_sm().text_right().child(t!(
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

        // Trucks and trailers can be removed from the company; coaches aren't part of it.
        let removal = match vehicle.kind {
            VehicleKind::Truck => Some((
                save::Removal::Truck(self.id.clone()),
                t!(cx, "remove-truck"),
            )),
            VehicleKind::Trailer => Some((
                save::Removal::Trailer(self.id.clone()),
                t!(cx, "remove-trailer"),
            )),
            VehicleKind::Coach => None,
        };
        let id = self.id.clone();
        let refuel_id = self.id.clone();
        let has_badge =
            vehicle.kind == VehicleKind::Truck && self.session.read(cx).has_badge(&self.id);
        let load = (vehicle.kind == VehicleKind::Trailer)
            .then(|| self.session.read(cx).trailer_load(&self.id))
            .flatten();
        v_flex()
            .pt_2()
            .gap_6()
            .child(group(
                t!(cx, "detail-about"),
                v_flex()
                    .gap_1()
                    .child(fact(
                        t!(cx, "column-plate"),
                        div().w(rems(11.)).child(Input::new(&self.plate).small()),
                        cx,
                    ))
                    .child(fact(
                        t!(cx, "plate-colours"),
                        div().w(rems(11.)).child(
                            Select::new(&self.plate_colours)
                                .small()
                                .placeholder(t!(cx, "plate-colours-pick")),
                        ),
                        cx,
                    ))
                    .child(match &self.garage {
                        Some(picker) => {
                            fact(t!(cx, "column-garage"), picker.clone(), cx).into_any_element()
                        }
                        None => fact(t!(cx, "column-garage"), where_kept, cx).into_any_element(),
                    })
                    .child(fact(
                        t!(cx, "column-odometer"),
                        div().w(rems(11.)).child(self.odometer.clone()),
                        cx,
                    ))
                    .children(self.parts.iter().map(|picker| {
                        let label = match picker.kind {
                            gamedata::PartKind::Engine => t!(cx, "truck-engine"),
                            gamedata::PartKind::Transmission => t!(cx, "truck-transmission"),
                        };
                        let placeholder = if picker.is_filled {
                            t!(cx, "truck-part-pick")
                        } else {
                            t!(cx, "truck-part-loading")
                        };
                        fact(
                            label,
                            div().w(rems(15.)).child(
                                Select::new(&picker.select)
                                    .small()
                                    .placeholder(placeholder)
                                    .disabled(!picker.is_filled),
                            ),
                            cx,
                        )
                    }))
                    .children(self.fuel.clone().map(|field| {
                        let is_full = vehicle.fuel.is_some_and(|fuel| fuel >= 1.0);
                        fact(
                            t!(cx, "column-fuel"),
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().w(rems(7.)).child(field))
                                .child(
                                    Button::new("fill-tank")
                                        .xsmall()
                                        .ghost()
                                        .label(t!(cx, "fill-tank"))
                                        .disabled(is_full)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let id = refuel_id.clone();
                                            this.session
                                                .update(cx, |session, cx| session.refuel(&id, cx));
                                        })),
                                ),
                            cx,
                        )
                    }))
                    .children((vehicle.kind == VehicleKind::Truck).then(|| {
                        let session = self.session.read(cx);
                        let is_yours = session.own_truck().as_deref() == Some(self.id.as_str());
                        let problem = session.can_drive(&self.id).err();
                        let id = self.id.clone();
                        let content = if is_yours {
                            div()
                                .text_sm()
                                .child(t!(cx, "player-truck-yours"))
                                .into_any_element()
                        } else {
                            let why = problem.map(|problem| match problem {
                                save::PlayerProblem::OnJob => t!(cx, "player-why-on-job"),
                                _ => t!(cx, "player-why-not-in-garage"),
                            });
                            v_flex()
                                .items_end()
                                .gap_1()
                                .child(
                                    Button::new("drive-truck")
                                        .xsmall()
                                        .outline()
                                        .label(t!(cx, "player-drive"))
                                        .tooltip(t!(cx, "player-drive-tooltip"))
                                        .disabled(problem.is_some())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let id = id.clone();
                                            this.session.update(cx, |session, cx| {
                                                session.drive_truck(&id, cx)
                                            });
                                        })),
                                )
                                .children(why.map(|why| {
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(why)
                                }))
                                .into_any_element()
                        };
                        fact(t!(cx, "player-truck"), content, cx)
                    }))
                    .children((vehicle.kind == VehicleKind::Trailer).then(|| {
                        let session = self.session.read(cx);
                        let is_yours = save::own_trailer(&session.preview()).as_deref()
                            == Some(self.id.as_str());
                        let problem = session.can_take_trailer(&self.id).err();
                        let id = self.id.clone();
                        let content = if is_yours {
                            div()
                                .text_sm()
                                .child(t!(cx, "player-trailer-yours"))
                                .into_any_element()
                        } else {
                            let why = problem.and_then(|problem| match problem {
                                save::PlayerProblem::OnJob => Some(t!(cx, "player-why-on-job")),
                                _ => None,
                            });
                            v_flex()
                                .items_end()
                                .gap_1()
                                .child(
                                    Button::new("pull-trailer")
                                        .xsmall()
                                        .outline()
                                        .label(t!(cx, "player-pull"))
                                        .tooltip(t!(cx, "player-pull-tooltip"))
                                        .disabled(problem.is_some())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let edit = save::PlayerEdit::TakeTrailer {
                                                trailer: id.clone(),
                                            };
                                            this.session.update(cx, |session, cx| {
                                                session.edit_player(edit, cx)
                                            });
                                        })),
                                )
                                .children(why.map(|why| {
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(why)
                                }))
                                .into_any_element()
                        };
                        fact(t!(cx, "player-trailer"), content, cx)
                    }))
                    .children(has_badge.then(|| {
                        let id = self.id.clone();
                        fact(
                            t!(cx, "truck-badge"),
                            Button::new("remove-badge")
                                .xsmall()
                                .ghost()
                                .label(t!(cx, "truck-remove-badge"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let edit = save::VehicleEdit::RemoveBadge {
                                        vehicle: id.clone(),
                                    };
                                    this.session
                                        .update(cx, |session, cx| session.edit_vehicle(edit, cx));
                                })),
                            cx,
                        )
                    })),
            ))
            .children(load.map(|load| {
                let id = self.id.clone();
                let countries = if load.countries.is_empty() {
                    t!(cx, "trailer-everywhere")
                } else {
                    load.countries
                        .iter()
                        .map(|token| game_data::country_name(cx, token))
                        .collect::<Vec<_>>()
                        .join(", ")
                        .into()
                };
                group(
                    t!(cx, "trailer-load"),
                    v_flex()
                        .gap_1()
                        .children(self.cargo.clone().map(|field| {
                            fact(t!(cx, "trailer-cargo"), div().w(rems(11.)).child(field), cx)
                        }))
                        .children(self.empty_weight.clone().map(|field| {
                            fact(
                                t!(cx, "trailer-empty-weight"),
                                div().w(rems(11.)).child(field),
                                cx,
                            )
                        }))
                        .child(
                            v_flex()
                                .gap_1()
                                .child(fact(
                                    t!(cx, "trailer-countries"),
                                    Button::new("unrestrict")
                                        .xsmall()
                                        .ghost()
                                        .label(t!(cx, "trailer-allow-everywhere"))
                                        .disabled(load.countries.is_empty())
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let edit = save::VehicleEdit::Unrestrict {
                                                trailer: id.clone(),
                                            };
                                            this.session.update(cx, |session, cx| {
                                                session.edit_vehicle(edit, cx)
                                            });
                                        })),
                                    cx,
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(countries),
                                ),
                        ),
                )
            }))
            .child(group(t!(cx, "column-wear"), wear))
            .child(super::actions_row(
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
                &self.session,
                removal,
                cx,
            ))
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

/// Opens a vehicle's panel, titled with its brand's logo and model.
pub fn open(
    session: Entity<Session>,
    vehicle: &Vehicle,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    let id = vehicle.id.clone();
    let kind_name = match vehicle.kind {
        VehicleKind::Truck => t!(cx, "detail-truck"),
        VehicleKind::Coach => t!(cx, "detail-coach"),
        VehicleKind::Trailer => t!(cx, "detail-trailer"),
    };
    // The object itself is the title; its kind is the fallback when the model isn't known.
    let name: SharedString = game_data::model_name(cx, vehicle)
        .map(SharedString::from)
        .unwrap_or(kind_name);
    let logo = game_data::brand_logo(cx, vehicle);
    let view = cx.new(|cx| VehicleDetail::new(session, id, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        let title = h_flex()
            .gap_2()
            .items_center()
            .children(logo.clone().map(|logo| {
                GameIconView::new(logo)
                    .size(1.1)
                    .channel(gamedata::RED, cx.theme().muted_foreground)
            }))
            .child(name.clone());
        sheet.title(title).size(px(SHEET_WIDTH)).child(view.clone())
    });
}

/// A colour scheme for a plate, in the plate colours list.
#[derive(Clone)]
struct PlateColours {
    key: SharedString,
    name: SharedString,
}

impl PlateColours {
    /// The schemes, with the game's own plate first.
    fn all(cx: &gpui_kit::App) -> Vec<PlateColours> {
        let scheme = |key: &'static str, name: SharedString| PlateColours {
            key: key.into(),
            name,
        };
        vec![
            scheme("plain", t!(cx, "plate-plain")),
            scheme("yellow", t!(cx, "plate-black-on-yellow")),
            scheme("white", t!(cx, "plate-black-on-white")),
            scheme("black", t!(cx, "plate-white-on-black")),
            scheme("blue", t!(cx, "plate-white-on-blue")),
            scheme("red", t!(cx, "plate-white-on-red")),
            scheme("green", t!(cx, "plate-white-on-green")),
            scheme("gold", t!(cx, "plate-gold-on-black")),
        ]
    }

    /// (background, letters) as red, green, blue; `None` for the game's own plate.
    fn colours(key: &str) -> Option<([u8; 3], [u8; 3])> {
        const BLACK: [u8; 3] = [0, 0, 0];
        const WHITE: [u8; 3] = [255, 255, 255];
        match key {
            "yellow" => Some(([255, 204, 0], BLACK)),
            "white" => Some((WHITE, BLACK)),
            "black" => Some((BLACK, WHITE)),
            "blue" => Some(([0, 51, 153], WHITE)),
            "red" => Some(([190, 20, 30], WHITE)),
            "green" => Some(([0, 110, 60], WHITE)),
            "gold" => Some((BLACK, [212, 175, 55])),
            _ => None,
        }
    }
}

impl SelectItem for PlateColours {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.key
    }
}

/// An engine or transmission list in a truck's panel.
struct PartPicker {
    kind: gamedata::PartKind,
    /// The accessory folder it's about (`engine`, `transmission`).
    part: &'static str,
    model: String,
    select: Entity<SelectState<SearchableVec<PartOption>>>,
    choices: Vec<game_data::PartChoice>,
    is_filled: bool,
}

/// One engine or transmission in a list: shown by its name and what it's like, chosen by its
/// definition.
#[derive(Clone)]
struct PartOption {
    path: SharedString,
    title: SharedString,
}

impl SelectItem for PartOption {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.path
    }
}
