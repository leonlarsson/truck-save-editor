//! The player's vehicles (trucks, coaches from the Coaches DLC, and owned trailers): what they
//! are, where they're kept, how worn and how full of fuel they are, and repairing and refuelling
//! them.

use std::collections::HashMap;

use sii::{Document, Value, value};

/// What kind of vehicle it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleKind {
    Truck,
    /// A coach (Coaches DLC).
    Coach,
    /// An owned trailer (`player.trailers[]`).
    Trailer,
}

/// One vehicle the player owns.
#[derive(Clone, Debug, PartialEq)]
pub struct Vehicle {
    pub kind: VehicleKind,
    /// The vehicle unit's id: stable, used for identity everywhere.
    pub id: String,
    /// `volvo.fh16_2012`, from the vehicle's definition path.
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
    /// How full the tank is, from 0 to 1 (`None` for trailers).
    pub fuel: Option<f32>,
    /// A trailer's body type (`refrigerated`, `curtainside`), from its `trailer_def`.
    pub body_type: Option<String>,
}

impl Vehicle {
    /// `Volvo FH16 2012`, from the model id (`None` when the model isn't known).
    pub fn display_name(&self) -> Option<String> {
        self.model_id.as_deref().map(model_display_name)
    }
}

/// Wear of each part, from 0 (new) to 1 (worn out).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wear {
    pub engine: f32,
    pub transmission: f32,
    pub cabin: f32,
    pub chassis: f32,
    /// A trailer's body.
    pub body: f32,
    /// The most worn wheel.
    pub wheels: f32,
}

impl Wear {
    /// Part by part, the more worn of two.
    fn worst_of(self, other: Wear) -> Wear {
        Wear {
            engine: self.engine.max(other.engine),
            transmission: self.transmission.max(other.transmission),
            cabin: self.cabin.max(other.cabin),
            chassis: self.chassis.max(other.chassis),
            body: self.body.max(other.body),
            wheels: self.wheels.max(other.wheels),
        }
    }

    /// The most worn part: what decides whether the truck needs the garage.
    pub fn worst(&self) -> f32 {
        [
            self.engine,
            self.transmission,
            self.cabin,
            self.chassis,
            self.body,
            self.wheels,
        ]
        .into_iter()
        .fold(0.0, f32::max)
    }
}

const WEAR_FIELDS: [&str; 5] = [
    "engine_wear",
    "transmission_wear",
    "cabin_wear",
    "chassis_wear",
    "trailer_body_wear",
];

/// Whether a vehicle keeps its wear on itself (the current format; trailers always do), not on
/// each part's accessory (the older trucks' format).
fn has_own_wear(vehicle: &sii::Unit) -> bool {
    vehicle.get("engine_wear").is_some() || vehicle.get("trailer_body_wear").is_some()
}

/// A trailer and the trailers it pulls (`slave_trailer`, for doubles and B-doubles), in order.
fn trailer_chain(document: &Document, first: &str) -> Vec<String> {
    let mut chain = vec![first.to_string()];
    let mut current = first.to_string();
    // A chain is a few trailers long; the limit stops a damaged save from looping.
    for _ in 0..8 {
        let Some(next) = document
            .unit(&current)
            .and_then(|unit| unit.single("slave_trailer"))
            .filter(|id| *id != "null" && document.unit(id).is_some())
        else {
            break;
        };
        if chain.iter().any(|id| id == next) {
            break;
        }
        chain.push(next.to_string());
        current = next.to_string();
    }
    chain
}

/// Every vehicle the player owns: trucks, then coaches, each in the order the save lists them.
pub fn vehicles(document: &Document) -> Vec<Vehicle> {
    let Some(player) = player(document) else {
        return Vec::new();
    };

    // Where each truck is kept, and who drives it.
    let mut garage_of: HashMap<&str, String> = HashMap::new();
    for garage in document.units_of_kind("garage") {
        let city = garage.id.strip_prefix("garage.").unwrap_or(&garage.id);
        for vehicle in garage
            .items("vehicles")
            .iter()
            .chain(garage.items("trailers"))
        {
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

    let trucks = player
        .items("trucks")
        .iter()
        .map(|id| (VehicleKind::Truck, id));
    let coaches = player
        .items("buses")
        .iter()
        .map(|id| (VehicleKind::Coach, id));
    let trailers = player
        .items("trailers")
        .iter()
        .map(|id| (VehicleKind::Trailer, id));
    trucks
        .chain(coaches)
        .chain(trailers)
        .filter_map(|(kind, id)| Some((kind, document.unit(id)?)))
        .map(|(kind, vehicle)| {
            let (plate, plate_country) = split_plate(vehicle.single("license_plate").unwrap_or(""));
            Vehicle {
                kind,
                id: vehicle.id.clone(),
                model_id: model_id(document, vehicle),
                plate,
                plate_country,
                odometer_km: vehicle
                    .single("odometer")
                    .and_then(value::parse_i64)
                    .unwrap_or(0),
                wear: if kind == VehicleKind::Trailer {
                    // The worst of the whole chain: a double is as worn as its most worn trailer.
                    trailer_chain(document, &vehicle.id)
                        .iter()
                        .filter_map(|id| document.unit(id))
                        .map(|unit| read_wear(document, unit))
                        .fold(Wear::default(), Wear::worst_of)
                } else {
                    read_wear(document, vehicle)
                },
                garage_city: garage_of.get(vehicle.id.as_str()).cloned(),
                driver: driver_of.get(vehicle.id.as_str()).cloned(),
                fuel: (kind != VehicleKind::Trailer)
                    .then(|| vehicle.single("fuel_relative").and_then(value::parse_f32))
                    .flatten(),
                body_type: vehicle
                    .single("trailer_definition")
                    .and_then(|id| document.unit(id))
                    .and_then(|definition| definition.single("body_type"))
                    .map(|body| value::parse_string(body).to_string()),
            }
        })
        .collect()
}

/// Repairs a truck: all wear the game's garage can fix goes to zero. With `including_permanent`,
/// the wear a garage can't fix goes too, making it like new.
///
/// A trailer is repaired with the trailers it pulls.
pub fn repair(document: &mut Document, vehicle_id: &str, including_permanent: bool) -> bool {
    let is_trailer = document
        .unit(vehicle_id)
        .is_some_and(|unit| unit.kind == "trailer");
    if is_trailer {
        let mut repaired = false;
        for id in trailer_chain(document, vehicle_id) {
            repaired |= repair_one(document, &id, including_permanent);
        }
        return repaired;
    }
    repair_one(document, vehicle_id, including_permanent)
}

fn repair_one(document: &mut Document, truck_id: &str, including_permanent: bool) -> bool {
    let Some(vehicle) = document.unit(truck_id) else {
        return false;
    };
    if !has_own_wear(vehicle) {
        // The older format: each part (accessory) holds its own `wear`.
        let parts: Vec<String> = vehicle.items("accessories").to_vec();
        let mut repaired = false;
        for part in parts {
            if let Some(mut accessory) = document.unit_mut(&part)
                && accessory.get("wear").is_some()
            {
                accessory.set("wear", Value::single("0"));
                repaired = true;
            }
        }
        return repaired;
    }

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
        // An array (one per axle) or, on some trailers, a single value.
        match vehicle.get(&field) {
            Some(Value::Array(items)) if !items.is_empty() => {
                let count = items.len();
                vehicle.set(&field, Value::Array(vec!["0".to_string(); count]));
            }
            Some(Value::Single(_)) => vehicle.set(&field, Value::single("0")),
            _ => {}
        }
    }
    true
}

/// Fills a truck's or coach's tank. Returns whether it has one.
pub fn refuel(document: &mut Document, vehicle_id: &str) -> bool {
    let Some(mut vehicle) = document.unit_mut(vehicle_id) else {
        return false;
    };
    if vehicle.get("fuel_relative").is_none() {
        return false;
    }
    vehicle.set("fuel_relative", Value::single("1"));
    true
}

/// A truck's wear, in either format the game has used:
/// - current: `engine_wear`, `transmission_wear`, `cabin_wear`, `chassis_wear` and `wheels_wear[]`
///   on the vehicle;
/// - older (until about 2021): a `wear` on each part's accessory, the part told by its
///   definition path (`…/engine/…`), and on each wheel accessory.
fn read_wear(document: &Document, vehicle: &sii::Unit) -> Wear {
    let float = |unit: &sii::Unit, field: &str| {
        unit.single(field).and_then(value::parse_f32).unwrap_or(0.0)
    };
    if has_own_wear(vehicle) {
        return Wear {
            engine: float(vehicle, "engine_wear"),
            transmission: float(vehicle, "transmission_wear"),
            cabin: float(vehicle, "cabin_wear"),
            chassis: float(vehicle, "chassis_wear"),
            body: float(vehicle, "trailer_body_wear"),
            wheels: match vehicle.get("wheels_wear") {
                Some(Value::Single(_)) => float(vehicle, "wheels_wear"),
                _ => vehicle
                    .items("wheels_wear")
                    .iter()
                    .filter_map(|text| value::parse_f32(text))
                    .fold(0.0, f32::max),
            },
        };
    }

    let mut wear = Wear::default();
    for accessory in vehicle
        .items("accessories")
        .iter()
        .filter_map(|id| document.unit(id))
    {
        let Some(part_wear) = accessory.single("wear").and_then(value::parse_f32) else {
            continue;
        };
        let path = accessory
            .single("data_path")
            .map(value::parse_string)
            .unwrap_or_default();
        let slot = if accessory.kind == "vehicle_wheel_accessory" {
            &mut wear.wheels
        } else if path.contains("/engine/") {
            &mut wear.engine
        } else if path.contains("/transmission/") {
            &mut wear.transmission
        } else if path.contains("/cabin/") {
            &mut wear.cabin
        } else if path.contains("/chassis/") {
            &mut wear.chassis
        } else {
            continue;
        };
        *slot = slot.max(part_wear);
    }
    wear
}

/// Whether a truck's wear is in a format the editor knows (see `read_wear`).
pub fn wear_is_known(document: &Document, truck_id: &str) -> bool {
    let Some(vehicle) = document.unit(truck_id) else {
        return false;
    };
    has_own_wear(vehicle)
        || vehicle
            .items("accessories")
            .iter()
            .filter_map(|id| document.unit(id))
            .any(|accessory| accessory.get("wear").is_some())
}

fn player(document: &Document) -> Option<&sii::Unit> {
    let economy = document.first_of_kind("economy")?;
    document.unit(economy.single("player")?)
}

/// The model from the vehicle's accessories: the one whose definition is
/// `/def/vehicle/truck/<model>/data.sii` (`/def/vehicle/bus/…` for coaches,
/// `/def/vehicle/trailer_owned/…` for owned trailers).
fn model_id(document: &Document, vehicle: &sii::Unit) -> Option<String> {
    vehicle.items("accessories").iter().find_map(|id| {
        let path = document.unit(id)?.single("data_path")?;
        let path = value::parse_string(path);
        let rest = path
            .strip_prefix("/def/vehicle/truck/")
            .or_else(|| path.strip_prefix("/def/vehicle/bus/"))
            .or_else(|| path.strip_prefix("/def/vehicle/trailer_owned/"))?;
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
        let trucks = vehicles(&document);
        assert_eq!(trucks.len(), 1);
        let truck = &trucks[0];
        assert_eq!(truck.display_name().as_deref(), Some("Volvo FH16 2012"));
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
        let truck = &vehicles(&document)[0];
        assert_eq!(truck.wear.worst(), 0.0);
        let vehicle = document.unit("_nameless.10").expect("vehicle");
        assert_eq!(vehicle.single("engine_wear_unfixable"), Some("&3c23d70a"));

        assert!(repair(&mut document, "_nameless.10", true));
        let vehicle = document.unit("_nameless.10").expect("vehicle");
        assert_eq!(vehicle.single("engine_wear_unfixable"), Some("0"));
        assert!(!repair(&mut document, "nope", false));
    }

    const OLDER_SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: _nameless.10\r\n}\r\n\r\nvehicle : _nameless.10 {\r\n accessories: 3\r\n accessories[0]: _nameless.11\r\n accessories[1]: _nameless.12\r\n accessories[2]: _nameless.13\r\n odometer: 500\r\n}\r\n\r\nvehicle_accessory : _nameless.11 {\r\n wear: &3e800000\r\n data_path: \"/def/vehicle/truck/scania.r/engine/dc16.sii\"\r\n}\r\n\r\nvehicle_wheel_accessory : _nameless.12 {\r\n wear: &3e4ccccd\r\n data_path: \"/def/vehicle/t_wheel/plain.sii\"\r\n}\r\n\r\nvehicle_accessory : _nameless.13 {\r\n wear: 0\r\n data_path: \"/def/vehicle/truck/scania.r/data.sii\"\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_and_repairs_wear_in_the_older_format() {
        let mut document = sii::parse_text(OLDER_SAVE).expect("parses");
        assert!(wear_is_known(&document, "_nameless.10"));
        let truck = &vehicles(&document)[0];
        assert_eq!(truck.wear.engine, 0.25);
        assert_eq!(truck.wear.wheels, 0.2);
        assert_eq!(truck.wear.worst(), 0.25);

        assert!(repair(&mut document, "_nameless.10", false));
        assert_eq!(vehicles(&document)[0].wear.worst(), 0.0);
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

    const TRAILERS: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: v1\r\n trailers: 1\r\n trailers[0]: t1\r\n}\r\n\r\nvehicle : v1 {\r\n engine_wear: 0\r\n transmission_wear: 0\r\n cabin_wear: 0\r\n chassis_wear: 0\r\n fuel_relative: &3f000000\r\n wheels_wear: 0\r\n odometer: 10\r\n license_plate: \"AB 123|france\"\r\n}\r\n\r\ntrailer : t1 {\r\n slave_trailer: t2\r\n trailer_body_wear: &3e800000\r\n chassis_wear: 0\r\n wheels_wear: 0\r\n odometer: 20\r\n license_plate: \"TR 1|france\"\r\n}\r\n\r\ntrailer : t2 {\r\n slave_trailer: null\r\n trailer_body_wear: 0\r\n chassis_wear: &3f000000\r\n wheels_wear: 0\r\n odometer: 20\r\n}\r\n\r\ngarage : garage.lyon {\r\n vehicles: 1\r\n vehicles[0]: v1\r\n trailers: 1\r\n trailers[0]: t1\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_trailers_with_the_wear_of_their_whole_chain() {
        let document = sii::parse_text(TRAILERS).expect("parses");
        let vehicles = vehicles(&document);
        assert_eq!(vehicles.len(), 2);
        let trailer = &vehicles[1];
        assert_eq!(trailer.kind, VehicleKind::Trailer);
        assert_eq!(trailer.garage_city.as_deref(), Some("lyon"));
        assert_eq!(trailer.fuel, None);
        assert_eq!(trailer.wear.body, 0.25);
        // The pulled trailer's chassis is the most worn part of the double.
        assert_eq!(trailer.wear.worst(), 0.5);
        assert_eq!(vehicles[0].fuel, Some(0.5));
    }

    #[test]
    fn repairs_a_trailer_with_the_trailers_it_pulls() {
        let mut document = sii::parse_text(TRAILERS).expect("parses");
        assert!(repair(&mut document, "t1", false));
        let trailer = &vehicles(&document)[1];
        assert_eq!(trailer.wear.worst(), 0.0);
    }

    #[test]
    fn refuels_trucks_not_trailers() {
        let mut document = sii::parse_text(TRAILERS).expect("parses");
        assert!(refuel(&mut document, "v1"));
        assert!(!refuel(&mut document, "t1"));
        assert_eq!(vehicles(&document)[0].fuel, Some(1.0));
    }
}
