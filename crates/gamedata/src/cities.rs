//! The installed game's cities, from its city definitions (`def/city.sii` and the `city.*.sii`
//! files each map DLC adds): every city's token, the text key of its proper name, its country,
//! and which have a truck dealer (`vehicle_brands[]`). And the countries (`def/country.sii` and
//! its includes): each one's short code as the game shows it after a city (`Verona (I)`).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct Cities {
    /// Token → the key of its name in the game's text (`berlin` → `berlin`).
    pub names: BTreeMap<String, String>,
    /// Tokens of the cities with a truck dealer.
    pub dealers: BTreeSet<String>,
    /// City token → its country's token (`verona` → `italy`).
    pub countries: BTreeMap<String, String>,
    /// Country token → its short code (`italy` → `I`).
    pub country_codes: BTreeMap<String, String>,
    /// Country token → the key of its name in the game's text (`italy`).
    pub country_names: BTreeMap<String, String>,
}

/// Cities whose definition lists dealer brands but whose map has no dealer: the game doesn't count
/// a dealer there (checked against the 1.62 map's dealer prefabs, and a save's Career Progress:
/// 144 truck dealers, not 146).
const DEALERS_NOT_ON_MAP: [&str; 2] = ["edirne", "veli_tarnovo"];

impl Cities {
    /// Adds the cities a `city.sii` (or a DLC's `city.*.sii`) defines.
    pub(crate) fn add(&mut self, document: &sii::Document) {
        for unit in document.units_of_kind("city_data") {
            let Some(token) = unit.id.strip_prefix("city.") else {
                continue;
            };
            let name_key = unit
                .single("city_name_localized")
                .map(|key| sii::value::parse_string(key).trim_matches('@').to_string())
                .filter(|key| !key.is_empty())
                .unwrap_or_else(|| token.to_string());
            self.names.insert(token.to_string(), name_key);
            if let Some(country) = unit.single("country") {
                let country = sii::value::parse_string(country).to_string();
                if !country.is_empty() {
                    self.countries.insert(token.to_string(), country);
                }
            }
            if !unit.items("vehicle_brands").is_empty() && !DEALERS_NOT_ON_MAP.contains(&token) {
                self.dealers.insert(token.to_string());
            }
        }
    }

    /// Adds the countries a `country.sii` (or a DLC's `country.*.sii`) defines.
    pub(crate) fn add_countries(&mut self, document: &sii::Document) {
        for unit in document.units_of_kind("country_data") {
            let Some(token) = unit.id.strip_prefix("country.data.") else {
                continue;
            };
            let name_key = unit
                .single("name_localized")
                .map(|key| sii::value::parse_string(key).trim_matches('@').to_string())
                .filter(|key| !key.is_empty())
                .unwrap_or_else(|| token.to_string());
            self.country_names.insert(token.to_string(), name_key);
            if let Some(code) = unit.single("country_code") {
                let code = sii::value::parse_string(code).to_string();
                if !code.is_empty() {
                    self.country_codes.insert(token.to_string(), code);
                }
            }
        }
    }
}
