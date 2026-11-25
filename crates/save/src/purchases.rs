//! Buying a truck from a dealer's stock (`gamedata::DealerTruck`) into one of the company's
//! garages, as the game does when it sells one. The save keeps no blank truck to start from, and
//! writing a truck from a fixed list of fields would drop whatever the next game version adds (see
//! `docs/compatibility.md`), so the new truck takes its fields from a truck already in the save
//! (the company's own, else one a dealer offers used: every save has those) and starts them as a
//! new truck's:
//!
//! - wear at 0 (`…_wear`, `…_wear_unfixable`; the per-axle lists empty, as on trucks the game has
//!   never driven),
//! - a full tank (`fuel_relative`), the odometer and trip counters at 0, no passengers,
//! - bought now (`purchase_time`: the save's game time).
//!
//! Everything else (the plate, mirror and seat settings) is the other truck's. Its accessories are
//! the dealer's, under new ids, and it's linked in as a copy is (see `copies.rs`): in
//! `player.trucks` with an empty profit log, and in a free slot of the garage.
//!
//! A trailer (`gamedata::DealerTrailer`: one trailer, or a road train's chain) is bought the same
//! way, its fields from a trailer in the save started as new (also empty: no cargo or damage, not
//! oversize, axles in their default place). A save with no trailer at all has nothing to take them
//! from: the trailers then get only the dealer file's fields, and the game fills in the rest, as it
//! does for its own dealer files. Only the front trailer carries the type (a save writes `null` for
//! the trailers behind it); the type is the save's own `trailer_def` with that name
//! (`source_name`), or a new one made from the game's definition, listed in `player.trailer_defs`.
//! The front trailer goes in `player.trailers` with an empty usage log, and in the garage's
//! trailers (fewer than its truck slots).

use std::collections::HashMap;

use sii::{Document, Unit, Value};

use crate::copies::{empty_log, player, push_item, renamed};

/// Why a truck can't be bought there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuyProblem {
    /// The garage isn't the company's.
    NotAGarage,
    /// The garage has no free truck slot.
    GarageFull,
    /// The save has no truck, and no truck log, to take the layout from.
    NoTemplate,
    /// The dealer's truck or trailer has nothing in it, or a trailer has no type.
    Empty,
}

/// The company's garages with a free truck slot, in the save's order.
pub fn buy_targets(document: &Document) -> Vec<String> {
    document
        .units_of_kind("garage")
        .filter(|garage| garage.single("status").unwrap_or("0") != "0")
        .filter(|garage| garage.items("vehicles").iter().any(|item| item == "null"))
        .map(|garage| garage.id.clone())
        .collect()
}

/// A truck in the save to take the layout of a truck from: the company's first, else the first
/// truck a dealer offers used.
fn template_truck(document: &Document) -> Option<&Unit> {
    let player = document.unit(&player(document)?)?;
    let own = player
        .items("trucks")
        .iter()
        .find_map(|id| document.unit(id));
    own.or_else(|| {
        document
            .units_of_kind("used_truck_offer")
            .filter_map(|offer| offer.single("truck"))
            .find_map(|id| document.unit(id))
    })
    .filter(|unit| unit.kind == "vehicle")
}

/// A profit log to make the new truck's empty one like: one of the company's trucks', else a
/// garage's (every garage has one, of the same kind).
fn template_log(document: &Document) -> Option<&Unit> {
    let player = document.unit(&player(document)?)?;
    let own = player
        .items("truck_profit_logs")
        .iter()
        .find_map(|id| document.unit(id));
    own.or_else(|| document.units_of_kind("profit_log").next())
}

/// A template's fields, started as a new vehicle's (see the module's notes), pointing at
/// `accessories`.
fn start_as_new(
    template: &Unit,
    kind: &str,
    id: &str,
    accessories: &[String],
    now: Option<i64>,
) -> Unit {
    let mut vehicle = Unit::new(kind, id);
    vehicle.fields = template.fields.clone();
    for field in &mut vehicle.fields {
        let name = field.name.as_str();
        let starts_at_zero = name.ends_with("_wear")
            || name.ends_with("_wear_unfixable")
            || name.starts_with("odometer")
            || name.starts_with("integrity_odometer")
            || name.starts_with("trip_")
            || name.ends_with("passenger_mass")
            || name.starts_with("cargo_")
            || name.ends_with("_offset") && name != "user_head_offset";
        field.value = if name == "accessories" {
            Value::Array(accessories.to_vec())
        } else if name == "fuel_relative" {
            Value::single("1")
        } else if name == "oversize" || name == "is_private" {
            Value::single("false")
        } else if name == "purchase_time"
            && let Some(now) = now
        {
            Value::single(now.to_string())
        } else if starts_at_zero {
            match field.value {
                Value::Array(_) => Value::Array(Vec::new()),
                Value::Single(_) => Value::single("0"),
            }
        } else {
            continue;
        };
    }
    vehicle
}

/// Sets a field on a plain unit: in place if it's there, else at the end.
fn set_field(unit: &mut Unit, name: &str, value: Value) {
    match unit
        .fields
        .iter_mut()
        .rev()
        .find(|field| field.name == name)
    {
        Some(field) => field.value = value,
        None => unit.fields.push(sii::Field {
            name: name.to_string(),
            value,
        }),
    }
}

/// The company's garages with room for another trailer (fewer trailers than truck slots), in the
/// save's order.
pub fn trailer_targets(document: &Document) -> Vec<String> {
    document
        .units_of_kind("garage")
        .filter(|garage| garage.single("status").unwrap_or("0") != "0")
        .filter(|garage| garage.items("trailers").len() < garage.items("vehicles").len())
        .map(|garage| garage.id.clone())
        .collect()
}

/// A trailer in the save to take a trailer's fields from: the company's first, else any.
fn template_trailer(document: &Document) -> Option<&Unit> {
    let player = document.unit(&player(document)?)?;
    let own = player
        .items("trailers")
        .iter()
        .find_map(|id| document.unit(id));
    own.or_else(|| document.units_of_kind("trailer").next())
        .filter(|unit| unit.kind == "trailer")
}

/// The save's trailer type with this name (`source_name`), if it has one.
fn trailer_type_in(document: &Document, name: &str) -> Option<String> {
    document
        .units_of_kind("trailer_def")
        .find(|unit| {
            unit.single("source_name")
                .is_some_and(|source| sii::value::parse_string(source) == name)
        })
        .map(|unit| unit.id.clone())
}

/// A save's trailer type made from the game's definition: its fields, the fields the save's other
/// types have that definitions don't (as the first of them has them; `trailer: ""` and
/// `country_validity: 0` as the game wrote them, when the save has none), and `source_name`.
fn new_trailer_type(document: &Document, definition: &Unit, id: &str) -> Unit {
    let mut unit = Unit::new("trailer_def", id);
    unit.fields = definition.fields.clone();
    let extras: Vec<sii::Field> = match document.units_of_kind("trailer_def").next() {
        Some(other) => other
            .fields
            .iter()
            .filter(|field| field.name != "source_name")
            .filter(|field| definition.get(&field.name).is_none())
            .cloned()
            .collect(),
        None => vec![
            sii::Field {
                name: "trailer".to_string(),
                value: Value::single("\"\""),
            },
            sii::Field {
                name: "country_validity".to_string(),
                value: Value::single("0"),
            },
        ],
    };
    unit.fields.extend(extras);
    set_field(
        &mut unit,
        "source_name",
        Value::single(format!("\"{}\"", definition.id)),
    );
    unit
}

/// Buys a dealer's trailer (`units`: its trailers, the front one first, and their accessories,
/// each with its `refund`; `definitions`: the front trailer's type as the game defines it) into a
/// garage with room. Returns the front trailer's id. Nothing is paid.
pub fn buy_trailer(
    document: &mut Document,
    units: &[Unit],
    definitions: &[Unit],
    garage: &str,
) -> Result<String, BuyProblem> {
    let front = units
        .first()
        .filter(|unit| unit.kind == "trailer")
        .ok_or(BuyProblem::Empty)?;
    let type_name = front
        .single("trailer_definition")
        .map(str::to_string)
        .ok_or(BuyProblem::Empty)?;
    let garage_unit = document
        .unit(garage)
        .filter(|unit| unit.kind == "garage" && unit.single("status").unwrap_or("0") != "0")
        .ok_or(BuyProblem::NotAGarage)?;
    if garage_unit.items("trailers").len() >= garage_unit.items("vehicles").len() {
        return Err(BuyProblem::GarageFull);
    }
    let player_id = player(document).ok_or(BuyProblem::NoTemplate)?;
    let template = template_trailer(document).cloned();
    let log_template = document
        .unit(&player_id)
        .and_then(|player| {
            player
                .items("trailer_utilization_logs")
                .iter()
                .find_map(|id| document.unit(id))
        })
        .or_else(|| document.units_of_kind("trailer_utilization_log").next())
        .cloned();
    let now = crate::jobs::game_time(document);

    // New ids: every unit, the usage log, and a type if the save doesn't have it yet.
    let mut ids = document.unused_nameless_ids(units.len() + 2);
    let log_id = ids.pop().unwrap_or_default();
    let type_id_if_new = ids.pop().unwrap_or_default();
    let existing_type = trailer_type_in(document, &type_name);
    let new_type = match &existing_type {
        Some(_) => None,
        None => {
            let definition = definitions
                .iter()
                .find(|definition| definition.id == type_name)
                .ok_or(BuyProblem::Empty)?;
            Some(new_trailer_type(document, definition, &type_id_if_new))
        }
    };
    let type_id = existing_type.unwrap_or(type_id_if_new);
    let renames: HashMap<String, String> = units
        .iter()
        .map(|unit| unit.id.clone())
        .zip(ids.iter().cloned())
        .collect();
    let front_id = ids.first().cloned().ok_or(BuyProblem::Empty)?;

    let mut new_units: Vec<Unit> = Vec::new();
    for (unit, id) in units.iter().zip(&ids) {
        let dealer = renamed(unit, id, &renames);
        if unit.kind != "trailer" {
            new_units.push(dealer);
            continue;
        }
        // From the template (started as new), with the dealer's fields over it; else the
        // dealer's alone.
        let mut trailer = match &template {
            Some(template) => start_as_new(template, "trailer", id, &[], now),
            None => Unit::new("trailer", id.clone()),
        };
        for field in &dealer.fields {
            set_field(&mut trailer, &field.name, field.value.clone());
        }
        // The last trailer of a train (or a single one) pulls nothing, whatever the template did.
        if dealer.get("slave_trailer").is_none() {
            set_field(&mut trailer, "slave_trailer", Value::single("null"));
        }
        let definition = if *id == front_id {
            type_id.as_str()
        } else {
            "null"
        };
        set_field(
            &mut trailer,
            "trailer_definition",
            Value::single(definition),
        );
        new_units.push(trailer);
    }
    for unit in new_units {
        document.push(unit);
    }
    let is_new_type = new_type.is_some();
    if let Some(unit) = new_type {
        document.push(unit);
    }
    document.push(match &log_template {
        Some(log) => empty_log(log, &log_id),
        None => Unit::new("trailer_utilization_log", log_id.clone()),
    });

    let player_unit = document
        .unit(&player_id)
        .cloned()
        .ok_or(BuyProblem::NoTemplate)?;
    if let Some(mut player) = document.unit_mut(&player_id) {
        player.set("trailers", push_item(&player_unit, "trailers", &front_id));
        player.set(
            "trailer_utilization_logs",
            push_item(&player_unit, "trailer_utilization_logs", &log_id),
        );
        if is_new_type || !player_unit.items("trailer_defs").contains(&type_id) {
            player.set(
                "trailer_defs",
                push_item(&player_unit, "trailer_defs", &type_id),
            );
        }
    }
    let garage_unit = document
        .unit(garage)
        .cloned()
        .ok_or(BuyProblem::NotAGarage)?;
    if let Some(mut unit) = document.unit_mut(garage) {
        unit.set("trailers", push_item(&garage_unit, "trailers", &front_id));
    }
    Ok(front_id)
}

/// Buys a dealer's truck (its accessories, as `gamedata::DealerTruck` gives them, each with its
/// `refund`) into a garage with a free slot. Returns the new truck's id. Nothing is paid: the
/// editor gives it to the company.
pub fn buy_truck(
    document: &mut Document,
    accessories: &[Unit],
    garage: &str,
) -> Result<String, BuyProblem> {
    if accessories.is_empty() {
        return Err(BuyProblem::Empty);
    }
    let garage_unit = document
        .unit(garage)
        .filter(|unit| unit.kind == "garage" && unit.single("status").unwrap_or("0") != "0")
        .ok_or(BuyProblem::NotAGarage)?;
    let mut slots = garage_unit.items("vehicles").to_vec();
    let free = slots
        .iter()
        .position(|item| item == "null")
        .ok_or(BuyProblem::GarageFull)?;
    let template = template_truck(document)
        .cloned()
        .ok_or(BuyProblem::NoTemplate)?;
    let log_template = template_log(document)
        .cloned()
        .ok_or(BuyProblem::NoTemplate)?;
    let player_id = player(document).ok_or(BuyProblem::NoTemplate)?;
    let now = crate::jobs::game_time(document);

    // New ids: the truck, its accessories, its log.
    let mut ids = document.unused_nameless_ids(accessories.len() + 2);
    let truck_id = ids.remove(0);
    let log_id = ids.pop().unwrap_or_default();
    // The dealer file's ids (`.daf1.engine`) → the save's new ones. An accessory pointing at
    // another (none do today) would follow along.
    let renames: HashMap<String, String> = accessories
        .iter()
        .map(|unit| unit.id.clone())
        .zip(ids.iter().cloned())
        .collect();
    let new_accessories: Vec<Unit> = accessories
        .iter()
        .zip(&ids)
        .map(|(unit, id)| renamed(unit, id, &renames))
        .collect();

    document.push(start_as_new(&template, "vehicle", &truck_id, &ids, now));
    for unit in new_accessories {
        document.push(unit);
    }
    document.push(empty_log(&log_template, &log_id));

    let player_unit = document
        .unit(&player_id)
        .cloned()
        .ok_or(BuyProblem::NoTemplate)?;
    if let Some(mut player) = document.unit_mut(&player_id) {
        player.set("trucks", push_item(&player_unit, "trucks", &truck_id));
        player.set(
            "truck_profit_logs",
            push_item(&player_unit, "truck_profit_logs", &log_id),
        );
    }
    if let Some(slot) = slots.get_mut(free) {
        *slot = truck_id.clone();
    }
    if let Some(mut unit) = document.unit_mut(garage) {
        unit.set("vehicles", Value::Array(slots));
    }
    Ok(truck_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No truck of the company's: the layout comes from a used truck a dealer offers.
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n game_time: 4321\r\n}\r\n\r\nplayer : p {\r\n trucks: 0\r\n truck_profit_logs: 0\r\n}\r\n\r\nused_truck_offer : _nameless.1.10 {\r\n truck: _nameless.1.20\r\n price: 50000\r\n}\r\n\r\nvehicle : _nameless.1.20 {\r\n engine_wear: &3e800000\r\n fuel_relative: &3f000000\r\n odometer: 120000\r\n odometer_float_part: &3f000000\r\n trip_distance_km: 50\r\n purchase_time: 12\r\n license_plate: \"XY 987|france\"\r\n wheels_wear: 2\r\n wheels_wear[0]: &3e000000\r\n wheels_wear[1]: &3e000000\r\n accessories: 1\r\n accessories[0]: _nameless.1.28\r\n user_fov: 0\r\n}\r\n\r\nvehicle_accessory : _nameless.1.28 {\r\n data_path: \"/def/vehicle/truck/x/engine/old.sii\"\r\n refund: 9000\r\n}\r\n\r\nprofit_log : _nameless.1.30 {\r\n stats_data: 1\r\n stats_data[0]: _nameless.1.38\r\n acc_distance_free: 77\r\n history_age: 4294967295\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 1\r\n vehicles[0]: null\r\n drivers: 1\r\n drivers[0]: null\r\n trailers: 0\r\n status: 6\r\n profit_log: _nameless.1.30\r\n}\r\n\r\ngarage : garage.rome {\r\n vehicles: 0\r\n status: 0\r\n}\r\n\r\n}\r\n";

    fn dealer_parts() -> Vec<Unit> {
        let text = "SiiNunit\r\n{\r\nvehicle_accessory : .t.data {\r\n data_path: \"/def/vehicle/truck/x/data.sii\"\r\n refund: 0\r\n}\r\n\r\nvehicle_accessory : .t.engine {\r\n data_path: \"/def/vehicle/truck/x/engine/new.sii\"\r\n refund: 25000\r\n}\r\n\r\n}\r\n";
        sii::parse_text(text).expect("parses").units().to_vec()
    }

    #[test]
    fn buys_a_new_truck_into_a_free_slot() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(buy_targets(&document), ["garage.lyon"]);
        let id = buy_truck(&mut document, &dealer_parts(), "garage.lyon").expect("bought");
        let truck = document.unit(&id).expect("the truck");
        assert_eq!(truck.single("engine_wear"), Some("0"));
        assert_eq!(truck.single("fuel_relative"), Some("1"));
        assert_eq!(truck.single("odometer"), Some("0"));
        assert_eq!(truck.single("odometer_float_part"), Some("0"));
        assert_eq!(truck.single("trip_distance_km"), Some("0"));
        assert_eq!(truck.single("purchase_time"), Some("4321"));
        assert!(truck.items("wheels_wear").is_empty());
        // Kept from the other truck.
        assert_eq!(truck.single("license_plate"), Some("\"XY 987|france\""));
        assert_eq!(truck.single("user_fov"), Some("0"));
        // The dealer's parts, under new ids, with their refunds.
        let parts = truck.items("accessories").to_vec();
        assert_eq!(parts.len(), 2);
        let engine = document.unit(&parts[1]).expect("engine");
        assert_eq!(
            engine.single("data_path"),
            Some("\"/def/vehicle/truck/x/engine/new.sii\"")
        );
        assert_eq!(engine.single("refund"), Some("25000"));
        assert!(parts.iter().all(|part| part.starts_with("_nameless.")));
        // The used truck it took its layout from is untouched.
        assert_eq!(
            document
                .unit("_nameless.1.20")
                .and_then(|u| u.single("odometer")),
            Some("120000")
        );
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trucks"), [id.as_str()]);
        let log = document
            .unit(&player.items("truck_profit_logs")[0])
            .expect("log");
        assert!(log.items("stats_data").is_empty());
        assert_eq!(log.single("acc_distance_free"), Some("0"));
        assert_eq!(
            document
                .unit("garage.lyon")
                .expect("lyon")
                .items("vehicles"),
            [id.as_str()]
        );
        // Lyon is full now.
        assert_eq!(
            buy_truck(&mut document, &dealer_parts(), "garage.lyon"),
            Err(BuyProblem::GarageFull)
        );
        assert_eq!(
            buy_truck(&mut document, &dealer_parts(), "garage.rome"),
            Err(BuyProblem::NotAGarage)
        );
        assert!(document.reachable_from("e").contains(&parts[0]));
    }

    /// A road train as a dealer file has it: the front trailer, a dolly, a second trailer.
    fn dealer_train() -> (Vec<Unit>, Vec<Unit>) {
        let text = "SiiNunit\r\n{\r\ntrailer : .front {\r\n trailer_definition: trailer_def.scs.flatbed.rm_double.curtain\r\n slave_trailer: .dolly\r\n accessories: 1\r\n accessories[0]: .data\r\n}\r\n\r\ntrailer : .dolly {\r\n slave_trailer: .second\r\n}\r\n\r\ntrailer : .second {\r\n trailer_definition: trailer_def.scs.flatbed.single_28.curtain\r\n}\r\n\r\nvehicle_accessory : .data {\r\n data_path: \"/def/vehicle/trailer_owned/scs.flatbed/data.sii\"\r\n refund: 0\r\n}\r\n\r\n}\r\n";
        let units = sii::parse_text(text).expect("parses").units().to_vec();
        let definition = "SiiNunit\r\n{\r\ntrailer_def : trailer_def.scs.flatbed.rm_double.curtain {\r\n axles: 3\r\n body_type: curtainside\r\n}\r\n\r\n}\r\n";
        let definitions = sii::parse_text(definition)
            .expect("parses")
            .units()
            .to_vec();
        (units, definitions)
    }

    #[test]
    fn buys_a_road_train_into_a_save_without_trailers() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(trailer_targets(&document), ["garage.lyon"]);
        let (units, definitions) = dealer_train();
        let front =
            buy_trailer(&mut document, &units, &definitions, "garage.lyon").expect("bought");
        let trailer = document.unit(&front).expect("front");
        // The front trailer carries the type: a new one, made from the game's definition.
        let type_id = trailer
            .single("trailer_definition")
            .expect("type")
            .to_string();
        let kind = document.unit(&type_id).expect("the type");
        assert_eq!(kind.kind, "trailer_def");
        assert_eq!(kind.single("axles"), Some("3"));
        assert_eq!(kind.single("country_validity"), Some("0"));
        assert_eq!(
            kind.single("source_name"),
            Some("\"trailer_def.scs.flatbed.rm_double.curtain\"")
        );
        // The chain, under new ids; the trailers behind have no type; the last pulls nothing.
        let dolly = trailer.single("slave_trailer").expect("dolly").to_string();
        let dolly_unit = document.unit(&dolly).expect("dolly unit");
        assert_eq!(dolly_unit.single("trailer_definition"), Some("null"));
        let second = dolly_unit
            .single("slave_trailer")
            .expect("second")
            .to_string();
        let second_unit = document.unit(&second).expect("second unit");
        assert_eq!(second_unit.single("trailer_definition"), Some("null"));
        assert_eq!(second_unit.single("slave_trailer"), Some("null"));
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trailers"), [front.as_str()]);
        assert_eq!(player.items("trailer_defs"), [type_id.as_str()]);
        assert_eq!(player.items("trailer_utilization_logs").len(), 1);
        assert_eq!(
            document
                .unit("garage.lyon")
                .expect("lyon")
                .items("trailers"),
            [front.as_str()]
        );
        // One truck slot: no room for a second trailer.
        assert_eq!(
            buy_trailer(&mut document, &units, &definitions, "garage.lyon"),
            Err(BuyProblem::GarageFull)
        );
        let reachable = document.reachable_from("e");
        assert!(reachable.contains(&second) && reachable.contains(&type_id));
    }

    #[test]
    fn a_second_trailer_of_a_type_shares_it_and_starts_new() {
        let mut document = sii::parse_text(&SAVE.replace(
            " vehicles: 1\r\n vehicles[0]: null\r\n",
            " vehicles: 3\r\n vehicles[0]: null\r\n vehicles[1]: null\r\n vehicles[2]: null\r\n",
        ))
        .expect("parses");
        let (units, definitions) = dealer_train();
        let first = buy_trailer(&mut document, &units, &definitions, "garage.lyon").expect("first");
        // Worn and loaded, as if driven: the next one takes its fields from it, started as new.
        if let Some(mut unit) = document.unit_mut(&first) {
            unit.set("cargo_mass", Value::single("20000"));
            unit.set("chassis_wear", Value::single("&3f000000"));
        }
        let second =
            buy_trailer(&mut document, &units, &definitions, "garage.lyon").expect("second");
        let [one, two] = [&first, &second].map(|id| document.unit(id).expect("trailer"));
        assert_eq!(
            one.single("trailer_definition"),
            two.single("trailer_definition")
        );
        assert_eq!(two.single("cargo_mass"), Some("0"));
        assert_eq!(two.single("chassis_wear"), Some("0"));
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trailers").len(), 2);
        assert_eq!(player.items("trailer_defs").len(), 1);
    }
}
