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
/// whichever truck is the player's each time a save loads, whatever the save says (found in
/// game, 2026-11-22: a coloured plate on the player's truck came back plain with the World of
/// Trucks text and country, while other trucks kept theirs; and a truck made the player's in the
/// editor got the World of Trucks plate on the next load).
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

    /// The profile's own picture, as the game reads it from the profile folder (`avatar.png`).
    pub const AVATAR_FILE: &str = "avatar.png";

    /// What saving does to the profile's picture.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum AvatarWrite<'a> {
        Keep,
        /// A PNG, written as the profile's `avatar.png`.
        Set(&'a [u8]),
        /// No picture of its own: the game shows the profile's face.
        Remove,
    }

    /// The profile's own picture (`avatar.png`), if it has one.
    pub fn read_avatar(profile_dir: &Path) -> Option<Vec<u8>> {
        std::fs::read(profile_dir.join(AVATAR_FILE)).ok()
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
        save_profile(
            game,
            profile_dir,
            Some(document),
            AvatarWrite::Keep,
            backups_root,
            profile,
        )
    }

    /// Writes what changed in a profile: its `profile.sii` and its picture, either or both. As
    /// `save_profile_file`: refused while the game runs, backed up first (the profile folder's
    /// files, the picture too), each written whole and checked, and the backup put back if
    /// anything fails.
    pub fn save_profile(
        game: Game,
        profile_dir: &Path,
        document: Option<&Document>,
        avatar: AvatarWrite<'_>,
        backups_root: &Path,
        profile: &str,
    ) -> Result<Backup> {
        if is_game_running(game) {
            return Err(Error::GameRunning);
        }
        let backup = back_up(profile_dir, backups_root, profile)?;
        let written = write_profile(profile_dir, document, avatar);
        if let Err(error) = written {
            put_back(profile_dir, &backup.dir);
            return Err(error);
        }
        Ok(backup)
    }

    fn write_profile(
        profile_dir: &Path,
        document: Option<&Document>,
        avatar: AvatarWrite<'_>,
    ) -> Result<()> {
        if let Some(document) = document {
            let path = profile_dir.join("profile.sii");
            let text = sii::to_text(document);
            crate::store::write_atomically(&path, text.as_bytes())?;
            let read_back = load_profile_file(profile_dir)
                .map_err(|error| Error::NotVerified(error.to_string()))?;
            if sii::to_text(&read_back) != text {
                return Err(Error::NotVerified("profile.sii differs".into()));
            }
        }
        let path = profile_dir.join(AVATAR_FILE);
        match avatar {
            AvatarWrite::Keep => {}
            AvatarWrite::Set(png) => {
                crate::store::write_atomically(&path, png)?;
                if read_avatar(profile_dir).as_deref() != Some(png) {
                    return Err(Error::NotVerified(format!("{AVATAR_FILE} differs")));
                }
            }
            AvatarWrite::Remove => {
                if path.exists() {
                    std::fs::remove_file(&path).map_err(io("couldn't remove", &path))?;
                }
            }
        }
        Ok(())
    }

    /// After a failed write: the profile file and picture as the backup has them (a picture that
    /// wasn't there before goes again).
    fn put_back(profile_dir: &Path, backup_dir: &Path) {
        if let Ok(bytes) = std::fs::read(backup_dir.join("profile.sii")) {
            let _ = crate::store::write_atomically(&profile_dir.join("profile.sii"), &bytes);
        }
        // Only the picture is ever removed: the profile file is never deleted.
        let avatar = profile_dir.join(AVATAR_FILE);
        match std::fs::read(backup_dir.join(AVATAR_FILE)) {
            Ok(bytes) => {
                let _ = crate::store::write_atomically(&avatar, &bytes);
            }
            Err(_) => {
                let _ = std::fs::remove_file(&avatar);
            }
        }
    }
}

#[cfg(feature = "native")]
pub use disk::{
    AVATAR_FILE, AvatarWrite, load_profile_file, read_avatar, save_profile, save_profile_file,
};

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

    #[cfg(feature = "native")]
    #[test]
    fn writes_checks_and_removes_the_avatar_with_a_backup() {
        use super::{AVATAR_FILE, AvatarWrite, read_avatar, save_profile};
        let root = std::env::temp_dir().join(format!("tse-avatar-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let profile = root.join("profile");
        let backups = root.join("backups");
        std::fs::create_dir_all(&profile).expect("a profile folder");
        std::fs::write(profile.join("profile.sii"), "SiiNunit\r\n{\r\n}\r\n").expect("writes");

        let picture = [0x89, b'P', b'N', b'G', 1, 2, 3];
        let game = crate::Game::Ets2;
        save_profile(
            game,
            &profile,
            None,
            AvatarWrite::Set(&picture),
            &backups,
            "p",
        )
        .expect("saves");
        assert_eq!(read_avatar(&profile).as_deref(), Some(&picture[..]));

        // Removing it backs up the picture first.
        let backup =
            save_profile(game, &profile, None, AvatarWrite::Remove, &backups, "p").expect("saves");
        assert_eq!(read_avatar(&profile), None);
        assert_eq!(
            std::fs::read(backup.dir.join(AVATAR_FILE)).ok().as_deref(),
            Some(&picture[..])
        );
        // The profile file is never touched by the picture.
        assert!(profile.join("profile.sii").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
