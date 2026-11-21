//! The open save: what was loaded, what's been changed, and what saving would write.
//!
//! One `Entity<Session>` holds it; the pages read from it and change it through its methods, and
//! the header counts its changes. Nothing is written to disk here: `prepared` builds the slot as
//! it would be saved, and the workspace hands that to `save::save` in the background.

use std::collections::HashSet;

use gpui_kit::Context;
use save::{Career, Game, LoadedSlot, Profile, Slot, Truck};

use crate::format::thousands;
use crate::labels::{ADR_CLASSES, SKILLS};

pub struct Session {
    game: Game,
    profile: Profile,
    slot: Slot,
    loaded: LoadedSlot,
    /// The career as it is on disk.
    saved_career: Career,
    career: Career,
    /// The trucks as they are on disk.
    saved_trucks: Vec<Truck>,
    /// Trucks to repair when saving, by id.
    repaired: HashSet<String>,
    is_saving: bool,
}

/// One change, for the list shown before saving.
pub struct Change {
    pub what: String,
    pub before: String,
    pub after: String,
}

impl Session {
    pub fn open(profile: Profile, slot: Slot, loaded: LoadedSlot) -> save::Result<Self> {
        let career = Career::read(&loaded.game)?;
        Ok(Self {
            game: profile.game,
            saved_trucks: save::trucks(&loaded.game),
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

    /// The trucks as they'll be after saving (repaired ones without wear).
    pub fn trucks(&self) -> Vec<Truck> {
        self.saved_trucks
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

    /// Marks every worn truck for repair.
    pub fn repair_all(&mut self, cx: &mut Context<Self>) {
        for truck in &self.saved_trucks {
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
    pub fn changes(&self) -> Vec<Change> {
        let (before, after) = (&self.saved_career, &self.career);
        let currency = self.game.currency_symbol();
        let mut changes = Vec::new();
        let mut add = |what: &str, before: String, after: String| {
            if before != after {
                changes.push(Change {
                    what: what.to_string(),
                    before,
                    after,
                });
            }
        };

        add(
            "Money",
            format!("{currency} {}", thousands(before.money)),
            format!("{currency} {}", thousands(after.money)),
        );
        add(
            "Experience",
            format!("{} XP", thousands(before.experience)),
            format!("{} XP", thousands(after.experience)),
        );
        for (index, label) in SKILLS.iter().enumerate() {
            add(
                label.name,
                format!("Level {}", before.skills.levels()[index].1),
                format!("Level {}", after.skills.levels()[index].1),
            );
        }
        add(
            "ADR",
            adr_summary(before.skills.adr),
            adr_summary(after.skills.adr),
        );
        add(
            "HQ",
            save::city_display_name(&before.hq_city),
            save::city_display_name(&after.hq_city),
        );
        if !self.repaired.is_empty() {
            add(
                "Repairs",
                "Worn".to_string(),
                format!("{} trucks repaired", self.repaired.len()),
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
        self.saved_trucks = save::trucks(&written.game);
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

/// `3 of 6 classes`, `All classes`, `None`.
fn adr_summary(bits: u8) -> String {
    let count = (0..ADR_CLASSES.len())
        .filter(|bit| bits & (1 << bit) != 0)
        .count();
    match count {
        0 => "None".to_string(),
        n if n == ADR_CLASSES.len() => "All classes".to_string(),
        n => format!("{n} of {} classes", ADR_CLASSES.len()),
    }
}
