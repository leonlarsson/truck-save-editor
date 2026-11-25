//! How the game makes plates: each country (or state) has `license_plate_data` units in
//! `/def/country/<country>/license_plates.sii`, one per kind of vehicle (`car`, `truck`,
//! `trailer` …), with `templates[]` (`"2111000"`) and character lists `def0[]` … `def9[]`. In a
//! template, a digit picks a character from the list of that number; anything else stays as it is.
//! A plate is written in a save as `<text>|<country>`.

use std::collections::HashMap;
use std::path::Path;

use crate::read_document;

/// One kind of vehicle's plate format in a country.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlateFormat {
    /// The kind of vehicle (`truck`, `trailer`, `car` …).
    pub kind: String,
    pub templates: Vec<String>,
    /// `def0` … `def9`: the characters each template digit picks from.
    pub sets: Vec<Vec<String>>,
}

/// Every country's plate formats, by country token, from the base game and the DLCs (the first
/// archive with a country's file wins, as it's read first).
pub(crate) fn read(install_dir: &Path) -> HashMap<String, Vec<PlateFormat>> {
    let mut plates: HashMap<String, Vec<PlateFormat>> = HashMap::new();
    for path in crate::def_archives(install_dir) {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list("/def/country") else {
            continue;
        };
        for country in listing.directories {
            if plates.contains_key(&country) {
                continue;
            }
            let file = format!("/def/country/{country}/license_plates.sii");
            let Ok(document) = read_document(&mut archive, &file) else {
                continue;
            };
            let formats: Vec<PlateFormat> = document
                .units_of_kind("license_plate_data")
                .map(|unit| PlateFormat {
                    kind: unit.single("type").unwrap_or_default().to_string(),
                    templates: strings(unit.items("templates")),
                    sets: (0..10)
                        .map(|digit| strings(unit.items(&format!("def{digit}"))))
                        .collect(),
                })
                .filter(|format| !format.templates.is_empty())
                .collect();
            if !formats.is_empty() {
                plates.insert(country, formats);
            }
        }
    }
    plates
}

fn strings(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| sii::value::parse_string(item).to_string())
        .collect()
}

/// A plate as the game would make one, from a country's formats for a kind of vehicle (the car
/// format when the country has none for that kind), written as a save keeps it
/// (`7TT3983|california`, unquoted). `seed` picks the template and characters: the same seed,
/// the same plate.
pub fn make_plate(formats: &[PlateFormat], country: &str, kind: &str, seed: u64) -> Option<String> {
    let format = formats
        .iter()
        .find(|format| format.kind == kind)
        .or_else(|| formats.iter().find(|format| format.kind == "car"))
        .or_else(|| formats.first())?;
    // A small, plain pseudo-random sequence (an LCG), so a seed always gives the same plate.
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut next = |range: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) % range.max(1) as u64) as usize
    };
    let template = format.templates.get(next(format.templates.len()))?;
    let mut text = String::new();
    // Inside a formatting tag (`<offset hshift=4>`), digits are the tag's own.
    let mut in_tag = false;
    for character in template.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {}
        }
        let set = character
            .to_digit(10)
            .filter(|_| !in_tag)
            .and_then(|digit| format.sets.get(digit as usize))
            .filter(|set| !set.is_empty());
        match set {
            Some(set) => text.push_str(&set[next(set.len())]),
            None => text.push(character),
        }
    }
    Some(format!("{text}|{country}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_plates_from_a_template() {
        let digits: Vec<String> = (0..10).map(|digit| digit.to_string()).collect();
        let letters: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
        let truck = PlateFormat {
            kind: "truck".into(),
            templates: vec!["11 000".into()],
            sets: vec![digits, letters],
        };
        let plate = make_plate(std::slice::from_ref(&truck), "france", "truck", 7).expect("plate");
        let (text, country) = plate.split_once('|').expect("a country");
        assert_eq!(country, "france");
        assert_eq!(text.len(), 6);
        assert!(text[..2].chars().all(|c| "ABC".contains(c)));
        assert_eq!(&text[2..3], " ");
        assert!(text[3..].chars().all(|c| c.is_ascii_digit()));
        // The same seed, the same plate; no trailer format: the truck's isn't used, nor is there a
        // car one, so the first format stands in.
        assert_eq!(
            make_plate(std::slice::from_ref(&truck), "france", "truck", 7),
            Some(plate)
        );
        assert!(make_plate(std::slice::from_ref(&truck), "france", "trailer", 7).is_some());
        assert_eq!(make_plate(&[], "france", "truck", 7), None);
    }
}
