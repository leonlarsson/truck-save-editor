//! `info.sii`: the summary the game's load menu shows for a slot (name, time, money, XP). It's
//! read to list slots without opening the large `game.sii`, and updated to match when saving.

use sii::{Document, Value, value};

use crate::career::Career;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotInfo {
    /// The name the player gave the save, if any. Unnamed saves (and autosaves) have none.
    pub name: Option<String>,
    /// When the save was written, in seconds since 1970 (UTC).
    pub saved_at: Option<i64>,
    /// In-game time, in minutes since the career started.
    pub game_minutes: Option<i64>,
    pub money: Option<i64>,
    /// Truck and bus experience together, as the load menu shows it.
    pub experience: Option<i64>,
    /// The save format version (105 for game 1.62).
    pub version: Option<i64>,
    /// DLC and mods the save depends on, as the game lists them (`dlc|eut2_north|DLC - Scandinavia`).
    pub dependencies: Vec<String>,
}

impl SlotInfo {
    pub fn read(info: &Document) -> Self {
        let Some(unit) = info.first_of_kind("save_container") else {
            return Self::default();
        };
        let number = |field: &str| unit.single(field).and_then(value::parse_i64);
        let name = unit
            .single("name")
            .map(|text| value::parse_string(text).into_owned())
            .filter(|name| !name.is_empty() && !name.starts_with("@@"));
        Self {
            name,
            saved_at: number("file_time"),
            game_minutes: number("time"),
            money: number("info_money_account"),
            experience: number("info_players_experience"),
            version: number("version"),
            dependencies: unit
                .items("dependencies")
                .iter()
                .map(|text| value::parse_string(text).into_owned())
                .collect(),
        }
    }

    /// The DLC names, without the technical prefix (`DLC - Scandinavia` → `Scandinavia`).
    pub fn dlc_names(&self) -> impl Iterator<Item = &str> {
        self.dependencies.iter().filter_map(|dependency| {
            let name = dependency.rsplit('|').next()?;
            Some(name.strip_prefix("DLC - ").unwrap_or(name))
        })
    }
}

/// Reads a slot's `info.sii` file (any form) into its summary.
pub fn read_info(bytes: &[u8]) -> Option<SlotInfo> {
    sii::read(bytes)
        .ok()
        .map(|document| SlotInfo::read(&document))
}

/// Gives a slot's summary a name, as the load menu shows it.
pub fn set_name(info: &mut Document, name: &str) {
    let Some(id) = info
        .first_of_kind("save_container")
        .map(|unit| unit.id.clone())
    else {
        return;
    };
    if let Some(mut unit) = info.unit_mut(&id) {
        unit.set("name", Value::single(value::format_string(name)));
    }
}

/// Updates the load menu's summary to match a changed career.
pub fn update_info(info: &mut Document, career: &Career) {
    let Some(id) = info
        .first_of_kind("save_container")
        .map(|unit| unit.id.clone())
    else {
        return;
    };
    let Some(mut unit) = info.unit_mut(&id) else {
        return;
    };
    unit.set(
        "info_money_account",
        Value::single(career.money.to_string()),
    );
    let experience = career.experience + career.coach_experience.unwrap_or(0);
    unit.set(
        "info_players_experience",
        Value::single(experience.to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const INFO: &str = "SiiNunit\r\n{\r\nsave_container : _nameless.1 {\r\n name: \"Before the long haul\"\r\n time: 1234567\r\n file_time: 1700000000\r\n version: 105\r\n dependencies: 2\r\n dependencies[0]: \"dlc|eut2_north|DLC - Scandinavia\"\r\n dependencies[1]: \"rdlc|eut2_rims|DLC - Wheel Tuning Pack\"\r\n info_players_experience: 500000\r\n info_money_account: 2480350\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_the_summary() {
        let info = SlotInfo::read(&sii::parse_text(INFO).expect("parses"));
        assert_eq!(info.name.as_deref(), Some("Before the long haul"));
        assert_eq!(info.saved_at, Some(1_700_000_000));
        assert_eq!(info.money, Some(2_480_350));
        assert_eq!(info.experience, Some(500_000));
        assert_eq!(
            info.dlc_names().collect::<Vec<_>>(),
            ["Scandinavia", "Wheel Tuning Pack"]
        );
    }

    #[test]
    fn unnamed_saves_have_no_name() {
        let text = INFO.replace("\"Before the long haul\"", "\"@@noname_save_game@@ 3\"");
        assert_eq!(
            SlotInfo::read(&sii::parse_text(&text).expect("parses")).name,
            None
        );
        let text = INFO.replace("\"Before the long haul\"", "\"\"");
        assert_eq!(
            SlotInfo::read(&sii::parse_text(&text).expect("parses")).name,
            None
        );
    }
}
