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
#[derive(Clone)]
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
    let written = write_slot(&slot.dir, &slot.game, slot.info.as_ref())
        .and_then(|()| verify(&slot.dir, &slot.game, slot.info.as_ref()));
    if let Err(error) = written {
        // Put the slot back as it was before saving; the backup is still there either way.
        copy_files(&backup.dir, &slot.dir)?;
        return Err(error);
    }
    Ok(backup)
}

fn write_slot(dir: &Path, game: &sii::Document, info: Option<&sii::Document>) -> Result<()> {
    write_atomically(&dir.join("game.sii"), sii::to_text(game).as_bytes())?;
    if let Some(info) = info {
        write_atomically(&dir.join("info.sii"), sii::to_text(info).as_bytes())?;
    }
    Ok(())
}

/// Reads a slot back from disk and checks it's exactly what was meant to be written: a disk that
/// lies, an antivirus that interferes or a bug in writing shows up here, not in the game.
///
/// It compares the two as the text they write to, not as documents: in the text format an empty
/// array (`loans: 0`) reads back as a single value, and only the game's own schema knows which it
/// is, but both write the same text, which is what the game reads.
pub fn verify(dir: &Path, game: &sii::Document, info: Option<&sii::Document>) -> Result<()> {
    let read_back = load(dir).map_err(|error| Error::NotVerified(error.to_string()))?;
    if sii::to_text(&read_back.game) != sii::to_text(game) {
        return Err(Error::NotVerified("game.sii differs".into()));
    }
    if let Some(info) = info {
        let same = read_back
            .info
            .as_ref()
            .is_some_and(|read| sii::to_text(read) == sii::to_text(info));
        if !same {
            return Err(Error::NotVerified("info.sii differs".into()));
        }
    }
    Ok(())
}

/// Writes a slot's save into a new slot next to it (the next free number in the profile's `save`
/// folder), named `name`, leaving the original untouched. The original's other files (the preview
/// pictures) are copied along. Returns the new slot's folder.
pub fn save_as_new_slot(game: Game, slot: &LoadedSlot, name: &str) -> Result<PathBuf> {
    if is_game_running(game) {
        return Err(Error::GameRunning);
    }
    let saves = slot
        .dir
        .parent()
        .ok_or_else(|| Error::NotASlot(slot.dir.clone()))?;
    let next = std::fs::read_dir(saves)
        .map_err(io("couldn't read", saves))?
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let dir = saves.join(next.to_string());
    std::fs::create_dir(&dir).map_err(io("couldn't create the new slot", &dir))?;

    // The pictures and anything else the slot has; the save and its summary are written below.
    let entries = std::fs::read_dir(&slot.dir).map_err(io("couldn't read", &slot.dir))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        if path.is_file() && file_name != "game.sii" && file_name != "info.sii" {
            std::fs::copy(&path, dir.join(&file_name)).map_err(io("couldn't copy", &path))?;
        }
    }

    let info = slot.info.clone().map(|mut info| {
        crate::info::set_name(&mut info, name);
        info
    });
    let written = write_slot(&dir, &slot.game, info.as_ref())
        .and_then(|()| verify(&dir, &slot.game, info.as_ref()));
    if let Err(error) = written {
        // A new slot that didn't come out right isn't left behind for the game to find.
        let _ = std::fs::remove_dir_all(&dir);
        return Err(error);
    }
    Ok(dir)
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
    copy_files(&backup.dir, slot_dir)?;
    Ok(before)
}

/// Copies every file of one folder into another, each replaced atomically.
fn copy_files(from: &Path, to: &Path) -> Result<()> {
    let entries = std::fs::read_dir(from).map_err(io("couldn't read", from))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let bytes = std::fs::read(&path).map_err(io("couldn't read", &path))?;
            write_atomically(&to.join(entry.file_name()), &bytes)?;
        }
    }
    Ok(())
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
    fn verifying_catches_a_save_that_reads_back_different() {
        let scratch = Scratch::new("verify");
        std::fs::write(scratch.0.join("game.sii"), SAVE).expect("writes the save");
        let slot = load(&scratch.0).expect("loads");
        assert!(verify(&scratch.0, &slot.game, None).is_ok());

        let mut other = slot.game.clone();
        let mut career = Career::read(&other).expect("reads the career");
        career.money = 1;
        career.write(&mut other).expect("changes it");
        assert!(matches!(
            verify(&scratch.0, &other, None),
            Err(Error::NotVerified(_))
        ));
    }

    #[test]
    fn saves_as_a_new_slot_leaving_the_original_alone() {
        let scratch = Scratch::new("new-slot");
        let saves = scratch.0.join("profile/save");
        let original = saves.join("3");
        std::fs::create_dir_all(&original).expect("creates the slot");
        std::fs::create_dir_all(saves.join("autosave")).expect("creates another slot");
        std::fs::write(original.join("game.sii"), SAVE).expect("writes the save");
        std::fs::write(
            original.join("info.sii"),
            "SiiNunit\r\n{\r\nsave_container : _nameless.9 {\r\n name: \"Old\"\r\n}\r\n\r\n}\r\n",
        )
        .expect("writes the summary");
        std::fs::write(original.join("preview.tga"), b"picture").expect("writes the preview");

        let mut slot = load(&original).expect("loads");
        let mut career = Career::read(&slot.game).expect("reads");
        career.money = 9_000;
        career.write(&mut slot.game).expect("changes it");
        let dir = save_as_new_slot(Game::Ets2, &slot, "Old (edited)").expect("saves");

        assert_eq!(dir, saves.join("4"));
        assert_eq!(
            std::fs::read_to_string(original.join("game.sii")).expect("original"),
            SAVE
        );
        let copy = load(&dir).expect("loads the copy");
        assert_eq!(Career::read(&copy.game).expect("reads").money, 9_000);
        let info = crate::info::SlotInfo::read(copy.info.as_ref().expect("has a summary"));
        assert_eq!(info.name.as_deref(), Some("Old (edited)"));
        assert!(dir.join("preview.tga").is_file());
    }

    #[test]
    fn refuses_a_folder_that_isnt_a_slot() {
        let scratch = Scratch::new("not-a-slot");
        assert!(matches!(load(&scratch.0), Err(Error::NotASlot(_))));
    }
}
