//! The game's own settings file: `config.cfg` in its Documents folder, one `uset name "value"`
//! a line (CRLF). The editor changes a few of them: the developer console (`g_developer`,
//! `g_console`) and how many trucks a convoy takes (`g_max_convoy_size`).
//!
//! The game writes this file when it exits, so it's changed only while the game isn't running,
//! and backed up first like a save.

/// How many trucks a convoy can hold at most, and the game's own default.
pub const MAX_CONVOY_SIZE: u32 = 128;
pub const DEFAULT_CONVOY_SIZE: u32 = 8;

/// A setting's value (`g_developer` → `0`), without its quotes.
pub fn config_value(text: &str, name: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("uset ")?;
        let (key, value) = rest.split_once(' ')?;
        (key == name).then(|| value.trim().trim_matches('"').to_string())
    })
}

/// The file with a setting changed (or added at the end, if it isn't there). Every other line,
/// and the file's line endings, stay as they are.
pub fn with_config_value(text: &str, name: &str, value: &str) -> String {
    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let new_line = format!("uset {name} \"{value}\"");
    let mut found = false;
    let mut lines: Vec<String> = text
        .split(ending)
        .map(|line| {
            let is_it = line
                .trim()
                .strip_prefix("uset ")
                .and_then(|rest| rest.split_once(' '))
                .is_some_and(|(key, _)| key == name);
            if is_it && !found {
                found = true;
                new_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect();
    if !found {
        // Before the final empty line, if the file ends with a line break.
        let at = if lines.last().is_some_and(String::is_empty) {
            lines.len() - 1
        } else {
            lines.len()
        };
        lines.insert(at, new_line);
    }
    lines.join(ending)
}

/// What the editor shows of the game's settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameSettings {
    /// The developer console (`~`) and developer features are on.
    pub developer_console: bool,
    pub convoy_size: u32,
}

pub fn game_settings(text: &str) -> GameSettings {
    let is_on = |name: &str| config_value(text, name).is_some_and(|value| value == "1");
    GameSettings {
        developer_console: is_on("g_developer") && is_on("g_console"),
        convoy_size: config_value(text, "g_max_convoy_size")
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_CONVOY_SIZE),
    }
}

/// The file with the developer console on or off.
pub fn with_developer_console(text: &str, is_on: bool) -> String {
    let value = if is_on { "1" } else { "0" };
    let text = with_config_value(text, "g_developer", value);
    with_config_value(&text, "g_console", value)
}

/// The file with a convoy size.
pub fn with_convoy_size(text: &str, size: u32) -> String {
    with_config_value(text, "g_max_convoy_size", &size.to_string())
}

#[cfg(feature = "native")]
mod disk {
    use std::path::{Path, PathBuf};

    use crate::{Error, Game, Result};

    /// The game's `config.cfg`, if there's one (the game makes it on its first start).
    pub fn game_config_path(game: Game) -> Option<PathBuf> {
        crate::profiles::documents_dirs()
            .into_iter()
            .map(|documents| documents.join(game.documents_folder()).join("config.cfg"))
            .find(|path| path.is_file())
    }

    /// Writes the game's `config.cfg`, after backing up the file as it was (under
    /// `<backups>/config/<time>/config.cfg`). Refuses while the game runs: it writes the file
    /// when it exits.
    pub fn write_game_config(
        game: Game,
        path: &Path,
        text: &str,
        backups_root: &Path,
    ) -> Result<()> {
        if crate::is_game_running(game) {
            return Err(Error::GameRunning);
        }
        let io = |action: &'static str, path: &Path| {
            let path = path.to_path_buf();
            move |source| Error::Io {
                action,
                path,
                source,
            }
        };
        let made_at = crate::time::utc_timestamp(std::time::SystemTime::now());
        let dir = backups_root
            .join("config")
            .join(game.documents_folder())
            .join(made_at);
        std::fs::create_dir_all(&dir).map_err(io("couldn't create the backup folder", &dir))?;
        std::fs::copy(path, dir.join("config.cfg")).map_err(io("couldn't back up", path))?;
        crate::store::write_atomically(path, text.as_bytes())
    }
}

#[cfg(feature = "native")]
pub use disk::{game_config_path, write_game_config};

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "# prism3d variable config data\r\n\r\nuset g_max_convoy_size \"8\"\r\nuset g_developer \"0\"\r\nuset g_console_state \"0\"\r\nuset g_console \"0\"\r\n";

    #[test]
    fn reads_the_settings() {
        assert_eq!(config_value(CONFIG, "g_console"), Some("0".to_string()));
        assert_eq!(config_value(CONFIG, "g_nothing"), None);
        assert_eq!(
            game_settings(CONFIG),
            GameSettings {
                developer_console: false,
                convoy_size: 8
            }
        );
    }

    #[test]
    fn changes_only_the_lines_it_means() {
        let on = with_developer_console(CONFIG, true);
        assert!(on.contains("uset g_developer \"1\"\r\n"));
        assert!(on.contains("uset g_console \"1\"\r\n"));
        // `g_console_state` starts like `g_console` but isn't it.
        assert!(on.contains("uset g_console_state \"0\"\r\n"));
        assert_eq!(on.len(), CONFIG.len());
        let convoy = with_convoy_size(&on, MAX_CONVOY_SIZE);
        assert_eq!(game_settings(&convoy).convoy_size, 128);
        assert!(game_settings(&convoy).developer_console);
    }

    /// Writes a copy in a scratch folder (never the real file), and backs the old one up.
    #[cfg(feature = "native")]
    #[test]
    fn writes_with_a_backup_first() {
        let root = std::env::temp_dir().join(format!("tse-config-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("scratch folder");
        let path = root.join("config.cfg");
        std::fs::write(&path, CONFIG).expect("writes");
        let backups = root.join("backups");
        let changed = with_convoy_size(CONFIG, MAX_CONVOY_SIZE);
        disk::write_game_config(crate::Game::Ets2, &path, &changed, &backups).expect("writes");
        assert_eq!(std::fs::read_to_string(&path).expect("reads"), changed);
        let backed_up = std::fs::read_dir(backups.join("config").join("Euro Truck Simulator 2"))
            .expect("a backup folder")
            .flatten()
            .map(|entry| entry.path().join("config.cfg"))
            .next()
            .expect("one backup");
        assert_eq!(std::fs::read_to_string(backed_up).expect("reads"), CONFIG);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn adds_a_missing_setting_before_the_end() {
        let text = with_config_value("uset a \"1\"\r\n", "b", "2");
        assert_eq!(text, "uset a \"1\"\r\nuset b \"2\"\r\n");
    }
}
