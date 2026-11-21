//! The installed game's cities, from its city definitions (`def/city.sii` and the `city.*.sii`
//! files each map DLC adds): every city's token, the text key of its proper name, and which have a
//! truck dealer (`vehicle_brands[]`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::read_document;

#[derive(Clone, Debug, Default)]
pub(crate) struct Cities {
    /// Token → the key of its name in the game's text (`berlin` → `berlin`).
    pub names: BTreeMap<String, String>,
    /// Tokens of the cities with a truck dealer.
    pub dealers: BTreeSet<String>,
}

/// Reads the city definitions from `def.scs` and every DLC archive in the install folder. An
/// archive or file that can't be read is skipped: the rest still count.
pub(crate) fn read_cities(install_dir: &Path) -> Cities {
    let mut cities = Cities::default();
    let mut archives = vec![install_dir.join("def.scs")];
    if let Ok(entries) = std::fs::read_dir(install_dir) {
        let mut dlcs: Vec<_> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("dlc_") && name.ends_with(".scs")
            })
            .collect();
        dlcs.sort();
        archives.extend(dlcs);
    }
    for path in archives {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list("/def") else {
            continue;
        };
        let files: Vec<String> = listing
            .files
            .into_iter()
            .filter(|name| {
                name == "city.sii" || (name.starts_with("city.") && name.ends_with(".sii"))
            })
            .collect();
        for file in files {
            let Ok(document) = read_document(&mut archive, &format!("/def/{file}")) else {
                continue;
            };
            for unit in document.units_of_kind("city_data") {
                let Some(token) = unit.id.strip_prefix("city.") else {
                    continue;
                };
                let name_key = unit
                    .single("city_name_localized")
                    .map(|key| sii::value::parse_string(key).trim_matches('@').to_string())
                    .filter(|key| !key.is_empty())
                    .unwrap_or_else(|| token.to_string());
                cities.names.insert(token.to_string(), name_key);
                if !unit.items("vehicle_brands").is_empty() {
                    cities.dealers.insert(token.to_string());
                }
            }
        }
    }
    cities
}
