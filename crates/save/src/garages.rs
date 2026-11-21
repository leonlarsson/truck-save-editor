//! The company's garages: one per city, owned or not, with slots for trucks and drivers.
//!
//! Only what real saves show is relied on: an unowned garage has `status: 0` and no slots; a large
//! garage has `status: 3` and five truck and five driver slots (`null` when empty). Buying or
//! upgrading a garage gives it exactly that shape, keeping the trucks and drivers it already has.
//!
//! A garage's `vehicles[i]` and `drivers[i]` are one slot: the driver in a slot drives its truck
//! (the drivers' own `assigned_truck` is `null` in current saves). So moving a truck or a driver to
//! another garage moves its whole slot, truck and driver together, into an empty slot there.

use sii::{Document, Value};

/// Slots in a large garage, and the status the game gives one.
pub const LARGE_GARAGE_SLOTS: usize = 5;
const LARGE_GARAGE_STATUS: &str = "3";

#[derive(Clone, Debug, PartialEq)]
pub struct Garage {
    /// The garage unit's id (`garage.berlin`).
    pub id: String,
    /// The city's token (`berlin`).
    pub city: String,
    pub is_owned: bool,
    /// How many trucks (and drivers) it can hold.
    pub slots: usize,
    /// How many of the truck slots are filled.
    pub trucks: usize,
    /// How many of the driver slots are filled.
    pub drivers: usize,
}

impl Garage {
    /// Whether it's as big as a garage gets.
    pub fn is_large(&self) -> bool {
        self.is_owned && self.slots >= LARGE_GARAGE_SLOTS
    }
}

/// Every garage, in the order the save lists them.
pub fn garages(document: &Document) -> Vec<Garage> {
    document
        .units_of_kind("garage")
        .map(|unit| {
            let filled = |field: &str| unit.items(field).iter().filter(|id| *id != "null").count();
            let status = unit.single("status").unwrap_or("0");
            Garage {
                id: unit.id.clone(),
                city: unit
                    .id
                    .strip_prefix("garage.")
                    .unwrap_or(&unit.id)
                    .to_string(),
                is_owned: status != "0",
                slots: unit.items("vehicles").len(),
                trucks: filled("vehicles"),
                drivers: filled("drivers"),
            }
        })
        .collect()
}

/// Makes a garage owned and large, keeping the trucks and drivers it has. `false` if there's no
/// such garage.
pub fn upgrade_garage(document: &mut Document, id: &str) -> bool {
    let Some(mut unit) = document.unit_mut(id) else {
        return false;
    };
    if unit.kind() != "garage" {
        return false;
    }
    for field in ["vehicles", "drivers"] {
        let mut slots = unit
            .get(field)
            .map(|value| value.items().to_vec())
            .unwrap_or_default();
        slots.resize(slots.len().max(LARGE_GARAGE_SLOTS), "null".to_string());
        unit.set(field, Value::Array(slots));
    }
    unit.set("status", Value::single(LARGE_GARAGE_STATUS));
    true
}

/// Why a truck or driver can't move to a garage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveProblem {
    /// It isn't in any garage (a coach, a truck at the dealer, a driver without a slot).
    NotInGarage,
    /// Its slot holds the player: your own truck goes where you drive it.
    IsPlayers,
    /// It's in that garage already.
    AlreadyThere,
    /// The garage isn't the company's.
    NotOwned,
    /// The garage has no slot with neither a truck nor a driver.
    Full,
}

/// Where a truck or driver is: its garage's id and the slot's index.
fn slot_of(document: &Document, id: &str) -> Option<(String, usize)> {
    document.units_of_kind("garage").find_map(|garage| {
        ["vehicles", "drivers"].iter().find_map(|field| {
            garage
                .items(field)
                .iter()
                .position(|item| item == id)
                .map(|index| (garage.id.clone(), index))
        })
    })
}

/// The first slot in a garage with neither a truck nor a driver.
fn empty_slot(garage: &sii::Unit) -> Option<usize> {
    let vehicles = garage.items("vehicles");
    let drivers = garage.items("drivers");
    (0..vehicles.len()).find(|&index| {
        vehicles[index] == "null" && drivers.get(index).is_none_or(|driver| driver == "null")
    })
}

/// Whether a truck or driver (by id) can move to a garage, and if so from which slot to which.
fn plan_move(
    document: &Document,
    id: &str,
    target: &str,
) -> Result<(String, usize, usize), MoveProblem> {
    let (source, from) = slot_of(document, id).ok_or(MoveProblem::NotInGarage)?;
    let source_unit = document.unit(&source).ok_or(MoveProblem::NotInGarage)?;
    let is_players = source_unit
        .items("drivers")
        .get(from)
        .and_then(|driver| document.unit(driver))
        .is_some_and(|driver| driver.kind == "driver_player");
    if is_players {
        return Err(MoveProblem::IsPlayers);
    }
    if source == target {
        return Err(MoveProblem::AlreadyThere);
    }
    let target_unit = document
        .unit(target)
        .filter(|unit| unit.kind == "garage")
        .ok_or(MoveProblem::NotOwned)?;
    if target_unit.single("status").unwrap_or("0") == "0" {
        return Err(MoveProblem::NotOwned);
    }
    let to = empty_slot(target_unit).ok_or(MoveProblem::Full)?;
    Ok((source, from, to))
}

/// Whether a truck or driver can move to a garage (see [`move_to_garage`]).
pub fn can_move_to_garage(document: &Document, id: &str, target: &str) -> Result<(), MoveProblem> {
    plan_move(document, id, target).map(|_| ())
}

/// Where a truck or driver could move: the garage it's in, who shares its slot (its driver, or
/// their truck), and every other company garage with a free slot.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveOptions {
    pub from: String,
    /// The truck or driver that moves along, if the slot has one.
    pub partner: Option<String>,
    /// Garage ids, in the save's order.
    pub targets: Vec<String>,
    /// The company's other garages, with no free slot (to show, not to choose).
    pub full: Vec<String>,
}

/// Where a truck or driver (by id) could move, or why it can't.
pub fn move_options(document: &Document, id: &str) -> Result<MoveOptions, MoveProblem> {
    let (from, index) = slot_of(document, id).ok_or(MoveProblem::NotInGarage)?;
    let source = document.unit(&from).ok_or(MoveProblem::NotInGarage)?;
    let slot = |field: &str| {
        source
            .items(field)
            .get(index)
            .filter(|item| *item != "null")
            .cloned()
    };
    let (truck, driver) = (slot("vehicles"), slot("drivers"));
    let is_players = driver
        .as_deref()
        .and_then(|driver| document.unit(driver))
        .is_some_and(|driver| driver.kind == "driver_player");
    if is_players {
        return Err(MoveProblem::IsPlayers);
    }
    let partner = [truck, driver]
        .into_iter()
        .flatten()
        .find(|item| item != id);
    let (mut targets, mut full) = (Vec::new(), Vec::new());
    for garage in document.units_of_kind("garage") {
        if garage.id == from || garage.single("status").unwrap_or("0") == "0" {
            continue;
        }
        if empty_slot(garage).is_some() {
            targets.push(garage.id.clone());
        } else {
            full.push(garage.id.clone());
        }
    }
    Ok(MoveOptions {
        from,
        partner,
        targets,
        full,
    })
}

/// The garage a truck or driver is in, by id.
pub fn garage_of(document: &Document, id: &str) -> Option<String> {
    slot_of(document, id).map(|(garage, _)| garage)
}

/// Moves a truck or driver (by id) to another of the company's garages, with the rest of its slot:
/// a truck takes its driver along, a driver their truck. They go into the first slot there with
/// neither, and leave theirs empty.
pub fn move_to_garage(document: &mut Document, id: &str, target: &str) -> Result<(), MoveProblem> {
    let (source, from, to) = plan_move(document, id, target)?;
    let mut moving = Vec::new();
    {
        let Some(mut unit) = document.unit_mut(&source) else {
            return Err(MoveProblem::NotInGarage);
        };
        for field in ["vehicles", "drivers"] {
            let mut slots = unit
                .get(field)
                .map(|value| value.items().to_vec())
                .unwrap_or_default();
            let item = slots
                .get_mut(from)
                .map(|item| std::mem::replace(item, "null".to_string()))
                .unwrap_or_else(|| "null".to_string());
            moving.push(item);
            unit.set(field, Value::Array(slots));
        }
    }
    let Some(mut unit) = document.unit_mut(target) else {
        return Err(MoveProblem::NotOwned);
    };
    for (field, item) in ["vehicles", "drivers"].into_iter().zip(moving) {
        let mut slots = unit
            .get(field)
            .map(|value| value.items().to_vec())
            .unwrap_or_default();
        if let Some(slot) = slots.get_mut(to) {
            *slot = item;
        }
        unit.set(field, Value::Array(slots));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\ngarage : garage.berlin {\r\n vehicles: 0\r\n drivers: 0\r\n trailers: 0\r\n status: 0\r\n profit_log: _nameless.1\r\n productivity: 0\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 1\r\n vehicles[0]: _nameless.5\r\n drivers: 1\r\n drivers[0]: null\r\n trailers: 0\r\n status: 6\r\n profit_log: _nameless.2\r\n productivity: 0\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_garages() {
        let document = sii::parse_text(SAVE).expect("parses");
        let garages = garages(&document);
        assert_eq!(garages.len(), 2);
        assert_eq!(garages[0].city, "berlin");
        assert!(!garages[0].is_owned);
        assert_eq!(garages[1].slots, 1);
        assert_eq!(garages[1].trucks, 1);
        assert_eq!(garages[1].drivers, 0);
        assert!(!garages[1].is_large());
    }

    const MOVES: &str = "SiiNunit\r\n{\r\ngarage : garage.berlin {\r\n vehicles: 2\r\n vehicles[0]: truck.a\r\n vehicles[1]: truck.me\r\n drivers: 2\r\n drivers[0]: driver.a\r\n drivers[1]: me\r\n status: 3\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 3\r\n vehicles[0]: truck.b\r\n vehicles[1]: null\r\n vehicles[2]: null\r\n drivers: 3\r\n drivers[0]: null\r\n drivers[1]: driver.c\r\n drivers[2]: null\r\n status: 3\r\n}\r\n\r\ngarage : garage.rome {\r\n vehicles: 0\r\n drivers: 0\r\n status: 0\r\n}\r\n\r\ndriver_ai : driver.a {\r\n}\r\n\r\ndriver_player : me {\r\n}\r\n\r\n}\r\n";

    #[test]
    fn a_truck_moves_with_its_driver_into_an_empty_slot() {
        let mut document = sii::parse_text(MOVES).expect("parses");
        assert_eq!(
            move_to_garage(&mut document, "truck.a", "garage.lyon"),
            Ok(())
        );
        let berlin = document.unit("garage.berlin").expect("berlin");
        assert_eq!(berlin.items("vehicles"), ["null", "truck.me"]);
        assert_eq!(berlin.items("drivers"), ["null", "me"]);
        // Slot 1 has a driver of its own, so the pair goes to slot 2.
        let lyon = document.unit("garage.lyon").expect("lyon");
        assert_eq!(lyon.items("vehicles"), ["truck.b", "null", "truck.a"]);
        assert_eq!(lyon.items("drivers"), ["null", "driver.c", "driver.a"]);
        assert_eq!(
            garage_of(&document, "driver.a").as_deref(),
            Some("garage.lyon")
        );
    }

    #[test]
    fn moving_a_driver_takes_their_truck_and_a_lone_truck_goes_alone() {
        let mut document = sii::parse_text(MOVES).expect("parses");
        assert_eq!(
            move_to_garage(&mut document, "driver.a", "garage.lyon"),
            Ok(())
        );
        assert_eq!(
            garage_of(&document, "truck.a").as_deref(),
            Some("garage.lyon")
        );
        // Berlin has the slot the pair left; Lyon's lone truck (no driver) can take it.
        assert_eq!(
            can_move_to_garage(&document, "truck.b", "garage.berlin"),
            Ok(())
        );
        assert_eq!(
            move_to_garage(&mut document, "truck.b", "garage.berlin"),
            Ok(())
        );
        let berlin = document.unit("garage.berlin").expect("berlin");
        assert_eq!(berlin.items("vehicles"), ["truck.b", "truck.me"]);
        assert_eq!(berlin.items("drivers"), ["null", "me"]);
    }

    #[test]
    fn lists_where_a_truck_could_go() {
        let document = sii::parse_text(MOVES).expect("parses");
        let options = move_options(&document, "truck.a").expect("can move");
        assert_eq!(options.from, "garage.berlin");
        assert_eq!(options.partner.as_deref(), Some("driver.a"));
        // Not Berlin (it's there), not Rome (not owned).
        assert_eq!(options.targets, ["garage.lyon"]);
        assert!(options.full.is_empty());
        assert_eq!(
            move_options(&document, "truck.b")
                .expect("can move")
                .partner,
            None
        );
        assert_eq!(move_options(&document, "me"), Err(MoveProblem::IsPlayers));
        assert_eq!(
            move_options(&document, "truck.x"),
            Err(MoveProblem::NotInGarage)
        );
    }

    #[test]
    fn says_why_a_move_cant_happen() {
        let document = sii::parse_text(MOVES).expect("parses");
        let problem = |id: &str, target: &str| can_move_to_garage(&document, id, target);
        assert_eq!(
            problem("truck.me", "garage.lyon"),
            Err(MoveProblem::IsPlayers)
        );
        assert_eq!(problem("me", "garage.lyon"), Err(MoveProblem::IsPlayers));
        assert_eq!(
            problem("truck.a", "garage.berlin"),
            Err(MoveProblem::AlreadyThere)
        );
        assert_eq!(
            problem("truck.a", "garage.rome"),
            Err(MoveProblem::NotOwned)
        );
        assert_eq!(
            problem("truck.a", "garage.nowhere"),
            Err(MoveProblem::NotOwned)
        );
        assert_eq!(
            problem("truck.x", "garage.lyon"),
            Err(MoveProblem::NotInGarage)
        );
        let mut document = document;
        assert_eq!(
            move_to_garage(&mut document, "truck.a", "garage.lyon"),
            Ok(())
        );
        assert_eq!(
            can_move_to_garage(&document, "driver.c", "garage.lyon"),
            Err(MoveProblem::AlreadyThere)
        );
        // Berlin's only free slot is the one truck.a left; after driver.c takes it, it's full.
        assert_eq!(
            move_to_garage(&mut document, "driver.c", "garage.berlin"),
            Ok(())
        );
        assert_eq!(
            can_move_to_garage(&document, "truck.b", "garage.berlin"),
            Err(MoveProblem::Full)
        );
    }

    #[test]
    fn buys_and_upgrades_keeping_what_is_there() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert!(upgrade_garage(&mut document, "garage.berlin"));
        assert!(upgrade_garage(&mut document, "garage.lyon"));
        assert!(!upgrade_garage(&mut document, "garage.nowhere"));
        let garages = garages(&document);
        assert!(garages.iter().all(Garage::is_large));
        assert_eq!(garages[1].trucks, 1);
        let lyon = document.unit("garage.lyon").expect("lyon");
        assert_eq!(lyon.items("vehicles")[0], "_nameless.5");
        assert_eq!(lyon.items("vehicles")[4], "null");
        // Fields stay where they were; only values change.
        assert_eq!(lyon.fields[3].name, "status");
        assert_eq!(lyon.single("status"), Some("3"));
    }
}
