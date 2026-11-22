//! The profile file (`profile.sii`, one `user_profile` unit): the editor reads and changes the
//! company name and the profile's name (`profile_name`, what the game's profile list shows), and
//! keeps `cached_experience` (what the profile list shows) in step with the
//! career.
//!
//! The same unit holds the World of Trucks login (`online_user_name`, an encrypted
//! `online_password`). Nothing here reads, shows or changes those: the document passes through
//! untouched, and only the fields named below are ever looked at.

use sii::{Document, Value, value};

/// The fields the editor may read or write. Everything else is left exactly as it is.
const COMPANY_NAME: &str = "company_name";
const PROFILE_NAME: &str = "profile_name";
const CACHED_EXPERIENCE: &str = "cached_experience";
/// Odds and ends the game keeps for the profile; `user_data[1]` is the plate set on World of
/// Trucks (`TEXT|_sweden`), which the game puts back on the player's truck each time a save loads.
const USER_DATA: &str = "user_data";

fn user_profile_id(document: &Document) -> Option<String> {
    document
        .first_of_kind("user_profile")
        .map(|unit| unit.id.clone())
}

/// The plate set on World of Trucks, by its text (`None` when there's none). The game puts it on
/// the truck that carries it each time a save loads, whatever the save says (found in game,
/// 2026-11-22: a coloured plate on that truck came back plain, with the World of Trucks text and
/// country, while other trucks kept theirs).
pub fn world_of_trucks_plate(document: &Document) -> Option<String> {
    let unit = document.first_of_kind("user_profile")?;
    let entry = unit.items(USER_DATA).get(1)?.clone();
    let entry = value::parse_string(&entry).into_owned();
    let (text, _country) = entry.split_once('|')?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The company's name, as the player typed it in the game.
pub fn company_name(document: &Document) -> Option<String> {
    let unit = document.first_of_kind("user_profile")?;
    Some(value::parse_string(unit.single(COMPANY_NAME)?).to_string())
}

/// The profile's name, as the game's profile list shows it.
pub fn profile_name(document: &Document) -> Option<String> {
    let unit = document.first_of_kind("user_profile")?;
    Some(value::parse_string(unit.single(PROFILE_NAME)?).to_string())
}

/// Changes the profile's name (see `set_company_name` for what's left out). The profile's folder
/// keeps its name: Steam Cloud only knows the files under it.
pub fn set_profile_name(document: &mut Document, name: &str) -> bool {
    set_text(document, PROFILE_NAME, name)
}

/// Changes the company's name. Quotes and backslashes are left out: the game's own name field
/// doesn't take them either, and they'd need escaping the game may not undo.
pub fn set_company_name(document: &mut Document, name: &str) -> bool {
    set_text(document, COMPANY_NAME, name)
}

fn set_text(document: &mut Document, field: &str, name: &str) -> bool {
    let clean: String = name
        .chars()
        .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
        .collect();
    let Some(id) = user_profile_id(document) else {
        return false;
    };
    let Some(mut unit) = document.unit_mut(&id) else {
        return false;
    };
    if unit.get(field).is_none() {
        return false;
    }
    unit.set(field, Value::single(format!("\"{}\"", clean.trim())));
    true
}

/// Keeps the experience the profile list shows in step with the career's.
pub fn set_cached_experience(document: &mut Document, experience: i64) {
    let Some(id) = user_profile_id(document) else {
        return;
    };
    if let Some(mut unit) = document.unit_mut(&id)
        && unit.get(CACHED_EXPERIENCE).is_some()
    {
        unit.set(
            CACHED_EXPERIENCE,
            Value::single(experience.max(0).to_string()),
        );
    }
}

#[cfg(feature = "native")]
mod disk {
    use std::path::Path;

    use super::Document;
    use crate::error::{Error, Result, io};
    use crate::game::Game;
    use crate::store::{Backup, back_up, is_game_running};

    /// Reads a profile's `profile.sii`.
    pub fn load_profile_file(profile_dir: &Path) -> Result<Document> {
        let path = profile_dir.join("profile.sii");
        let bytes = std::fs::read(&path).map_err(io("couldn't read", &path))?;
        Ok(sii::read(&bytes)?)
    }

    /// Writes a profile's `profile.sii`: refused while the game runs, backed up first (with the
    /// profile folder's other files, not its saves), written whole or not at all, and read back
    /// and checked. If the check fails, the backup is put back.
    pub fn save_profile_file(
        game: Game,
        profile_dir: &Path,
        document: &Document,
        backups_root: &Path,
        profile: &str,
    ) -> Result<Backup> {
        if is_game_running(game) {
            return Err(Error::GameRunning);
        }
        let backup = back_up(profile_dir, backups_root, profile)?;
        let path = profile_dir.join("profile.sii");
        let text = sii::to_text(document);
        let written = crate::store::write_atomically(&path, text.as_bytes()).and_then(|()| {
            let read_back = load_profile_file(profile_dir)
                .map_err(|error| Error::NotVerified(error.to_string()))?;
            if sii::to_text(&read_back) == text {
                Ok(())
            } else {
                Err(Error::NotVerified("profile.sii differs".into()))
            }
        });
        if let Err(error) = written {
            let original = backup.dir.join("profile.sii");
            if let Ok(bytes) = std::fs::read(&original) {
                let _ = crate::store::write_atomically(&path, &bytes);
            }
            return Err(error);
        }
        Ok(backup)
    }
}

#[cfg(feature = "native")]
pub use disk::{load_profile_file, save_profile_file};

#[cfg(test)]
mod tests {

    #[test]
    fn reads_the_world_of_trucks_plate() {
        let with = "SiiNunit\r\n{\r\nuser_profile : p {\r\n user_data: 2\r\n user_data[0]: \"\"\r\n user_data[1]: \"AB 12|_france\"\r\n}\r\n\r\n}\r\n";
        let document = sii::parse_text(with).expect("parses");
        assert_eq!(world_of_trucks_plate(&document).as_deref(), Some("AB 12"));
        let without = with.replace("AB 12|_france", "");
        let document = sii::parse_text(&without).expect("parses");
        assert_eq!(world_of_trucks_plate(&document), None);
    }

    use super::*;

    /// A made-up profile; the login fields hold invented placeholders.
    const PROFILE: &str = "SiiNunit\r\n{\r\nuser_profile : _nameless.1 {\r\n face: 0\r\n company_name: \"Northwind Haulage\"\r\n cached_experience: 100\r\n online_user_name: \"someone\"\r\n online_password: \"not-a-real-secret\"\r\n profile_name: \"Tester\"\r\n}\r\n\r\n}\r\n";

    #[test]
    fn changes_only_the_named_fields() {
        let mut document = sii::parse_text(PROFILE).expect("parses");
        assert_eq!(
            company_name(&document).as_deref(),
            Some("Northwind Haulage")
        );
        assert!(set_company_name(&mut document, " Fjord \"Freight\" "));
        set_cached_experience(&mut document, 5000);
        assert_eq!(company_name(&document).as_deref(), Some("Fjord Freight"));

        // Everything else, the login above all, is exactly as it was.
        let before = sii::parse_text(PROFILE).expect("parses");
        let unit = document.first_of_kind("user_profile").expect("unit");
        let original = before.first_of_kind("user_profile").expect("unit");
        for field in &unit.fields {
            if field.name != COMPANY_NAME && field.name != CACHED_EXPERIENCE {
                assert_eq!(
                    Some(&field.value),
                    original.get(&field.name),
                    "{}",
                    field.name
                );
            }
        }
        assert_eq!(unit.fields.len(), original.fields.len());
    }
}
