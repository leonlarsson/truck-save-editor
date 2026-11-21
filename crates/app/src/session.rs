//! The open save: what was loaded, what's been changed, and what saving would write.
//!
//! One `Entity<Session>` holds it; the pages read from it and change it through its methods, and
//! the header counts its changes. Nothing is written to disk here: `prepared` builds the slot as
//! it would be saved, and the workspace hands that to `save::save` in the background.

use std::collections::{BTreeMap, HashSet};

use gpui_kit::{App, Context, SharedString};
use save::{Career, Driver, Game, Garage, LoadedSlot, Profile, Slot, Vehicle, VehicleKind};

use crate::format::thousands;
use crate::game_data;
use crate::i18n::t;
use crate::labels::{adr_summary, coach_skill_name, skill_name};

pub struct Session {
    game: Game,
    profile: Profile,
    slot: Slot,
    loaded: LoadedSlot,
    /// The career as it is on disk.
    saved_career: Career,
    career: Career,
    /// The vehicles (trucks and coaches) as they are on disk.
    saved_vehicles: Vec<Vehicle>,
    /// Trucks to repair when saving, by id.
    repaired: HashSet<String>,
    /// The garages as they are on disk.
    saved_garages: Vec<Garage>,
    /// Garages to buy or upgrade to large when saving, by id.
    upgraded_garages: HashSet<String>,
    /// The hired drivers as they are on disk.
    saved_drivers: Vec<Driver>,
    /// Drivers to max out when saving, by id.
    maxed_drivers: HashSet<String>,
    /// Values changed in All data, by where they are; applied before the other changes.
    raw_edits: BTreeMap<FieldPath, String>,
    is_saving: bool,
}

/// One change, for the list shown before saving.
pub struct Change {
    pub what: SharedString,
    pub before: SharedString,
    pub after: SharedString,
}

impl Session {
    pub fn open(profile: Profile, slot: Slot, loaded: LoadedSlot) -> save::Result<Self> {
        let career = Career::read(&loaded.game)?;
        Ok(Self {
            game: profile.game,
            saved_vehicles: save::vehicles(&loaded.game),
            saved_garages: save::garages(&loaded.game),
            saved_drivers: save::drivers(&loaded.game),
            maxed_drivers: HashSet::new(),
            raw_edits: BTreeMap::new(),
            saved_career: career.clone(),
            career,
            profile,
            slot,
            loaded,
            repaired: HashSet::new(),
            upgraded_garages: HashSet::new(),
            is_saving: false,
        })
    }

    pub fn game(&self) -> Game {
        self.game
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn slot(&self) -> &Slot {
        &self.slot
    }

    pub fn career(&self) -> &Career {
        &self.career
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving
    }

    pub fn set_saving(&mut self, is_saving: bool, cx: &mut Context<Self>) {
        self.is_saving = is_saving;
        cx.notify();
    }

    /// Changes the career.
    pub fn edit_career(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Career)) {
        edit(&mut self.career);
        cx.notify();
    }

    /// The vehicles as they'll be after saving (repaired ones without wear).
    pub fn vehicles(&self) -> Vec<Vehicle> {
        self.saved_vehicles
            .iter()
            .map(|truck| {
                let mut truck = truck.clone();
                if self.repaired.contains(&truck.id) {
                    truck.wear = save::Wear::default();
                }
                truck
            })
            .collect()
    }

    /// Marks every worn vehicle of a kind for repair.
    pub fn repair_all(&mut self, kind: VehicleKind, cx: &mut Context<Self>) {
        for truck in self.saved_vehicles.iter().filter(|v| v.kind == kind) {
            if truck.wear.worst() > 0.0 {
                self.repaired.insert(truck.id.clone());
            }
        }
        cx.notify();
    }

    pub fn change_count(&self) -> usize {
        self.career.changes_from(&self.saved_career)
            + usize::from(!self.repaired.is_empty())
            + usize::from(!self.upgraded_garages.is_empty())
            + usize::from(!self.maxed_drivers.is_empty())
            + self.raw_edits.len()
    }

    /// The save as loaded (or last saved), for All data. Edits aren't in it; see `raw_value`.
    pub fn document(&self) -> &sii::Document {
        &self.loaded.game
    }

    /// A value as it'll be saved: the All data edit if there is one, else what's in the save.
    pub fn raw_value(&self, path: &FieldPath) -> Option<String> {
        if let Some(edited) = self.raw_edits.get(path) {
            return Some(edited.clone());
        }
        let value = self.loaded.game.unit(&path.unit)?.get(&path.field)?;
        match path.index {
            None => value.as_single().map(str::to_string),
            Some(index) => value.items().get(index).cloned(),
        }
    }

    pub fn is_edited(&self, path: &FieldPath) -> bool {
        self.raw_edits.contains_key(path)
    }

    /// Changes one value (in its text form, as All data shows it). Setting it back to what's in
    /// the save removes the change.
    pub fn set_raw_value(&mut self, path: FieldPath, text: String, cx: &mut Context<Self>) {
        let original = {
            let edit = self.raw_edits.remove(&path);
            let original = self.raw_value(&path);
            if let Some(edit) = edit {
                self.raw_edits.insert(path.clone(), edit);
            }
            original
        };
        if original.as_deref() == Some(text.as_str()) {
            self.raw_edits.remove(&path);
        } else {
            self.raw_edits.insert(path, text);
        }
        cx.notify();
    }

    /// The hired drivers as they'll be after saving.
    pub fn drivers(&self) -> Vec<Driver> {
        self.saved_drivers
            .iter()
            .map(|driver| {
                let mut driver = driver.clone();
                if self.maxed_drivers.contains(&driver.id) {
                    driver.skills = save::Skills::maxed();
                }
                driver
            })
            .collect()
    }

    /// Gives every hired driver every skill at its highest level and all ADR classes.
    pub fn max_all_drivers(&mut self, cx: &mut Context<Self>) {
        for driver in &self.saved_drivers {
            if driver.skills != save::Skills::maxed() {
                self.maxed_drivers.insert(driver.id.clone());
            }
        }
        cx.notify();
    }

    /// The garages as they'll be after saving.
    pub fn garages(&self) -> Vec<Garage> {
        self.saved_garages
            .iter()
            .map(|garage| {
                let mut garage = garage.clone();
                if self.upgraded_garages.contains(&garage.id) {
                    garage.is_owned = true;
                    garage.slots = garage.slots.max(save::LARGE_GARAGE_SLOTS);
                }
                garage
            })
            .collect()
    }

    /// Makes every owned garage large; with `buy_all`, buys every garage too.
    pub fn upgrade_garages(&mut self, buy_all: bool, cx: &mut Context<Self>) {
        for garage in &self.saved_garages {
            if !garage.is_large() && (garage.is_owned || buy_all) {
                self.upgraded_garages.insert(garage.id.clone());
            }
        }
        cx.notify();
    }

    /// Every change, described for people.
    pub fn changes(&self, cx: &App) -> Vec<Change> {
        let (before, after) = (&self.saved_career, &self.career);
        let currency = self.game.currency_symbol();
        let money = |amount: i64| format!("{currency} {}", thousands(amount));
        let mut changes = Vec::new();
        let mut add = |what: SharedString, before: SharedString, after: SharedString| {
            if before != after {
                changes.push(Change {
                    what,
                    before,
                    after,
                });
            }
        };

        add(
            t!(cx, "money"),
            money(before.money).into(),
            money(after.money).into(),
        );
        add(
            t!(cx, "experience"),
            t!(
                cx,
                "experience-amount",
                amount = thousands(before.experience)
            ),
            t!(
                cx,
                "experience-amount",
                amount = thousands(after.experience)
            ),
        );
        if let (Some(old), Some(new)) = (before.coach_experience, after.coach_experience) {
            add(
                t!(cx, "coach-experience"),
                t!(cx, "experience-amount", amount = thousands(old)),
                t!(cx, "experience-amount", amount = thousands(new)),
            );
        }
        for index in 0..before.skills.levels().len() {
            add(
                skill_name(cx, index),
                t!(cx, "skill-level", level = before.skills.levels()[index].1),
                t!(cx, "skill-level", level = after.skills.levels()[index].1),
            );
        }
        if let (Some(old), Some(new)) = (before.coach_skills, after.coach_skills) {
            for index in 0..old.levels().len() {
                add(
                    coach_skill_name(cx, index),
                    t!(cx, "skill-level", level = old.levels()[index].1),
                    t!(cx, "skill-level", level = new.levels()[index].1),
                );
            }
        }
        add(
            t!(cx, "adr"),
            adr_summary(cx, before.skills.adr),
            adr_summary(cx, after.skills.adr),
        );
        add(
            t!(cx, "hq"),
            game_data::city_name(cx, &before.hq_city).into(),
            game_data::city_name(cx, &after.hq_city).into(),
        );
        for (path, text) in &self.raw_edits {
            let before = self
                .loaded
                .game
                .unit(&path.unit)
                .and_then(|unit| unit.get(&path.field))
                .and_then(|value| match path.index {
                    None => value.as_single().map(str::to_string),
                    Some(index) => value.items().get(index).cloned(),
                })
                .unwrap_or_default();
            let kind = self
                .loaded
                .game
                .unit(&path.unit)
                .map(|unit| unit.kind.clone())
                .unwrap_or_default();
            add(
                format!("{kind} › {}", path.field_name()).into(),
                before.into(),
                text.clone().into(),
            );
        }
        if !self.maxed_drivers.is_empty() {
            add(
                t!(cx, "page-drivers"),
                t!(cx, "garages-before"),
                t!(cx, "drivers-maxed", count = self.maxed_drivers.len()),
            );
        }
        if !self.upgraded_garages.is_empty() {
            let (bought, upgraded): (Vec<&Garage>, Vec<&Garage>) = self
                .saved_garages
                .iter()
                .filter(|g| self.upgraded_garages.contains(&g.id))
                .partition(|g| !g.is_owned);
            add(
                t!(cx, "garages"),
                t!(cx, "garages-before"),
                t!(
                    cx,
                    "garages-after",
                    bought = bought.len(),
                    upgraded = upgraded.len()
                ),
            );
        }
        if !self.repaired.is_empty() {
            add(
                t!(cx, "repairs"),
                t!(cx, "repairs-before"),
                t!(cx, "repairs-after", count = self.repaired.len()),
            );
        }
        changes
    }

    pub fn discard(&mut self, cx: &mut Context<Self>) {
        self.career = self.saved_career.clone();
        self.repaired.clear();
        self.upgraded_garages.clear();
        self.maxed_drivers.clear();
        self.raw_edits.clear();
        cx.notify();
    }

    /// The slot as it would be saved: the loaded save with every change applied.
    pub fn prepared(&self) -> save::Result<LoadedSlot> {
        let mut slot = self.loaded.clone();
        // All data edits first, so the typed changes (which name exactly what they change) win.
        for (path, text) in &self.raw_edits {
            path.apply(&mut slot.game, text);
        }
        self.career
            .write_changes(&self.saved_career, &mut slot.game)?;
        for id in &self.repaired {
            save::repair(&mut slot.game, id, false);
        }
        for id in &self.upgraded_garages {
            save::upgrade_garage(&mut slot.game, id);
        }
        for id in &self.maxed_drivers {
            save::max_driver(&mut slot.game, id);
        }
        if let Some(info) = &mut slot.info {
            save::update_info(info, &self.career);
        }
        Ok(slot)
    }

    /// After a successful save: what was written is now what's on disk.
    pub fn mark_saved(&mut self, written: LoadedSlot, cx: &mut Context<Self>) {
        self.saved_vehicles = save::vehicles(&written.game);
        self.saved_garages = save::garages(&written.game);
        self.upgraded_garages.clear();
        self.saved_drivers = save::drivers(&written.game);
        self.maxed_drivers.clear();
        self.raw_edits.clear();
        self.loaded = written;
        self.saved_career = self.career.clone();
        self.repaired.clear();
        self.is_saving = false;
        cx.notify();
    }

    /// After "Save as new slot": the session now edits the new slot, as written.
    pub fn moved_to(
        &mut self,
        dir: std::path::PathBuf,
        written: LoadedSlot,
        cx: &mut Context<Self>,
    ) {
        let folder = dir
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.slot = Slot {
            kind: folder
                .parse()
                .map(save::SlotKind::Manual)
                .unwrap_or(save::SlotKind::Other),
            info: written
                .info
                .as_ref()
                .map(save::SlotInfo::read)
                .unwrap_or_default(),
            modified: Some(std::time::SystemTime::now()),
            folder,
            dir,
        };
        self.mark_saved(written, cx);
    }

    /// The profile's folder name, used to keep its backups apart.
    pub fn profile_folder(&self) -> String {
        self.profile
            .dir
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "profile".to_string())
    }
}

/// Where a value is: a unit, a field, and for arrays the item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldPath {
    pub unit: String,
    pub field: String,
    pub index: Option<usize>,
}

impl FieldPath {
    /// `money_account`, or `trucks[3]` for an array item.
    pub fn field_name(&self) -> String {
        match self.index {
            Some(index) => format!("{}[{index}]", self.field),
            None => self.field.clone(),
        }
    }

    fn apply(&self, document: &mut sii::Document, text: &str) {
        let Some(mut unit) = document.unit_mut(&self.unit) else {
            return;
        };
        match self.index {
            None => unit.set(&self.field, sii::Value::single(text)),
            Some(index) => {
                let mut items = unit
                    .get(&self.field)
                    .map(|value| value.items().to_vec())
                    .unwrap_or_default();
                if let Some(item) = items.get_mut(index) {
                    *item = text.to_string();
                    unit.set(&self.field, sii::Value::Array(items));
                }
            }
        }
    }
}
