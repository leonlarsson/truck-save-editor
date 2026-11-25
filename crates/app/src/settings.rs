//! The app's settings: language, light or dark, where the game is when it isn't found by itself,
//! and whether to check for a new version. Kept in a small text file (`settings.txt`, one `name = value` a line) in the user's
//! config folder, so they're easy to read or reset by hand.

use std::path::PathBuf;

use gpui_kit::{App, Global};

use crate::APP_NAME;
use crate::i18n::Language;

/// Light, dark, or whatever the system uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

impl Appearance {
    fn code(self) -> &'static str {
        match self {
            Appearance::System => "system",
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code {
            "system" => Some(Appearance::System),
            "light" => Some(Appearance::Light),
            "dark" => Some(Appearance::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub language: Language,
    pub appearance: Appearance,
    /// Each game's folder, when the player picked it (otherwise it's found through Steam).
    pub ets2_game_dir: Option<PathBuf>,
    pub ats_game_dir: Option<PathBuf>,
    /// Folders of profiles the player added on the start screen, looked in as well as the usual
    /// places.
    pub profile_dirs: Vec<PathBuf>,
    /// Whether to ask GitHub for the latest version when the editor starts (`updates.rs`).
    pub check_for_updates: bool,
    /// Whether maps draw each road as a dot where it is, as they did before they drew roads as
    /// lines (the legend's Roads as dots).
    pub map_roads_as_dots: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            appearance: Appearance::default(),
            ets2_game_dir: None,
            ats_game_dir: None,
            profile_dirs: Vec::new(),
            // On unless turned off: a game update can need a new version of the editor.
            check_for_updates: true,
            map_roads_as_dots: false,
        }
    }
}

impl Global for Settings {}

impl Settings {
    /// Reads `name = value` lines; anything unknown or unreadable keeps its default.
    fn parse(text: &str) -> Self {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match name.trim() {
                "language" => {
                    if let Some(language) = Language::from_code(value) {
                        settings.language = language;
                    }
                }
                "appearance" => {
                    if let Some(appearance) = Appearance::from_code(value) {
                        settings.appearance = appearance;
                    }
                }
                // `game_dir`: ETS2's, as written before ATS had a folder of its own (0.3.0).
                "ets2_game_dir" | "game_dir" if !value.is_empty() => {
                    settings.ets2_game_dir = Some(PathBuf::from(value))
                }
                "ats_game_dir" if !value.is_empty() => {
                    settings.ats_game_dir = Some(PathBuf::from(value))
                }
                // One line for each folder (a path can hold any separator a list would use).
                "profile_dir" if !value.is_empty() => {
                    settings.profile_dirs.push(PathBuf::from(value))
                }
                "check_for_updates" => settings.check_for_updates = value != "false",
                "map_roads_as_dots" => settings.map_roads_as_dots = value == "true",
                _ => {}
            }
        }
        settings
    }

    /// A game's folder, when the player picked it.
    pub fn game_dir(&self, game: save::Game) -> Option<&PathBuf> {
        match game {
            save::Game::Ets2 => self.ets2_game_dir.as_ref(),
            save::Game::Ats => self.ats_game_dir.as_ref(),
        }
    }

    /// Picks a game's folder, or forgets it (`None`: found through Steam again).
    pub fn set_game_dir(&mut self, game: save::Game, dir: Option<PathBuf>) {
        match game {
            save::Game::Ets2 => self.ets2_game_dir = dir,
            save::Game::Ats => self.ats_game_dir = dir,
        }
    }

    fn to_text(&self) -> String {
        let path = |dir: &Option<PathBuf>| {
            dir.as_ref()
                .map(|dir| dir.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let ets2_game_dir = path(&self.ets2_game_dir);
        let ats_game_dir = path(&self.ats_game_dir);
        let profile_dirs: String = self
            .profile_dirs
            .iter()
            .map(|dir| format!("profile_dir = {}\n", dir.to_string_lossy()))
            .collect();
        format!(
            "language = {}\nappearance = {}\nets2_game_dir = {ets2_game_dir}\nats_game_dir = {ats_game_dir}\n{profile_dirs}check_for_updates = {}\nmap_roads_as_dots = {}\n",
            self.language.code(),
            self.appearance.code(),
            self.check_for_updates,
            self.map_roads_as_dots
        )
    }
}

fn file() -> Option<PathBuf> {
    // Tests keep their settings in memory, never touching the developer's own.
    if cfg!(test) {
        return None;
    }
    Some(dirs::config_dir()?.join(APP_NAME).join("settings.txt"))
}

/// Loads the settings (defaults when there's no file yet).
pub fn init(cx: &mut App) {
    let settings = file()
        .and_then(|file| std::fs::read_to_string(file).ok())
        .map(|text| Settings::parse(&text))
        .unwrap_or_default();
    cx.set_global(settings);
}

pub fn get(cx: &App) -> Settings {
    cx.try_global::<Settings>().cloned().unwrap_or_default()
}

/// Puts every setting back as on the first start (language, appearance, the game's folder,
/// folders added on the start screen, the update check), and applies them. Saves and backups
/// aren't settings and stay as they are.
pub fn reset(window: &mut gpui_kit::Window, cx: &mut App) {
    let defaults = Settings::default();
    update(cx, |settings| *settings = defaults.clone());
    // Each applied as if chosen: the words, the numbers' style, the game's data (its folder and
    // language), the theme.
    crate::i18n::set_language(defaults.language, cx);
    crate::theme::set_appearance(defaults.appearance, window, cx);
}

/// Changes the settings and writes them down. Not being able to write them isn't worth stopping
/// for: the change still holds until the app closes.
pub fn update(cx: &mut App, change: impl FnOnce(&mut Settings)) {
    let mut settings = get(cx);
    change(&mut settings);
    if let Some(file) = file() {
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&file, settings.to_text()));
        if let Err(error) = written {
            eprintln!("Couldn't save the settings to {}: {error}", file.display());
        }
    }
    cx.set_global(settings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_it_writes_and_ignores_the_unknown() {
        let settings = Settings {
            language: Language::Swedish,
            appearance: Appearance::Dark,
            ets2_game_dir: Some(PathBuf::from("/games/ets2")),
            ats_game_dir: Some(PathBuf::from("/games/ats")),
            profile_dirs: vec![
                PathBuf::from("D:/saves; and more/profiles"),
                PathBuf::from("/backup/profiles"),
            ],
            check_for_updates: false,
            map_roads_as_dots: true,
        };
        assert_eq!(Settings::parse(&settings.to_text()), settings);
        let odd = Settings::parse("language = klingon\nwhatever\nappearance=light\n");
        assert_eq!(odd.language, Language::English);
        assert_eq!(odd.appearance, Appearance::Light);
        // Checking for updates is on unless the file says otherwise.
        assert!(odd.check_for_updates);
        // ETS2's folder as written before ATS had one of its own.
        let before = Settings::parse("game_dir = /games/ets2\n");
        assert_eq!(
            before.game_dir(save::Game::Ets2),
            Some(&PathBuf::from("/games/ets2"))
        );
        assert_eq!(before.game_dir(save::Game::Ats), None);
    }
}
