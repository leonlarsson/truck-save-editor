//! The installed game's cities, from its city definitions (`def/city.sii` and the `city.*.sii`
//! files each map DLC adds): every city's token, the text key of its proper name, and which have a
//! truck dealer (`vehicle_brands[]`).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct Cities {
    /// Token → the key of its name in the game's text (`berlin` → `berlin`).
    pub names: BTreeMap<String, String>,
    /// Tokens of the cities with a truck dealer.
    pub dealers: BTreeSet<String>,
}

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
            if !unit.items("vehicle_brands").is_empty() {
                self.dealers.insert(token.to_string());
            }
        }
    }
}
