//! The game's photo album (its Gallery): every viewpoint and photo trophy, from
//! `def/photo_album/photo_album.sii` and the files it includes (`viewpoints.sui`,
//! `landmarks.sui`). Each `photo_album_item : album.<kind>.<id>` has a `name` (a text key), maybe
//! a `desc`, the DLC it comes with (`dlc_id`, `base` for the base game's) and `objects_uid[]`,
//! map items: for a viewpoint, its cutscene item, whose id the save lists once found
//! (`economy.discovered_cutscene_items[]`). `photo_album_group` units gather items by country
//! (`album.landmarks.germany`, with a `name` key), and the Gallery's tabs list the groups.

use std::collections::HashMap;

use crate::read_document;

/// A viewpoint or a photo trophy.
#[derive(Clone, Debug, PartialEq)]
pub struct AlbumItem {
    /// Its id after `album.<kind>.` (`prague`): a photo trophy's picture in the profile is
    /// `landmarks.<id>.png`, and its map overlay is named by it.
    pub id: String,
    /// Its name and description, as text keys (`@@landmark_prague@@`).
    pub name: String,
    pub description: Option<String>,
    /// The DLC it comes with, `None` for the base game's.
    pub dlc: Option<String>,
    /// Its map items (a viewpoint's cutscene item).
    pub map_items: Vec<u64>,
    /// The name of its group (a country), as a text key.
    pub group: Option<String>,
}

/// Every viewpoint and photo trophy, in the album's order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhotoAlbum {
    pub viewpoints: Vec<AlbumItem>,
    pub landmarks: Vec<AlbumItem>,
}

/// Reads the album from the base game's definitions (DLC archives add none: each item names its
/// DLC). Empty when it isn't there (older games).
pub(crate) fn read(archive: &mut hashfs::Archive) -> PhotoAlbum {
    let Ok(document) = read_document(archive, "/def/photo_album/photo_album.sii") else {
        return PhotoAlbum::default();
    };
    let text = |unit: &sii::Unit, field: &str| {
        unit.single(field)
            .map(|value| sii::value::parse_string(value).to_string())
            .filter(|value| !value.is_empty())
    };
    // Item → its group's name key.
    let mut groups: HashMap<String, String> = HashMap::new();
    for group in document.units_of_kind("photo_album_group") {
        let Some(name) = text(group, "name") else {
            continue;
        };
        for item in group.items("items") {
            groups.insert(item.clone(), name.clone());
        }
    }
    let mut album = PhotoAlbum::default();
    for unit in document.units_of_kind("photo_album_item") {
        let (into, id) = if let Some(id) = unit.id.strip_prefix("album.viewpoints.") {
            (&mut album.viewpoints, id)
        } else if let Some(id) = unit.id.strip_prefix("album.landmarks.") {
            (&mut album.landmarks, id)
        } else {
            continue;
        };
        let map_items = unit
            .items("objects_uid")
            .iter()
            .filter_map(|uid| {
                let hex = uid.trim().trim_start_matches("0x").trim_start_matches("0X");
                u64::from_str_radix(hex, 16).ok()
            })
            .collect();
        into.push(AlbumItem {
            id: id.to_string(),
            name: text(unit, "name").unwrap_or_default(),
            description: text(unit, "desc"),
            dlc: text(unit, "dlc_id").filter(|dlc| dlc != "base"),
            map_items,
            group: groups.get(&unit.id).cloned(),
        });
    }
    album
}
