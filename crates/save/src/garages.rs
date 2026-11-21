//! The company's garages: one per city, owned or not, with slots for trucks and drivers.
//!
//! Only what real saves show is relied on: an unowned garage has `status: 0` and no slots; a large
//! garage has `status: 3` and five truck and five driver slots (`null` when empty). Buying or
//! upgrading a garage gives it exactly that shape, keeping the trucks and drivers it already has.

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
