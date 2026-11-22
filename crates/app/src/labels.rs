//! What the game calls things, in the current language. One place, so every screen uses the same
//! words. (Each key is written out literally, so the i18n test can check it has text.)

use std::time::SystemTime;

use gpui_kit::{App, SharedString};
use save::{Location, Slot, SlotKind};

use crate::i18n::t;

/// How many ADR classes there are (one bit each).
pub const ADR_CLASS_COUNT: usize = 6;

/// A leveled skill's name, in the order the game lists them (matching `save::Skills::levels`).
pub fn skill_name(cx: &App, index: usize) -> SharedString {
    match index {
        0 => t!(cx, "skill-long-distance"),
        1 => t!(cx, "skill-high-value-cargo"),
        2 => t!(cx, "skill-fragile-cargo"),
        3 => t!(cx, "skill-just-in-time"),
        _ => t!(cx, "skill-ecodriving"),
    }
}

pub fn skill_description(cx: &App, index: usize) -> SharedString {
    match index {
        0 => t!(cx, "skill-long-distance-description"),
        1 => t!(cx, "skill-high-value-cargo-description"),
        2 => t!(cx, "skill-fragile-cargo-description"),
        3 => t!(cx, "skill-just-in-time-description"),
        _ => t!(cx, "skill-ecodriving-description"),
    }
}

/// A coach skill's name (Coaches DLC), in the order of `save::CoachSkills::levels`.
pub fn coach_skill_name(cx: &App, index: usize) -> SharedString {
    match index {
        0 => t!(cx, "coach-skill-longer-duties"),
        1 => t!(cx, "coach-skill-demanding-timetables"),
        2 => t!(cx, "coach-skill-premium-service"),
        3 => t!(cx, "coach-skill-cushioned-journey"),
        4 => t!(cx, "coach-skill-passenger-favourite"),
        _ => t!(cx, "coach-skill-ecodriving"),
    }
}

pub fn coach_skill_description(cx: &App, index: usize) -> SharedString {
    match index {
        0 => t!(cx, "coach-skill-longer-duties-description"),
        1 => t!(cx, "coach-skill-demanding-timetables-description"),
        2 => t!(cx, "coach-skill-premium-service-description"),
        3 => t!(cx, "coach-skill-cushioned-journey-description"),
        4 => t!(cx, "coach-skill-passenger-favourite-description"),
        _ => t!(cx, "coach-skill-ecodriving-description"),
    }
}

/// An ADR class, by bit (bit 0 first): classes 1, 2, 3, 4, 6 and 8, as the game's `adr_info_icon[]`
/// in `def/economy_data.sii` orders them.
pub fn adr_class(cx: &App, bit: usize) -> SharedString {
    // The game's own name when it's there (`skill_adr_classt<class number>`).
    const CLASS_NUMBERS: [u8; 6] = [1, 2, 3, 4, 6, 8];
    if let Some(name) = CLASS_NUMBERS
        .get(bit)
        .and_then(|class| crate::game_data::text(cx, &format!("skill_adr_classt{class}")))
    {
        return name.into();
    }
    match bit {
        0 => t!(cx, "adr-explosives"),
        1 => t!(cx, "adr-gases"),
        2 => t!(cx, "adr-flammable-liquids"),
        3 => t!(cx, "adr-flammable-solids"),
        4 => t!(cx, "adr-toxic"),
        _ => t!(cx, "adr-corrosive"),
    }
}

/// `3 of 6 classes`, `All classes`, `None`.
pub fn adr_summary(cx: &App, bits: u8) -> SharedString {
    let count = (0..ADR_CLASS_COUNT)
        .filter(|bit| bits & (1 << bit) != 0)
        .count();
    match count {
        0 => t!(cx, "adr-none"),
        n if n == ADR_CLASS_COUNT => t!(cx, "adr-all"),
        n => t!(cx, "adr-some", count = n, total = ADR_CLASS_COUNT),
    }
}

/// The name to show for a slot: the player's own, or what kind of save it is.
pub fn slot_name(cx: &App, slot: &Slot) -> SharedString {
    if let Some(name) = &slot.info.name {
        return name.clone().into();
    }
    match slot.kind {
        SlotKind::Manual(_) => t!(cx, "slot-unnamed"),
        SlotKind::Autosave => t!(cx, "slot-autosave"),
        SlotKind::AutosaveDrive => t!(cx, "slot-autosave-drive"),
        SlotKind::AutosaveJob => t!(cx, "slot-autosave-job"),
        SlotKind::Quicksave => t!(cx, "slot-quicksave"),
        SlotKind::MultiplayerBackup => t!(cx, "slot-multiplayer-backup"),
        SlotKind::Other => slot.folder.clone().into(),
    }
}

pub fn location(cx: &App, location: Location) -> SharedString {
    match location {
        Location::SteamCloud => t!(cx, "location-steam-cloud"),
        Location::Local => t!(cx, "location-this-pc"),
        Location::Loose => t!(cx, "location-loose"),
    }
}

/// A profile as lists show it: its name and where it's kept (`Anna · Steam Cloud`), with the game
/// when it isn't ETS2.
pub fn profile_place(cx: &App, profile: &save::Profile) -> SharedString {
    let place = location(cx, profile.location);
    match profile.game {
        save::Game::Ets2 => t!(cx, "profile-place", name = &profile.name, place = place),
        save::Game::Ats => t!(
            cx,
            "profile-game-place",
            name = &profile.name,
            game = "ATS",
            place = place
        ),
    }
}

/// A length of time in game minutes: `8 h 05 min`, or `45 min`.
pub fn duration(cx: &App, minutes: i64) -> SharedString {
    let minutes = minutes.max(0);
    if minutes < 60 {
        t!(cx, "duration-minutes", minutes = minutes)
    } else {
        t!(
            cx,
            "duration-hours",
            hours = minutes / 60,
            minutes = format!("{:02}", minutes % 60)
        )
    }
}

/// A day of the week and a time (`Thursday 07:26`), as the game's clock shows them.
pub fn day_time(cx: &App, day: i64, minutes: i64) -> SharedString {
    let name = match day.rem_euclid(7) {
        0 => t!(cx, "day-monday"),
        1 => t!(cx, "day-tuesday"),
        2 => t!(cx, "day-wednesday"),
        3 => t!(cx, "day-thursday"),
        4 => t!(cx, "day-friday"),
        5 => t!(cx, "day-saturday"),
        _ => t!(cx, "day-sunday"),
    };
    t!(
        cx,
        "day-and-time",
        day = name,
        time = format!("{:02}:{:02}", minutes / 60 % 24, minutes % 60)
    )
}

/// What was done to picked rows, for saying how it went.
#[derive(Clone, Copy)]
pub enum PickAction {
    Move,
    Remove,
}

/// How an action on picked rows went: all of them, or how many and why not the rest.
pub fn picks_outcome(cx: &App, action: PickAction, done: usize, count: usize) -> SharedString {
    match (action, done == count) {
        (PickAction::Move, true) => t!(cx, "picks-moved-all", count = count),
        (PickAction::Move, false) => t!(cx, "picks-moved", moved = done, count = count),
        (PickAction::Remove, true) => t!(cx, "picks-removed-all", count = count),
        (PickAction::Remove, false) => t!(cx, "picks-removed", removed = done, count = count),
    }
}

/// How long ago something happened, roughly: `just now`, `5 minutes ago`, `3 years ago`.
/// (Relative times need no time zone, and read better than dates in lists.)
pub fn ago(cx: &App, when: SystemTime) -> SharedString {
    let seconds = SystemTime::now()
        .duration_since(when)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let count = |size: u64| (seconds / size) as i64;
    if seconds >= 365 * 86_400 {
        t!(cx, "time-years-ago", count = count(365 * 86_400))
    } else if seconds >= 30 * 86_400 {
        t!(cx, "time-months-ago", count = count(30 * 86_400))
    } else if seconds >= 7 * 86_400 {
        t!(cx, "time-weeks-ago", count = count(7 * 86_400))
    } else if seconds >= 86_400 {
        t!(cx, "time-days-ago", count = count(86_400))
    } else if seconds >= 3_600 {
        t!(cx, "time-hours-ago", count = count(3_600))
    } else if seconds >= 60 {
        t!(cx, "time-minutes-ago", count = count(60))
    } else {
        t!(cx, "time-just-now")
    }
}

/// The plate colour schemes the editor offers, by key, the game's own plate first.
pub const PLATE_SCHEMES: [&str; 8] = [
    "plain", "yellow", "white", "black", "blue", "red", "green", "gold",
];

/// A plate colour scheme's colours: (background, letters) as red, green, blue; `None` for the
/// game's own plate.
pub fn plate_scheme_colours(key: &str) -> Option<([u8; 3], [u8; 3])> {
    const BLACK: [u8; 3] = [0, 0, 0];
    const WHITE: [u8; 3] = [255, 255, 255];
    match key {
        "yellow" => Some(([255, 204, 0], BLACK)),
        "white" => Some((WHITE, BLACK)),
        "black" => Some((BLACK, WHITE)),
        "blue" => Some(([0, 51, 153], WHITE)),
        "red" => Some(([190, 20, 30], WHITE)),
        "green" => Some(([0, 110, 60], WHITE)),
        "gold" => Some((BLACK, [212, 175, 55])),
        _ => None,
    }
}

/// A scheme's name by its key (`None`: colours none of the schemes has).
pub fn plate_scheme_name(cx: &App, key: Option<&str>) -> SharedString {
    match key {
        Some("plain") => t!(cx, "plate-plain"),
        Some("yellow") => t!(cx, "plate-black-on-yellow"),
        Some("white") => t!(cx, "plate-black-on-white"),
        Some("black") => t!(cx, "plate-white-on-black"),
        Some("blue") => t!(cx, "plate-white-on-blue"),
        Some("red") => t!(cx, "plate-white-on-red"),
        Some("green") => t!(cx, "plate-white-on-green"),
        Some("gold") => t!(cx, "plate-gold-on-black"),
        _ => t!(cx, "plate-custom"),
    }
}

/// The scheme of a plate as the save holds it (`license_plate`'s value): the game's own, one of
/// the editor's, or `None` for colours none of them has.
pub fn plate_scheme_of(plate: &str) -> Option<&'static str> {
    match save::plate_colours(plate) {
        None => Some("plain"),
        Some(colours) => PLATE_SCHEMES
            .iter()
            .copied()
            .find(|key| plate_scheme_colours(key) == Some(colours)),
    }
}
