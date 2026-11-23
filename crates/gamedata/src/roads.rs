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
/// A city's area (a city can have several: its centre, outskirts), and a garage, each naming its
/// city.
const CITY_AREA: u32 = 12;
const GARAGE: u32 = 22;
/// A cutscene (a viewpoint when its type, the low bits of its flags, is 0) and a map overlay (a
/// photo trophy's landmark when its type, the flags' first byte, is 4). Every item's head holds,
/// after the box, its flags (`u32`) and a view distance (`u8`): 57 bytes in all; a landmark
/// overlay's name (the photo trophy's id, a token) comes right after (documented by the community
/// for map format 907, and checked against the game's own lists: every viewpoint's map item and
/// every photo trophy's name are found).
const CUTSCENE: u32 = 46;
const MAP_OVERLAY: u32 = 18;
const FLAGS_AT: usize = 52;
const OVERLAY_NAME_AT: usize = 57;
const VIEWPOINT_TYPE: u32 = 0;
const LANDMARK_OVERLAY: u32 = 4;
/// Where a city area or garage names its city: the city's token, this many bytes into the item
/// (found in sector format 907: Berlin, Oslo, Paris and London at the same place).
const CITY_TOKEN_AT: usize = 57;
/// The largest a city's area gets, in metres.
const LARGEST_CITY_AREA: f32 = 30_000.0;
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

/// A city's area or a garage on the map: which city (its token, `berlin`), where (the middle of
/// its box, in the game's metres) and how big.
#[derive(Clone, Debug, PartialEq)]
pub struct MapPlace {
    pub kind: MapPlaceKind,
    pub city: String,
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapPlaceKind {
    CityArea,
    Garage,
}

/// A viewpoint or a photo trophy on the map: its viewpoint's cutscene item, or its photo
/// trophy's id (the landmark overlay's name), and where it is.
#[derive(Clone, Debug, PartialEq)]
pub enum MapMarker {
    Viewpoint { item: u64, x: f32, z: f32 },
    Landmark { id: String, x: f32, z: f32 },
}

/// What the map holds that the editor shows: every road and junction (by id, each once), the
/// cities' areas and garages, and the viewpoints and photo trophies.
#[derive(Clone, Debug, Default)]
pub struct MapContents {
    pub roads: Vec<MapRoad>,
    pub places: Vec<MapPlace>,
    pub markers: Vec<MapMarker>,
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
    map_contents(game, install_dir).roads
}

/// The map of the installed game and its map DLCs: roads, junctions, city areas and garages.
pub fn map_contents(game: Game, install_dir: &Path) -> MapContents {
    let folder = map_folder(game);
    let mut roads: Vec<MapRoad> = Vec::new();
    let mut places: Vec<MapPlace> = Vec::new();
    let mut markers: Vec<MapMarker> = Vec::new();
    for path in crate::archives_after(install_dir, "base_map.scs") {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list(folder) else {
            continue;
        };
        for file in listing.files.iter().filter(|name| name.ends_with(".base")) {
            if let Ok(Some(bytes)) = archive.read(&format!("{folder}/{file}")) {
                let (sector_roads, sector_places, sector_markers) = sector_contents(&bytes);
                roads.extend(sector_roads);
                places.extend(sector_places);
                markers.extend(sector_markers);
            }
        }
    }
    roads.sort_unstable_by_key(|road| road.id);
    roads.dedup_by_key(|road| road.id);
    MapContents {
        roads,
        places,
        markers,
    }
}

/// The roads and junctions in one sector's `.base` file.
pub fn sector_roads(bytes: &[u8]) -> Vec<MapRoad> {
    sector_contents(bytes).0
}

/// The roads, junctions, city areas and garages, viewpoints and landmarks in one sector's `.base`
/// file. Viewpoints and landmarks are only as good as the match: the app keeps those the game's
/// photo album names, so a stray match never shows.
pub fn sector_contents(bytes: &[u8]) -> (Vec<MapRoad>, Vec<MapPlace>, Vec<MapMarker>) {
    let item_count = read_u32(bytes, ITEM_COUNT_AT).unwrap_or(0) as usize;
    let mut seen: HashSet<u64> = HashSet::new();
    let mut seen_places: HashSet<u64> = HashSet::new();
    let mut roads = Vec::new();
    let mut places = Vec::new();
    let mut markers = Vec::new();
    let mut at = HEADER_LENGTH;
    // The header's count caps the roads found, as a guard against stray matches; places (which
    // only label the map) don't use it up, so no road is missed for them.
    while at + ITEM_HEAD_LENGTH <= bytes.len() && roads.len() < item_count {
        // Roads and places apart: a stray match taken for a place mustn't hide a road.
        if let Some(item) = item_at(bytes, at)
            && (if matches!(item.kind, ROAD | PREFAB) {
                seen.insert(item.id)
            } else {
                seen_places.insert(item.id)
            })
        {
            let middle = (
                (item.low[0] + item.high[0]) / 2.0,
                (item.low[2] + item.high[2]) / 2.0,
            );
            let flags = read_u32(bytes, at + FLAGS_AT).unwrap_or(u32::MAX);
            match item.kind {
                ROAD | PREFAB => roads.push(MapRoad {
                    id: item.id,
                    x: middle.0,
                    z: middle.1,
                }),
                CUTSCENE => {
                    if flags & 0x0F == VIEWPOINT_TYPE {
                        markers.push(MapMarker::Viewpoint {
                            item: item.id,
                            x: middle.0,
                            z: middle.1,
                        });
                    }
                }
                MAP_OVERLAY => {
                    let name = read_u64(bytes, at + OVERLAY_NAME_AT).and_then(decode_token);
                    if flags & 0xFF == LANDMARK_OVERLAY
                        && let Some(id) = name
                    {
                        markers.push(MapMarker::Landmark {
                            id,
                            x: middle.0,
                            z: middle.1,
                        });
                    }
                }
                kind => {
                    if let Some(city) = read_u64(bytes, at + CITY_TOKEN_AT).and_then(decode_token) {
                        places.push(MapPlace {
                            kind: if kind == GARAGE {
                                MapPlaceKind::Garage
                            } else {
                                MapPlaceKind::CityArea
                            },
                            city,
                            x: middle.0,
                            z: middle.1,
                            width: item.high[0] - item.low[0],
                            depth: item.high[2] - item.low[2],
                        });
                    }
                }
            }
        }
        at += 1;
    }
    (roads, places, markers)
}

/// The start of an item the editor reads.
struct Item {
    kind: u32,
    id: u64,
    low: [f32; 3],
    high: [f32; 3],
}

/// The road, junction, city area or garage starting at `at`, if one does.
fn item_at(bytes: &[u8], at: usize) -> Option<Item> {
    let kind = read_u32(bytes, at)?;
    let largest = match kind {
        ROAD | PREFAB | GARAGE | CUTSCENE | MAP_OVERLAY => LARGEST_ITEM,
        CITY_AREA => LARGEST_CITY_AREA,
        _ => return None,
    };
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
    let is_box = (0..3).all(|axis| low[axis] <= high[axis] && high[axis] - low[axis] < largest);
    // Some extent on the ground: a box of a single point is no road.
    // Some extent on the ground for roads and areas; a viewpoint or a landmark can be a point.
    let has_area =
        (high[0] - low[0]) + (high[2] - low[2]) > 0.5 || matches!(kind, CUTSCENE | MAP_OVERLAY);
    (plausible && is_box && has_area).then_some(Item {
        kind,
        id,
        low,
        high,
    })
}

/// The game's tokens: short names (`berlin`) packed into a number, a character per place in base
/// 38 (`0`–`9`, `a`–`z`, `_`), the first character lowest. `None` for a number that isn't one.
pub fn decode_token(mut value: u64) -> Option<String> {
    const CHARACTERS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz_";
    let mut text = String::new();
    while value > 0 {
        let index = (value % 38) as usize;
        value /= 38;
        // 0 is no character: only past the end.
        if index == 0 {
            return None;
        }
        text.push(CHARACTERS[index - 1] as char);
    }
    (!text.is_empty()).then_some(text)
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
    fn decodes_the_games_tokens() {
        let token = |text: &str| -> u64 {
            const CHARACTERS: &str = "0123456789abcdefghijklmnopqrstuvwxyz_";
            text.chars().rev().fold(0, |value, c| {
                value * 38 + CHARACTERS.find(c).expect("a token character") as u64 + 1
            })
        };
        assert_eq!(decode_token(token("berlin")).as_deref(), Some("berlin"));
        assert_eq!(
            decode_token(token("la_rochelle")).as_deref(),
            Some("la_rochelle")
        );
        assert_eq!(decode_token(0), None);
    }

    #[test]
    fn reads_nothing_from_too_little() {
        assert!(sector_roads(&[0u8; 8]).is_empty());
        assert!(sector_roads(&sector(&[])).is_empty());
    }
}
