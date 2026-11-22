//! Finding where the game is installed: in one of Steam's library folders, as Steam's own
//! `libraryfolders.vdf` lists them.

use std::path::{Path, PathBuf};

use save::Game;

/// Set to a game folder to use it instead of looking (for development and unusual setups).
pub const GAME_DIR_VARIABLE: &str = "TSE_GAME_DIR";

/// The game's install folder (the one holding `def.scs`), if it can be found.
pub fn find_install(game: Game) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(GAME_DIR_VARIABLE).map(PathBuf::from) {
        return dir.join("def.scs").is_file().then_some(dir);
    }
    let folder = game.name();
    steam_dirs()
        .into_iter()
        .flat_map(|steam| library_folders(&steam))
        .map(|library| library.join("steamapps").join("common").join(folder))
        .find(|dir| dir.join("def.scs").is_file())
}

/// Every Steam library: Steam's own folder, and those listed in `libraryfolders.vdf`.
fn library_folders(steam: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam.to_path_buf()];
    let vdf = std::fs::read_to_string(steam.join("steamapps").join("libraryfolders.vdf"))
        .unwrap_or_default();
    for line in vdf.lines() {
        let mut parts = line.split('"').filter(|part| !part.trim().is_empty());
        if parts.next() == Some("path")
            && let Some(path) = parts.next()
        {
            libraries.push(native_path(&path.replace("\\\\", "\\")));
        }
    }
    libraries.sort();
    libraries.dedup();
    libraries
}

/// A Windows path as this system sees it: unchanged on Windows; under WSL (development),
/// `E:\Games\Steam` becomes `/mnt/e/Games/Steam`.
fn native_path(windows_path: &str) -> PathBuf {
    if cfg!(target_os = "linux") {
        let mut chars = windows_path.chars();
        if let (Some(drive), Some(':')) = (chars.next(), chars.next()) {
            let rest = chars.as_str().replace('\\', "/");
            return PathBuf::from(format!("/mnt/{}{}", drive.to_ascii_lowercase(), rest));
        }
    }
    PathBuf::from(windows_path)
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
        dirs.push(PathBuf::from("/mnt/c/Program Files (x86)/Steam"));
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/Application Support/Steam"));
        }
    }
    dirs.retain(|dir| dir.is_dir());
    // Steam's registry entry is written `c:/program files (x86)/steam`: as Windows writes paths.
    dirs.iter().map(|dir| save::plain_path(dir)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_windows_paths_under_wsl() {
        if cfg!(target_os = "linux") {
            assert_eq!(
                native_path("E:\\Games\\Steam"),
                PathBuf::from("/mnt/e/Games/Steam")
            );
        }
        assert_eq!(native_path("relative"), PathBuf::from("relative"));
    }
}
