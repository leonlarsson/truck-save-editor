//! Which of the game's prefab models are recruitment agencies' buildings: from its prefab
//! definitions (`prefab_model : prefab.<token>` in `def/world/prefab*.sii`, in the definitions
//! archive and each DLC's), those whose descriptor lies under `/prefab/recruitment_agency/`. With
//! the map's prefabs (`roads::agency_cities`), that says which cities have an agency, in either
//! game, with no list kept in the editor.

use std::collections::HashSet;
use std::path::Path;

use crate::read_document;

/// The tokens of the prefab models that are recruitment agencies.
pub(crate) fn agency_models(install_dir: &Path) -> HashSet<String> {
    let mut models = HashSet::new();
    for path in crate::def_archives(install_dir) {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list("/def/world") else {
            continue;
        };
        let files: Vec<String> = listing
            .files
            .into_iter()
            .filter(|name| name.starts_with("prefab") && name.ends_with(".sii"))
            .collect();
        for file in files {
            let Ok(document) = read_document(&mut archive, &format!("/def/world/{file}")) else {
                continue;
            };
            for unit in document.units_of_kind("prefab_model") {
                let is_agency = unit
                    .single("prefab_desc")
                    .is_some_and(|desc| desc.contains("/recruitment_agency/"));
                if is_agency && let Some(token) = unit.id.strip_prefix("prefab.") {
                    models.insert(token.to_string());
                }
            }
        }
    }
    models
}
