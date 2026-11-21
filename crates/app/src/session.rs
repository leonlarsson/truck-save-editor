//! The open save: what was loaded, what's been changed, and what saving would write.
//!
//! One `Entity<Session>` holds it; the pages read from it and change it through its methods, and
//! the header counts its changes. Nothing is written to disk here: `prepared` builds the slot as
//! it would be saved, and the workspace hands that to `save::save` in the background.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use gpui_kit::{App, Context, SharedString};
use save::{Career, Driver, Game, Garage, LoadedSlot, Place, Profile, Slot, Vehicle, VehicleKind};

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
    /// The vehicles (trucks and coaches) as they are on disk.
    saved_vehicles: Vec<Vehicle>,
    /// The garages as they are on disk.
    saved_garages: Vec<Garage>,
    /// The hired drivers as they are on disk.
    saved_drivers: Vec<Driver>,
    /// Every change not saved yet.
    pending: Pending,
    /// What `pending` was before each change, latest last: Undo goes back through these.
    undo: Vec<Pending>,
    /// What Undo went back from, latest last: Redo goes forward again. A new change clears it.
    redo: Vec<Pending>,
    is_saving: bool,
    /// The save as it would be written (the loaded save with every change), built when first
    /// asked for after a change. Every page reads from it, so they all agree: a repair shows in
    /// All data, an All data edit shows in Fleet. A `RefCell` because building it is a cache
    /// fill, not a change, and reading the session only gives `&self`.
    preview: std::cell::RefCell<Option<Arc<sii::Document>>>,
}

/// Everything changed since the save was loaded (or last saved). Undo and Redo swap whole copies
/// of it: it's small (a career and a few lists of ids), so copies are cheap, and an undo can never
/// get out of step with what it undoes.
#[derive(Clone, Debug, PartialEq)]
struct Pending {
    career: Career,
    /// Vehicles to repair when saving, by id.
    repaired: HashSet<String>,
    /// Trucks and coaches to fill up when saving, by id.
    refueled: HashSet<String>,
    /// Garages to buy or upgrade to large when saving, by id.
    upgraded_garages: HashSet<String>,
    /// Drivers to max out when saving, by id.
    maxed_drivers: HashSet<String>,
    /// Whether to mark every city as visited when saving.
    visit_all_cities: bool,
    /// Values changed in All data, by where they are; applied before the other changes.
    raw_edits: BTreeMap<FieldPath, String>,
    /// Whether to make every job offer last at least a week (game time) when saving.
    keep_offers: bool,
    /// Cities whose truck dealer to unlock when saving.
    unlock_dealers: BTreeSet<String>,
    /// Cities whose recruitment agency to unlock when saving.
    unlock_recruitments: BTreeSet<String>,
}

impl Pending {
    /// No changes: the career as it is on disk.
    fn none(career: Career) -> Self {
        Self {
            career,
            repaired: HashSet::new(),
            refueled: HashSet::new(),
            upgraded_garages: HashSet::new(),
            maxed_drivers: HashSet::new(),
            visit_all_cities: false,
            raw_edits: BTreeMap::new(),
            keep_offers: false,
            unlock_dealers: BTreeSet::new(),
            unlock_recruitments: BTreeSet::new(),
        }
    }

    fn unlocks(&self, place: Place) -> &BTreeSet<String> {
        match place {
            Place::Dealer => &self.unlock_dealers,
            Place::Recruitment => &self.unlock_recruitments,
        }
    }

    fn unlocks_mut(&mut self, place: Place) -> &mut BTreeSet<String> {
        match place {
            Place::Dealer => &mut self.unlock_dealers,
            Place::Recruitment => &mut self.unlock_recruitments,
        }
    }
}

/// How long "Keep every offer" keeps them: a week of game time, in minutes.
pub const KEEP_OFFERS_MINUTES: i64 = 7 * 24 * 60;

/// How many steps Undo remembers.
const UNDO_LIMIT: usize = 200;

/// A set of changes made together, for a fresh start (see `quick_start.rs`): one step of Undo.
#[derive(Clone, Debug, Default)]
pub struct QuickStartPlan {
    pub money: Option<i64>,
    pub truck_skills: bool,
    pub adr: bool,
    pub coach_skills: bool,
    /// Buy (or make large) the garage in the headquarters city.
    pub hq_garage: bool,
    pub visit_cities: bool,
    /// Dealers and agencies to unlock (the cities the game has them in), when asked for.
    pub dealers: Option<BTreeSet<String>>,
    pub recruitments: Option<BTreeSet<String>>,
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
            pending: Pending::none(career.clone()),
            undo: Vec::new(),
            redo: Vec::new(),
            saved_career: career,
            profile,
            slot,
            loaded,
            is_saving: false,
            preview: std::cell::RefCell::new(None),
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
        &self.pending.career
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving
    }

    pub fn set_saving(&mut self, is_saving: bool, cx: &mut Context<Self>) {
        self.is_saving = is_saving;
        cx.notify();
    }

    /// Makes a change: everything that changes the save goes through here, so each change is
    /// one step of Undo.
    fn change(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Pending, &Self)) {
        let before = self.pending.clone();
        let mut pending = self.pending.clone();
        edit(&mut pending, self);
        if pending == before {
            return;
        }
        self.pending = pending;
        self.forget_preview();
        self.undo.push(before);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        cx.notify();
    }

    /// The save as it would be written, every change in it (see `preview`).
    pub fn preview(&self) -> Arc<sii::Document> {
        if let Some(preview) = self.preview.borrow().as_ref() {
            return preview.clone();
        }
        // If a change can't be applied (it can't, short of a bug), show the save as loaded
        // rather than nothing; saving reports the error.
        let document = self
            .prepared()
            .map(|slot| slot.game)
            .unwrap_or_else(|_| self.loaded.game.clone());
        let preview = Arc::new(document);
        *self.preview.borrow_mut() = Some(preview.clone());
        preview
    }

    fn forget_preview(&mut self) {
        *self.preview.get_mut() = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Goes back one change (Discard counts as one, so it can be undone too).
    pub fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(previous) = self.undo.pop() {
            let current = std::mem::replace(&mut self.pending, previous);
            self.forget_preview();
            self.redo.push(current);
            cx.notify();
        }
    }

    pub fn redo(&mut self, cx: &mut Context<Self>) {
        if let Some(next) = self.redo.pop() {
            let current = std::mem::replace(&mut self.pending, next);
            self.forget_preview();
            self.undo.push(current);
            cx.notify();
        }
    }

    /// Changes the career.
    pub fn edit_career(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut Career)) {
        self.change(cx, |pending, _| edit(&mut pending.career));
    }

    /// The vehicles as they'll be after saving.
    pub fn vehicles(&self) -> Vec<Vehicle> {
        save::vehicles(&self.preview())
    }

    /// Applies a quick-start plan: everything in it is one change.
    pub fn apply_quick_start(&mut self, plan: QuickStartPlan, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            let career = &mut pending.career;
            if let Some(money) = plan.money {
                career.money = money;
            }
            if plan.truck_skills {
                career.skills = save::Skills {
                    adr: career.skills.adr,
                    ..save::Skills::maxed()
                };
            }
            if plan.adr {
                career.skills.adr = save::ADR_ALL;
            }
            if plan.coach_skills && career.coach_skills.is_some() {
                career.coach_skills = Some(save::CoachSkills::maxed());
            }
            if plan.hq_garage {
                let hq = career.hq_city.clone();
                if let Some(garage) = this
                    .saved_garages
                    .iter()
                    .find(|garage| garage.city == hq && !garage.is_large())
                {
                    pending.upgraded_garages.insert(garage.id.clone());
                }
            }
            if plan.visit_cities {
                pending.visit_all_cities = true;
            }
            for (place, cities) in [
                (Place::Dealer, &plan.dealers),
                (Place::Recruitment, &plan.recruitments),
            ] {
                if let Some(cities) = cities {
                    let saved: BTreeSet<String> = save::unlocked(&this.loaded.game, place)
                        .into_iter()
                        .collect();
                    *pending.unlocks_mut(place) = cities.difference(&saved).cloned().collect();
                }
            }
        });
    }

    /// Fills the tank of every truck or coach of a kind that isn't full.
    pub fn refuel_all(&mut self, kind: VehicleKind, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            for vehicle in this.saved_vehicles.iter().filter(|v| v.kind == kind) {
                if vehicle.fuel.is_some_and(|fuel| fuel < 1.0) {
                    pending.refueled.insert(vehicle.id.clone());
                }
            }
        });
    }

    /// The job offers as they'll be after saving, and the game's clock (minutes).
    pub fn jobs(&self) -> (Vec<save::Job>, i64) {
        let preview = self.preview();
        (save::jobs(&preview), save::game_time(&preview).unwrap_or(0))
    }

    /// Makes every job offer last at least a week of game time from now.
    pub fn keep_offers(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| pending.keep_offers = true);
    }

    /// Fills one truck's or coach's tank.
    pub fn refuel(&mut self, id: &str, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| {
            pending.refueled.insert(id.to_string());
        });
    }

    /// Gives one driver every skill at its highest level and all ADR classes.
    pub fn max_driver(&mut self, id: &str, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| {
            pending.maxed_drivers.insert(id.to_string());
        });
    }

    /// Buys one garage (if needed) and makes it large.
    pub fn upgrade_garage(&mut self, id: &str, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| {
            pending.upgraded_garages.insert(id.to_string());
        });
    }

    /// Changes one value of a unit by name, as a detail panel does (see `set_raw_value`).
    pub fn set_field(
        &mut self,
        unit: &str,
        field: &str,
        index: Option<usize>,
        text: String,
        cx: &mut Context<Self>,
    ) {
        let path = FieldPath {
            unit: unit.to_string(),
            field: field.to_string(),
            index,
        };
        self.set_raw_value(path, text, cx);
    }

    /// A unit as it'll be saved (for detail panels: every field, every change in it).
    pub fn unit(&self, id: &str) -> Option<sii::Unit> {
        self.preview().unit(id).cloned()
    }

    /// Marks one vehicle for repair.
    pub fn repair(&mut self, id: &str, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| {
            pending.repaired.insert(id.to_string());
        });
    }

    /// Marks every worn vehicle of a kind for repair.
    pub fn repair_all(&mut self, kind: VehicleKind, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            for truck in this.saved_vehicles.iter().filter(|v| v.kind == kind) {
                if truck.wear.worst() > 0.0 {
                    pending.repaired.insert(truck.id.clone());
                }
            }
        });
    }

    pub fn change_count(&self) -> usize {
        self.pending.career.changes_from(&self.saved_career)
            + usize::from(!self.pending.repaired.is_empty())
            + usize::from(!self.pending.refueled.is_empty())
            + usize::from(!self.pending.upgraded_garages.is_empty())
            + usize::from(!self.pending.maxed_drivers.is_empty())
            + self.pending.raw_edits.len()
            + usize::from(self.pending.visit_all_cities)
            + usize::from(self.pending.keep_offers)
            + usize::from(!self.pending.unlock_dealers.is_empty())
            + usize::from(!self.pending.unlock_recruitments.is_empty())
    }

    /// Every city with how often it was visited, as it'll be after saving.
    pub fn cities(&self) -> Vec<save::City> {
        save::cities(&self.preview())
    }

    /// The cities whose dealer (or agency) is unlocked, as it'll be after saving.
    pub fn unlocked(&self, place: Place) -> BTreeSet<String> {
        save::unlocked(&self.preview(), place).into_iter().collect()
    }

    /// Unlocks the dealers (or agencies) in every city of `cities` (which the game's data knows).
    pub fn unlock_all(&mut self, place: Place, cities: &BTreeSet<String>, cx: &mut Context<Self>) {
        let saved: BTreeSet<String> = save::unlocked(&self.loaded.game, place)
            .into_iter()
            .collect();
        self.change(cx, |pending, _| {
            *pending.unlocks_mut(place) = cities.difference(&saved).cloned().collect();
        });
    }

    pub fn visit_all_cities(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |pending, _| pending.visit_all_cities = true);
    }

    /// A value as it'll be saved (every change in it), in its text form.
    pub fn raw_value(&self, path: &FieldPath) -> Option<String> {
        path.read(&self.preview())
    }

    /// Whether a value will be saved different from what's on disk (by an All data edit or any
    /// other change).
    pub fn is_edited(&self, path: &FieldPath) -> bool {
        self.pending.raw_edits.contains_key(path)
            || path.read(&self.preview()) != self.saved_value(path)
    }

    /// Changes one value (in its text form, as All data shows it). A value the career pages own
    /// (money, experience, skills …) becomes a career change, so those pages show it and it's
    /// undone like one; anything else is kept as an All data edit. Setting a value back to
    /// what's on disk removes the edit.
    pub fn set_raw_value(&mut self, path: FieldPath, text: String, cx: &mut Context<Self>) {
        let mut edited = (*self.preview()).clone();
        path.apply(&mut edited, &text);
        if let Ok(career) = Career::read(&edited)
            && career != self.pending.career
        {
            self.change(cx, |pending, _| {
                pending.career = career;
                pending.raw_edits.remove(&path);
            });
            return;
        }
        let original = self.saved_value(&path);
        self.change(cx, |pending, _| {
            if original.as_deref() == Some(text.as_str()) {
                pending.raw_edits.remove(&path);
            } else {
                pending.raw_edits.insert(path, text);
            }
        });
    }

    /// A value as it is in the save on disk.
    pub fn saved_value(&self, path: &FieldPath) -> Option<String> {
        path.read(&self.loaded.game)
    }

    /// The hired drivers as they'll be after saving.
    pub fn drivers(&self) -> Vec<Driver> {
        save::drivers(&self.preview())
    }

    /// Gives every hired driver every skill at its highest level and all ADR classes.
    pub fn max_all_drivers(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            for driver in &this.saved_drivers {
                if driver.skills != save::Skills::maxed() {
                    pending.maxed_drivers.insert(driver.id.clone());
                }
            }
        });
    }

    /// The garages as they'll be after saving.
    pub fn garages(&self) -> Vec<Garage> {
        save::garages(&self.preview())
    }

    /// Makes every owned garage large; with `buy_all`, buys every garage too.
    pub fn upgrade_garages(&mut self, buy_all: bool, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            for garage in &this.saved_garages {
                if !garage.is_large() && (garage.is_owned || buy_all) {
                    pending.upgraded_garages.insert(garage.id.clone());
                }
            }
        });
    }

    /// Every change, described for people.
    pub fn changes(&self, cx: &App) -> Vec<Change> {
        let (before, after) = (&self.saved_career, &self.pending.career);
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
        for (path, text) in &self.pending.raw_edits {
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
        if self.pending.visit_all_cities {
            let unvisited = save::cities(&self.loaded.game)
                .iter()
                .filter(|city| city.visits == 0)
                .count();
            add(
                t!(cx, "page-map"),
                t!(cx, "garages-before"),
                t!(cx, "map-visited-all", count = unvisited),
            );
        }
        for (place, what) in [
            (Place::Dealer, t!(cx, "map-dealers")),
            (Place::Recruitment, t!(cx, "map-recruitments")),
        ] {
            if self.pending.unlocks(place).is_empty() {
                continue;
            }
            let saved: BTreeSet<String> = save::unlocked(&self.loaded.game, place)
                .into_iter()
                .collect();
            let after = self.unlocked(place);
            // Counted as the Map page counts them: only where the game has one, of how many.
            match game_data::place_cities(cx, place) {
                Some(cities) => add(
                    what,
                    t!(
                        cx,
                        "map-unlocked-of",
                        unlocked = saved.intersection(&cities).count(),
                        count = cities.len()
                    ),
                    t!(
                        cx,
                        "map-unlocked-of",
                        unlocked = after.intersection(&cities).count(),
                        count = cities.len()
                    ),
                ),
                None => add(
                    what,
                    t!(cx, "map-unlocked-count", count = saved.len()),
                    t!(cx, "map-unlocked-count", count = after.len()),
                ),
            }
        }
        if !self.pending.maxed_drivers.is_empty() {
            add(
                t!(cx, "page-drivers"),
                t!(cx, "garages-before"),
                t!(
                    cx,
                    "drivers-maxed",
                    count = self.pending.maxed_drivers.len()
                ),
            );
        }
        if !self.pending.upgraded_garages.is_empty() {
            let (bought, upgraded): (Vec<&Garage>, Vec<&Garage>) = self
                .saved_garages
                .iter()
                .filter(|g| self.pending.upgraded_garages.contains(&g.id))
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
        if self.pending.keep_offers {
            add(
                t!(cx, "jobs-offers"),
                t!(cx, "garages-before"),
                t!(cx, "jobs-kept-week"),
            );
        }
        if !self.pending.refueled.is_empty() {
            add(
                t!(cx, "fuel"),
                t!(cx, "fuel-before"),
                t!(cx, "fuel-after", count = self.pending.refueled.len()),
            );
        }
        if !self.pending.repaired.is_empty() {
            add(
                t!(cx, "repairs"),
                t!(cx, "repairs-before"),
                t!(cx, "repairs-after", count = self.pending.repaired.len()),
            );
        }
        changes
    }

    /// Drops every change. One step of Undo brings them back.
    pub fn discard(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |pending, this| {
            *pending = Pending::none(this.saved_career.clone());
        });
    }

    /// The slot as it would be saved: the loaded save with every change applied.
    pub fn prepared(&self) -> save::Result<LoadedSlot> {
        let mut slot = self.loaded.clone();
        // All data edits first, so the typed changes (which name exactly what they change) win.
        for (path, text) in &self.pending.raw_edits {
            path.apply(&mut slot.game, text);
        }
        self.pending
            .career
            .write_changes(&self.saved_career, &mut slot.game)?;
        for id in &self.pending.repaired {
            save::repair(&mut slot.game, id, false);
        }
        for id in &self.pending.refueled {
            save::refuel(&mut slot.game, id);
        }
        for id in &self.pending.upgraded_garages {
            save::upgrade_garage(&mut slot.game, id);
        }
        for id in &self.pending.maxed_drivers {
            save::max_driver(&mut slot.game, id);
        }
        if self.pending.visit_all_cities {
            save::visit_all_cities(&mut slot.game);
        }
        if self.pending.keep_offers
            && let Some(now) = save::game_time(&slot.game)
        {
            save::extend_offers(&mut slot.game, now + KEEP_OFFERS_MINUTES);
        }
        for place in [Place::Dealer, Place::Recruitment] {
            let cities: Vec<String> = self.pending.unlocks(place).iter().cloned().collect();
            save::unlock(&mut slot.game, place, &cities);
        }
        if let Some(info) = &mut slot.info {
            save::update_info(info, &self.pending.career);
        }
        Ok(slot)
    }

    /// After a successful save: what was written is now what's on disk.
    pub fn mark_saved(&mut self, written: LoadedSlot, cx: &mut Context<Self>) {
        self.saved_vehicles = save::vehicles(&written.game);
        self.saved_garages = save::garages(&written.game);
        self.saved_drivers = save::drivers(&written.game);
        self.loaded = written;
        self.forget_preview();
        self.saved_career = self.pending.career.clone();
        self.pending = Pending::none(self.saved_career.clone());
        // What's on disk changed under the history; undoing past a save would describe changes
        // against a save that no longer exists.
        self.undo.clear();
        self.redo.clear();
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

    /// The value at this path in a document, in its text form.
    pub fn read(&self, document: &sii::Document) -> Option<String> {
        let value = document.unit(&self.unit)?.get(&self.field)?;
        match self.index {
            None => value.as_single().map(str::to_string),
            Some(index) => value.items().get(index).cloned(),
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

#[cfg(test)]
mod tests {
    use gpui_kit::{AppContext as _, Entity, TestAppContext};
    use save::{Game, Location, Profile, Slot, SlotKind};

    use super::*;

    /// A tiny made-up save: a career and one worn truck.
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : _nameless.1 {\r\n bank: _nameless.2\r\n player: _nameless.3\r\n experience_points: 5000\r\n adr: 0\r\n long_dist: 0\r\n heavy: 0\r\n fragile: 0\r\n urgent: 0\r\n mechanical: 0\r\n}\r\n\r\nbank : _nameless.2 {\r\n money_account: 1000\r\n loans: 0\r\n}\r\n\r\nplayer : _nameless.3 {\r\n hq_city: lyon\r\n}\r\n\r\n}\r\n";

    fn session(cx: &mut TestAppContext) -> Entity<Session> {
        let loaded = LoadedSlot {
            dir: "slot".into(),
            game: sii::parse_text(SAVE).expect("parses"),
            info: None,
        };
        let profile = Profile {
            game: Game::Ets2,
            name: "Tester".into(),
            dir: "profile".into(),
            location: Location::Local,
            last_saved: None,
        };
        let slot = Slot {
            dir: "slot".into(),
            folder: "1".into(),
            kind: SlotKind::Manual(1),
            info: Default::default(),
            modified: None,
        };
        cx.new(|_| Session::open(profile, slot, loaded).expect("opens"))
    }

    fn money(session: &Entity<Session>, cx: &mut TestAppContext) -> i64 {
        cx.update(|cx| session.read(cx).career().money)
    }

    fn set_money(session: &Entity<Session>, amount: i64, cx: &mut TestAppContext) {
        session.update(cx, |session, cx| {
            session.edit_career(cx, |career| career.money = amount)
        });
    }

    #[gpui_kit::test]
    fn undo_and_redo_step_through_changes(cx: &mut TestAppContext) {
        let session = session(cx);
        set_money(&session, 2000, cx);
        set_money(&session, 3000, cx);

        session.update(cx, |session, cx| session.undo(cx));
        assert_eq!(money(&session, cx), 2000);
        session.update(cx, |session, cx| session.undo(cx));
        assert_eq!(money(&session, cx), 1000);
        cx.update(|cx| assert!(!session.read(cx).can_undo()));

        session.update(cx, |session, cx| session.redo(cx));
        assert_eq!(money(&session, cx), 2000);

        // A new change after undoing drops what Redo could have brought back.
        set_money(&session, 5000, cx);
        cx.update(|cx| assert!(!session.read(cx).can_redo()));
    }

    #[gpui_kit::test]
    fn changes_that_change_nothing_arent_steps(cx: &mut TestAppContext) {
        let session = session(cx);
        set_money(&session, 1000, cx);
        cx.update(|cx| assert!(!session.read(cx).can_undo()));
    }

    #[gpui_kit::test]
    fn discard_can_be_undone(cx: &mut TestAppContext) {
        let session = session(cx);
        set_money(&session, 2000, cx);
        session.update(cx, |session, cx| session.visit_all_cities(cx));
        session.update(cx, |session, cx| session.discard(cx));
        assert_eq!(money(&session, cx), 1000);
        cx.update(|cx| assert_eq!(session.read(cx).change_count(), 0));

        session.update(cx, |session, cx| session.undo(cx));
        assert_eq!(money(&session, cx), 2000);
        cx.update(|cx| assert_eq!(session.read(cx).change_count(), 2));
    }

    #[gpui_kit::test]
    fn saving_starts_a_new_history(cx: &mut TestAppContext) {
        let session = session(cx);
        set_money(&session, 2000, cx);
        session.update(cx, |session, cx| {
            let written = session.prepared().expect("prepares");
            session.mark_saved(written, cx);
        });
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.change_count(), 0);
            assert!(!session.can_undo());
            assert_eq!(session.career().money, 2000);
        });
    }

    #[gpui_kit::test]
    fn unlocks_only_whats_not_unlocked_yet(cx: &mut TestAppContext) {
        let session = session(cx);
        let cities: BTreeSet<String> = ["lyon", "berlin"].map(String::from).into();
        session.update(cx, |session, cx| {
            session.unlock_all(Place::Dealer, &cities, cx)
        });
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.unlocked(Place::Dealer), cities);
            assert_eq!(session.change_count(), 1);
            let prepared = session.prepared().expect("prepares");
            assert_eq!(
                save::unlocked(&prepared.game, Place::Dealer),
                ["berlin", "lyon"]
            );
        });
    }

    #[gpui_kit::test]
    fn quick_start_is_one_step(cx: &mut TestAppContext) {
        let session = session(cx);
        let plan = QuickStartPlan {
            money: Some(10_000_000),
            truck_skills: true,
            adr: true,
            visit_cities: true,
            ..QuickStartPlan::default()
        };
        session.update(cx, |session, cx| session.apply_quick_start(plan, cx));
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.career().money, 10_000_000);
            assert_eq!(session.career().skills, save::Skills::maxed());
        });
        session.update(cx, |session, cx| session.undo(cx));
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.change_count(), 0);
            assert!(!session.can_undo());
        });
    }

    #[gpui_kit::test]
    fn all_data_edits_to_career_fields_are_career_changes(cx: &mut TestAppContext) {
        let session = session(cx);
        let money = FieldPath {
            unit: "_nameless.2".into(),
            field: "money_account".into(),
            index: None,
        };
        session.update(cx, |session, cx| {
            session.set_raw_value(money.clone(), "4321".into(), cx)
        });
        assert_eq!(money_of(&session, cx), 4321);
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.change_count(), 1);
            assert_eq!(session.raw_value(&money).as_deref(), Some("4321"));
            assert!(session.is_edited(&money));
        });
        session.update(cx, |session, cx| session.undo(cx));
        assert_eq!(money_of(&session, cx), 1000);
    }

    #[gpui_kit::test]
    fn typed_changes_show_in_all_data(cx: &mut TestAppContext) {
        let session = session(cx);
        set_money(&session, 2000, cx);
        let money = FieldPath {
            unit: "_nameless.2".into(),
            field: "money_account".into(),
            index: None,
        };
        cx.update(|cx| {
            let session = session.read(cx);
            assert_eq!(session.raw_value(&money).as_deref(), Some("2000"));
            assert!(session.is_edited(&money));
        });
    }

    fn money_of(session: &Entity<Session>, cx: &mut TestAppContext) -> i64 {
        cx.update(|cx| session.read(cx).career().money)
    }
}
