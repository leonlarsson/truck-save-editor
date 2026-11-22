//! Whole-profile tools: a backup of a profile's folder, a copy of a profile under a new name, and
//! copying a profile's own settings (`config.cfg`: graphics, controls, gameplay options) to
//! another profile.
//!
//! A copy is always made as a local profile (`Documents\<game>\profiles\<name in hex>\`): the game
//! lists the profiles it finds there, while Steam Cloud only shows files the game itself made. The
//! copy's World of Trucks login is left empty, so two profiles never share one; the player can link
//! the copy in the game. Nothing here reads the login.

use std::path::{Path, PathBuf};

use crate::{Error, Game, Result};

fn io(action: &'static str, path: &Path) -> impl FnOnce(std::io::Error) -> Error {
    let path = path.to_path_buf();
    move |source| Error::Io {
        action,
        path,
        source,
    }
}

/// The game's folder name for a profile: its name's UTF-8 bytes in hex (`Anna` → `416E6E61`).
pub fn profile_folder_name(name: &str) -> String {
    name.bytes().map(|byte| format!("{byte:02X}")).collect()
}

/// Copies a folder and everything in it.
fn copy_dir_all(from: &Path, to: &Path) -> Result<u64> {
    std::fs::create_dir_all(to).map_err(io("couldn't create", to))?;
    let mut bytes = 0;
    for entry in std::fs::read_dir(from)
        .map_err(io("couldn't read", from))?
        .flatten()
    {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            bytes += copy_dir_all(&path, &target)?;
        } else if path.is_file() {
            bytes += std::fs::copy(&path, &target).map_err(io("couldn't copy", &path))?;
        }
    }
    Ok(bytes)
}

/// A backup of a whole profile, made: where it is and how big.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileBackup {
    pub dir: PathBuf,
    pub bytes: u64,
}

/// Copies a whole profile (every save, the album, its settings) into the backups folder, under
/// `profiles/<profile>/<time>/`. Reads the profile only.
pub fn back_up_profile(
    profile_dir: &Path,
    backups_root: &Path,
    profile_name: &str,
) -> Result<ProfileBackup> {
    let made_at = crate::time::utc_timestamp(std::time::SystemTime::now());
    let folder: String = profile_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let dir = backups_root
        .join("profiles")
        .join(folder.trim())
        .join(made_at);
    let bytes = copy_dir_all(profile_dir, &dir)?;
    Ok(ProfileBackup { dir, bytes })
}

/// Why a profile can't be copied under a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyProblem {
    EmptyName,
    /// A profile with that name (folder) is there already.
    Taken,
    /// The game's Documents folder wasn't found.
    NoDocuments,
}

/// Where a copy named `name` would go, or why it can't.
pub fn copy_destination(game: Game, name: &str) -> std::result::Result<PathBuf, CopyProblem> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CopyProblem::EmptyName);
    }
    let documents = crate::profiles::documents_dirs()
        .into_iter()
        .map(|documents| documents.join(game.documents_folder()))
        .find(|dir| dir.is_dir())
        .ok_or(CopyProblem::NoDocuments)?;
    let dir = documents.join("profiles").join(profile_folder_name(name));
    if dir.exists() {
        return Err(CopyProblem::Taken);
    }
    Ok(dir)
}

/// Copies a profile to `destination` (from `copy_destination`) as a local profile named `name`,
/// with an empty World of Trucks login. Refuses while the game runs.
pub fn copy_profile(game: Game, profile_dir: &Path, destination: &Path, name: &str) -> Result<()> {
    if crate::is_game_running(game) {
        return Err(Error::GameRunning);
    }
    copy_dir_all(profile_dir, destination)?;
    let renamed = (|| {
        let mut profile = crate::load_profile_file(destination)?;
        crate::set_profile_name(&mut profile, name.trim());
        clear_login(&mut profile);
        let text = sii::to_text(&profile);
        crate::store::write_atomically(&destination.join("profile.sii"), text.as_bytes())?;
        // The game's own spare copy would bring the old name and login back.
        let _ = std::fs::remove_file(destination.join("profile.bak.sii"));
        Ok(())
    })();
    if renamed.is_err() {
        // Half a copy isn't left for the game to find.
        let _ = std::fs::remove_dir_all(destination);
    }
    renamed
}

/// Empties the World of Trucks login in a profile file (without reading it).
fn clear_login(profile: &mut sii::Document) {
    let Some(id) = profile
        .first_of_kind("user_profile")
        .map(|unit| unit.id.clone())
    else {
        return;
    };
    if let Some(mut unit) = profile.unit_mut(&id) {
        for field in ["online_user_name", "online_password"] {
            if unit.get(field).is_some() {
                unit.set(field, sii::Value::single("\"\""));
            }
        }
    }
}

/// Copies one profile's settings (`config.cfg`) over another's, after backing the other's up
/// (`<backups>/profiles/<to>/settings/<time>/config.cfg`). Only to a profile that has settings of
/// its own (in Steam Cloud, a file Steam knows). Refuses while the game runs.
pub fn copy_profile_settings(
    game: Game,
    from_dir: &Path,
    to_dir: &Path,
    to_name: &str,
    backups_root: &Path,
) -> Result<()> {
    if crate::is_game_running(game) {
        return Err(Error::GameRunning);
    }
    let from = from_dir.join("config.cfg");
    let to = to_dir.join("config.cfg");
    let text = std::fs::read(&from).map_err(io("couldn't read", &from))?;
    let made_at = crate::time::utc_timestamp(std::time::SystemTime::now());
    let backup = backups_root
        .join("profiles")
        .join(to_name)
        .join("settings")
        .join(made_at);
    std::fs::create_dir_all(&backup).map_err(io("couldn't create the backup folder", &backup))?;
    std::fs::copy(&to, backup.join("config.cfg")).map_err(io("couldn't back up", &to))?;
    crate::store::write_atomically(&to, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_folders_like_the_game() {
        assert_eq!(profile_folder_name("Anna"), "416E6E61");
        assert_eq!(
            crate::decode_profile_folder(&profile_folder_name("Åsa 2")).as_deref(),
            Some("Åsa 2")
        );
    }

    /// In a scratch folder: a backup copies everything; a copy gets its new name and no login.
    #[test]
    fn backs_up_and_copies_a_profile() {
        let root = std::env::temp_dir().join(format!("tse-profile-tools-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let profile = root.join("4E6F7274");
        std::fs::create_dir_all(profile.join("save").join("1")).expect("folders");
        std::fs::write(
            profile.join("profile.sii"),
            "SiiNunit\r\n{\r\nuser_profile : _nameless.1 {\r\n profile_name: \"North\"\r\n company_name: \"Northwind\"\r\n online_user_name: \"placeholder\"\r\n online_password: \"placeholder\"\r\n}\r\n\r\n}\r\n",
        )
        .expect("writes");
        std::fs::write(profile.join("profile.bak.sii"), "old").expect("writes");
        std::fs::write(profile.join("config.cfg"), "uset a \"1\"\r\n").expect("writes");
        std::fs::write(profile.join("save").join("1").join("game.sii"), "SiiN").expect("writes");

        let backup = back_up_profile(&profile, &root.join("backups"), "North").expect("backs up");
        assert!(backup.dir.join("save").join("1").join("game.sii").is_file());
        assert!(backup.bytes > 0);

        let destination = root.join("copy");
        copy_profile(Game::Ets2, &profile, &destination, "South").expect("copies");
        let copied = crate::load_profile_file(&destination).expect("reads");
        assert_eq!(crate::profile_name(&copied).as_deref(), Some("South"));
        assert_eq!(crate::company_name(&copied).as_deref(), Some("Northwind"));
        let unit = copied.first_of_kind("user_profile").expect("profile");
        assert_eq!(unit.single("online_user_name"), Some("\"\""));
        assert!(!destination.join("profile.bak.sii").exists());
        assert!(
            destination
                .join("save")
                .join("1")
                .join("game.sii")
                .is_file()
        );

        // Settings over another profile, with the other's backed up.
        std::fs::write(destination.join("config.cfg"), "uset a \"2\"\r\n").expect("writes");
        copy_profile_settings(
            Game::Ets2,
            &profile,
            &destination,
            "South",
            &root.join("backups"),
        )
        .expect("copies settings");
        assert_eq!(
            std::fs::read_to_string(destination.join("config.cfg")).expect("reads"),
            "uset a \"1\"\r\n"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
