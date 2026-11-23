//! The Gallery's progress: viewpoints in the save, photo trophies in the profile.
//!
//! A viewpoint the player has found is in `economy.discovered_cutscene_items[]` (its cutscene's
//! map item id), with a state in `discovered_cutscene_items_states[]`, item by item: 1 found on
//! the map but not watched, 2 watched (what the game's Career Progress counts as visited). Not
//! documented by SCS; agreed by the community, and it fits a save's counts exactly. The list also
//! holds other cutscenes, which the editor leaves alone.
//!
//! A photo trophy is captured per profile, not per save: `album/landmarks.sii` in the profile
//! folder lists `album_landmarks_data` units, each with the `landmark_id` captured (and its
//! picture, `album/landmarks.<id>.png`).

use sii::{Document, Value, value};

/// A viewpoint's state in the save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewpointState {
    /// Found on the map, not watched.
    Found,
    /// Watched: visited.
    Visited,
}

/// The cutscenes the save lists, by map item id, with their states (`None` for a state the
/// editor doesn't know).
pub fn cutscene_states(document: &Document) -> Vec<(u64, Option<ViewpointState>)> {
    let Some(economy) = document.first_of_kind("economy") else {
        return Vec::new();
    };
    let states = economy.items("discovered_cutscene_items_states");
    economy
        .items("discovered_cutscene_items")
        .iter()
        .enumerate()
        .filter_map(|(at, id)| {
            let id = value::parse_u64(id)?;
            let state = match states.get(at).map(String::as_str) {
                Some("1") => Some(ViewpointState::Found),
                Some("2") => Some(ViewpointState::Visited),
                _ => None,
            };
            Some((id, state))
        })
        .collect()
}

/// Marks viewpoints (by their map item ids) as visited, and forgets others (as never found):
/// the lists are kept item by item, other cutscenes as they are. Returns how many changed.
pub fn set_viewpoints(document: &mut Document, visit: &[u64], forget: &[u64]) -> usize {
    let Some(id) = document
        .first_of_kind("economy")
        .map(|unit| unit.id.clone())
    else {
        return 0;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return 0;
    };
    let ids = economy
        .get("discovered_cutscene_items")
        .map(|value| value.items().to_vec())
        .unwrap_or_default();
    let mut states = economy
        .get("discovered_cutscene_items_states")
        .map(|value| value.items().to_vec())
        .unwrap_or_default();
    // The two lists go item by item; a missing state counts as found.
    states.resize(ids.len(), "1".to_string());
    let mut changed = 0;
    let mut kept_ids = Vec::new();
    let mut kept_states = Vec::new();
    for (id, state) in ids.into_iter().zip(states) {
        let number = value::parse_u64(&id);
        if number.is_some_and(|number| forget.contains(&number)) {
            changed += 1;
            continue;
        }
        let state = if number.is_some_and(|number| visit.contains(&number)) && state != "2" {
            changed += 1;
            "2".to_string()
        } else {
            state
        };
        kept_ids.push(id);
        kept_states.push(state);
    }
    for id in visit {
        let text = id.to_string();
        if !kept_ids.contains(&text) && !forget.contains(id) {
            kept_ids.push(text);
            kept_states.push("2".to_string());
            changed += 1;
        }
    }
    if changed > 0 {
        economy.set("discovered_cutscene_items", Value::Array(kept_ids));
        economy.set(
            "discovered_cutscene_items_states",
            Value::Array(kept_states),
        );
    }
    changed
}

/// The photo trophies a profile's `album/landmarks.sii` lists as captured, by id (`prague`).
pub fn captured_landmarks(album: &Document) -> std::collections::BTreeSet<String> {
    album
        .units_of_kind("album_landmarks_data")
        .filter_map(|unit| unit.single("landmark_id"))
        .map(|id| value::parse_string(id).to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

#[cfg(feature = "native")]
mod disk {
    use std::collections::BTreeSet;
    use std::path::Path;

    /// The photo trophies a profile has captured: those its `album/landmarks.sii` lists, or, when
    /// that can't be read, those it has a picture of (`album/landmarks.<id>.png`). Read only;
    /// nothing in the album is ever written.
    pub fn read_captured_landmarks(profile_dir: &Path) -> BTreeSet<String> {
        let album = profile_dir.join("album");
        if let Ok(bytes) = std::fs::read(album.join("landmarks.sii"))
            && let Ok(document) = sii::read(&bytes)
        {
            return super::captured_landmarks(&document);
        }
        std::fs::read_dir(&album)
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|entry| {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        name.strip_prefix("landmarks.")?
                            .strip_suffix(".png")
                            .map(str::to_string)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(feature = "native")]
pub use disk::read_captured_landmarks;

#[cfg(test)]
mod tests {
    use super::*;

    /// Viewpoint 11 visited, 22 found, and another cutscene (33) visited.
    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n discovered_cutscene_items: 3\r\n discovered_cutscene_items[0]: 11\r\n discovered_cutscene_items[1]: 22\r\n discovered_cutscene_items[2]: 33\r\n discovered_cutscene_items_states: 3\r\n discovered_cutscene_items_states[0]: 2\r\n discovered_cutscene_items_states[1]: 1\r\n discovered_cutscene_items_states[2]: 2\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_the_states() {
        let document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(
            cutscene_states(&document),
            [
                (11, Some(ViewpointState::Visited)),
                (22, Some(ViewpointState::Found)),
                (33, Some(ViewpointState::Visited))
            ]
        );
    }

    #[test]
    fn visits_and_forgets_viewpoints_keeping_the_rest() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        // 22 watched now, 44 newly visited, 11 forgotten; 33 untouched.
        assert_eq!(set_viewpoints(&mut document, &[22, 44], &[11]), 3);
        assert_eq!(
            cutscene_states(&document),
            [
                (22, Some(ViewpointState::Visited)),
                (33, Some(ViewpointState::Visited)),
                (44, Some(ViewpointState::Visited))
            ]
        );
        assert_eq!(set_viewpoints(&mut document, &[22, 44], &[11]), 0);
    }

    #[test]
    fn reads_captured_photo_trophies() {
        let album = "SiiNunit\r\n{\r\nalbum_landmarks_info : _nameless.1 {\r\n landmarks: 2\r\n landmarks[0]: _nameless.2\r\n landmarks[1]: _nameless.3\r\n}\r\n\r\nalbum_landmarks_data : _nameless.2 {\r\n landmark_id: prague\r\n screenshot_file: \"shot1.png\"\r\n}\r\n\r\nalbum_landmarks_data : _nameless.3 {\r\n landmark_id: brno\r\n screenshot_file: \"shot2.png\"\r\n}\r\n\r\n}\r\n";
        let document = sii::parse_text(album).expect("parses");
        let captured: Vec<String> = captured_landmarks(&document).into_iter().collect();
        assert_eq!(captured, ["brno", "prague"]);
    }
}
