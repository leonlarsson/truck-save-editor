//! The installed game's own data, read from its archives: how much XP each level needs, the rank
//! names that go with levels, how high each skill goes, and the game's text in a language (which
//! includes the cities' proper names). Reading it from the game, rather than copying it into the
//! editor, keeps it right when SCS changes it, and keeps SCS's text and artwork out of this repo.
//!
//! Everything here is read-only. When the game isn't found, the app falls back to built-in values.

mod cities;
mod companies;
mod content;
mod icon;
mod install;
mod recruitment;
mod roads;
mod texture;
mod vehicles;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use save::{Game, LevelTable};

pub use companies::{CompanyCargo, company_cargo};
pub use content::{Installed, installed};
pub use icon::{ALPHA, BLUE, GREEN, GameIcon, RED};
pub use install::find_install;
pub use roads::{
    MapContents, MapPlace, MapPlaceKind, MapRoad, decode_token, map_contents, map_roads, road_ids,
    sector_contents, sector_roads,
};
pub use texture::{decode_rgba, distance_field_coverage};
pub use vehicles::{PaintJob, Part, PartKind, model_paint_jobs, model_parts};

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
    /// The map DLC each city comes with (token → DLC id); the base game's aren't in it. Every
    /// DLC's files ship with the game, owned or not, so a save's cities are the base game's and
    /// those of the map DLCs it was made with.
    city_dlcs: BTreeMap<String, String>,
    /// Cities with a truck dealer.
    dealer_cities: BTreeSet<String>,
    /// City token → its country's short code (`verona` → `I`).
    city_country_codes: BTreeMap<String, String>,
    /// Country token → the key of its name in the game's text.
    country_names: BTreeMap<String, String>,
    /// Cities with a recruitment agency (of those the installed game has).
    recruitment_cities: BTreeSet<String>,
    /// Vehicle models: model id → (brand text key, series text key).
    models: HashMap<String, (String, String)>,
    /// Companies' names (`renar` → `Renar Logistik`), as the game writes them (not translated).
    company_names: HashMap<String, String>,
    /// Cargo: token → key of its name in the text (`battery` → `cn_batteries`).
    cargo_names: HashMap<String, String>,
    /// Brand logos, by brand token (`volvo`): for trucks, and for coaches.
    truck_logos: HashMap<String, Arc<GameIcon>>,
    coach_logos: HashMap<String, Arc<GameIcon>>,
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
            for (folder, logos) in [
                ("/material/ui/brand_logo", &mut data.truck_logos),
                ("/material/ui/bus_brand_logo", &mut data.coach_logos),
            ] {
                let names = base
                    .list(folder)
                    .ok()
                    .flatten()
                    .map(|listing| listing.files)
                    .unwrap_or_default();
                for name in names {
                    let Some(brand) = name.strip_suffix(".mat") else {
                        continue;
                    };
                    if let Some(icon) = GameIcon::load(&mut base, folder, brand) {
                        logos.insert(brand.to_string(), Arc::new(icon));
                    }
                }
            }
            data.adr_icons = skills
                .adr_icons
                .iter()
                .map_while(|name| GameIcon::load(&mut base, folder, name).map(Arc::new))
                .collect();
        }

        // Every archive with definitions, opened once: the base game's, then each DLC's.
        let mut cities = cities::Cities::default();
        for path in def_archives(install_dir) {
            let Ok(mut archive) = hashfs::Archive::open(&path) else {
                continue;
            };
            vehicles::add_models(&mut archive, &mut data.models);
            let dlc = archive_dlc(&mut archive, &path);
            for_each_definition(&mut archive, "city", |document| {
                if let Some(dlc) = &dlc {
                    for unit in document.units_of_kind("city_data") {
                        if let Some(token) = unit.id.strip_prefix("city.") {
                            data.city_dlcs.insert(token.to_string(), dlc.clone());
                        }
                    }
                }
                cities.add(document)
            });
            for_each_definition(&mut archive, "country", |document| {
                cities.add_countries(document)
            });
            for_each_definition(&mut archive, "company", |document| {
                for unit in document.units_of_kind("company_permanent") {
                    let (Some(token), Some(name)) = (
                        unit.id.strip_prefix("company.permanent."),
                        unit.single("name"),
                    ) else {
                        continue;
                    };
                    data.company_names.insert(
                        token.to_string(),
                        sii::value::parse_string(name).to_string(),
                    );
                }
            });
            for_each_definition(&mut archive, "cargo", |document| {
                for unit in document.units_of_kind("cargo_data") {
                    let (Some(token), Some(name)) =
                        (unit.id.strip_prefix("cargo."), unit.single("name"))
                    else {
                        continue;
                    };
                    let key = sii::value::parse_string(name).trim_matches('@').to_string();
                    data.cargo_names.insert(token.to_string(), key);
                }
            });
        }
        data.recruitment_cities = recruitment::RECRUITMENT_CITIES
            .iter()
            .filter(|token| cities.names.contains_key(**token))
            .map(|token| token.to_string())
            .collect();
        data.country_names = cities.country_names.clone();
        data.dealer_cities = cities.dealers;
        data.city_names = cities.names;
        data.city_country_codes = cities
            .countries
            .iter()
            .filter_map(|(city, country)| {
                let code = cities.country_codes.get(country)?;
                Some((city.clone(), code.clone()))
            })
            .collect();

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

    /// A line of the game's text with `@@key@@`s in it (`750 @@hp@@ (551@@kw@@)`), in the game's
    /// language. A key it doesn't know is left out.
    pub fn resolve(&self, raw: &str) -> String {
        let mut text = raw.to_string();
        // Each pass replaces one key; a line has a handful at most.
        for _ in 0..16 {
            let Some(start) = text.find("@@") else { break };
            let Some(length) = text[start + 2..].find("@@") else {
                break;
            };
            let key = text[start + 2..start + 2 + length].to_string();
            let replacement = self.text(&key).unwrap_or_default();
            text.replace_range(start..start + 4 + length, &replacement);
        }
        text
    }

    /// A country's name in the game's language (`Italia` → `Italy`), by its token (`italy`).
    pub fn country_name(&self, token: &str) -> Option<String> {
        let key = self.country_names.get(token)?;
        self.text(key).filter(|name| !name.is_empty())
    }

    /// The short code of a city's country, as the game shows it after the city's name (`I`).
    pub fn city_country_code(&self, token: &str) -> Option<&str> {
        self.city_country_codes.get(token).map(String::as_str)
    }

    /// Every city the installed game defines, by token, with the map DLC it comes with (`None`
    /// for the base game's). See `city_dlcs`: what a save may use is what it was made with.
    pub fn cities(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.city_names.keys().map(|token| {
            (
                token.as_str(),
                self.city_dlcs.get(token).map(String::as_str),
            )
        })
    }

    /// A city's name as the game shows it (`malmo` → `Malmö`): its `city_name_localized` text.
    pub fn city_name(&self, token: &str) -> Option<String> {
        let key = self.city_names.get(token).map_or(token, String::as_str);
        self.text(key).filter(|name| !name.is_empty())
    }

    /// A vehicle model's name as the game shows it (`Volvo FH16 Classic`), by model id.
    pub fn model_name(&self, model_id: &str) -> Option<String> {
        let (brand, series) = self.models.get(model_id)?;
        let series = self.text(series).filter(|text| !text.is_empty())?;
        match self.text(brand).filter(|text| !text.is_empty()) {
            Some(brand) => Some(format!("{brand} {series}")),
            // Trailers: the series alone (`Box`).
            None => Some(series),
        }
    }

    /// A company's name (`renar` → `Renar Logistik`).
    pub fn company_name(&self, token: &str) -> Option<String> {
        self.company_names.get(token).cloned()
    }

    /// A cargo's name in the loaded language (`battery` → `Batteries`).
    pub fn cargo_name(&self, token: &str) -> Option<String> {
        self.text(self.cargo_names.get(token)?)
            .filter(|name| !name.is_empty())
    }

    /// A brand's logo (`volvo`), from the truck or coach set.
    pub fn brand_logo(&self, brand: &str, is_coach: bool) -> Option<Arc<GameIcon>> {
        let logos = if is_coach {
            &self.coach_logos
        } else {
            &self.truck_logos
        };
        logos.get(brand).cloned()
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

/// The DLC an archive belongs to, by its id (`eut2_sstrip`), from its manifest
/// (`<archive name>.manifest.sii`, a `dlc_package`); `None` for the base game's archives, which
/// have none. SCS ships every DLC's archive with the game, owned or not (Steam checks ownership
/// when the game runs), so what a save may use is what it was made with, not what's installed.
pub(crate) fn archive_dlc(archive: &mut hashfs::Archive, path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let document = read_document(archive, &format!("/{stem}.manifest.sii")).ok()?;
    let id = document.first_of_kind("dlc_package")?.single("id")?;
    Some(sii::value::parse_string(id).to_string())
}

/// The archives that hold definitions: `def.scs`, then every DLC archive (`dlc_*.scs`), sorted.
pub(crate) fn def_archives(install_dir: &Path) -> Vec<PathBuf> {
    archives_after(install_dir, "def.scs")
}

/// The archives that hold the game's pictures and models: `base.scs`, then every DLC archive.
pub fn content_archives(install_dir: &Path) -> Vec<PathBuf> {
    archives_after(install_dir, "base.scs")
}

/// `first`, then every DLC archive (`dlc_*.scs`) in the install folder, sorted.
fn archives_after(install_dir: &Path, first: &str) -> Vec<PathBuf> {
    let mut archives = vec![install_dir.join(first)];
    if let Ok(entries) = std::fs::read_dir(install_dir) {
        let mut dlcs: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("dlc_") && name.ends_with(".scs")
            })
            .collect();
        dlcs.sort();
        archives.extend(dlcs);
    }
    archives
}

/// Reads every definition file of one kind in an archive: `def/<base>.sii` and the `def/<base>.*.sii`
/// files DLCs and patches add. A file that can't be read is skipped.
fn for_each_definition(
    archive: &mut hashfs::Archive,
    base: &str,
    mut each: impl FnMut(&sii::Document),
) {
    let Ok(Some(listing)) = archive.list("/def") else {
        return;
    };
    let prefix = format!("{base}.");
    let files: Vec<String> = listing
        .files
        .into_iter()
        .filter(|name| name.starts_with(&prefix) && name.ends_with(".sii"))
        .collect();
    for file in files {
        if let Ok(document) = read_document(archive, &format!("/def/{file}")) {
            each(&document);
        }
    }
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
