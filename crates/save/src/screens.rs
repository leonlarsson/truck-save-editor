//! The game's screens a career hasn't unlocked yet: `economy.screen_access_list`, the screens'
//! names (`"screen.trailerman"`). The game takes a screen off the list when the career first earns
//! it: Truck Manager when the first truck is bought, the agency screen when the first recruitment
//! agency is discovered. A screen left on the list stays shut, whatever the save holds otherwise
//! ("To access the Trailer Manager you need to buy your first trailer", with a trailer in the
//! garage; an unlocked agency's screen black, and hiring online refused). Found in ATS,
//! 2026-11-25, by comparing saves before and after the game sold a truck and a trailer and
//! discovered an agency.
//!
//! So when a save is prepared for writing, the screens the career has earned by what it holds
//! (an own truck, an own trailer, an unlocked agency) come off the list, as the game would have
//! taken them off: whatever the editor added, and saves an earlier edit left that way.

use sii::{Document, Value};

/// Screens a truck of its own unlocks: Truck Manager, its upgrades and diagnostics, and the
/// freight market (which needs a truck of one's own).
const TRUCK_SCREENS: [&str; 4] = [
    "screen.truckman",
    "truck.upgrades",
    "truck.diag",
    "jobs.company",
];

/// Screens a trailer of its own unlocks: Trailer Manager and its upgrades.
const TRAILER_SCREENS: [&str; 2] = ["screen.trailerman", "trailer.upgrades"];

/// The screen a discovered recruitment agency unlocks.
const AGENCY_SCREENS: [&str; 1] = ["screen.recruit"];

/// The screens still locked (`screen.bank` …), unquoted.
pub fn locked_screens(document: &Document) -> Vec<String> {
    document
        .first_of_kind("economy")
        .map(|economy| {
            economy
                .items("screen_access_list")
                .iter()
                .map(|name| sii::value::parse_string(name).to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Takes the screens the career has earned off the locked list (see the module's notes). Returns
/// how many came off.
pub fn unlock_earned_screens(document: &mut Document) -> usize {
    let Some(economy) = document.first_of_kind("economy") else {
        return 0;
    };
    let economy_id = economy.id.clone();
    let player = economy.single("player").and_then(|id| document.unit(id));
    let owns = |list: &str| player.is_some_and(|player| !player.items(list).is_empty());
    let mut earned: Vec<&str> = Vec::new();
    if owns("trucks") {
        earned.extend(TRUCK_SCREENS);
    }
    if owns("trailers") {
        earned.extend(TRAILER_SCREENS);
    }
    if !economy.items("unlocked_recruitments").is_empty() {
        earned.extend(AGENCY_SCREENS);
    }
    let locked = economy.items("screen_access_list");
    let kept: Vec<String> = locked
        .iter()
        .filter(|name| !earned.contains(&sii::value::parse_string(name).as_ref()))
        .cloned()
        .collect();
    let unlocked = locked.len() - kept.len();
    if unlocked > 0
        && let Some(mut economy) = document.unit_mut(&economy_id)
    {
        economy.set("screen_access_list", Value::Array(kept));
    }
    unlocked
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n unlocked_recruitments: 1\r\n unlocked_recruitments[0]: lyon\r\n screen_access_list: 5\r\n screen_access_list[0]: \"screen.bank\"\r\n screen_access_list[1]: \"screen.truckman\"\r\n screen_access_list[2]: \"screen.trailerman\"\r\n screen_access_list[3]: \"screen.recruit\"\r\n screen_access_list[4]: \"jobs.company\"\r\n}\r\n\r\nplayer : p {\r\n trucks: 0\r\n trailers: 1\r\n trailers[0]: t\r\n}\r\n\r\n}\r\n";

    #[test]
    fn unlocks_what_the_career_has_earned_and_nothing_else() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        // A trailer and an agency, no truck: Trailer Manager and the agency screen open; Truck
        // Manager and the freight market stay shut, the bank too.
        assert_eq!(unlock_earned_screens(&mut document), 2);
        assert_eq!(
            locked_screens(&document),
            ["screen.bank", "screen.truckman", "jobs.company"]
        );
        // Nothing more to do the second time.
        assert_eq!(unlock_earned_screens(&mut document), 0);
    }
}
