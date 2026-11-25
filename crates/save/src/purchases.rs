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
    /// The dealer's truck has no accessories.
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

/// The template truck's fields, started as a new truck's (see the module's notes), pointing at
/// `accessories`.
fn new_truck(template: &Unit, id: &str, accessories: &[String], now: Option<i64>) -> Unit {
    let mut truck = Unit::new("vehicle", id);
    truck.fields = template.fields.clone();
    for field in &mut truck.fields {
        let name = field.name.as_str();
        let starts_at_zero = name.ends_with("_wear")
            || name.ends_with("_wear_unfixable")
            || name.starts_with("odometer")
            || name.starts_with("integrity_odometer")
            || name.starts_with("trip_")
            || name.ends_with("passenger_mass");
        field.value = if name == "accessories" {
            Value::Array(accessories.to_vec())
        } else if name == "fuel_relative" {
            Value::single("1")
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
    truck
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

    document.push(new_truck(&template, &truck_id, &ids, now));
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
}
