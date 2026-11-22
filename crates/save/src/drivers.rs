//! The drivers the company has hired (`driver_ai` units listed in `player.drivers`): their skills,
//! experience, where they're from, and maxing them out.

use sii::{Document, Value, value};

use crate::career::{ADR_ALL, MAX_SKILL_LEVEL, Skills};

#[derive(Clone, Debug, PartialEq)]
pub struct Driver {
    /// The unit's id (`driver.59`).
    pub id: String,
    /// The hometown's city token.
    pub hometown: String,
    /// Where the driver is now (city token).
    pub current_city: String,
    pub experience: i64,
    pub skills: Skills,
    /// The truck they drive, if any (vehicle id).
    pub truck: Option<String>,
}

/// Every driver the company has hired, in the order the save lists them.
pub fn drivers(document: &Document) -> Vec<Driver> {
    let Some(player) = document
        .first_of_kind("economy")
        .and_then(|economy| economy.single("player"))
        .and_then(|id| document.unit(id))
    else {
        return Vec::new();
    };
    // A garage's `vehicles[i]` and `drivers[i]` are one slot: the driver drives that truck.
    let mut slot_trucks: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    for garage in document.units_of_kind("garage") {
        for (truck, driver) in garage.items("vehicles").iter().zip(garage.items("drivers")) {
            if truck != "null" && driver != "null" {
                slot_trucks.insert(driver.as_str(), truck.clone());
            }
        }
    }
    player
        .items("drivers")
        .iter()
        .filter_map(|id| document.unit(id))
        .filter(|unit| unit.kind == "driver_ai")
        .map(|unit| {
            let level = |field: &str| {
                unit.single(field)
                    .and_then(value::parse_i64)
                    .map_or(0, |v| v.clamp(0, i64::from(u8::MAX)) as u8)
            };
            let city = |field: &str| {
                unit.single(field)
                    .map(|text| value::parse_string(text).into_owned())
                    .unwrap_or_default()
            };
            // The truck in the driver's garage slot (that's what the game shows); the driver's own
            // fields when they aren't in a garage (they're often empty in current saves).
            let truck = slot_trucks.get(unit.id.as_str()).cloned().or_else(|| {
                ["assigned_truck", "adopted_truck"]
                    .iter()
                    .filter_map(|field| unit.single(field))
                    .find(|id| *id != "null")
                    .map(str::to_string)
            });
            Driver {
                id: unit.id.clone(),
                hometown: city("hometown"),
                current_city: city("current_city"),
                experience: unit
                    .single("experience_points")
                    .and_then(value::parse_i64)
                    .unwrap_or(0),
                skills: Skills {
                    adr: level("adr"),
                    long_distance: level("long_dist"),
                    high_value_cargo: level("heavy"),
                    fragile_cargo: level("fragile"),
                    just_in_time: level("urgent"),
                    ecodriving: level("mechanical"),
                },
                truck,
            }
        })
        .collect()
}

/// Gives a driver every skill at its highest level and every ADR class. `false` if there's no
/// such driver.
pub fn max_driver(document: &mut Document, id: &str) -> bool {
    let Some(mut unit) = document.unit_mut(id) else {
        return false;
    };
    if unit.kind() != "driver_ai" {
        return false;
    }
    let maxed = Skills {
        adr: ADR_ALL,
        long_distance: MAX_SKILL_LEVEL,
        high_value_cargo: MAX_SKILL_LEVEL,
        fragile_cargo: MAX_SKILL_LEVEL,
        just_in_time: MAX_SKILL_LEVEL,
        ecodriving: MAX_SKILL_LEVEL,
    };
    unit.set("adr", Value::single(maxed.adr.to_string()));
    for (field, level) in maxed.levels() {
        unit.set(field, Value::single(level.to_string()));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n drivers: 1\r\n drivers[0]: driver.12\r\n}\r\n\r\ndriver_ai : driver.12 {\r\n adr: 7\r\n long_dist: 4\r\n heavy: 4\r\n fragile: 4\r\n urgent: 4\r\n mechanical: 3\r\n hometown: berlin\r\n current_city: lyon\r\n experience_points: 5000\r\n assigned_truck: null\r\n adopted_truck: _nameless.10\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_hired_drivers() {
        let document = sii::parse_text(SAVE).expect("parses");
        let drivers = drivers(&document);
        assert_eq!(drivers.len(), 1);
        let driver = &drivers[0];
        assert_eq!(driver.hometown, "berlin");
        assert_eq!(driver.current_city, "lyon");
        assert_eq!(driver.experience, 5000);
        assert_eq!(driver.skills.adr, 7);
        assert_eq!(driver.skills.ecodriving, 3);
        assert_eq!(driver.truck.as_deref(), Some("_nameless.10"));
    }

    #[test]
    fn a_drivers_truck_is_the_one_in_their_garage_slot() {
        let save = SAVE.replace(
            " adopted_truck: _nameless.10\r\n",
            " adopted_truck: null\r\n",
        ) .replace(
            "}\r\n\r\n}\r\n",
            "}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 2\r\n vehicles[0]: truck.a\r\n vehicles[1]: truck.b\r\n drivers: 2\r\n drivers[0]: null\r\n drivers[1]: driver.12\r\n}\r\n\r\n}\r\n",
        );
        let document = sii::parse_text(&save).expect("parses");
        assert_eq!(drivers(&document)[0].truck.as_deref(), Some("truck.b"));
    }

    #[test]
    fn maxes_a_driver() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert!(max_driver(&mut document, "driver.12"));
        assert!(!max_driver(&mut document, "p"));
        let driver = &drivers(&document)[0];
        assert_eq!(driver.skills.adr, ADR_ALL);
        assert_eq!(driver.skills.levels().map(|(_, l)| l), [6; 5]);
        // Only skills changed.
        assert_eq!(driver.experience, 5000);
    }
}
