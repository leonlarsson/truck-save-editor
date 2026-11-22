//! Every road and junction on the game's map, by id: what the save lists as discovered when the
//! player has driven it (`economy.discovered_items[]`).
//!
//! The map is split into sectors, each with a `.base` file (`map/europe/sec+0000+0000.base` in
//! `base_map.scs` and in each map DLC's archive). A `.base` file is a 20-byte header, then the
//! sector's items, one after another, each starting with its type (`u32`), its id (`u64`) and its
//! bounding box. Items of every other kind have layouts of their own, so instead of reading each
//! kind, the items wanted are recognised where they start: type 3 (a road) or 4 (a prefab: a
//! junction, an interchange, a company's yard), a non-zero id, then a box. The box is six numbers
//! out of eight: the three lowest corners, two others, then the three highest (found in game
//! version 1.5x, sector format 907, 2026-11-22). A box whose low corner is above its high one, or
//! that's implausibly big, isn't one, so stray bytes don't pass for an item.
//!
//! Checked against a real save: its discovered ids are among the items found, and no sector gives
//! more matches than its header's item count.

use std::collections::HashSet;
use std::path::Path;

use save::Game;

/// Item types that count as discovered roads.
const ROAD: u32 = 3;
const PREFAB: u32 = 4;
/// Where a `.base` file's items start.
const HEADER_LENGTH: usize = 20;
/// The header's item count.
const ITEM_COUNT_AT: usize = 16;
/// From an item's start: type (4), id (8), then eight `f32`s holding the box.
const ITEM_HEAD_LENGTH: usize = 4 + 8 + 8 * 4;
/// No road or junction is bigger than this, in metres, along any axis.
const LARGEST_ITEM: f32 = 3000.0;
/// No map coordinate is farther from the middle than this.
const FARTHEST: f32 = 2_000_000.0;

/// The map's folder in the game's archives.
fn map_folder(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "map/europe",
        Game::Ats => "map/usa",
    }
}

/// A road or junction on the map: its id, and where it is (the middle of its box, in the game's
/// metres: `x` east, `z` south).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapRoad {
    pub id: u64,
    pub x: f32,
    pub z: f32,
}

/// Every road and junction id on the map of the installed game and its map DLCs, sorted. Empty
/// when the map can't be read (or its layout has changed so nothing is recognised).
pub fn road_ids(game: Game, install_dir: &Path) -> Vec<u64> {
    map_roads(game, install_dir)
        .into_iter()
        .map(|road| road.id)
        .collect()
}

/// Every road and junction on the map of the installed game and its map DLCs, by id, each once.
pub fn map_roads(game: Game, install_dir: &Path) -> Vec<MapRoad> {
    let folder = map_folder(game);
    let mut roads: Vec<MapRoad> = Vec::new();
    for path in crate::archives_after(install_dir, "base_map.scs") {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list(folder) else {
            continue;
        };
        for file in listing.files.iter().filter(|name| name.ends_with(".base")) {
            if let Ok(Some(bytes)) = archive.read(&format!("{folder}/{file}")) {
                roads.extend(sector_roads(&bytes));
            }
        }
    }
    roads.sort_unstable_by_key(|road| road.id);
    roads.dedup_by_key(|road| road.id);
    roads
}

/// The roads and junctions in one sector's `.base` file.
pub fn sector_roads(bytes: &[u8]) -> Vec<MapRoad> {
    let item_count = read_u32(bytes, ITEM_COUNT_AT).unwrap_or(0) as usize;
    let mut seen: HashSet<u64> = HashSet::new();
    let mut ids = Vec::new();
    let mut at = HEADER_LENGTH;
    while at + ITEM_HEAD_LENGTH <= bytes.len() && ids.len() < item_count {
        if let Some(road) = road_at(bytes, at)
            && seen.insert(road.id)
        {
            ids.push(road);
        }
        at += 1;
    }
    ids
}

/// The road or junction starting at `at`, if one does.
fn road_at(bytes: &[u8], at: usize) -> Option<MapRoad> {
    let kind = read_u32(bytes, at)?;
    if kind != ROAD && kind != PREFAB {
        return None;
    }
    let id = read_u64(bytes, at + 4)?;
    if id == 0 {
        return None;
    }
    let number = |index: usize| read_f32(bytes, at + 12 + index * 4);
    let low = [number(0)?, number(1)?, number(2)?];
    let high = [number(5)?, number(6)?, number(7)?];
    let plausible = low
        .iter()
        .chain(high.iter())
        .all(|value| value.is_finite() && value.abs() < FARTHEST);
    let is_box =
        (0..3).all(|axis| low[axis] <= high[axis] && high[axis] - low[axis] < LARGEST_ITEM);
    // Some extent on the ground: a box of a single point is no road.
    let has_area = (high[0] - low[0]) + (high[2] - low[2]) > 0.5;
    (plausible && is_box && has_area).then(|| MapRoad {
        id,
        x: (low[0] + high[0]) / 2.0,
        z: (low[2] + high[2]) / 2.0,
    })
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

fn read_f32(bytes: &[u8], at: usize) -> Option<f32> {
    Some(f32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One made-up item: its type, id and box (low corner, two other numbers, high corner).
    fn item(kind: u32, id: u64, low: [f32; 3], high: [f32; 3]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(kind.to_le_bytes());
        bytes.extend(id.to_le_bytes());
        for value in low.iter().chain([-1.0_f32, -2.0].iter()).chain(high.iter()) {
            bytes.extend(value.to_le_bytes());
        }
        // The rest of an item: never read.
        bytes.extend([0u8; 12]);
        bytes
    }

    fn sector(items: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = vec![0u8; HEADER_LENGTH];
        bytes[..4].copy_from_slice(&907u32.to_le_bytes());
        bytes[ITEM_COUNT_AT..ITEM_COUNT_AT + 4]
            .copy_from_slice(&(items.len() as u32).to_le_bytes());
        for item in items {
            bytes.extend(item);
        }
        bytes
    }

    #[test]
    fn finds_roads_and_junctions_but_not_other_items() {
        let bytes = sector(&[
            item(ROAD, 101, [-100.0, 10.0, 50.0], [-80.0, 20.0, 90.0]),
            // A building: another type.
            item(1, 102, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
            item(PREFAB, 103, [5.0, 0.0, 5.0], [40.0, 3.0, 30.0]),
            // Not a box: its low corner is above its high one.
            item(ROAD, 104, [10.0, 0.0, 10.0], [0.0, 0.0, 0.0]),
        ]);
        let found: Vec<(u64, f32, f32)> = sector_roads(&bytes)
            .iter()
            .map(|road| (road.id, road.x, road.z))
            .collect();
        assert_eq!(found, [(101, -90.0, 70.0), (103, 22.5, 17.5)]);
    }

    #[test]
    fn reads_nothing_from_too_little() {
        assert!(sector_roads(&[0u8; 8]).is_empty());
        assert!(sector_roads(&sector(&[])).is_empty());
    }
}
