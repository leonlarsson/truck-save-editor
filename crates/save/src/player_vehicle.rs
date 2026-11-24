//! The player's own truck and trailer, and where they stand.
//!
//! Newer saves keep them in `player_vehicles` units, one per kind of vehicle the player drives
//! (`player.my_vehicles[]`; `player.assigned_vehicles` is the one in use): `vehicle`, `trailer`,
//! `stored_vehicle_placement` (`(x, y, z) (w; x, y, z)`), `stored_vehicle_city` (`city.<token>`)
//! and `stored_trailer_attached`. Older saves keep `assigned_truck`, `my_truck`,
//! `assigned_trailer` and `truck_placement` on the player unit.
//!
//! The player's truck sits in a garage slot like any other, with the player (`driver_player`) as
//! its driver. Driving another of the company's trucks swaps the two slots' drivers (whoever drove
//! the new truck gets the old one), points the player's vehicles at it, and makes its garage the
//! headquarters, as the player's slot is there.

use sii::{Document, Value};

fn player(document: &Document) -> Option<String> {
    document
        .first_of_kind("economy")?
        .single("player")
        .map(str::to_string)
}

/// The `player_vehicles` unit holding the player's truck (newer saves).
fn truck_vehicles(document: &Document) -> Option<String> {
    let player = document.unit(&player(document)?)?;
    let trucks = player.items("trucks");
    player
        .items("my_vehicles")
        .iter()
        .filter_map(|id| document.unit(id))
        .find(|unit| {
            unit.single("vehicle")
                .is_some_and(|vehicle| trucks.iter().any(|truck| truck == vehicle))
        })
        .map(|unit| unit.id.clone())
}

/// The `player_vehicles` unit the game uses for the truck being driven: the player's
/// `assigned_vehicles`, when it holds one of the player's trucks. It's a unit of its own, apart
/// from the truck's entry in `my_vehicles` (found in a 1.5x save, 2026-11-22), and the one whose
/// placement is current. Falls back to the `my_vehicles` entry.
fn active_truck_vehicles(document: &Document) -> Option<String> {
    let player = document.unit(&player(document)?)?;
    let trucks = player.items("trucks");
    player
        .single("assigned_vehicles")
        .filter(|id| *id != "null")
        .and_then(|id| document.unit(id))
        .filter(|unit| unit.kind == "player_vehicles")
        .filter(|unit| {
            unit.single("vehicle")
                .is_some_and(|vehicle| trucks.iter().any(|truck| truck == vehicle))
        })
        .map(|unit| unit.id.clone())
        .or_else(|| truck_vehicles(document))
}

/// The truck the player drives, if any.
pub fn own_truck(document: &Document) -> Option<String> {
    if let Some(holder) = truck_vehicles(document) {
        return document
            .unit(&holder)?
            .single("vehicle")
            .filter(|id| *id != "null")
            .map(str::to_string);
    }
    let player = document.unit(&player(document)?)?;
    ["my_truck", "assigned_truck"]
        .iter()
        .filter_map(|field| player.single(field))
        .find(|id| *id != "null")
        .map(str::to_string)
}

/// Why the player can't change trucks or move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerProblem {
    /// A job is in progress: finish or cancel it first.
    OnJob,
    /// That truck is already the player's.
    AlreadyYours,
    /// The truck (or the player's own) isn't in a garage slot.
    NotInGarage,
    /// A trailer is hitched: moving would need a place for it too.
    TrailerHitched,
    /// The save doesn't have the player's position (an older format).
    NoPosition,
    /// The player has no truck of their own yet (a new career, driving quick jobs).
    NoOwnTruck,
}

fn on_job(document: &Document) -> bool {
    player(document)
        .and_then(|id| document.unit(&id))
        .and_then(|unit| unit.single("current_job"))
        .is_some_and(|job| job != "null")
}

/// A truck's garage slot: (garage id, index).
fn slot(document: &Document, truck: &str) -> Option<(String, usize)> {
    document.units_of_kind("garage").find_map(|garage| {
        garage
            .items("vehicles")
            .iter()
            .position(|item| item == truck)
            .map(|index| (garage.id.clone(), index))
    })
}

/// Whether the player can drive another of the company's trucks, and if not, why.
pub fn can_drive(document: &Document, truck: &str) -> Result<(), PlayerProblem> {
    if on_job(document) {
        return Err(PlayerProblem::OnJob);
    }
    let own = own_truck(document).ok_or(PlayerProblem::NotInGarage)?;
    if own == truck {
        return Err(PlayerProblem::AlreadyYours);
    }
    slot(document, &own).ok_or(PlayerProblem::NotInGarage)?;
    slot(document, truck).ok_or(PlayerProblem::NotInGarage)?;
    Ok(())
}

fn set_slot_driver(document: &mut Document, garage: &str, index: usize, driver: &str) {
    if let Some(mut unit) = document.unit_mut(garage) {
        let mut drivers = unit
            .get("drivers")
            .map(|value| value.items().to_vec())
            .unwrap_or_default();
        if let Some(slot) = drivers.get_mut(index) {
            *slot = driver.to_string();
            unit.set("drivers", Value::Array(drivers));
        }
    }
}

/// Makes another of the company's trucks the player's (see the module docs).
pub fn drive_truck(document: &mut Document, truck: &str) -> Result<(), PlayerProblem> {
    can_drive(document, truck)?;
    let own = own_truck(document).ok_or(PlayerProblem::NotInGarage)?;
    let (own_garage, own_index) = slot(document, &own).ok_or(PlayerProblem::NotInGarage)?;
    let (new_garage, new_index) = slot(document, truck).ok_or(PlayerProblem::NotInGarage)?;
    let driver_at = |document: &Document, garage: &str, index: usize| {
        document
            .unit(garage)
            .and_then(|unit| unit.items("drivers").get(index).cloned())
            .unwrap_or_else(|| "null".to_string())
    };
    let me = driver_at(document, &own_garage, own_index);
    let them = driver_at(document, &new_garage, new_index);
    set_slot_driver(document, &own_garage, own_index, &them);
    set_slot_driver(document, &new_garage, new_index, &me);

    // Every vehicles unit that names the old truck names the new one.
    let holders: Vec<String> = document
        .units_of_kind("player_vehicles")
        .filter(|unit| unit.single("vehicle") == Some(own.as_str()))
        .map(|unit| unit.id.clone())
        .collect();
    for holder in holders {
        if let Some(mut unit) = document.unit_mut(&holder) {
            unit.set("vehicle", Value::single(truck));
        }
    }
    if let Some(player) = player(document)
        && let Some(mut unit) = document.unit_mut(&player)
    {
        for field in ["my_truck", "assigned_truck"] {
            if unit.get(field).and_then(Value::as_single) == Some(own.as_str()) {
                unit.set(field, Value::single(truck));
            }
        }
        // The player's slot is at the headquarters.
        let city = new_garage.strip_prefix("garage.").unwrap_or(&new_garage);
        if unit.get("hq_city").is_some() {
            unit.set("hq_city", Value::single(city));
        }
    }
    Ok(())
}

/// The trailer the player pulls (newer saves: the truck's vehicles unit; older: the player's).
pub fn own_trailer(document: &Document) -> Option<String> {
    if let Some(holder) = truck_vehicles(document) {
        return document
            .unit(&holder)?
            .single("trailer")
            .filter(|id| *id != "null")
            .map(str::to_string);
    }
    let player = document.unit(&player(document)?)?;
    ["my_trailer", "assigned_trailer"]
        .iter()
        .filter_map(|field| player.single(field))
        .find(|id| *id != "null")
        .map(str::to_string)
}

/// Whether the player can pull one of the company's trailers, and if not, why.
pub fn can_take_trailer(document: &Document, trailer: &str) -> Result<(), PlayerProblem> {
    if on_job(document) {
        return Err(PlayerProblem::OnJob);
    }
    if own_trailer(document).as_deref() == Some(trailer) {
        return Err(PlayerProblem::AlreadyYours);
    }
    let owned = player(document)
        .and_then(|id| document.unit(&id))
        .is_some_and(|player| player.items("trailers").iter().any(|id| id == trailer));
    if !owned {
        return Err(PlayerProblem::NotInGarage);
    }
    Ok(())
}

/// Makes one of the company's trailers the one the player pulls, not hitched (a hitched trailer
/// would need a place behind the truck). `false` if it can't.
pub(crate) fn take_trailer(document: &mut Document, trailer: &str) -> bool {
    if can_take_trailer(document, trailer).is_err() {
        return false;
    }
    if let Some(holder) = truck_vehicles(document) {
        if let Some(mut unit) = document.unit_mut(&holder) {
            unit.set("trailer", Value::single(trailer));
            if unit.get("stored_trailer_attached").is_some() {
                unit.set("stored_trailer_attached", Value::single("false"));
            }
            if unit.get("stored_trailer_placements").is_some() {
                unit.set("stored_trailer_placements", Value::Array(Vec::new()));
            }
        }
        return true;
    }
    let Some(player) = player(document) else {
        return false;
    };
    if let Some(mut unit) = document.unit_mut(&player) {
        if unit.get("my_trailer").is_some() {
            unit.set("my_trailer", Value::single(trailer));
        }
        if unit.get("my_trailer_attached").is_some() {
            unit.set("my_trailer_attached", Value::single("false"));
        }
    }
    true
}

/// Where the player's truck stands: its placement and city token, if the save says.
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub placement: String,
    pub city: Option<String>,
}

pub fn position(document: &Document) -> Option<Position> {
    if let Some(holder) = active_truck_vehicles(document) {
        let unit = document.unit(&holder)?;
        let city = |unit: &sii::Unit| {
            unit.single("stored_vehicle_city")
                .filter(|city| *city != "null")
                .map(|city| city.trim_start_matches("city.").to_string())
        };
        // The active entry may not name its city; the `my_vehicles` entry may.
        let city = city(unit).or_else(|| {
            truck_vehicles(document)
                .and_then(|id| document.unit(&id))
                .and_then(city)
        });
        return Some(Position {
            placement: unit.single("stored_vehicle_placement")?.to_string(),
            city,
        });
    }
    // Without a truck of their own (a new career on quick jobs), the game keeps no position:
    // `truck_placement` is all zeros.
    own_truck(document)?;
    let player = document.unit(&player(document)?)?;
    Some(Position {
        placement: player.single("truck_placement")?.to_string(),
        city: None,
    })
}

/// Whether the player can be moved, and if not, why.
pub fn can_move_player(document: &Document) -> Result<(), PlayerProblem> {
    if on_job(document) {
        return Err(PlayerProblem::OnJob);
    }
    position(document).ok_or(PlayerProblem::NoPosition)?;
    let hitched = active_truck_vehicles(document)
        .and_then(|holder| document.unit(&holder))
        .is_some_and(|unit| {
            unit.single("trailer")
                .is_some_and(|trailer| trailer != "null")
                && unit.single("stored_trailer_attached") == Some("true")
        })
        || player(document)
            .and_then(|id| document.unit(&id))
            .is_some_and(|unit| unit.single("assigned_trailer_connected") == Some("true"));
    if hitched {
        return Err(PlayerProblem::TrailerHitched);
    }
    Ok(())
}

/// Moves the player's truck to a position (from another save of the same game).
pub fn move_player(document: &mut Document, to: &Position) -> Result<(), PlayerProblem> {
    can_move_player(document)?;
    // Only a placement in the game's own form: `(x, y, z) (w; x, y, z)`.
    if !to.placement.starts_with('(') || !to.placement.contains(';') {
        return Err(PlayerProblem::NoPosition);
    }
    // Both the active entry and the truck's `my_vehicles` entry, so whichever the game reads
    // has the new place.
    let mut holders: Vec<String> = active_truck_vehicles(document).into_iter().collect();
    if let Some(entry) = truck_vehicles(document)
        && !holders.contains(&entry)
    {
        holders.push(entry);
    }
    if !holders.is_empty() {
        for holder in holders {
            if let Some(mut unit) = document.unit_mut(&holder) {
                unit.set(
                    "stored_vehicle_placement",
                    Value::single(to.placement.clone()),
                );
                if let Some(city) = &to.city
                    && unit.get("stored_vehicle_city").is_some()
                {
                    unit.set("stored_vehicle_city", Value::single(format!("city.{city}")));
                }
            }
        }
        return Ok(());
    }
    let player = player(document).ok_or(PlayerProblem::NoPosition)?;
    if let Some(mut unit) = document.unit_mut(&player) {
        unit.set("truck_placement", Value::single(to.placement.clone()));
    }
    Ok(())
}

/// Whether the game can be asked for a quick travel when the save loads (see
/// `schedule_quick_travel`), and if not, why.
pub fn can_travel(document: &Document) -> Result<(), PlayerProblem> {
    if on_job(document) {
        return Err(PlayerProblem::OnJob);
    }
    let player = player(document)
        .and_then(|id| document.unit(&id))
        .ok_or(PlayerProblem::NoPosition)?;
    let has_fields = player.get("schedule_quick_travel").is_some()
        && player.get("schedule_transfer_to_hq").is_some();
    if !has_fields {
        return Err(PlayerProblem::NoPosition);
    }
    // Quick travel moves the player's own vehicles: none yet, nothing to move.
    if truck_vehicles(document).is_none() {
        return Err(PlayerProblem::NoOwnTruck);
    }
    Ok(())
}

/// Asks the game for a free quick travel when the save loads (`schedule_quick_travel`).
///
/// Found in game (2026-11-22): the game then says "Game change detected" (unfinished deliveries
/// cancelled without penalty, the vehicle moved to a safe place) and shows its quick travel map
/// once for each of the player's vehicles (the truck, then the coach with Coaches), with every
/// visited city to pick from at no cost. The player picks: a city written into the vehicles'
/// `stored_vehicle_city` isn't used, so none is written. Off during a job, which this would cancel.
/// `schedule_transfer_to_hq` did nothing in the same test (the player stayed put), so it's only
/// turned off here.
pub fn schedule_quick_travel(document: &mut Document) -> Result<(), PlayerProblem> {
    can_travel(document)?;
    let player = player(document).ok_or(PlayerProblem::NoPosition)?;
    if let Some(mut unit) = document.unit_mut(&player) {
        unit.set("schedule_quick_travel", Value::single("true"));
        unit.set("schedule_transfer_to_hq", Value::single("false"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n hq_city: lyon\r\n trucks: 2\r\n trucks[0]: truck.me\r\n trucks[1]: truck.b\r\n my_vehicles: 2\r\n my_vehicles[0]: pv.truck\r\n my_vehicles[1]: pv.bus\r\n assigned_vehicles: pv.truck\r\n current_job: null\r\n}\r\n\r\nplayer_vehicles : pv.truck {\r\n vehicle: truck.me\r\n stored_vehicle_placement: (1, 2, 3) (1; 0, 0, 0)\r\n stored_vehicle_city: city.lyon\r\n trailer: null\r\n stored_trailer_attached: false\r\n}\r\n\r\nplayer_vehicles : pv.bus {\r\n vehicle: bus.1\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 1\r\n vehicles[0]: truck.me\r\n drivers: 1\r\n drivers[0]: me\r\n}\r\n\r\ngarage : garage.rome {\r\n vehicles: 2\r\n vehicles[0]: null\r\n vehicles[1]: truck.b\r\n drivers: 2\r\n drivers[0]: null\r\n drivers[1]: driver.b\r\n}\r\n\r\ndriver_player : me {\r\n}\r\n\r\ndriver_ai : driver.b {\r\n}\r\n\r\n}\r\n";

    fn document() -> Document {
        sii::parse_text(SAVE).expect("parses")
    }

    #[test]
    fn drives_another_truck_by_swapping_drivers() {
        let mut document = document();
        assert_eq!(own_truck(&document).as_deref(), Some("truck.me"));
        assert_eq!(drive_truck(&mut document, "truck.b"), Ok(()));
        assert_eq!(own_truck(&document).as_deref(), Some("truck.b"));
        assert_eq!(
            document.unit("garage.rome").expect("rome").items("drivers"),
            ["null", "me"]
        );
        assert_eq!(
            document.unit("garage.lyon").expect("lyon").items("drivers"),
            ["driver.b"]
        );
        assert_eq!(
            document.unit("p").and_then(|u| u.single("hq_city")),
            Some("rome")
        );
        // The coach stays the coach.
        assert_eq!(
            document.unit("pv.bus").and_then(|u| u.single("vehicle")),
            Some("bus.1")
        );
        assert_eq!(
            can_drive(&document, "truck.b"),
            Err(PlayerProblem::AlreadyYours)
        );
    }

    #[test]
    fn has_no_position_without_a_truck() {
        let text = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n my_truck: null\r\n assigned_truck: null\r\n truck_placement: (0, 0, 0) (1; 0, 0, 0)\r\n current_job: null\r\n}\r\n\r\n}\r\n";
        let without_truck = sii::parse_text(text).expect("parses");
        assert_eq!(position(&without_truck), None);
        assert!(position(&document()).is_some());
    }

    #[test]
    fn asks_the_game_for_a_quick_travel() {
        let text = SAVE.replace(
            " current_job: null\r\n",
            " current_job: null\r\n schedule_transfer_to_hq: true\r\n schedule_quick_travel: false\r\n",
        );
        let mut document = sii::parse_text(&text).expect("parses");
        let city_before = document
            .unit("pv.truck")
            .and_then(|u| u.single("stored_vehicle_city"))
            .map(str::to_string);
        assert_eq!(schedule_quick_travel(&mut document), Ok(()));
        let player = document.unit("p").expect("player");
        assert_eq!(player.single("schedule_quick_travel"), Some("true"));
        assert_eq!(player.single("schedule_transfer_to_hq"), Some("false"));
        // No city is written: the player picks one in the game.
        assert_eq!(
            document
                .unit("pv.truck")
                .and_then(|u| u.single("stored_vehicle_city"))
                .map(str::to_string),
            city_before
        );
        // A save without the fields (older) says so instead of adding them.
        let mut older = sii::parse_text(SAVE).expect("parses");
        assert_eq!(
            schedule_quick_travel(&mut older),
            Err(PlayerProblem::NoPosition)
        );
    }

    #[test]
    fn moves_the_player_and_says_when_it_cannot() {
        let mut document = document();
        let to = Position {
            placement: "(500, 10, -200) (&3f3504f3; 0, &3f3504f3, 0)".into(),
            city: Some("oslo".into()),
        };
        assert_eq!(move_player(&mut document, &to), Ok(()));
        assert_eq!(position(&document), Some(to.clone()));
        let hitched = SAVE
            .replace(" trailer: null\r\n", " trailer: trailer.1\r\n")
            .replace(
                "stored_trailer_attached: false",
                "stored_trailer_attached: true",
            );
        let document = sii::parse_text(&hitched).expect("parses");
        assert_eq!(
            can_move_player(&document),
            Err(PlayerProblem::TrailerHitched)
        );
        let on_job = SAVE.replace("current_job: null", "current_job: job.1");
        let document = sii::parse_text(&on_job).expect("parses");
        assert_eq!(can_drive(&document, "truck.b"), Err(PlayerProblem::OnJob));
    }

    #[test]
    fn reads_and_moves_the_active_entry_too() {
        // Newer saves: `assigned_vehicles` is an entry of its own, apart from `my_vehicles`.
        let save = SAVE
            .replace(" assigned_vehicles: pv.truck\r\n", " assigned_vehicles: pv.active\r\n")
            .replace(
                "player_vehicles : pv.bus {",
                "player_vehicles : pv.active {\r\n vehicle: truck.me\r\n stored_vehicle_placement: (9, 9, 9) (1; 0, 0, 0)\r\n stored_vehicle_city: null\r\n trailer: null\r\n stored_trailer_attached: false\r\n}\r\n\r\nplayer_vehicles : pv.bus {",
            );
        let mut document = sii::parse_text(&save).expect("parses");
        let here = position(&document).expect("a position");
        assert_eq!(here.placement, "(9, 9, 9) (1; 0, 0, 0)");
        assert_eq!(here.city.as_deref(), Some("lyon"));
        let to = Position {
            placement: "(500, 10, -200) (&3f3504f3; 0, &3f3504f3, 0)".into(),
            city: Some("oslo".into()),
        };
        assert_eq!(move_player(&mut document, &to), Ok(()));
        for entry in ["pv.active", "pv.truck"] {
            assert_eq!(
                document
                    .unit(entry)
                    .and_then(|unit| unit.single("stored_vehicle_placement")),
                Some(to.placement.as_str()),
                "{entry}"
            );
        }
    }
}
