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

/// Set (to anything) to look only in `TSE_PROFILE_DIRS` and added folders, for seeing the start
/// screen as on a PC without the game.
pub const PROFILE_DIRS_ONLY_VARIABLE: &str = "TSE_PROFILE_DIRS_ONLY";

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

/// A folder looked in for profiles, and what was there.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchedFolder {
    pub dir: PathBuf,
    pub location: Location,
    pub source: Source,
    /// Whether the folder is there at all.
    pub is_there: bool,
    /// How many profiles were in it.
    pub profiles: usize,
}

/// Why a folder is looked in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Steam's own folder for the game's cloud saves (one for each Steam account on this PC).
    Steam,
    /// The game's folder in Documents.
    Documents,
    /// A folder the player added.
    Added,
}

/// What looking for profiles found, and where it looked.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Search {
    /// Most recently played first.
    pub profiles: Vec<Profile>,
    pub folders: Vec<SearchedFolder>,
    /// Where Steam is (empty when it isn't on this PC).
    pub steam_dirs: Vec<PathBuf>,
}

/// Every profile of the game on this computer, most recently played first.
pub fn find_profiles(game: Game) -> Vec<Profile> {
    search(game, &[]).profiles
}

/// Looks for the game's profiles in the usual places and in `added` (folders the player chose,
/// as `profile_folders_at` gives them), saying what each place held.
pub fn search(game: Game, added: &[PathBuf]) -> Search {
    let steam = steam_dirs();
    let mut profiles = Vec::new();
    let mut folders = Vec::new();
    for (root, location, source) in profile_roots(game, &steam, added) {
        let found = profiles_in(game, &root, location);
        folders.push(SearchedFolder {
            is_there: root.is_dir(),
            profiles: found.len(),
            dir: root,
            location,
            source,
        });
        profiles.extend(found);
    }
    // The same profile can be reached two ways (a symlinked Documents, or Steam's folder written
    // two ways: the registry says `c:/program files (x86)/steam`); keep one.
    profiles.sort_by_key(|profile| same_place_key(&profile.dir));
    profiles.dedup_by(|a, b| same_place_key(&a.dir) == same_place_key(&b.dir));
    profiles.sort_by_key(|profile| std::cmp::Reverse(profile.last_saved));
    // The same goes for places (a folder added that's looked in anyway): the first one says it.
    let mut seen = std::collections::HashSet::new();
    folders.retain(|folder| seen.insert(same_place_key(&folder.dir)));
    Search {
        profiles,
        folders,
        steam_dirs: steam,
    }
}

/// The profiles in one folder of profiles. A profile is a folder with a `save` folder or a
/// `profile.sii` (a new one may have no saves yet).
fn profiles_in(game: Game, root: &Path, location: Location) -> Vec<Profile> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !is_profile(&dir) {
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
    profiles
}

fn is_profile(dir: &Path) -> bool {
    dir.join("save").is_dir() || dir.join("profile.sii").is_file()
}

/// The folders of profiles a chosen folder means: the folder itself when it holds profiles, the
/// one around it when it's a profile or one save in a profile, or the `profiles` folders in it
/// when it's the game's own folder in Documents. Empty when there are no profiles there.
pub fn profile_folders_at(chosen: &Path) -> Vec<PathBuf> {
    // One save (`…\profiles\<profile>\save\<slot>`): three folders up.
    if chosen.join("game.sii").is_file()
        && let Some(root) = chosen
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
    {
        return vec![root.to_path_buf()];
    }
    // A profile's `save` folder.
    if chosen.file_name().is_some_and(|name| name == "save")
        && let Some(profile) = chosen.parent()
        && is_profile(profile)
        && let Some(root) = profile.parent()
    {
        return vec![root.to_path_buf()];
    }
    // A profile.
    if is_profile(chosen)
        && let Some(root) = chosen.parent()
    {
        return vec![root.to_path_buf()];
    }
    // A folder of profiles.
    let holds_profiles = std::fs::read_dir(chosen)
        .is_ok_and(|entries| entries.flatten().any(|entry| is_profile(&entry.path())));
    if holds_profiles {
        return vec![chosen.to_path_buf()];
    }
    // The game's folder in Documents (or Steam's `remote` folder), which has `profiles` in it.
    ["profiles", "steam_profiles"]
        .iter()
        .map(|name| chosen.join(name))
        .filter(|dir| {
            std::fs::read_dir(dir)
                .is_ok_and(|entries| entries.flatten().any(|entry| is_profile(&entry.path())))
        })
        .collect()
}

/// Which game a folder of profiles is for, by its path: under `American Truck Simulator` (in
/// Documents) or ATS's Steam app id, ATS; otherwise ETS2.
fn game_of_folder(dir: &Path) -> Game {
    let ats_id = Game::Ats.steam_app_id().to_string();
    let is_ats = dir.components().any(|part| {
        let part = part.as_os_str().to_string_lossy();
        part.eq_ignore_ascii_case(Game::Ats.documents_folder()) || part == ats_id.as_str()
    });
    if is_ats { Game::Ats } else { Game::Ets2 }
}

/// Whether a folder of profiles is Steam Cloud's (`…\remote\profiles`): there, the game only
/// lists saves Steam knows about.
fn location_of(root: &Path) -> Location {
    let parent = root
        .parent()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().to_lowercase());
    if parent.as_deref() == Some("remote") {
        Location::SteamCloud
    } else {
        Location::Local
    }
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

/// Every folder that may hold profiles, and why it's looked in.
fn profile_roots(
    game: Game,
    steam: &[PathBuf],
    added: &[PathBuf],
) -> Vec<(PathBuf, Location, Source)> {
    let mut roots = Vec::new();

    // For development: more folders, and with `TSE_PROFILE_DIRS_ONLY` set, only those (to see
    // the start screen as on a PC without the game).
    let only_variable = std::env::var_os(PROFILE_DIRS_ONLY_VARIABLE).is_some();
    let variable: Vec<PathBuf> = std::env::var_os(PROFILE_DIRS_VARIABLE)
        .map(|extra| std::env::split_paths(&extra).collect())
        .unwrap_or_default();
    // Each added folder is looked in for one game: the other's when its path says so (each
    // game's folders are named after it), ETS2's otherwise.
    roots.extend(
        variable
            .iter()
            .chain(added)
            .filter(|dir| game_of_folder(dir) == game)
            .map(|dir| (dir.clone(), location_of(dir), Source::Added)),
    );
    if only_variable {
        return roots;
    }

    for steam in steam {
        let Ok(users) = std::fs::read_dir(steam.join("userdata")) else {
            continue;
        };
        for user in users.flatten() {
            let dir = user
                .path()
                .join(game.steam_app_id().to_string())
                .join("remote")
                .join("profiles");
            roots.push((dir, Location::SteamCloud, Source::Steam));
        }
    }

    for documents in documents_dirs() {
        let game_dir = documents.join(game.documents_folder());
        roots.push((
            game_dir.join("profiles"),
            Location::Local,
            Source::Documents,
        ));
        // Only there for some players (profiles made while signed in to Steam, in older
        // versions); not worth a line saying it isn't.
        let steam_profiles = game_dir.join("steam_profiles");
        if steam_profiles.is_dir() {
            roots.push((steam_profiles, Location::Local, Source::Documents));
        }
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
        // Windows' own accounts (and stray files like `desktop.ini`) have no games in them.
        let builtin = ["All Users", "Default", "Default User", "Public"];
        dirs.extend(
            users
                .flatten()
                .filter(|user| user.path().is_dir())
                .filter(|user| !builtin.iter().any(|name| user.file_name() == *name))
                .map(|user| user.path().join("Documents")),
        );
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

    /// A scratch folder for one test, removed when it ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("tse-profiles-{name}-{}", std::process::id()));
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
    fn works_out_the_profiles_folder_from_what_was_chosen() {
        let scratch = Scratch::new("chosen");
        let game_dir = scratch.0.join("Euro Truck Simulator 2");
        let root = game_dir.join("profiles");
        let profile = root.join("416E6E61");
        let slot = profile.join("save").join("3");
        std::fs::create_dir_all(&slot).expect("makes the slot");
        std::fs::write(slot.join("game.sii"), "").expect("writes a save");
        // A new profile with no saves yet.
        let fresh = root.join("4A6F686E");
        std::fs::create_dir_all(&fresh).expect("makes the profile");
        std::fs::write(fresh.join("profile.sii"), "").expect("writes a profile");

        for chosen in [
            &slot,
            &profile.join("save"),
            &profile,
            &fresh,
            &root,
            &game_dir,
        ] {
            assert_eq!(
                profile_folders_at(chosen),
                std::slice::from_ref(&root),
                "{chosen:?}"
            );
        }
        assert!(profile_folders_at(&scratch.0).is_empty());
        assert!(profile_folders_at(&scratch.0.join("not there")).is_empty());

        let mut found = profiles_in(Game::Ets2, &root, Location::Local);
        found.sort_by(|a, b| a.name.cmp(&b.name));
        let names: Vec<&str> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Anna", "John"]);
        assert!(profile_slots(&fresh).is_empty());
    }

    #[test]
    fn tells_which_game_a_folder_is_for() {
        assert_eq!(
            game_of_folder(Path::new("/docs/American Truck Simulator/profiles")),
            Game::Ats
        );
        assert_eq!(
            game_of_folder(Path::new("/steam/userdata/1/270880/remote/profiles")),
            Game::Ats
        );
        assert_eq!(game_of_folder(Path::new("/backup/profiles")), Game::Ets2);
    }

    #[test]
    fn tells_steam_cloud_folders_apart() {
        assert_eq!(
            location_of(Path::new("/steam/userdata/1/227300/remote/profiles")),
            Location::SteamCloud
        );
        assert_eq!(
            location_of(Path::new("/docs/Euro Truck Simulator 2/profiles")),
            Location::Local
        );
    }
}
