//! The player's trucks: what they are, where they're kept, how worn they are, and repairing them.

use std::collections::HashMap;

use sii::{Document, Value, value};

/// One truck the player owns.
#[derive(Clone, Debug, PartialEq)]
pub struct Truck {
    /// The vehicle unit's id: stable, used for identity everywhere.
    pub id: String,
    /// `volvo.fh16_2012`, from the truck's definition path.
    pub model_id: Option<String>,
    pub plate: String,
    /// The plate's country token (`germany`).
    pub plate_country: Option<String>,
    pub odometer_km: i64,
    pub wear: Wear,
    /// The garage's city token (`berlin`), if it's assigned to one.
    pub garage_city: Option<String>,
    /// The driver's id (`driver.12`), if one drives it.
    pub driver: Option<String>,
}

impl Truck {
    /// `Volvo FH16 2012`, from the model id; `Unknown truck` without one.
    pub fn display_name(&self) -> String {
        self.model_id
            .as_deref()
            .map(model_display_name)
            .unwrap_or_else(|| "Unknown truck".to_string())
    }
}

/// Wear of each part, from 0 (new) to 1 (worn out).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wear {
    pub engine: f32,
    pub transmission: f32,
    pub cabin: f32,
    pub chassis: f32,
    /// The most worn wheel.
    pub wheels: f32,
}

impl Wear {
    /// The most worn part: what decides whether the truck needs the garage.
    pub fn worst(&self) -> f32 {
        [
            self.engine,
            self.transmission,
            self.cabin,
            self.chassis,
            self.wheels,
        ]
        .into_iter()
        .fold(0.0, f32::max)
    }
}

const WEAR_FIELDS: [&str; 4] = [
    "engine_wear",
    "transmission_wear",
    "cabin_wear",
    "chassis_wear",
];

/// Every truck the player owns, in the order the save lists them.
pub fn trucks(document: &Document) -> Vec<Truck> {
    let Some(player) = player(document) else {
        return Vec::new();
    };

    // Where each truck is kept, and who drives it.
    let mut garage_of: HashMap<&str, String> = HashMap::new();
    for garage in document.units_of_kind("garage") {
        let city = garage.id.strip_prefix("garage.").unwrap_or(&garage.id);
        for vehicle in garage.items("vehicles") {
            garage_of.insert(vehicle.as_str(), city.to_string());
        }
    }
    let mut driver_of: HashMap<&str, String> = HashMap::new();
    for driver in document.units_of_kind("driver_ai") {
        for field in ["assigned_truck", "adopted_truck"] {
            if let Some(truck) = driver.single(field).filter(|id| *id != "null") {
                driver_of.entry(truck).or_insert_with(|| driver.id.clone());
            }
        }
    }

    player
        .items("trucks")
        .iter()
        .filter_map(|id| document.unit(id))
        .map(|vehicle| {
            let float = |field: &str| {
                vehicle
                    .single(field)
                    .and_then(value::parse_f32)
                    .unwrap_or(0.0)
            };
            let (plate, plate_country) = split_plate(vehicle.single("license_plate").unwrap_or(""));
            Truck {
                id: vehicle.id.clone(),
                model_id: model_id(document, vehicle),
                plate,
                plate_country,
                odometer_km: vehicle
                    .single("odometer")
                    .and_then(value::parse_i64)
                    .unwrap_or(0),
                wear: Wear {
                    engine: float("engine_wear"),
                    transmission: float("transmission_wear"),
                    cabin: float("cabin_wear"),
                    chassis: float("chassis_wear"),
                    wheels: vehicle
                        .items("wheels_wear")
                        .iter()
                        .filter_map(|text| value::parse_f32(text))
                        .fold(0.0, f32::max),
                },
                garage_city: garage_of.get(vehicle.id.as_str()).cloned(),
                driver: driver_of.get(vehicle.id.as_str()).cloned(),
            }
        })
        .collect()
}

/// Repairs a truck: all wear the game's garage can fix goes to zero. With `including_permanent`,
/// the wear a garage can't fix goes too, making it like new.
pub fn repair(document: &mut Document, truck_id: &str, including_permanent: bool) -> bool {
    let Some(mut vehicle) = document.unit_mut(truck_id) else {
        return false;
    };
    for field in WEAR_FIELDS {
        if vehicle.get(field).is_some() {
            vehicle.set(field, Value::single("0"));
        }
        let permanent = format!("{field}_unfixable");
        if including_permanent && vehicle.get(&permanent).is_some() {
            vehicle.set(&permanent, Value::single("0"));
        }
    }
    let mut wheel_fields = vec!["wheels_wear".to_string()];
    if including_permanent {
        wheel_fields.push("wheels_wear_unfixable".to_string());
    }
    for field in wheel_fields {
        let count = vehicle.get(&field).map(|v| v.items().len()).unwrap_or(0);
        if count > 0 {
            vehicle.set(&field, Value::Array(vec!["0".to_string(); count]));
        }
    }
    true
}

fn player(document: &Document) -> Option<&sii::Unit> {
    let economy = document.first_of_kind("economy")?;
    document.unit(economy.single("player")?)
}

/// The model from the truck's accessories: the one whose definition is
/// `/def/vehicle/truck/<model>/data.sii`.
fn model_id(document: &Document, vehicle: &sii::Unit) -> Option<String> {
    vehicle.items("accessories").iter().find_map(|id| {
        let path = document.unit(id)?.single("data_path")?;
        let path = value::parse_string(path);
        let rest = path.strip_prefix("/def/vehicle/truck/")?;
        let (model, file) = rest.split_once('/')?;
        (file == "data.sii").then(|| model.to_string())
    })
}

/// `"XY 123|germany"` → (`XY 123`, `germany`). Plates can hold formatting tags like `<offset …>`,
/// which are dropped.
fn split_plate(text: &str) -> (String, Option<String>) {
    let text = value::parse_string(text);
    let (plate, country) = match text.split_once('|') {
        Some((plate, country)) => (plate, Some(country.to_string())),
        None => (text.as_ref(), None),
    };
    let mut clean = String::with_capacity(plate.len());
    let mut in_tag = false;
    for c in plate.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => clean.push(c),
            _ => {}
        }
    }
    (clean.trim().to_string(), country)
}

/// `volvo.fh16_2012` → `Volvo FH16 2012`; `mercedes.actros2014` → `Mercedes-Benz Actros2014`.
pub fn model_display_name(model_id: &str) -> String {
    let (brand, model) = model_id.split_once('.').unwrap_or((model_id, ""));
    let brand = match brand {
        "daf" => "DAF".to_string(),
        "man" => "MAN".to_string(),
        "mercedes" => "Mercedes-Benz".to_string(),
        "iveco" => "Iveco".to_string(),
        "renault" => "Renault".to_string(),
        "scania" => "Scania".to_string(),
        "volvo" => "Volvo".to_string(),
        other => capitalize(other),
    };
    let model: Vec<String> = model
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            if part.len() <= 4 || part.chars().any(|c| c.is_ascii_digit()) && part.len() <= 5 {
                part.to_uppercase()
            } else {
                capitalize(part)
            }
        })
        .collect();
    if model.is_empty() {
        brand
    } else {
        format!("{brand} {}", model.join(" "))
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: _nameless.10\r\n}\r\n\r\nvehicle : _nameless.10 {\r\n engine_wear: &3e800000\r\n transmission_wear: 0\r\n cabin_wear: 0\r\n chassis_wear: &3d4ccccd\r\n engine_wear_unfixable: &3c23d70a\r\n accessories: 2\r\n accessories[0]: _nameless.11\r\n accessories[1]: _nameless.12\r\n odometer: 12345\r\n license_plate: \"XY 123|germany\"\r\n wheels_wear: 2\r\n wheels_wear[0]: &3e4ccccd\r\n wheels_wear[1]: 0\r\n}\r\n\r\nvehicle_accessory : _nameless.11 {\r\n data_path: \"/def/vehicle/truck/volvo.fh16_2012/head_light/xenon.sii\"\r\n}\r\n\r\nvehicle_accessory : _nameless.12 {\r\n data_path: \"/def/vehicle/truck/volvo.fh16_2012/data.sii\"\r\n}\r\n\r\ngarage : garage.berlin {\r\n vehicles: 1\r\n vehicles[0]: _nameless.10\r\n}\r\n\r\ndriver_ai : driver.12 {\r\n assigned_truck: _nameless.10\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_the_trucks() {
        let document = sii::parse_text(SAVE).expect("parses");
        let trucks = trucks(&document);
        assert_eq!(trucks.len(), 1);
        let truck = &trucks[0];
        assert_eq!(truck.display_name(), "Volvo FH16 2012");
        assert_eq!(truck.plate, "XY 123");
        assert_eq!(truck.plate_country.as_deref(), Some("germany"));
        assert_eq!(truck.odometer_km, 12345);
        assert_eq!(truck.wear.engine, 0.25);
        assert_eq!(truck.wear.worst(), 0.25);
        assert_eq!(truck.garage_city.as_deref(), Some("berlin"));
        assert_eq!(truck.driver.as_deref(), Some("driver.12"));
    }

    #[test]
    fn repairs_fixable_wear_or_everything() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert!(repair(&mut document, "_nameless.10", false));
        let truck = &trucks(&document)[0];
        assert_eq!(truck.wear.worst(), 0.0);
        let vehicle = document.unit("_nameless.10").expect("vehicle");
        assert_eq!(vehicle.single("engine_wear_unfixable"), Some("&3c23d70a"));

        assert!(repair(&mut document, "_nameless.10", true));
        let vehicle = document.unit("_nameless.10").expect("vehicle");
        assert_eq!(vehicle.single("engine_wear_unfixable"), Some("0"));
        assert!(!repair(&mut document, "nope", false));
    }

    #[test]
    fn names_models_and_cleans_plates() {
        assert_eq!(model_display_name("scania.s_2016"), "Scania S 2016");
        assert_eq!(
            model_display_name("mercedes.actros"),
            "Mercedes-Benz Actros"
        );
        assert_eq!(model_display_name("man.tgx_euro6"), "MAN TGX EURO6");
        assert_eq!(
            split_plate("\"<offset hshift=-2>AB 123|france\"").0,
            "AB 123"
        );
    }
}
