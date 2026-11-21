//! The installed game's own data, read from its archives: how much XP each level needs, the rank
//! names that go with levels, how high each skill goes, and the game's text in a language (which
//! includes the cities' proper names). Reading it from the game, rather than copying it into the
//! editor, keeps it right when SCS changes it, and keeps SCS's text and artwork out of this repo.
//!
//! Everything here is read-only. When the game isn't found, the app falls back to built-in values.

mod install;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use save::{Game, LevelTable};

pub use install::find_install;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Archive(#[from] hashfs::Error),

    #[error(transparent)]
    Sii(#[from] sii::Error),

    #[error("the game's {0} wasn't found")]
    Missing(String),
}

/// What the editor reads from the game.
#[derive(Clone, Debug, Default)]
pub struct GameData {
    pub install_dir: PathBuf,
    pub truck_levels: LevelTable,
    pub coach_levels: LevelTable,
    /// (first level, name key), lowest first.
    truck_ranks: Vec<(i64, String)>,
    coach_ranks: Vec<(i64, String)>,
    /// Each skill's highest level, by its save field name (`long_dist`, `long_duty` …).
    skill_max: HashMap<String, u8>,
    /// The game's text: key → text, in the loaded language.
    strings: HashMap<String, String>,
}

impl GameData {
    /// Reads the game's data from its install folder. `language` is the game's code
    /// (`en_us`, `de_de` …); English is used when the language isn't there.
    pub fn load(game: Game, install_dir: &Path, language: &str) -> Result<Self> {
        let mut definitions = hashfs::Archive::open(install_dir.join("def.scs"))?;
        let mut data = GameData {
            install_dir: install_dir.to_path_buf(),
            ..GameData::default()
        };

        let (truck_levels, truck_ranks) =
            read_levels(&mut definitions, "/def/economy_data_xp.sii")?;
        data.truck_levels = truck_levels;
        data.truck_ranks = truck_ranks;
        // Coaches may not be installed; trucks still have their levels then.
        if game == Game::Ets2
            && let Ok((coach_levels, coach_ranks)) =
                read_levels(&mut definitions, "/def/bus_economy_data_xp.sii")
        {
            data.coach_levels = coach_levels;
            data.coach_ranks = coach_ranks;
        }
        data.skill_max = read_skill_limits(&mut definitions).unwrap_or_default();

        if let Ok(mut locale) = hashfs::Archive::open(install_dir.join("locale.scs")) {
            data.strings = read_strings(&mut locale, language)
                .or_else(|_| read_strings(&mut locale, "en_us"))
                .unwrap_or_default();
        }
        Ok(data)
    }

    /// The game's text for a key, with references to other texts (`@@key@@`) filled in.
    pub fn text(&self, key: &str) -> Option<String> {
        let key = key.trim_matches('@');
        let mut text = self.strings.get(key)?.clone();
        // Texts can refer to other texts; resolve a few levels deep.
        for _ in 0..4 {
            let Some(start) = text.find("@@") else { break };
            let Some(length) = text[start + 2..].find("@@") else {
                break;
            };
            let inner = text[start + 2..start + 2 + length].to_string();
            let replacement = self.strings.get(&inner).cloned().unwrap_or_default();
            text.replace_range(start..start + 4 + length, &replacement);
        }
        Some(text)
    }

    /// A city's name as the game shows it (`malmo` → `Malmö`).
    pub fn city_name(&self, token: &str) -> Option<String> {
        self.text(token).filter(|name| !name.is_empty())
    }

    /// The rank name for a truck driver level (`Newbie` … ), if the game has one.
    pub fn truck_rank(&self, level: i64) -> Option<String> {
        rank(&self.truck_ranks, level).and_then(|key| self.text(key))
    }

    pub fn coach_rank(&self, level: i64) -> Option<String> {
        rank(&self.coach_ranks, level).and_then(|key| self.text(key))
    }

    /// A skill's highest level, by its save field name.
    pub fn max_skill_level(&self, field: &str) -> Option<u8> {
        self.skill_max.get(field).copied()
    }
}

fn rank(ranks: &[(i64, String)], level: i64) -> Option<&str> {
    ranks
        .iter()
        .rev()
        .find(|(first, _)| level >= *first)
        .map(|(_, key)| key.as_str())
}

/// Reads a game file (any SII form), with its `@include`s.
fn read_document(archive: &mut hashfs::Archive, path: &str) -> Result<sii::Document> {
    let bytes = archive
        .read(path)?
        .ok_or_else(|| Error::Missing(path.to_string()))?;
    let bytes = sii::decrypt(&bytes)?;
    if !matches!(sii::detect(&bytes), Some(sii::Format::Text)) {
        return Ok(sii::read(&bytes)?);
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let mut read = |include: &str| -> Option<String> {
        let bytes = archive.read(include).ok().flatten()?;
        let bytes = sii::decrypt(&bytes).ok()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    };
    let expanded = sii::expand_includes(&text, path, &mut read)?;
    Ok(sii::parse_text(&expanded)?)
}

/// `level_xp[]` and `level_names[]` ("first level|@@name key@@") from an `xp_level_data` unit.
fn read_levels(
    archive: &mut hashfs::Archive,
    path: &str,
) -> Result<(LevelTable, Vec<(i64, String)>)> {
    let document = read_document(archive, path)?;
    let unit = document
        .first_of_kind("xp_level_data")
        .ok_or_else(|| Error::Missing(format!("level table in {path}")))?;
    let steps = unit
        .items("level_xp")
        .iter()
        .filter_map(|text| sii::value::parse_i64(text))
        .collect();
    let ranks = unit
        .items("level_names")
        .iter()
        .filter_map(|text| {
            let text = sii::value::parse_string(text);
            let (first, key) = text.split_once('|')?;
            Some((
                first.trim().parse().ok()?,
                key.trim_matches('@').to_string(),
            ))
        })
        .collect();
    Ok((LevelTable::new(steps), ranks))
}

/// Each skill's `max_rank`, keyed by the field name the save uses for it.
fn read_skill_limits(archive: &mut hashfs::Archive) -> Result<HashMap<String, u8>> {
    let document = read_document(archive, "/def/skill_data.sii")?;
    let index = document
        .first_of_kind("skills_data")
        .ok_or_else(|| Error::Missing("skills_data".to_string()))?;
    // `skill_data.sii` field → the save's field.
    let fields = [
        ("long_distance_skill_data", "long_dist"),
        ("valuable_cargo_skill_data", "heavy"),
        ("fragile_cargo_skill_data", "fragile"),
        ("urgent_job_skill_data", "urgent"),
        ("ecodriving_skill_data", "mechanical"),
        ("adr_skill_data", "adr"),
        ("long_duty_skill_data", "long_duty"),
        ("tight_schedule_skill_data", "tight_schedule"),
        ("satisfaction_decrease_skill_data", "satisfaction_decrease"),
        ("low_acceleration_skill_data", "low_acceleration"),
        ("satisfaction_increase_skill_data", "satisfaction_increase"),
        ("bus_ecodriving_skill_data", "ecodriving"),
    ];
    let mut limits = HashMap::new();
    for (definition, save_field) in fields {
        let max = index
            .single(definition)
            .and_then(|id| document.unit(id))
            .and_then(|skill| skill.single("max_rank"))
            .and_then(sii::value::parse_i64)
            .and_then(|max| u8::try_from(max).ok());
        if let Some(max) = max {
            limits.insert(save_field.to_string(), max);
        }
    }
    Ok(limits)
}

/// The game's text in a language: `key[]` and `val[]` pairs in `locale/<language>/local.sii` and
/// what it includes.
fn read_strings(archive: &mut hashfs::Archive, language: &str) -> Result<HashMap<String, String>> {
    let document = read_document(archive, &format!("/locale/{language}/local.sii"))?;
    let mut strings = HashMap::new();
    for unit in document.units_of_kind("localization_db") {
        for (key, value) in unit.items("key").iter().zip(unit.items("val")) {
            strings.insert(
                sii::value::parse_string(key).into_owned(),
                sii::value::parse_string(value).into_owned(),
            );
        }
    }
    Ok(strings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_rank_for_a_level() {
        let ranks = vec![
            (0, "a".to_string()),
            (5, "b".to_string()),
            (100, "c".to_string()),
        ];
        assert_eq!(rank(&ranks, 0), Some("a"));
        assert_eq!(rank(&ranks, 7), Some("b"));
        assert_eq!(rank(&ranks, 182), Some("c"));
        assert_eq!(rank(&[], 3), None);
    }

    #[test]
    fn fills_in_text_references() {
        let mut data = GameData::default();
        data.strings.insert("rank".into(), "Rank".into());
        data.strings.insert("title".into(), "@@rank@@ 3".into());
        assert_eq!(data.text("title").as_deref(), Some("Rank 3"));
        assert_eq!(data.text("@@rank@@").as_deref(), Some("Rank"));
        assert_eq!(data.text("missing"), None);
    }
}
