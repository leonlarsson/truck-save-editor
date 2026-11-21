//! The ETS2/ATS save, on top of the `sii` file format: the player's career, fleet and slot summary
//! as typed values, and (with the `native` feature) finding profiles on disk, loading slots and
//! writing them back safely, with backups.
//!
//! Without `native` it has no disk access, so the same code can run in a browser later.

mod career;
mod error;
mod fleet;
mod game;
mod info;
#[cfg(feature = "native")]
mod profiles;
#[cfg(feature = "native")]
mod store;
#[cfg(feature = "native")]
mod time;

pub use career::{ADR_ALL, Career, MAX_SKILL_LEVEL, Skills};
pub use error::{Error, Result};
pub use fleet::{Truck, Wear, model_display_name, repair, trucks};
pub use game::Game;
pub use info::{SlotInfo, read_info, update_info};
#[cfg(feature = "native")]
pub use profiles::{
    Location, PROFILE_DIRS_VARIABLE, Profile, Slot, SlotKind, decode_profile_folder, find_profiles,
    profile_slots,
};
#[cfg(feature = "native")]
pub use store::{Backup, LoadedSlot, back_up, backups, is_game_running, load, restore, save};
#[cfg(feature = "native")]
pub use time::{utc_parts, utc_timestamp};

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
