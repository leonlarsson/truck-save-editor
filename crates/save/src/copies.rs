//! Copying a truck or trailer the company owns into one of its garages, as if bought again: the
//! vehicle and every unit that's part of it (its accessories: parts, wheels, paint, plates; a
//! trailer's other parts, for a road train) are copied under new ids, and the copy is linked in the
//! way the game links a truck or trailer it sells:
//!
//! - a truck goes in `player.trucks`, with an empty profit log in `player.truck_profit_logs`
//!   (the two lists run in step), and in a free slot of the garage's `vehicles`;
//! - a trailer goes in `player.trailers`, with an empty usage log in
//!   `player.trailer_utilization_logs`, and in the garage's `trailers`. Its `trailer_def` (shared
//!   by every trailer of that type, and listed once in `player.trailer_defs`) is shared by the
//!   copy too.
//!
//! The copy keeps everything else (odometer, wear, fuel) but its plate, which gets new characters in
//! the same format (`plates.rs`). Worked out from saves where the
//! game had just sold a truck and a trailer (ATS, 2026-11-25), and from ETS2 fleets.

use std::collections::HashMap;

use sii::{Document, Unit, Value};

/// Why a truck or trailer can't be copied there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleCopyProblem {
    /// It isn't one of the company's trucks or trailers.
    NotOwned,
    /// The garage isn't the company's.
    NotAGarage,
    /// The garage has no free truck slot (for a truck), or no room for another trailer.
    GarageFull,
    /// Something that's part of the vehicle is a kind this doesn't know how to copy.
    UnknownPart,
}

/// What kind of vehicle an id is, and the player's lists it's in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Truck,
    Trailer,
}

impl Kind {
    /// The player's list of these, and the list of their logs that runs in step with it.
    fn lists(self) -> (&'static str, &'static str) {
        match self {
            Kind::Truck => ("trucks", "truck_profit_logs"),
            Kind::Trailer => ("trailers", "trailer_utilization_logs"),
        }
    }
}

pub(crate) fn player(document: &Document) -> Option<String> {
    document
        .first_of_kind("economy")?
        .single("player")
        .map(str::to_string)
}

/// Whether `id` is one of the company's trucks or trailers, and which.
fn kind_of(document: &Document, id: &str) -> Option<Kind> {
    let player = document.unit(&player(document)?)?;
    [Kind::Truck, Kind::Trailer]
        .into_iter()
        .find(|kind| player.items(kind.lists().0).iter().any(|item| item == id))
}

/// Whether a garage has room for one more of this kind: an empty truck slot, or (for trailers)
/// fewer trailers than truck slots, as the game allows.
fn has_room(garage: &Unit, kind: Kind) -> bool {
    let slots = garage.items("vehicles");
    match kind {
        Kind::Truck => slots.iter().any(|item| item == "null"),
        Kind::Trailer => garage.items("trailers").len() < slots.len(),
    }
}

/// The company's garages a truck or trailer could be copied into (with room for it), in the save's
/// order.
pub fn copy_targets(document: &Document, id: &str) -> Result<Vec<String>, VehicleCopyProblem> {
    let kind = kind_of(document, id).ok_or(VehicleCopyProblem::NotOwned)?;
    Ok(document
        .units_of_kind("garage")
        .filter(|garage| garage.single("status").unwrap_or("0") != "0")
        .filter(|garage| has_room(garage, kind))
        .map(|garage| garage.id.clone())
        .collect())
}

/// Whether a unit kind is part of a vehicle, to be copied with it.
fn is_part(kind: &str) -> bool {
    kind == "vehicle" || kind == "trailer" || kind.starts_with("vehicle_")
}

/// Units a vehicle points at that it shares with others, kept as they are.
fn is_shared(kind: &str) -> bool {
    kind == "trailer_def"
}

/// The ids of a vehicle and everything that's part of it, the vehicle first, then in the order
/// they're reached (so the copies come out in the same order).
fn parts(document: &Document, id: &str) -> Result<Vec<String>, VehicleCopyProblem> {
    let mut found = vec![id.to_string()];
    let mut next = 0;
    while let Some(current) = found.get(next).cloned() {
        next += 1;
        let unit = document
            .unit(&current)
            .ok_or(VehicleCopyProblem::NotOwned)?;
        for field in &unit.fields {
            for value in field_ids(&field.value) {
                let Some(target) = document.unit(value) else {
                    continue;
                };
                if is_shared(&target.kind) || found.iter().any(|seen| seen == value) {
                    continue;
                }
                if !is_part(&target.kind) {
                    return Err(VehicleCopyProblem::UnknownPart);
                }
                found.push(value.clone());
            }
        }
    }
    Ok(found)
}

/// The values of a field (one, or an array's items).
fn field_ids(value: &Value) -> &[String] {
    match value {
        Value::Single(text) => std::slice::from_ref(text),
        Value::Array(items) => items,
    }
}

/// A unit with every value that's a key of `ids` swapped for its new id.
pub(crate) fn renamed(unit: &Unit, id: &str, ids: &HashMap<String, String>) -> Unit {
    let mut copy = Unit::new(unit.kind.clone(), id);
    copy.fields = unit.fields.clone();
    for field in &mut copy.fields {
        match &mut field.value {
            Value::Single(text) => {
                if let Some(new) = ids.get(text.as_str()) {
                    *text = new.clone();
                }
            }
            Value::Array(items) => {
                for item in items {
                    if let Some(new) = ids.get(item.as_str()) {
                        *item = new.clone();
                    }
                }
            }
        }
    }
    copy
}

/// An empty log like `log`: its lists emptied and its running totals at 0 (`acc_…`, `total_…`);
/// anything else (like `history_age`) as it is.
pub(crate) fn empty_log(log: &Unit, id: &str) -> Unit {
    let mut copy = Unit::new(log.kind.clone(), id);
    copy.fields = log.fields.clone();
    for field in &mut copy.fields {
        if matches!(field.value, Value::Array(_)) {
            field.value = Value::Array(Vec::new());
        } else if field.name.starts_with("acc_") || field.name.starts_with("total_") {
            field.value = Value::single("0");
        }
    }
    copy
}

/// Adds an item to the end of an array field.
pub(crate) fn push_item(unit: &Unit, field: &str, item: &str) -> Value {
    let mut items = unit.items(field).to_vec();
    items.push(item.to_string());
    Value::Array(items)
}

/// Copies a truck or trailer (by id) into a garage, as described above. Returns the copy's id.
pub fn copy_vehicle(
    document: &mut Document,
    id: &str,
    garage: &str,
) -> Result<String, VehicleCopyProblem> {
    let kind = kind_of(document, id).ok_or(VehicleCopyProblem::NotOwned)?;
    let garage_unit = document
        .unit(garage)
        .filter(|unit| unit.kind == "garage" && unit.single("status").unwrap_or("0") != "0")
        .ok_or(VehicleCopyProblem::NotAGarage)?;
    if !has_room(garage_unit, kind) {
        return Err(VehicleCopyProblem::GarageFull);
    }
    let player_id = player(document).ok_or(VehicleCopyProblem::NotOwned)?;
    let player_unit = document
        .unit(&player_id)
        .ok_or(VehicleCopyProblem::NotOwned)?;
    let (list, logs) = kind.lists();
    // The source's log, in step with it: the copy's empty log is made like it.
    let index = player_unit
        .items(list)
        .iter()
        .position(|item| item == id)
        .ok_or(VehicleCopyProblem::NotOwned)?;
    let source_log = player_unit
        .items(logs)
        .get(index)
        .and_then(|log| document.unit(log))
        .cloned();

    let originals = parts(document, id)?;
    let mut new_ids = document.unused_nameless_ids(originals.len() + 1);
    // `unused_nameless_ids` gives exactly as many as asked for, so there's one to take.
    let log_id = new_ids.pop().unwrap_or_default();
    let ids: HashMap<String, String> = originals.iter().cloned().zip(new_ids).collect();
    let copies: Vec<Unit> = originals
        .iter()
        .filter_map(|original| {
            let unit = document.unit(original)?;
            let mut copy = renamed(unit, ids.get(original)?, &ids);
            // Its own plate, like a bought one (`plates.rs`); parts have none.
            crate::plates::renew_plate(&mut copy);
            Some(copy)
        })
        .collect();
    let copy_id = ids.get(id).cloned().ok_or(VehicleCopyProblem::NotOwned)?;
    for unit in copies {
        document.push(unit);
    }

    // The log, in step with the player's list (without the source's, an empty one of the kind).
    let log = match &source_log {
        Some(log) => empty_log(log, &log_id),
        None => match kind {
            Kind::Truck => Unit::new("profit_log", log_id.clone()),
            Kind::Trailer => Unit::new("trailer_utilization_log", log_id.clone()),
        },
    };
    document.push(log);
    let player_unit = document
        .unit(&player_id)
        .cloned()
        .ok_or(VehicleCopyProblem::NotOwned)?;
    if let Some(mut player) = document.unit_mut(&player_id) {
        player.set(list, push_item(&player_unit, list, &copy_id));
        player.set(logs, push_item(&player_unit, logs, &log_id));
    }

    let garage_unit = document
        .unit(garage)
        .cloned()
        .ok_or(VehicleCopyProblem::NotAGarage)?;
    if let Some(mut unit) = document.unit_mut(garage) {
        match kind {
            Kind::Truck => {
                let mut slots = garage_unit.items("vehicles").to_vec();
                if let Some(free) = slots.iter_mut().find(|item| *item == "null") {
                    *free = copy_id.clone();
                }
                unit.set("vehicles", Value::Array(slots));
            }
            Kind::Trailer => unit.set("trailers", push_item(&garage_unit, "trailers", &copy_id)),
        }
    }
    Ok(copy_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: _nameless.1.10\r\n truck_profit_logs: 1\r\n truck_profit_logs[0]: _nameless.1.18\r\n trailers: 1\r\n trailers[0]: _nameless.1.40\r\n trailer_utilization_logs: 1\r\n trailer_utilization_logs[0]: _nameless.1.50\r\n trailer_defs: 1\r\n trailer_defs[0]: _nameless.1.48\r\n}\r\n\r\nvehicle : _nameless.1.10 {\r\n license_plate: \"AB 123|sweden\"\r\n odometer: 5000\r\n accessories: 2\r\n accessories[0]: _nameless.1.20\r\n accessories[1]: _nameless.1.28\r\n}\r\n\r\nvehicle_accessory : _nameless.1.20 {\r\n data_path: \"/def/vehicle/truck/x/engine/a.sii\"\r\n}\r\n\r\nvehicle_paint_job_accessory : _nameless.1.28 {\r\n base_color: (1, 0, 0)\r\n}\r\n\r\nprofit_log : _nameless.1.18 {\r\n stats_data: 1\r\n stats_data[0]: _nameless.1.30\r\n acc_distance_free: 12\r\n acc_distance_on_job: 34\r\n history_age: 4294967295\r\n}\r\n\r\nprofit_log_entry : _nameless.1.30 {\r\n revenue: 100\r\n}\r\n\r\ntrailer : _nameless.1.40 {\r\n trailer_definition: _nameless.1.48\r\n slave_trailer: null\r\n accessories: 0\r\n}\r\n\r\ntrailer_def : _nameless.1.48 {\r\n source_name: \"x\"\r\n}\r\n\r\ntrailer_utilization_log : _nameless.1.50 {\r\n entries: 0\r\n total_driven_distance_km: 900\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 2\r\n vehicles[0]: _nameless.1.10\r\n vehicles[1]: null\r\n drivers: 2\r\n drivers[0]: null\r\n drivers[1]: null\r\n trailers: 1\r\n trailers[0]: _nameless.1.40\r\n status: 2\r\n}\r\n\r\ngarage : garage.rome {\r\n vehicles: 0\r\n drivers: 0\r\n trailers: 0\r\n status: 0\r\n}\r\n\r\n}\r\n";

    fn document() -> Document {
        sii::parse_text(SAVE).expect("parses")
    }

    #[test]
    fn copies_a_truck_with_its_parts_into_a_free_slot() {
        let mut document = document();
        assert_eq!(
            copy_targets(&document, "_nameless.1.10"),
            Ok(vec!["garage.lyon".to_string()])
        );
        let copy = copy_vehicle(&mut document, "_nameless.1.10", "garage.lyon").expect("copies");
        let vehicle = document.unit(&copy).expect("the copy");
        let plate = vehicle.single("license_plate").expect("a plate");
        assert!(plate.ends_with("|sweden\"") && plate != "\"AB 123|sweden\"");
        // Its own parts, not the original's.
        // Owned: the document is changed again below, which a borrow of it would block.
        let parts = vehicle.items("accessories").to_vec();
        assert_eq!(parts.len(), 2);
        assert!(
            parts
                .iter()
                .all(|part| !part.ends_with(".20") && !part.ends_with(".28"))
        );
        assert_eq!(
            document
                .unit(&parts[1])
                .and_then(|unit| unit.single("base_color")),
            Some("(1, 0, 0)")
        );
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trucks"), ["_nameless.1.10", copy.as_str()]);
        let logs = player.items("truck_profit_logs");
        assert_eq!(logs.len(), 2);
        let log = document.unit(&logs[1]).expect("the copy's log");
        assert!(log.items("stats_data").is_empty());
        assert_eq!(log.single("acc_distance_on_job"), Some("0"));
        assert_eq!(log.single("history_age"), Some("4294967295"));
        assert_eq!(
            document
                .unit("garage.lyon")
                .expect("lyon")
                .items("vehicles"),
            ["_nameless.1.10", copy.as_str()]
        );
        // The garage is full now, and a garage the company doesn't own was never offered.
        assert_eq!(
            copy_vehicle(&mut document, "_nameless.1.10", "garage.lyon"),
            Err(VehicleCopyProblem::GarageFull)
        );
        assert_eq!(
            copy_vehicle(&mut document, "_nameless.1.10", "garage.rome"),
            Err(VehicleCopyProblem::NotAGarage)
        );
        // Everything the copy needs is in the one tree from the economy.
        let reachable = document.reachable_from("e");
        assert!(reachable.contains(&copy) && reachable.contains(&parts[0]));
    }

    #[test]
    fn copies_a_trailer_sharing_its_type() {
        let mut document = document();
        let copy = copy_vehicle(&mut document, "_nameless.1.40", "garage.lyon").expect("copies");
        let trailer = document.unit(&copy).expect("the copy");
        assert_eq!(trailer.single("trailer_definition"), Some("_nameless.1.48"));
        let player = document.unit("p").expect("player");
        assert_eq!(player.items("trailers").len(), 2);
        assert_eq!(player.items("trailer_defs"), ["_nameless.1.48"]);
        let logs = player.items("trailer_utilization_logs");
        let log = document.unit(&logs[1]).expect("the copy's log");
        assert_eq!(log.single("total_driven_distance_km"), Some("0"));
        assert_eq!(
            document
                .unit("garage.lyon")
                .expect("lyon")
                .items("trailers"),
            ["_nameless.1.40", copy.as_str()]
        );
        // Two truck slots, two trailers: no room for a third.
        assert_eq!(
            copy_vehicle(&mut document, "_nameless.1.40", "garage.lyon"),
            Err(VehicleCopyProblem::GarageFull)
        );
    }

    #[test]
    fn copies_only_what_it_owns() {
        let mut document = document();
        assert_eq!(
            copy_vehicle(&mut document, "_nameless.1.20", "garage.lyon"),
            Err(VehicleCopyProblem::NotOwned)
        );
    }
}
