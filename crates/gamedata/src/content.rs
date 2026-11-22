//! What's installed besides the base game: the DLCs (each `dlc_*.scs` names itself in its
//! `<archive>.manifest.sii`), the mods in the Documents `mod` folder, and the Steam Workshop
//! items downloaded for the game. Compared with what a save needs (`save::requirements`).

use std::collections::BTreeSet;
use std::path::Path;

use save::{Game, Requirement, RequirementKind};

/// What's installed, by the ids saves use.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Installed {
    /// DLC ids (`eut2_iceland`).
    pub dlcs: BTreeSet<String>,
    /// Mod package names: the files and folders in the `mod` folder, without extension.
    pub mods: BTreeSet<String>,
    /// Steam Workshop item numbers.
    pub workshop_items: BTreeSet<u64>,
}

impl Installed {
    /// Whether what a save needs is here. A mod in the `mod` folder or from the Workshop counts.
    pub fn has(&self, requirement: &Requirement) -> bool {
        match requirement.kind {
            RequirementKind::Dlc | RequirementKind::RemovableDlc => {
                self.dlcs.contains(&requirement.id)
            }
            RequirementKind::Mod => match requirement.workshop_id() {
                Some(item) => self.workshop_items.contains(&item),
                None => self.mods.contains(&requirement.id),
            },
        }
    }
}

/// Looks at the install folder, the Documents `mod` folders and the Workshop downloads in the
/// install's own Steam library. Anything unreadable counts as not installed.
pub fn installed(game: Game, install_dir: &Path) -> Installed {
    let mut found = Installed::default();
    for archive in crate::def_archives(install_dir).into_iter().skip(1) {
        if let Some(id) = dlc_id(&archive) {
            found.dlcs.insert(id);
        }
    }
    for documents in save::documents_dirs() {
        let folder = documents.join(game.documents_folder()).join("mod");
        let Ok(entries) = std::fs::read_dir(folder) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            // Packed mods are `.scs` or `.zip`; an unpacked one is a folder.
            if (path.is_dir() || matches!(extension, "scs" | "zip"))
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                found.mods.insert(stem.to_string());
            }
        }
    }
    // `<library>/steamapps/common/<game>` → `<library>/steamapps/workshop/content/<app id>`.
    if let Some(steamapps) = install_dir.parent().and_then(Path::parent) {
        let workshop = steamapps
            .join("workshop")
            .join("content")
            .join(game.steam_app_id().to_string());
        if let Ok(entries) = std::fs::read_dir(workshop) {
            found.workshop_items.extend(
                entries
                    .flatten()
                    .filter_map(|entry| entry.file_name().to_str()?.parse::<u64>().ok()),
            );
        }
    }
    found
}

/// A DLC archive's id, from the manifest inside it (`id: eut2_iceland`).
fn dlc_id(archive: &Path) -> Option<String> {
    let stem = archive.file_stem()?.to_str()?;
    let mut archive = hashfs::Archive::open(archive).ok()?;
    let bytes = archive
        .read(&format!("/{stem}.manifest.sii"))
        .ok()
        .flatten()?;
    manifest_id(&String::from_utf8_lossy(&bytes))
}

/// The `id:` of a `dlc_package` manifest (read by line: the manifest is short and plain).
fn manifest_id(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let value = line.trim().strip_prefix("id:")?.trim();
        (!value.is_empty()).then(|| value.trim_matches('"').to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_manifest_id() {
        let text = "SiiNunit\n{\ndlc_package : _nameless.dlc_x {\n\tid: eut2_north\n\tonline_id: eu_north\n\tdisplay_name: \"DLC - Scandinavia\"\n}\n}\n";
        assert_eq!(manifest_id(text).as_deref(), Some("eut2_north"));
    }

    #[test]
    fn counts_workshop_mods_by_item_number() {
        let mut installed = Installed::default();
        installed.workshop_items.insert(12_345_678);
        installed.mods.insert("my_skin".into());
        let workshop = Requirement {
            kind: RequirementKind::Mod,
            id: "mod_workshop_package.0000000000BC614E".into(),
            name: "Better Sky".into(),
        };
        let local = Requirement {
            kind: RequirementKind::Mod,
            id: "my_skin".into(),
            name: "My skin".into(),
        };
        assert!(installed.has(&workshop) && installed.has(&local));
    }
}
