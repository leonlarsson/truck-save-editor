//! Finding the player's profiles and save slots on disk.
//!
//! Profiles live in two places:
//! - Steam Cloud on: `<Steam>\userdata\<user>\<app id>\remote\profiles\<profile>\`
//! - Steam Cloud off: `Documents\<game>\profiles\<profile>\` (and `steam_profiles`)
//!
//! A profile's folder name is its name in hex (`4A6F686E` = "John"). Each slot is a folder in
//! `save\` holding `game.sii` and `info.sii`.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::game::Game;
use crate::info::SlotInfo;

/// Extra folders to look for profiles in, separated like `PATH` (for development and unusual
/// setups).
pub const PROFILE_DIRS_VARIABLE: &str = "TSE_PROFILE_DIRS";

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub game: Game,
    /// The profile's name, decoded from its folder name.
    pub name: String,
    pub dir: PathBuf,
    pub location: Location,
    /// When a slot was last saved, to put the most recently played profile first.
    pub last_saved: Option<SystemTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    SteamCloud,
    Local,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub dir: PathBuf,
    /// The slot folder's name (`9`, `autosave`, `autosave_job_3` …).
    pub folder: String,
    pub kind: SlotKind,
    pub info: SlotInfo,
    /// When `game.sii` was last written.
    pub modified: Option<SystemTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    /// A save the player made, with its slot number.
    Manual(u32),
    Autosave,
    /// The game's rolling autosaves while driving.
    AutosaveDrive,
    /// The game's autosaves when a job starts or ends.
    AutosaveJob,
    Quicksave,
    /// Made before joining a convoy.
    MultiplayerBackup,
    Other,
}

impl SlotKind {
    fn from_folder(folder: &str) -> Self {
        if let Ok(number) = folder.parse() {
            return SlotKind::Manual(number);
        }
        match folder {
            "autosave" => SlotKind::Autosave,
            "quicksave" => SlotKind::Quicksave,
            "multiplayer_backup" => SlotKind::MultiplayerBackup,
            f if f.starts_with("autosave_drive") => SlotKind::AutosaveDrive,
            f if f.starts_with("autosave_job") => SlotKind::AutosaveJob,
            _ => SlotKind::Other,
        }
    }

    /// Whether the player made this save (rather than the game).
    pub fn is_manual(self) -> bool {
        matches!(self, SlotKind::Manual(_))
    }
}

/// Every profile of the game on this computer, most recently played first.
pub fn find_profiles(game: Game) -> Vec<Profile> {
    let mut profiles = Vec::new();
    for (root, location) in profile_roots(game) {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.join("save").is_dir() {
                continue;
            }
            let folder = entry.file_name().to_string_lossy().into_owned();
            // The name in the profile file (it can be renamed), else the one the folder was named
            // after (the game names a profile's folder by its name in hex).
            let name = crate::load_profile_file(&dir)
                .ok()
                .and_then(|document| crate::profile_name(&document))
                .filter(|name| !name.is_empty())
                .or_else(|| decode_profile_folder(&folder))
                .unwrap_or_else(|| folder.clone());
            let last_saved = slot_dirs(&dir)
                .iter()
                .filter_map(|slot| modified(&slot.join("game.sii")))
                .max();
            profiles.push(Profile {
                game,
                name,
                dir,
                location,
                last_saved,
            });
        }
    }
    // The same profile can be reached two ways (a symlinked Documents, or Steam's folder written
    // two ways: the registry says `c:/program files (x86)/steam`); keep one.
    profiles.sort_by_key(|profile| same_place_key(&profile.dir));
    profiles.dedup_by(|a, b| same_place_key(&a.dir) == same_place_key(&b.dir));
    profiles.sort_by_key(|profile| std::cmp::Reverse(profile.last_saved));
    profiles
}

/// A profile's slots: the player's own saves first (newest first), then the game's.
pub fn profile_slots(profile_dir: &Path) -> Vec<Slot> {
    let mut slots: Vec<Slot> = slot_dirs(profile_dir)
        .into_iter()
        .map(|dir| {
            let folder = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let info = std::fs::read(dir.join("info.sii"))
                .ok()
                .and_then(|bytes| sii::read(&bytes).ok())
                .map(|document| SlotInfo::read(&document))
                .unwrap_or_default();
            Slot {
                kind: SlotKind::from_folder(&folder),
                modified: modified(&dir.join("game.sii")),
                dir,
                folder,
                info,
            }
        })
        .collect();
    slots.sort_by(|a, b| {
        b.kind
            .is_manual()
            .cmp(&a.kind.is_manual())
            .then(b.modified.cmp(&a.modified))
    });
    slots
}

/// `416E6E61` → `Anna`. `None` when the folder name isn't hex text.
pub fn decode_profile_folder(folder: &str) -> Option<String> {
    if folder.is_empty() || !folder.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..folder.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(folder.get(i..i + 2)?, 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

fn slot_dirs(profile_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(profile_dir.join("save")) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|dir| dir.join("game.sii").is_file())
        .collect()
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Every folder that may hold profiles.
fn profile_roots(game: Game) -> Vec<(PathBuf, Location)> {
    let mut roots = Vec::new();

    if let Some(extra) = std::env::var_os(PROFILE_DIRS_VARIABLE) {
        roots.extend(std::env::split_paths(&extra).map(|dir| (dir, Location::Local)));
    }

    for steam in steam_dirs() {
        let Ok(users) = std::fs::read_dir(steam.join("userdata")) else {
            continue;
        };
        for user in users.flatten() {
            let dir = user
                .path()
                .join(game.steam_app_id().to_string())
                .join("remote")
                .join("profiles");
            roots.push((dir, Location::SteamCloud));
        }
    }

    for documents in documents_dirs() {
        let game_dir = documents.join(game.documents_folder());
        roots.push((game_dir.join("profiles"), Location::Local));
        roots.push((game_dir.join("steam_profiles"), Location::Local));
    }
    roots
}

/// The Documents folders the game's own folder may be in.
pub fn documents_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = dirs::document_dir().into_iter().collect();
    // Under WSL (development), the Windows users' Documents folders.
    if cfg!(target_os = "linux")
        && let Ok(users) = std::fs::read_dir("/mnt/c/Users")
    {
        dirs.extend(users.flatten().map(|user| user.path().join("Documents")));
    }
    dirs
}

fn steam_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam")
            && let Ok(path) = key.get_value::<String, _>("SteamPath")
        {
            dirs.push(PathBuf::from(path));
        }
        dirs.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".steam/steam"));
            dirs.push(home.join(".local/share/Steam"));
        }
        // Under WSL (development), Windows' Steam.
        dirs.push(PathBuf::from("/mnt/c/Program Files (x86)/Steam"));
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/Application Support/Steam"));
        }
    }
    dirs.retain(|dir| dir.is_dir());
    dirs.sort_by_key(|dir| same_place_key(dir));
    dirs.dedup_by(|a, b| same_place_key(a) == same_place_key(b));
    dirs
}

/// What tells two paths to the same folder apart from two folders: the resolved path (links,
/// `..` and slashes worked out), and on Windows, whose paths ignore case, in one case.
fn same_place_key(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let key = resolved.to_string_lossy().into_owned();
    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_profile_folders() {
        assert_eq!(decode_profile_folder("416E6E61").as_deref(), Some("Anna"));
        assert_eq!(decode_profile_folder("4A6F686E").as_deref(), Some("John"));
        assert_eq!(
            decode_profile_folder("C3A96D696C65").as_deref(),
            Some("émile")
        );
        assert_eq!(decode_profile_folder("xyz"), None);
        assert_eq!(decode_profile_folder("ABC"), None);
        assert_eq!(decode_profile_folder(""), None);
    }

    #[test]
    fn names_slots() {
        assert_eq!(SlotKind::from_folder("9"), SlotKind::Manual(9));
        assert_eq!(SlotKind::from_folder("autosave"), SlotKind::Autosave);
        assert_eq!(
            SlotKind::from_folder("autosave_job_3"),
            SlotKind::AutosaveJob
        );
        assert_eq!(
            SlotKind::from_folder("autosave_drive"),
            SlotKind::AutosaveDrive
        );
        assert_eq!(
            SlotKind::from_folder("multiplayer_backup"),
            SlotKind::MultiplayerBackup
        );
    }
}
