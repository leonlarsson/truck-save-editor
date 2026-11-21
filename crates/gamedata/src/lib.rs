//! The installed game's own data, read from its archives: how much XP each level needs, the rank
//! names that go with levels, how high each skill goes, and the game's text in a language (which
//! includes the cities' proper names). Reading it from the game, rather than copying it into the
//! editor, keeps it right when SCS changes it, and keeps SCS's text and artwork out of this repo.
//!
//! Everything here is read-only. When the game isn't found, the app falls back to built-in values.

mod cities;
mod icon;
mod install;
mod recruitment;
mod texture;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use save::{Game, LevelTable};

pub use icon::{ALPHA, BLUE, GREEN, GameIcon, RED};
pub use install::find_install;
pub use texture::{decode_rgba, distance_field_coverage};

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
    /// Each skill's icon, by its save field name.
    skill_icons: HashMap<String, Arc<GameIcon>>,
    /// The ADR class placards, in the order of the save's ADR bits.
    adr_icons: Vec<Arc<GameIcon>>,
    /// The game's text: key → text, in the loaded language.
    strings: HashMap<String, String>,
    /// Every city in the installed game and its maps: token → key of its name in the text.
    city_names: std::collections::BTreeMap<String, String>,
    /// Cities with a truck dealer.
    dealer_cities: BTreeSet<String>,
    /// Cities with a recruitment agency (of those the installed game has).
    recruitment_cities: BTreeSet<String>,
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
        let skills = read_skills(&mut definitions).unwrap_or_default();
        data.skill_max = skills.limits;
        // Icons are a nicety: without `base.scs`, or with an icon the editor can't read, the
        // app shows none.
        if let Ok(mut base) = hashfs::Archive::open(install_dir.join("base.scs")) {
            let folder = "/material/ui/skills";
            for (field, name) in skills.icons {
                if let Some(icon) = GameIcon::load(&mut base, folder, &name) {
                    data.skill_icons.insert(field, Arc::new(icon));
                }
            }
            data.adr_icons = skills
                .adr_icons
                .iter()
                .map_while(|name| GameIcon::load(&mut base, folder, name).map(Arc::new))
                .collect();
        }

        let cities = cities::read_cities(install_dir);
        data.recruitment_cities = recruitment::RECRUITMENT_CITIES
            .iter()
            .filter(|token| cities.names.contains_key(**token))
            .map(|token| token.to_string())
            .collect();
        data.dealer_cities = cities.dealers;
        data.city_names = cities.names;

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

    /// A city's name as the game shows it (`malmo` → `Malmö`): its `city_name_localized` text.
    pub fn city_name(&self, token: &str) -> Option<String> {
        let key = self.city_names.get(token).map_or(token, String::as_str);
        self.text(key).filter(|name| !name.is_empty())
    }

    /// Cities with a truck dealer, in the installed game and its map DLCs. Empty if the game's
    /// city definitions couldn't be read.
    pub fn dealer_cities(&self) -> &BTreeSet<String> {
        &self.dealer_cities
    }

    /// Cities with a recruitment agency, in the installed game and its map DLCs.
    pub fn recruitment_cities(&self) -> &BTreeSet<String> {
        &self.recruitment_cities
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

    /// A skill's icon, by its save field name (`long_dist`, `long_duty` …).
    pub fn skill_icon(&self, field: &str) -> Option<Arc<GameIcon>> {
        self.skill_icons.get(field).cloned()
    }

    /// An ADR class's placard, by its bit in the save (0 = explosives).
    pub fn adr_icon(&self, bit: usize) -> Option<Arc<GameIcon>> {
        self.adr_icons.get(bit).cloned()
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

/// What `skill_data.sii` says about each skill, keyed by the field name the save uses for it.
#[derive(Default)]
struct SkillDefinitions {
    limits: HashMap<String, u8>,
    /// Field → icon material name.
    icons: HashMap<String, String>,
    /// The ADR skill's `rank_icons[]`: one placard per class, in bit order.
    adr_icons: Vec<String>,
}

fn read_skills(archive: &mut hashfs::Archive) -> Result<SkillDefinitions> {
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
    let mut skills = SkillDefinitions::default();
    for (definition, save_field) in fields {
        let Some(skill) = index.single(definition).and_then(|id| document.unit(id)) else {
            continue;
        };
        let max = skill
            .single("max_rank")
            .and_then(sii::value::parse_i64)
            .and_then(|max| u8::try_from(max).ok());
        if let Some(max) = max {
            skills.limits.insert(save_field.to_string(), max);
        }
        if let Some(icon) = skill.single("icon_mat") {
            let icon = sii::value::parse_string(icon);
            skills
                .icons
                .insert(save_field.to_string(), icon.to_string());
        }
        if save_field == "adr" {
            skills.adr_icons = skill
                .items("rank_icons")
                .iter()
                .map(|icon| sii::value::parse_string(icon).to_string())
                .collect();
        }
    }
    Ok(skills)
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
