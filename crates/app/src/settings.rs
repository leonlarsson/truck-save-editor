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
    /// The game's folder, when the player picked it (otherwise it's found through Steam).
    pub game_dir: Option<PathBuf>,
    /// Whether to ask GitHub for the latest version when the editor starts (`updates.rs`).
    pub check_for_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            appearance: Appearance::default(),
            game_dir: None,
            // On unless turned off: a game update can need a new version of the editor.
            check_for_updates: true,
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
                "game_dir" if !value.is_empty() => settings.game_dir = Some(PathBuf::from(value)),
                "check_for_updates" => settings.check_for_updates = value != "false",
                _ => {}
            }
        }
        settings
    }

    fn to_text(&self) -> String {
        let game_dir = self
            .game_dir
            .as_ref()
            .map(|dir| dir.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!(
            "language = {}\nappearance = {}\ngame_dir = {game_dir}\ncheck_for_updates = {}\n",
            self.language.code(),
            self.appearance.code(),
            self.check_for_updates
        )
    }
}

fn file() -> Option<PathBuf> {
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
            game_dir: Some(PathBuf::from("/games/ets2")),
            check_for_updates: false,
        };
        assert_eq!(Settings::parse(&settings.to_text()), settings);
        let odd = Settings::parse("language = klingon\nwhatever\nappearance=light\n");
        assert_eq!(odd.language, Language::English);
        assert_eq!(odd.appearance, Appearance::Light);
        // Checking for updates is on unless the file says otherwise.
        assert!(odd.check_for_updates);
    }
}
