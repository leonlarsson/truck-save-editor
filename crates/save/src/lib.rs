//! The ETS2/ATS save, on top of the `sii` file format: the player's career, fleet and slot summary
//! as typed values, and (with the `native` feature) finding profiles on disk, loading slots and
//! writing them back safely, with backups.
//!
//! Without `native` it has no disk access, so the same code can run in a browser later.

mod career;
mod current_job;
mod drivers;
mod error;
mod fleet;
mod game;
mod garages;
mod info;
mod jobs;
mod levels;
mod map;
mod preview;
mod profile_file;
#[cfg(feature = "native")]
mod profiles;
mod removals;
#[cfg(feature = "native")]
mod store;
#[cfg(feature = "native")]
mod time;
mod vehicle_edits;

pub use career::{ADR_ALL, Career, CoachSkills, MAX_COACH_SKILL_LEVEL, MAX_SKILL_LEVEL, Skills};
pub use current_job::{CurrentJob, JobKind, cancel_job, current_job, remove_job_deadline};
pub use drivers::{Driver, drivers, max_driver};
pub use error::{Error, Result};
pub use fleet::{
    Vehicle, VehicleKind, Wear, model_display_name, refuel, repair, vehicles, wear_is_known,
};
pub use game::Game;
pub use garages::{
    Garage, LARGE_GARAGE_SLOTS, MoveOptions, MoveProblem, can_move_to_garage, garage_of, garages,
    move_options, move_to_garage, upgrade_garage,
};
pub use info::{SlotInfo, read_info, set_name, update_info};
pub use jobs::{Job, extend_offers, game_time, jobs};
pub use levels::{FALLBACK_STEPS, Level, LevelTable};
pub use map::{City, Place, cities, unlock, unlocked, visit_all_cities};
pub use preview::{Picture, decode_tga};
pub use profile_file::{company_name, set_cached_experience, set_company_name};
#[cfg(feature = "native")]
pub use profile_file::{load_profile_file, save_profile_file};
#[cfg(feature = "native")]
pub use profiles::{
    Location, PROFILE_DIRS_VARIABLE, Profile, Slot, SlotKind, decode_profile_folder, find_profiles,
    profile_slots,
};
pub use removals::{Removal, RemoveProblem, can_remove, discovered_roads, forget_roads, remove};
#[cfg(feature = "native")]
pub use store::{
    Backup, LoadedSlot, back_up, backups, is_game_running, load, restore, save, save_as_new_slot,
    verify,
};
#[cfg(feature = "native")]
pub use time::{utc_parts, utc_timestamp};
pub use vehicle_edits::{
    TrailerLoad, VehicleEdit, apply_vehicle_edit, badge, coloured_plate, trailer_chain,
    trailer_load,
};

/// The newest save format version (`version` in `info.sii`) the editor has been tested with.
/// Newer saves still open, with a warning (see `docs/compatibility.md`).
pub const NEWEST_TESTED_SAVE_VERSION: i64 = 105;

/// Every career field the editor writes, by unit kind. (Truck wear comes in two formats; see
/// `wear_is_known`.) If a game update renames or removes one, editing
/// it must stop: `missing_edited_fields` finds them, and the fixture tests check every save.
pub const EDITED_FIELDS: &[(&str, &str)] = &[
    ("bank", "money_account"),
    ("economy", "experience_points"),
    ("economy", "adr"),
    ("economy", "long_dist"),
    ("economy", "heavy"),
    ("economy", "fragile"),
    ("economy", "urgent"),
    ("economy", "mechanical"),
    ("player", "hq_city"),
];

/// Edited fields the save doesn't have (for kinds it has at least one unit of).
pub fn missing_edited_fields(document: &sii::Document) -> Vec<(&'static str, &'static str)> {
    EDITED_FIELDS
        .iter()
        .copied()
        .filter(|(kind, field)| {
            let mut units = document.units_of_kind(kind).peekable();
            units.peek().is_some() && !units.any(|unit| unit.get(field).is_some())
        })
        .collect()
}

/// Turns a city token into a name to show: `berlin` → `Berlin`, `la_coruna` → `La Coruna`.
/// (The game's own names, with accents, come from its files; this is the fallback.)
pub fn city_display_name(token: &str) -> String {
    token
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Removes the units nothing points at any more, following references from the economy unit.
/// A save the game writes is one tree from there, and it refuses a file with units outside it
/// ("There are multiple unit trees"); an edit that clears the last reference to a unit (a
/// cancelled job, an All data edit) leaves such units behind. Returns how many were removed.
pub fn drop_unreachable(document: &mut sii::Document) -> usize {
    let Some(root) = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone())
    else {
        return 0;
    };
    let reached = document.reachable_from(&root);
    let stray: std::collections::HashSet<String> = document
        .units()
        .iter()
        .filter(|unit| !reached.contains(&unit.id))
        .map(|unit| unit.id.clone())
        .collect();
    let count = stray.len();
    document.remove_units(&stray);
    count
}
