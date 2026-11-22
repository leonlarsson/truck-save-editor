//! What a save needs to load: the DLCs and mods the game wrote into `info.sii` when it saved.
//!
//! `info.sii`'s `dependencies[]` are `"<kind>|<id>|<name>"`: `dlc` (the save needs it: maps,
//! trucks), `rdlc` (a removable one, such as paint jobs; the save loads without it) and `mod`
//! (a mod's package name: its file's name in the `mod` folder, or `mod_workshop_package.<hex
//! id>` for one from the Steam Workshop).

use sii::{Document, value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequirementKind {
    /// A DLC the save needs.
    Dlc,
    /// A DLC the save can load without (paint jobs, cabin accessories).
    RemovableDlc,
    Mod,
}

/// One thing the save was made with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub kind: RequirementKind,
    /// The DLC's id (`eut2_iceland`) or the mod's package name.
    pub id: String,
    /// As the game names it (`DLC - Iceland`, a mod's title).
    pub name: String,
}

impl Requirement {
    /// For a mod from the Steam Workshop, its item number there (the folder Steam downloads it
    /// to is named by it).
    pub fn workshop_id(&self) -> Option<u64> {
        let hex = self.id.strip_prefix("mod_workshop_package.")?;
        u64::from_str_radix(hex, 16).ok()
    }
}

/// Everything `info.sii` says the save was made with, in its order.
pub fn requirements(info: &Document) -> Vec<Requirement> {
    let Some(unit) = info.units().first() else {
        return Vec::new();
    };
    unit.items("dependencies")
        .iter()
        .filter_map(|item| {
            let text = value::parse_string(item);
            let mut parts = text.splitn(3, '|');
            let kind = match parts.next()? {
                "dlc" => RequirementKind::Dlc,
                "rdlc" => RequirementKind::RemovableDlc,
                "mod" => RequirementKind::Mod,
                _ => return None,
            };
            let id = parts.next()?.to_string();
            let name = parts.next().unwrap_or(&id).to_string();
            Some(Requirement { kind, id, name })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_dlcs_and_mods() {
        let info = sii::parse_text(
            "SiiNunit\r\n{\r\nsave_container : _nameless.1 {\r\n dependencies: 3\r\n dependencies[0]: \"dlc|eut2_north|DLC - Scandinavia\"\r\n dependencies[1]: \"rdlc|eut2_flags_s|DLC - Swedish Paint Jobs\"\r\n dependencies[2]: \"mod|mod_workshop_package.00000000BC614E|Better Sky\"\r\n}\r\n\r\n}\r\n",
        )
        .expect("parses");
        let all = requirements(&info);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].kind, RequirementKind::Dlc);
        assert_eq!(all[1].kind, RequirementKind::RemovableDlc);
        assert_eq!(all[2].name, "Better Sky");
        assert_eq!(all[2].workshop_id(), Some(12_345_678));
    }
}
