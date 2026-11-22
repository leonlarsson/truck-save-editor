//! Changes to one truck or trailer beyond repairs: the cargo a trailer carries, its own weight,
//! the countries it may enter, a fuel level above a full tank, the badge on a truck's grille, and
//! licence plates in colours of the player's choosing.
//!
//! A trailer can pull more trailers (`slave_trailer`, for doubles and B-doubles); a change to a
//! trailer applies to all of them. Each trailer's model is a `trailer_def` unit
//! (`trailer_definition`): its `chassis_mass` and `body_mass` are the empty weight, and
//! `country_validity[]` lists the countries it may be driven in (empty for everywhere).
//!
//! A plate is `text|country` (`AB 123|sweden`); the text may carry the game's markup, the same
//! the game's own plate templates use (`def/country/*/license_plates.sii`). Colours there are
//! written `AABBGGRR` (Belgium's red letters are `FF0000CC`).

use sii::{Document, Value, value};

/// One change to a truck or trailer (by id).
#[derive(Clone, Debug, PartialEq)]
pub enum VehicleEdit {
    /// The weight of the cargo, in kilograms, spread over the trailer and the ones it pulls.
    CargoMass { trailer: String, kg: f64 },
    /// The trailer's own weight (chassis and body), in kilograms, for all its trailers.
    EmptyMass { trailer: String, kg: f64 },
    /// No country restrictions for the trailer and the ones it pulls.
    Unrestrict { trailer: String },
    /// The fuel level: 1 is a full tank, more is more than the tank holds.
    Fuel { vehicle: String, ratio: f64 },
    /// No badge on the truck.
    RemoveBadge { vehicle: String },
    /// Another engine or transmission (`part` is `engine` or `transmission`): the accessory
    /// points at another of the model's definitions, and its refund is that one's price.
    Part {
        vehicle: String,
        part: String,
        data_path: String,
        price: i64,
    },
}

impl VehicleEdit {
    /// The truck or trailer it changes.
    pub fn vehicle(&self) -> &str {
        match self {
            VehicleEdit::CargoMass { trailer, .. }
            | VehicleEdit::EmptyMass { trailer, .. }
            | VehicleEdit::Unrestrict { trailer } => trailer,
            VehicleEdit::Fuel { vehicle, .. }
            | VehicleEdit::RemoveBadge { vehicle }
            | VehicleEdit::Part { vehicle, .. } => vehicle,
        }
    }

    /// Whether it changes the same thing as `other` (a later one replaces it).
    pub fn same_kind(&self, other: &VehicleEdit) -> bool {
        let same_part = match (self, other) {
            (VehicleEdit::Part { part: a, .. }, VehicleEdit::Part { part: b, .. }) => a == b,
            _ => true,
        };
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && self.vehicle() == other.vehicle()
            && same_part
    }
}

/// A trailer and the ones it pulls, in order.
pub fn trailer_chain(document: &Document, trailer: &str) -> Vec<String> {
    let mut chain = Vec::new();
    let mut next = Some(trailer.to_string());
    while let Some(id) = next.take() {
        let Some(unit) = document.unit(&id).filter(|unit| unit.kind == "trailer") else {
            break;
        };
        if chain.contains(&id) {
            break;
        }
        next = unit
            .single("slave_trailer")
            .filter(|slave| *slave != "null")
            .map(str::to_string);
        chain.push(id);
    }
    chain
}

/// The model units of a trailer and the ones it pulls (each once).
fn chain_defs(document: &Document, trailer: &str) -> Vec<String> {
    let mut defs: Vec<String> = Vec::new();
    for id in trailer_chain(document, trailer) {
        let def = document
            .unit(&id)
            .and_then(|unit| unit.single("trailer_definition"))
            .filter(|def| *def != "null" && document.unit(def).is_some())
            .map(str::to_string);
        if let Some(def) = def
            && !defs.contains(&def)
        {
            defs.push(def);
        }
    }
    defs
}

fn number(document: &Document, unit: &str, field: &str) -> f64 {
    document
        .unit(unit)
        .and_then(|unit| unit.single(field))
        .and_then(value::parse_f32)
        .map_or(0.0, f64::from)
}

/// What's known about a trailer's weight and where it may go.
#[derive(Clone, Debug, PartialEq)]
pub struct TrailerLoad {
    /// The cargo's weight, all its trailers together, in kilograms.
    pub cargo_kg: f64,
    /// Its own weight, all its trailers together, in kilograms.
    pub empty_kg: f64,
    /// The countries it may be driven in (tokens); empty for everywhere.
    pub countries: Vec<String>,
}

pub fn trailer_load(document: &Document, trailer: &str) -> Option<TrailerLoad> {
    let chain = trailer_chain(document, trailer);
    if chain.is_empty() {
        return None;
    }
    let cargo_kg = chain
        .iter()
        .map(|id| number(document, id, "cargo_mass"))
        .sum();
    let defs = chain_defs(document, trailer);
    let empty_kg = defs
        .iter()
        .map(|def| number(document, def, "chassis_mass") + number(document, def, "body_mass"))
        .sum();
    let mut countries: Vec<String> = Vec::new();
    for def in &defs {
        for country in document
            .unit(def)
            .map(|unit| unit.items("country_validity").to_vec())
            .unwrap_or_default()
        {
            if !countries.contains(&country) {
                countries.push(country);
            }
        }
    }
    Some(TrailerLoad {
        cargo_kg,
        empty_kg,
        countries,
    })
}

/// A whole number when it is one (`20000`), else the float as the game writes it.
fn mass_text(kg: f64) -> String {
    let kg = kg.max(0.0);
    if kg.fract() == 0.0 && kg < 1e9 {
        format!("{}", kg as i64)
    } else {
        value::format_f32(kg as f32)
    }
}

fn set_if_present(document: &mut Document, unit: &str, field: &str, value: Value) {
    if let Some(mut unit) = document.unit_mut(unit)
        && unit.get(field).is_some()
    {
        unit.set(field, value);
    }
}

/// The badge accessory of a truck (its id), if it has one.
pub fn badge(document: &Document, vehicle: &str) -> Option<String> {
    let unit = document.unit(vehicle)?;
    unit.items("accessories")
        .iter()
        .find(|accessory| {
            document
                .unit(accessory)
                .and_then(|unit| unit.single("data_path"))
                .is_some_and(|path| path.contains("/badge/"))
        })
        .cloned()
}

/// A truck's engine or transmission (`part`: `engine`, `transmission`): its accessory's id and
/// the definition it points at.
pub fn vehicle_part(document: &Document, vehicle: &str, part: &str) -> Option<(String, String)> {
    let unit = document.unit(vehicle)?;
    let folder = format!("/{part}/");
    unit.items("accessories").iter().find_map(|accessory| {
        let path = document.unit(accessory)?.single("data_path")?;
        let path = value::parse_string(path);
        path.contains(&folder)
            .then(|| (accessory.clone(), path.to_string()))
    })
}

/// Makes one change (see `VehicleEdit`). `false` if there's no such truck or trailer.
pub fn apply_vehicle_edit(document: &mut Document, edit: &VehicleEdit) -> bool {
    match edit {
        VehicleEdit::CargoMass { trailer, kg } => {
            let chain = trailer_chain(document, trailer);
            if chain.is_empty() {
                return false;
            }
            // Spread like the game spreads a load: evenly over the trailers.
            let share = kg / chain.len() as f64;
            for id in &chain {
                set_if_present(document, id, "cargo_mass", Value::single(mass_text(share)));
            }
            true
        }
        VehicleEdit::EmptyMass { trailer, kg } => {
            let defs = chain_defs(document, trailer);
            let total: f64 = defs
                .iter()
                .map(|def| {
                    number(document, def, "chassis_mass") + number(document, def, "body_mass")
                })
                .sum();
            if defs.is_empty() {
                return false;
            }
            // Every part keeps its share of the weight.
            let scale = if total > 0.0 { kg / total } else { 0.0 };
            for def in &defs {
                for field in ["chassis_mass", "body_mass"] {
                    let kg = (number(document, def, field) * scale).round();
                    set_if_present(document, def, field, Value::single(mass_text(kg)));
                }
            }
            true
        }
        VehicleEdit::Unrestrict { trailer } => {
            let defs = chain_defs(document, trailer);
            for def in &defs {
                set_if_present(document, def, "country_validity", Value::Array(Vec::new()));
            }
            !defs.is_empty()
        }
        VehicleEdit::Fuel { vehicle, ratio } => {
            if document.unit(vehicle).is_none() {
                return false;
            }
            set_if_present(
                document,
                vehicle,
                "fuel_relative",
                Value::single(value::format_f32(ratio.max(0.0) as f32)),
            );
            true
        }
        VehicleEdit::Part {
            vehicle,
            part,
            data_path,
            price,
        } => {
            let Some((accessory, _)) = vehicle_part(document, vehicle, part) else {
                return false;
            };
            let clean: String = data_path.chars().filter(|c| *c != '"').collect();
            set_if_present(
                document,
                &accessory,
                "data_path",
                Value::single(format!("\"{clean}\"")),
            );
            set_if_present(
                document,
                &accessory,
                "refund",
                Value::single(price.max(&0).to_string()),
            );
            true
        }
        VehicleEdit::RemoveBadge { vehicle } => {
            let Some(badge) = badge(document, vehicle) else {
                return false;
            };
            if let Some(mut unit) = document.unit_mut(vehicle) {
                let kept: Vec<String> = unit
                    .get("accessories")
                    .map(|value| value.items().to_vec())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|accessory| *accessory != badge)
                    .collect();
                unit.set("accessories", Value::Array(kept));
            }
            // The badge's accessory unit now hangs from nothing.
            crate::drop_unreachable(document);
            true
        }
    }
}

/// A plate's text in two colours (`[red, green, blue]`): the whole plate filled with the
/// background colour and the text centred on it, in the game's markup. Characters the markup uses
/// (`<`, `>`, `|`) are left out of the text.
///
/// Found in game (2026-11-22): this form, written as a Belgian plate, shows coloured front and
/// back, and stays when the player takes the truck. The colour comes before the background image,
/// which takes it, and a negative margin lets the fill cover the plate's edge. The Belgian plate's
/// definition adds no layout of its own (Sweden's, for one, offsets and aligns its text). Whether
/// a plate keeps its own country this way is untried: the first test was on the truck carrying
/// the World of Trucks plate, which the game puts back on load (`world_of_trucks_plate`).
pub fn coloured_plate(text: &str, background: [u8; 3], letters: [u8; 3]) -> String {
    let text: String = text
        .chars()
        .filter(|c| !matches!(c, '<' | '>' | '|'))
        .collect();
    format!(
        "<color value={}><margin left=-15><img src=/material/ui/white.mat xscale=stretch yscale=stretch><ret><align hstyle=center vstyle=center><font xscale=1 yscale=1><color value={}>{text}</font></align></margin>|{COLOURED_PLATE_COUNTRY}",
        game_colour(background),
        game_colour(letters),
    )
}

/// The country a coloured plate is written as (see `coloured_plate`).
pub const COLOURED_PLATE_COUNTRY: &str = "belgium";

/// A colour as the game writes it: alpha, blue, green, red.
fn game_colour([red, green, blue]: [u8; 3]) -> String {
    format!("FF{blue:02X}{green:02X}{red:02X}")
}

/// The colours of a plate `coloured_plate` wrote, as (background, letters) in red, green, blue;
/// `None` for any other plate (the game's own, whatever its markup). `plate` is the field's value,
/// quoted or not, with or without its country. Reads the first form too (the background as the
/// image's `color=`), which earlier versions wrote.
pub fn plate_colours(plate: &str) -> Option<([u8; 3], [u8; 3])> {
    let text = value::parse_string(plate);
    let (background, rest) = if let Some(rest) = text.strip_prefix("<color value=") {
        (rest.get(..8)?, rest.get(8..)?)
    } else {
        let rest = text.strip_prefix("<img src=/material/ui/white.mat color=")?;
        (rest.get(..8)?, rest.get(8..)?)
    };
    if !rest.contains("/material/ui/white.mat") && !text.starts_with("<img") {
        return None;
    }
    let (_, after) = rest.split_once("<color value=")?;
    let letters = after.get(..8)?;
    // The game's order: alpha, blue, green, red.
    let rgb = |hex: &str| -> Option<[u8; 3]> {
        let byte = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
        Some([byte(6)?, byte(4)?, byte(2)?])
    };
    Some((rgb(background)?, rgb(letters)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_back_the_colours_of_a_coloured_plate() {
        let plate = coloured_plate("AB 123", [0, 51, 153], [255, 255, 255]);
        assert_eq!(
            plate_colours(&format!("\"{plate}\"")),
            Some(([0, 51, 153], [255, 255, 255]))
        );
        // The first form, as earlier versions wrote it.
        let old = "<img src=/material/ui/white.mat color=FF993300 xscale=stretch yscale=stretch><ret><align hstyle=center vstyle=center><color value=FFFFFFFF>AB 123</align>|sweden";
        assert_eq!(plate_colours(old), Some(([0, 51, 153], [255, 255, 255])));
        assert_eq!(plate_colours("\"AB 123|sweden\""), None);
    }

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: truck.1\r\n trailers: 1\r\n trailers[0]: trailer.a\r\n}\r\n\r\ntrailer : trailer.a {\r\n trailer_definition: def.a\r\n cargo_mass: 0\r\n slave_trailer: trailer.b\r\n}\r\n\r\ntrailer : trailer.b {\r\n trailer_definition: def.b\r\n cargo_mass: 0\r\n slave_trailer: null\r\n}\r\n\r\ntrailer_def : def.a {\r\n chassis_mass: 6000\r\n body_mass: 2000\r\n country_validity: 2\r\n country_validity[0]: sweden\r\n country_validity[1]: norway\r\n}\r\n\r\ntrailer_def : def.b {\r\n chassis_mass: 3000\r\n body_mass: 1000\r\n country_validity: 1\r\n country_validity[0]: finland\r\n}\r\n\r\nvehicle : truck.1 {\r\n fuel_relative: 1\r\n accessories: 2\r\n accessories[0]: acc.data\r\n accessories[1]: acc.badge\r\n}\r\n\r\nvehicle_accessory : acc.data {\r\n data_path: \"/def/vehicle/truck/volvo.fh16_2012/data.sii\"\r\n}\r\n\r\nvehicle_addon_accessory : acc.badge {\r\n data_path: \"/def/vehicle/truck/volvo.fh16_2012/accessory/badge/gold.sii\"\r\n}\r\n\r\n}\r\n";

    fn document() -> Document {
        sii::parse_text(SAVE).expect("parses")
    }

    #[test]
    fn reads_a_double_as_one_load() {
        let document = document();
        assert_eq!(
            trailer_chain(&document, "trailer.a"),
            ["trailer.a", "trailer.b"]
        );
        let load = trailer_load(&document, "trailer.a").expect("a trailer");
        assert_eq!(load.empty_kg, 12000.0);
        assert_eq!(load.countries, ["sweden", "norway", "finland"]);
    }

    #[test]
    fn spreads_the_cargo_and_scales_the_empty_weight() {
        let mut document = document();
        let cargo = VehicleEdit::CargoMass {
            trailer: "trailer.a".into(),
            kg: 30000.0,
        };
        assert!(apply_vehicle_edit(&mut document, &cargo));
        assert_eq!(
            document
                .unit("trailer.b")
                .and_then(|u| u.single("cargo_mass")),
            Some("15000")
        );
        let empty = VehicleEdit::EmptyMass {
            trailer: "trailer.a".into(),
            kg: 6000.0,
        };
        assert!(apply_vehicle_edit(&mut document, &empty));
        let load = trailer_load(&document, "trailer.a").expect("a trailer");
        assert_eq!((load.cargo_kg, load.empty_kg), (30000.0, 6000.0));
        // Each part keeps its share: half the weight, half of each.
        assert_eq!(
            document
                .unit("def.a")
                .and_then(|u| u.single("chassis_mass")),
            Some("3000")
        );
    }

    #[test]
    fn lifts_country_restrictions_for_the_whole_double() {
        let mut document = document();
        assert!(apply_vehicle_edit(
            &mut document,
            &VehicleEdit::Unrestrict {
                trailer: "trailer.a".into()
            }
        ));
        assert!(
            trailer_load(&document, "trailer.a")
                .expect("a trailer")
                .countries
                .is_empty()
        );
    }

    #[test]
    fn sets_fuel_and_removes_the_badge() {
        let mut document = document();
        assert!(apply_vehicle_edit(
            &mut document,
            &VehicleEdit::Fuel {
                vehicle: "truck.1".into(),
                ratio: 5.0
            }
        ));
        assert_eq!(
            document
                .unit("truck.1")
                .and_then(|u| u.single("fuel_relative")),
            Some("5")
        );
        assert_eq!(badge(&document, "truck.1").as_deref(), Some("acc.badge"));
        assert!(apply_vehicle_edit(
            &mut document,
            &VehicleEdit::RemoveBadge {
                vehicle: "truck.1".into()
            }
        ));
        assert_eq!(badge(&document, "truck.1"), None);
        assert!(
            document.unit("acc.badge").is_none(),
            "its accessory is gone"
        );
        assert!(document.unit("acc.data").is_some());
    }

    #[test]
    fn swaps_an_engine() {
        let with_engine = SAVE.replace(
            "accessories: 2\r\n accessories[0]: acc.data\r\n accessories[1]: acc.badge\r\n",
            "accessories: 3\r\n accessories[0]: acc.data\r\n accessories[1]: acc.badge\r\n accessories[2]: acc.engine\r\n",
        );
        // The engine's unit goes in before the document's last closing brace.
        let end = with_engine.rfind("}\r\n").expect("the document's end");
        let text = format!(
            "{}vehicle_accessory : acc.engine {{\r\n data_path: \"/def/vehicle/truck/volvo.fh16_2012/engine/d13c420.sii\"\r\n refund: 10000\r\n}}\r\n\r\n{}",
            &with_engine[..end],
            &with_engine[end..]
        );
        let mut document = sii::parse_text(&text).expect("parses");
        assert_eq!(
            vehicle_part(&document, "truck.1", "engine").map(|(_, path)| path),
            Some("/def/vehicle/truck/volvo.fh16_2012/engine/d13c420.sii".to_string())
        );
        let edit = VehicleEdit::Part {
            vehicle: "truck.1".into(),
            part: "engine".into(),
            data_path: "/def/vehicle/truck/volvo.fh16_2012/engine/d16g750.sii".into(),
            price: 25560,
        };
        assert!(apply_vehicle_edit(&mut document, &edit));
        let engine = document.unit("acc.engine").expect("the engine");
        assert_eq!(
            engine.single("data_path"),
            Some("\"/def/vehicle/truck/volvo.fh16_2012/engine/d16g750.sii\"")
        );
        assert_eq!(engine.single("refund"), Some("25560"));
        // An engine change doesn't replace a transmission change.
        let gearbox = VehicleEdit::Part {
            vehicle: "truck.1".into(),
            part: "transmission".into(),
            data_path: String::new(),
            price: 0,
        };
        assert!(!edit.same_kind(&gearbox));
    }

    #[test]
    fn colours_a_plate_in_the_games_order() {
        let plate = coloured_plate("AB|1<2>3", [255, 204, 0], [0, 0, 0]);
        // Yellow: red FF, green CC, blue 00, written blue first, before the background image.
        assert!(plate.starts_with("<color value=FF00CCFF><margin left=-15><img "));
        assert!(plate.contains("<color value=FF000000>AB123</font>"));
        assert!(plate.ends_with("|belgium"));
    }
}
