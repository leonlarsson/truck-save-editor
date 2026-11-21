//! The open save: what was loaded, what's been changed, and what saving would write.
//!
//! One `Entity<Session>` holds it; the pages read from it and change it through its methods, and
//! the header counts its changes. Nothing is written to disk here: `prepared` builds the slot as
//! it would be saved, and the workspace hands that to `save::save` in the background.

use std::collections::HashSet;

use gpui_kit::{App, Context, SharedString};
use save::{Career, Game, LoadedSlot, Profile, Slot, Vehicle, VehicleKind};

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
            saved_career: career.clone(),
            career,
            profile,
            slot,
            loaded,
            repaired: HashSet::new(),
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
        self.career.changes_from(&self.saved_career) + usize::from(!self.repaired.is_empty())
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
        cx.notify();
    }

    /// The slot as it would be saved: the loaded save with every change applied.
    pub fn prepared(&self) -> save::Result<LoadedSlot> {
        let mut slot = self.loaded.clone();
        self.career.write(&mut slot.game)?;
        for id in &self.repaired {
            save::repair(&mut slot.game, id, false);
        }
        if let Some(info) = &mut slot.info {
            save::update_info(info, &self.career);
        }
        Ok(slot)
    }

    /// After a successful save: what was written is now what's on disk.
    pub fn mark_saved(&mut self, written: LoadedSlot, cx: &mut Context<Self>) {
        self.saved_vehicles = save::vehicles(&written.game);
        self.loaded = written;
        self.saved_career = self.career.clone();
        self.repaired.clear();
        self.is_saving = false;
        cx.notify();
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
