//! One truck, coach or trailer: its plate (and its colours), odometer, fuel, badge, a trailer's
//! cargo, weight and the countries it may enter, and the wear of each part.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    slider::{Slider, SliderEvent, SliderState},
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
use crate::ui::Tip as _;
use crate::ui::{AmountChanged, AmountField, GameIconView};
use crate::units::Unit;

/// The wear fields a vehicle keeps on itself (the current format), with their labels' keys.
const PARTS: [(&str, &str); 6] = [
    ("engine_wear", "part-engine"),
    ("transmission_wear", "part-transmission"),
    ("cabin_wear", "part-cabin"),
    ("chassis_wear", "part-chassis"),
    ("trailer_body_wear", "part-body"),
    ("wheels_wear", "part-wheels"),
];

/// One part's wear as a slider, in percent: dragged to set it.
struct WearSlider {
    field: &'static str,
    label: &'static str,
    state: Entity<SliderState>,
    /// Where the slider is while it's dragged (shown as the percent beside it); the change is
    /// made when it's let go, so one drag is one step for Undo.
    dragging: Option<f32>,
}

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
    /// Each part's wear (current-format vehicles keep it on themselves; older ones have none).
    wear: Vec<WearSlider>,
    /// The paint job and its colours (`None` for a vehicle without paint the editor knows).
    paint: Option<Entity<super::paint::PaintEditor>>,
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
        // Distances and weights in the game's units (miles and pounds in ATS); stored in
        // kilometres and kilograms.
        let game = session.read(cx).game();
        let (distance_unit, weight_unit) = (Unit::distance(game), Unit::weight(game));
        let odometer_km = vehicle.as_ref().map_or(0, |v| v.odometer_km);
        let label = t!(cx, "column-odometer");
        let distance_symbol = distance_unit.symbol(cx);
        let odometer = cx.new(|cx| {
            AmountField::new(distance_unit.shown(odometer_km as f64), label, window, cx)
                .id("odometer-input")
                .min(0)
                .suffix(distance_symbol)
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
        let weight_symbol = weight_unit.symbol(cx);
        let cargo = load.as_ref().map(|load| {
            let label = t!(cx, "trailer-cargo");
            let symbol = weight_symbol.clone();
            cx.new(|cx| {
                AmountField::new(weight_unit.shown(load.cargo_kg), label, window, cx)
                    .id("cargo-input")
                    .min(0)
                    .suffix(symbol)
                    .compact()
            })
        });
        let empty_weight = load.as_ref().map(|load| {
            let label = t!(cx, "trailer-empty-weight");
            let symbol = weight_symbol.clone();
            cx.new(|cx| {
                AmountField::new(weight_unit.shown(load.empty_kg), label, window, cx)
                    .id("empty-weight-input")
                    .min(0)
                    .suffix(symbol)
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
        let unit = session.read(cx).unit(&id);
        let wear: Vec<WearSlider> = PARTS
            .iter()
            .filter_map(|&(field, label)| {
                let wear = part_wear(unit.as_ref()?, field)?;
                let state = cx.new(|_| {
                    SliderState::new()
                        .min(0.)
                        .max(100.)
                        .step(1.)
                        .default_value(wear * 100.)
                });
                Some(WearSlider {
                    field,
                    label,
                    state,
                    dragging: None,
                })
            })
            .collect();
        // Paint the editor knows: a paint job in a model's folder.
        let paint_model = save::vehicle_paint(&session.read(cx).preview(), &id)
            .and_then(|paint| save::paint_model(&paint.data_path).map(str::to_string));
        let paint = paint_model.map(|model| {
            let (session, id) = (session.clone(), id.clone());
            cx.new(|cx| super::paint::PaintEditor::new(session, id, model, window, cx))
        });
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
            cx.subscribe(&odometer, move |this, _, AmountChanged(shown), cx| {
                let id = this.id.clone();
                let km = distance_unit.stored((*shown).max(0)).round() as i64;
                let km = km.to_string();
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
        for (index, slider) in wear.iter().enumerate() {
            _subscriptions.push(cx.subscribe(
                &slider.state,
                move |this, _, event: &SliderEvent, cx| {
                    let Some(slider) = this.wear.get_mut(index) else {
                        return;
                    };
                    match event {
                        SliderEvent::Change(value) => slider.dragging = Some(value.start()),
                        SliderEvent::Release(value) => {
                            slider.dragging = None;
                            let (id, field) = (this.id.clone(), slider.field);
                            let wear = value.start() / 100.0;
                            this.session
                                .update(cx, |session, cx| session.set_wear(&id, field, wear, cx));
                        }
                    }
                    cx.notify();
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
            _subscriptions.push(
                cx.subscribe(field, move |this, _, AmountChanged(shown), cx| {
                    let edit = save::VehicleEdit::CargoMass {
                        trailer: this.id.clone(),
                        kg: weight_unit.stored((*shown).max(0)),
                    };
                    this.session
                        .update(cx, |session, cx| session.edit_vehicle(edit, cx));
                }),
            );
        }
        if let Some(field) = &empty_weight {
            _subscriptions.push(
                cx.subscribe(field, move |this, _, AmountChanged(shown), cx| {
                    let edit = save::VehicleEdit::EmptyMass {
                        trailer: this.id.clone(),
                        kg: weight_unit.stored((*shown).max(0)),
                    };
                    this.session
                        .update(cx, |session, cx| session.edit_vehicle(edit, cx));
                }),
            );
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
            wear,
            paint,
            _subscriptions,
        }
    }

    fn sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return;
        };
        let odometer =
            Unit::distance(self.session.read(cx).game()).shown(vehicle.odometer_km as f64);
        if self.odometer.read(cx).value() != odometer {
            self.odometer
                .update(cx, |field, cx| field.set_value(odometer, window, cx));
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
        // Undo, Repair and the like move the sliders; one being dragged is left where it is.
        if let Some(unit) = self.session.read(cx).unit(&self.id) {
            for slider in &self.wear {
                let Some(wear) = part_wear(&unit, slider.field) else {
                    continue;
                };
                let percent = wear * 100.0;
                if slider.dragging.is_none()
                    && (slider.state.read(cx).value().start() - percent).abs() > 0.01
                {
                    slider
                        .state
                        .update(cx, |state, cx| state.set_value(percent, window, cx));
                }
            }
        }
        if let Some(load) = self.session.read(cx).trailer_load(&self.id) {
            let unit = Unit::weight(self.session.read(cx).game());
            for (field, kg) in [
                (&self.cargo, load.cargo_kg),
                (&self.empty_weight, load.empty_kg),
            ] {
                let shown = unit.shown(kg);
                if let Some(field) = field
                    && field.read(cx).value() != shown
                {
                    field.update(cx, |field, cx| field.set_value(shown, window, cx));
                }
            }
        }
        cx.notify();
    }

    /// Writes the plate in a colour scheme (or plain again), keeping its text. A coloured plate is a
    /// Belgian one; plain again, it keeps the country it has (Belgian, after colouring).
    fn write_plate_colours(&mut self, key: &str, cx: &mut Context<Self>) {
        let Some(vehicle) = find(&self.session, &self.id, cx) else {
            return;
        };
        let country = vehicle.plate_country.clone().unwrap_or_default();
        let plate = match PlateColours::colours(key) {
            // Written as a Belgian plate: the one the game colours (see `coloured_plate`).
            Some((background, letters)) => {
                save::coloured_plate(&vehicle.plate, background, letters)
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

    /// Repairs one part (current-format vehicles keep each part's wear on themselves): every
    /// item of it (each wheel) in one change.
    fn repair_part(&mut self, field: &'static str, cx: &mut Context<Self>) {
        let id = self.id.clone();
        self.session
            .update(cx, |session, cx| session.set_wear(&id, field, 0.0, cx));
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
            // Only what the player has (the base game's, and DLCs the save was made with), and
            // whatever the truck has now.
            let session = self.session.read(cx);
            let choices: Vec<game_data::PartChoice> = choices
                .into_iter()
                .filter(|choice| {
                    session.has_dlc(choice.dlc.as_deref())
                        || current.as_deref() == Some(choice.path.as_str())
                })
                .collect();
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

        // Older saves keep wear on each part's accessory: shown and repaired as a whole.
        let wear: gpui_kit::AnyElement = if self.wear.is_empty() {
            fact(
                t!(cx, "column-wear"),
                t!(
                    cx,
                    "percent",
                    amount = (vehicle.wear.worst() * 100.0).ceil() as i64
                ),
                cx,
            )
            .into_any_element()
        } else {
            let rows: Vec<_> = self
                .wear
                .iter()
                .enumerate()
                .map(|(index, slider)| {
                    let worn = unit
                        .as_ref()
                        .and_then(|unit| part_wear(unit, slider.field))
                        .unwrap_or_default();
                    // While dragged, where the slider is; otherwise the wear, rounded up so a
                    // part that's worn at all never reads 0 %.
                    let percent = match slider.dragging {
                        Some(percent) => percent.round() as i64,
                        None => (worn * 100.0).ceil() as i64,
                    };
                    let part = part_label(slider.label, cx);
                    let field = slider.field;
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(div().w(rems(7.)).text_sm().child(part.clone()))
                        .child(div().flex_1().child(
                            Slider::new(&slider.state).accessibility_label(t!(
                                cx,
                                "change-wear",
                                part = part
                            )),
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
                                .disabled(worn == 0.0)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.repair_part(field, cx)),
                                ),
                        )
                })
                .collect();
            v_flex().gap_2().children(rows).into_any_element()
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
        // Your truck, when the profile has a World of Trucks plate: the game puts that plate on
        // whichever truck is yours each time a save loads (found in game, 2026-11-22), so its
        // plate can't be changed here. It goes by the truck that's yours after the changes.
        let has_world_of_trucks_plate = vehicle.kind == VehicleKind::Truck && {
            let session = self.session.read(cx);
            session.world_of_trucks_plate().is_some()
                && session.own_truck().as_deref() == Some(self.id.as_str())
        };
        crate::ui::Divided::groups()
            .pt_2()
            .child(group(
                t!(cx, "detail-about"),
                v_flex()
                    .gap_1()
                    .child(fact(
                        t!(cx, "column-plate"),
                        div().w(rems(11.)).child(
                            Input::new(&self.plate)
                                .small()
                                .disabled(has_world_of_trucks_plate),
                        ),
                        cx,
                    ))
                    .child(fact(
                        t!(cx, "plate-colours"),
                        div().w(rems(11.)).child(
                            Select::new(&self.plate_colours)
                                .small()
                                .disabled(has_world_of_trucks_plate)
                                .placeholder(t!(cx, "plate-colours-pick")),
                        ),
                        cx,
                    ))
                    .children(has_world_of_trucks_plate.then(|| {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "plate-world-of-trucks"))
                    }))
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
                                        .tip(t!(cx, "player-drive-tooltip"))
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
                                        .tip(t!(cx, "player-pull-tooltip"))
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
                                .tip(t!(cx, "truck-remove-badge-tooltip"))
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
                                    t!(
                                        cx,
                                        "trailer-countries",
                                        game = crate::labels::current_game_key(cx)
                                    ),
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
            .children(
                self.paint
                    .clone()
                    .map(|paint| group(t!(cx, "paint"), paint)),
            )
            .child(group(t!(cx, "column-wear"), wear))
            .child(super::actions_row(
                h_flex()
                    .gap_2()
                    .child(
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
                    )
                    // Trucks and trailers are copied into a garage; a coach joins the coaches.
                    .child(if vehicle.kind == VehicleKind::Coach {
                        self.copy_coach_button(cx)
                    } else {
                        self.copy_button(cx)
                    }),
                &self.session,
                removal,
                cx,
            ))
            .into_any_element()
    }
}

impl VehicleDetail {
    /// Copy coach: a copy joins the player's coaches (no garage), with a coach's plate for the
    /// headquarters' country.
    fn copy_coach_button(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let session = self.session.clone();
        let id = self.id.clone();
        Button::new("copy-coach")
            .small()
            .outline()
            .label(t!(cx, "copy-coach"))
            .tip(t!(cx, "copy-coach-tooltip"))
            .on_click(move |_, _, cx| {
                let hq = format!("garage.{}", session.read(cx).career().hq_city);
                let plates = game_data::new_plates(cx, &hq, "bus", 1);
                session.update(cx, |session, cx| session.copy_coach(&id, plates, cx));
            })
            .into_any_element()
    }

    /// Copy to a garage: a menu of the company's garages with room for one more, by city.
    /// Without room anywhere, it's off and says how to make some.
    fn copy_button(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let targets = self
            .session
            .read(cx)
            .copy_targets(&self.id)
            .unwrap_or_default();
        let button = Button::new("copy-vehicle")
            .small()
            .outline()
            .label(t!(cx, "copy-to-garage"));
        if targets.is_empty() {
            return button
                .tip(t!(cx, "copy-no-room"))
                .disabled(true)
                .into_any_element();
        }
        let session = self.session.clone();
        let id = self.id.clone();
        let is_trailer = self
            .session
            .read(cx)
            .vehicles()
            .iter()
            .any(|vehicle| vehicle.id == self.id && vehicle.kind == VehicleKind::Trailer);
        let plate_kind = if is_trailer { "trailer" } else { "truck" };
        button
            .tip(t!(cx, "copy-tooltip"))
            // Upward: the button sits at the panel's foot, where a menu opening down would cover it.
            .dropdown_menu_with_anchor(gpui_kit::Anchor::BottomLeft, move |menu, _, cx| {
                targets.iter().fold(menu, |menu, garage| {
                    let city =
                        game_data::city_name(cx, garage.strip_prefix("garage.").unwrap_or(garage));
                    // Each item needs its own copies to move into its click handler.
                    let (session, id, garage) = (session.clone(), id.clone(), garage.clone());
                    menu.item(PopupMenuItem::new(city).on_click(move |_, _, cx| {
                        // Plates for the copy's trucks or trailers (a road train has several; the
                        // extra ones are left over), made for that garage's country.
                        let plates = game_data::new_plates(cx, &garage, plate_kind, 8);
                        session.update(cx, |session, cx| {
                            session.copy_vehicle(&id, &garage, plates, cx)
                        });
                    }))
                })
            })
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
        crate::labels::PLATE_SCHEMES
            .iter()
            .map(|key| PlateColours {
                key: (*key).into(),
                name: crate::labels::plate_scheme_name(cx, Some(key)),
            })
            .collect()
    }

    /// (background, letters) as red, green, blue; `None` for the game's own plate.
    fn colours(key: &str) -> Option<([u8; 3], [u8; 3])> {
        crate::labels::plate_scheme_colours(key)
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
