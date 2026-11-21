//! Loading a slot, and writing it back safely.
//!
//! Writing never happens without a backup: before a slot's files are replaced, the current ones
//! are copied to `<backups>/<profile>/<slot>/<time>/`. Backups live outside the game's folders
//! (with Steam Cloud on, anything in the profile folder is uploaded). New files are written next
//! to the old ones and then moved into place, so a crash mid-write can't leave half a save.

use std::path::{Path, PathBuf};

use sii::Document;

use crate::error::{Error, Result, io};
use crate::game::Game;

/// A slot read into memory.
pub struct LoadedSlot {
    pub dir: PathBuf,
    pub game: Document,
    /// `info.sii`, if the slot has a readable one.
    pub info: Option<Document>,
}

pub fn load(slot_dir: &Path) -> Result<LoadedSlot> {
    let game_path = slot_dir.join("game.sii");
    if !game_path.is_file() {
        return Err(Error::NotASlot(slot_dir.to_path_buf()));
    }
    let bytes = std::fs::read(&game_path).map_err(io("couldn't read", &game_path))?;
    let game = sii::read(&bytes)?;
    let info = std::fs::read(slot_dir.join("info.sii"))
        .ok()
        .and_then(|bytes| sii::read(&bytes).ok());
    Ok(LoadedSlot {
        dir: slot_dir.to_path_buf(),
        game,
        info,
    })
}

/// Whether the game is running. (Under WSL this only sees Linux processes, so it can't tell.)
pub fn is_game_running(game: Game) -> bool {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    let Ok(mut system) = System::new() else {
        return false;
    };
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    game.process_names().iter().any(|name| {
        system
            .processes_by_exact_name(std::ffi::OsStr::new(name))
            .next()
            .is_some()
    })
}

/// A backup of a slot, made before writing it.
#[derive(Clone, Debug, PartialEq)]
pub struct Backup {
    pub dir: PathBuf,
    /// When it was made: the folder's name (`2026-11-21 18-30-12`, UTC).
    pub made_at: String,
}

/// Writes a slot's save (and summary) as text SII, after backing up what's there.
///
/// `backups_root` is where backups go (the app's own data folder); `profile` names the
/// profile's folder in it.
pub fn save(game: Game, slot: &LoadedSlot, backups_root: &Path, profile: &str) -> Result<Backup> {
    if is_game_running(game) {
        return Err(Error::GameRunning);
    }
    let backup = back_up(&slot.dir, backups_root, profile)?;
    write_atomically(
        &slot.dir.join("game.sii"),
        sii::to_text(&slot.game).as_bytes(),
    )?;
    if let Some(info) = &slot.info {
        write_atomically(&slot.dir.join("info.sii"), sii::to_text(info).as_bytes())?;
    }
    Ok(backup)
}

/// Copies a slot's files into a new, timestamped backup folder.
pub fn back_up(slot_dir: &Path, backups_root: &Path, profile: &str) -> Result<Backup> {
    let slot_name = slot_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "slot".to_string());
    let made_at = crate::time::utc_timestamp(std::time::SystemTime::now());
    let mut dir = backups_root.join(profile).join(&slot_name).join(&made_at);
    // Two saves in the same second get separate folders.
    let mut attempt = 1;
    while dir.exists() {
        attempt += 1;
        dir = backups_root
            .join(profile)
            .join(&slot_name)
            .join(format!("{made_at} ({attempt})"));
    }
    std::fs::create_dir_all(&dir).map_err(io("couldn't create the backup folder", &dir))?;

    let entries = std::fs::read_dir(slot_dir).map_err(io("couldn't read", slot_dir))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let target = dir.join(entry.file_name());
            std::fs::copy(&path, &target).map_err(io("couldn't back up", &path))?;
        }
    }
    Ok(Backup { dir, made_at })
}

/// A slot's backups, newest first.
pub fn backups(backups_root: &Path, profile: &str, slot_name: &str) -> Vec<Backup> {
    let Ok(entries) = std::fs::read_dir(backups_root.join(profile).join(slot_name)) else {
        return Vec::new();
    };
    let mut found: Vec<Backup> = entries
        .flatten()
        .filter(|entry| entry.path().join("game.sii").is_file())
        .map(|entry| Backup {
            dir: entry.path(),
            made_at: entry.file_name().to_string_lossy().into_owned(),
        })
        .collect();
    found.sort_by(|a, b| b.made_at.cmp(&a.made_at));
    found
}

/// Puts a backup back into its slot. What's in the slot now is backed up first, so restoring
/// can be undone too.
pub fn restore(
    game: Game,
    backup: &Backup,
    slot_dir: &Path,
    backups_root: &Path,
    profile: &str,
) -> Result<Backup> {
    if is_game_running(game) {
        return Err(Error::GameRunning);
    }
    let before = back_up(slot_dir, backups_root, profile)?;
    let entries = std::fs::read_dir(&backup.dir).map_err(io("couldn't read", &backup.dir))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let bytes = std::fs::read(&path).map_err(io("couldn't read", &path))?;
            write_atomically(&slot_dir.join(entry.file_name()), &bytes)?;
        }
    }
    Ok(before)
}

/// Writes a file by writing a temporary one beside it and moving it into place.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tse-writing");
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, bytes).map_err(io("couldn't write", &temporary))?;
    std::fs::rename(&temporary, path).map_err(io("couldn't replace", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::career::Career;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : _nameless.1 {\r\n bank: _nameless.2\r\n player: _nameless.3\r\n experience_points: 10\r\n adr: 0\r\n long_dist: 0\r\n heavy: 0\r\n fragile: 0\r\n urgent: 0\r\n mechanical: 0\r\n}\r\n\r\nbank : _nameless.2 {\r\n money_account: 100\r\n}\r\n\r\nplayer : _nameless.3 {\r\n hq_city: berlin\r\n}\r\n\r\n}\r\n";

    /// A scratch folder for one test, removed when it ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("tse-test-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("creates the scratch folder");
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn saves_with_a_backup_and_restores_it() {
        let scratch = Scratch::new("save");
        let slot_dir = scratch.0.join("profile/save/9");
        let backups_root = scratch.0.join("backups");
        std::fs::create_dir_all(&slot_dir).expect("creates the slot");
        std::fs::write(slot_dir.join("game.sii"), SAVE).expect("writes the save");
        std::fs::write(slot_dir.join("preview.tga"), b"picture").expect("writes the preview");

        let mut slot = load(&slot_dir).expect("loads");
        let mut career = Career::read(&slot.game).expect("reads the career");
        career.money = 5_000;
        career.write(&mut slot.game).expect("changes it");
        let backup = save(Game::Ets2, &slot, &backups_root, "John").expect("saves");

        // The slot has the change; the backup has the original, and every other file too.
        let saved = load(&slot_dir).expect("loads again");
        assert_eq!(Career::read(&saved.game).expect("reads").money, 5_000);
        assert_eq!(
            std::fs::read_to_string(backup.dir.join("game.sii")).expect("backup"),
            SAVE
        );
        assert!(backup.dir.join("preview.tga").is_file());
        assert!(!slot_dir.join("game.sii.tse-writing").exists());
        assert_eq!(backups(&backups_root, "John", "9"), vec![backup.clone()]);

        // Restoring puts the original back, and backs up the changed one first.
        let before_restore =
            restore(Game::Ets2, &backup, &slot_dir, &backups_root, "John").expect("restores");
        let restored = load(&slot_dir).expect("loads the restored save");
        assert_eq!(Career::read(&restored.game).expect("reads").money, 100);
        assert!(before_restore.dir.join("game.sii").is_file());
        assert_eq!(backups(&backups_root, "John", "9").len(), 2);
    }

    #[test]
    fn refuses_a_folder_that_isnt_a_slot() {
        let scratch = Scratch::new("not-a-slot");
        assert!(matches!(load(&scratch.0), Err(Error::NotASlot(_))));
    }
}
