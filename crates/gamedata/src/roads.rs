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
/// A prefab's model (a token) comes right after the head, too.
const PREFAB_MODEL_AT: usize = 57;
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

/// A node of the map: a point roads and junctions meet at. After a sector's items come its
/// nodes, 56 bytes each: an id (`u64`), a position (three `i32`s, in 256ths of a metre), a
/// rotation (a quaternion, four `f32`s: w, x, y, z), the item behind it and the item ahead of it
/// (`u64` ids, 0 for none) and flags. A road runs from the node it's ahead of to the node it's
/// behind; a junction is behind or ahead of each of its ends. (Worked out on sector format 907,
/// 2026-11-25: in one sector every node found was inside its item's box, and the nodes'
/// directions ran along their roads.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapNode {
    pub x: f32,
    pub z: f32,
    /// The way it points (east, south), one metre long.
    pub direction: [f32; 2],
    pub backward_item: u64,
    pub forward_item: u64,
}

/// A stretch of road as it runs on the map: of the road or junction `id`, from one point to
/// another (in the game's metres, east and south), leaving and arriving in these directions (one
/// metre long each; zero for a straight line).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapLine {
    pub id: u64,
    pub from: [f32; 2],
    pub from_direction: [f32; 2],
    pub to: [f32; 2],
    pub to_direction: [f32; 2],
}

impl MapLine {
    /// The point a share of the way along (0 at `from`, 1 at `to`): on the curve that leaves
    /// and arrives in the line's directions (a Hermite curve, as the game lays its roads), or on
    /// the straight line when it has none.
    pub fn point(&self, share: f32) -> [f32; 2] {
        let length = self.length();
        let t = share.clamp(0.0, 1.0);
        let (t2, t3) = (t * t, t * t * t);
        // The four Hermite weights: start point, start direction, end point, end direction.
        let weights = [
            2.0 * t3 - 3.0 * t2 + 1.0,
            t3 - 2.0 * t2 + t,
            -2.0 * t3 + 3.0 * t2,
            t3 - t2,
        ];
        let is_straight = self.from_direction == [0.0, 0.0] && self.to_direction == [0.0, 0.0];
        let axis = |axis: usize| {
            if is_straight {
                self.from[axis] + (self.to[axis] - self.from[axis]) * t
            } else {
                weights[0] * self.from[axis]
                    + weights[1] * self.from_direction[axis] * length
                    + weights[2] * self.to[axis]
                    + weights[3] * self.to_direction[axis] * length
            }
        };
        [axis(0), axis(1)]
    }

    /// How far apart its ends are, in metres.
    pub fn length(&self) -> f32 {
        (self.to[0] - self.from[0]).hypot(self.to[1] - self.from[1])
    }
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

/// A prefab on the map (a junction, a building, a company's yard …): its model, by token
/// (`prefab.<token>` in the game's definitions), and where it is.
#[derive(Clone, Debug, PartialEq)]
pub struct MapPrefab {
    /// Its map item id (as the save's `discovered_items` name it).
    pub id: u64,
    pub model: u64,
    pub x: f32,
    pub z: f32,
}

/// What the map holds that the editor shows: every road and junction (by id, each once), the
/// cities' areas and garages, the viewpoints and photo trophies, and every prefab's model (to
/// find the recruitment agencies' buildings, see `agency_cities`).
#[derive(Clone, Debug, Default)]
pub struct MapContents {
    pub roads: Vec<MapRoad>,
    pub places: Vec<MapPlace>,
    pub markers: Vec<MapMarker>,
    pub prefabs: Vec<MapPrefab>,
    /// The nodes found, for `lines` (a road's nodes can be in a neighbouring sector's file).
    pub nodes: Vec<MapNode>,
    /// How the roads and junctions run, made from the nodes once every sector is read (see
    /// `lines_of`). A road or junction without one still has its place in `roads`.
    pub lines: Vec<MapLine>,
}

impl MapContents {
    /// Adds another sector's contents.
    fn extend(&mut self, other: MapContents) {
        self.roads.extend(other.roads);
        self.places.extend(other.places);
        self.markers.extend(other.markers);
        self.prefabs.extend(other.prefabs);
        self.nodes.extend(other.nodes);
    }
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
    let mut map = MapContents::default();
    for path in crate::archives_after(install_dir, "base_map.scs") {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(listing)) = archive.list(folder) else {
            continue;
        };
        for file in listing.files.iter().filter(|name| name.ends_with(".base")) {
            if let Ok(Some(bytes)) = archive.read(&format!("{folder}/{file}")) {
                map.extend(sector_contents(&bytes));
            }
        }
    }
    map.roads.sort_unstable_by_key(|road| road.id);
    map.roads.dedup_by_key(|road| road.id);
    map.lines = lines_of(&map);
    // Only needed to make the lines.
    map.nodes = Vec::new();
    map
}

/// One end of a junction: where it is, and the direction that leads into the junction there.
type JunctionEnd = ([f32; 2], [f32; 2]);

/// How every road and junction runs, from the nodes: a road from the node it's ahead of to the
/// node it's behind, curved the way the nodes point. A junction's own lanes are in its prefab's
/// file, which isn't read: one with two ends is drawn as the curve between them, one with more
/// as a line from each end to their middle (a crossroads becomes a cross).
pub fn lines_of(map: &MapContents) -> Vec<MapLine> {
    use std::collections::HashMap;
    let junctions: HashSet<u64> = map.prefabs.iter().map(|prefab| prefab.id).collect();
    // Each item's nodes: for a road, where it starts (the node it's ahead of) and ends.
    let mut starts: HashMap<u64, &MapNode> = HashMap::new();
    let mut ends: HashMap<u64, &MapNode> = HashMap::new();
    // A junction's ends, each with the direction that leads into the junction.
    let mut junction_ends: HashMap<u64, Vec<JunctionEnd>> = HashMap::new();
    for node in &map.nodes {
        let reversed = [-node.direction[0], -node.direction[1]];
        for (item, is_ahead) in [(node.forward_item, true), (node.backward_item, false)] {
            if item == 0 {
                continue;
            }
            if junctions.contains(&item) {
                // Ahead of the node: its direction leads in. Behind it: the other way does.
                let inward = if is_ahead { node.direction } else { reversed };
                junction_ends
                    .entry(item)
                    .or_default()
                    .push(([node.x, node.z], inward));
            } else if is_ahead {
                starts.insert(item, node);
            } else {
                ends.insert(item, node);
            }
        }
    }

    let mut lines = Vec::new();
    for road in &map.roads {
        if let Some(ends_of) = junction_ends.get(&road.id) {
            match ends_of.as_slice() {
                [(from, inward), (to, other_inward)] => lines.push(MapLine {
                    id: road.id,
                    from: *from,
                    from_direction: *inward,
                    to: *to,
                    // Arriving at the far end means leaving the junction there.
                    to_direction: [-other_inward[0], -other_inward[1]],
                }),
                ends_of => {
                    let count = ends_of.len() as f32;
                    let middle = [
                        ends_of.iter().map(|(at, _)| at[0]).sum::<f32>() / count,
                        ends_of.iter().map(|(at, _)| at[1]).sum::<f32>() / count,
                    ];
                    lines.extend(ends_of.iter().map(|(at, _)| MapLine {
                        id: road.id,
                        from: *at,
                        from_direction: [0.0, 0.0],
                        to: middle,
                        to_direction: [0.0, 0.0],
                    }));
                }
            }
        } else if let (Some(start), Some(end)) = (starts.get(&road.id), ends.get(&road.id)) {
            lines.push(MapLine {
                id: road.id,
                from: [start.x, start.z],
                from_direction: start.direction,
                to: [end.x, end.z],
                to_direction: end.direction,
            });
        }
    }
    // Nodes wrongly matched would make a line across the map: no real one is this long.
    lines.retain(|line| line.length() < LARGEST_ITEM);
    lines
}

/// The roads and junctions in one sector's `.base` file.
pub fn sector_roads(bytes: &[u8]) -> Vec<MapRoad> {
    sector_contents(bytes).roads
}

/// The roads, junctions, city areas and garages, viewpoints, landmarks and prefabs in one
/// sector's `.base` file. Viewpoints and landmarks are only as good as the match: the app keeps
/// those the game's photo album names, so a stray match never shows.
pub fn sector_contents(bytes: &[u8]) -> MapContents {
    let item_count = read_u32(bytes, ITEM_COUNT_AT).unwrap_or(0) as usize;
    let mut seen: HashSet<u64> = HashSet::new();
    let mut seen_places: HashSet<u64> = HashSet::new();
    let mut roads = Vec::new();
    let mut places = Vec::new();
    let mut markers = Vec::new();
    let mut prefabs = Vec::new();
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
            if item.kind == PREFAB
                && let Some(model) = read_u64(bytes, at + PREFAB_MODEL_AT)
            {
                prefabs.push(MapPrefab {
                    id: item.id,
                    model,
                    x: middle.0,
                    z: middle.1,
                });
            }
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
    MapContents {
        roads,
        places,
        markers,
        prefabs,
        nodes: sector_nodes(bytes, HEADER_LENGTH),
        lines: Vec::new(),
    }
}

/// A node's length in a `.base` file, and where its parts are.
const NODE_LENGTH: usize = 56;
const NODE_POSITION_AT: usize = 8;
const NODE_ROTATION_AT: usize = 20;
const NODE_BACKWARD_AT: usize = 36;
const NODE_FORWARD_AT: usize = 44;
/// A node's position is in 256ths of a metre.
const NODE_UNITS_PER_METRE: f32 = 256.0;

/// The nodes in a sector's file, from `from` on. The whole file is looked through: starting
/// after the last item recognised missed one road in six (on the 1.62 map: 97 % of roads and
/// junctions get a line this way, 83 % that way). Like the items, they're
/// recognised where they start: an id, a position on the map, a rotation that is one (a
/// quaternion of length 1) and an item behind or ahead. A stray match does no harm: a node only
/// counts once an item of the map names it (`lines_of`).
///
/// Every place is tried, also right after a match: a node that only turns about the up axis has
/// zeros in its rotation, so the bytes four before it can pass for a rotation too. Skipping a
/// node's length after such a match would step over the real node (and then every node after
/// it, all four bytes off); its items would be nonsense, so it's harmless when the real one is
/// found as well.
fn sector_nodes(bytes: &[u8], from: usize) -> Vec<MapNode> {
    (from..bytes.len().saturating_sub(NODE_LENGTH - 1))
        .filter_map(|at| node_at(bytes, at))
        .collect()
}

fn node_at(bytes: &[u8], at: usize) -> Option<MapNode> {
    // The rotation first: it rules out nearly every place that isn't a node, cheaply.
    let part = |index: usize| read_f32(bytes, at + NODE_ROTATION_AT + index * 4);
    let (w, x, y, z) = (part(0)?, part(1)?, part(2)?, part(3)?);
    let length = w * w + x * x + y * y + z * z;
    if !(0.98..1.02).contains(&length) {
        return None;
    }
    if read_u64(bytes, at)? == 0 {
        return None;
    }
    let backward_item = read_u64(bytes, at + NODE_BACKWARD_AT)?;
    let forward_item = read_u64(bytes, at + NODE_FORWARD_AT)?;
    if backward_item == 0 && forward_item == 0 {
        return None;
    }
    let position = |index: usize| {
        read_u32(bytes, at + NODE_POSITION_AT + index * 4)
            .map(|value| value as i32 as f32 / NODE_UNITS_PER_METRE)
    };
    let (east, south) = (position(0)?, position(2)?);
    if east.abs() > FARTHEST || south.abs() > FARTHEST {
        return None;
    }
    // The way the node points: the game's "forward" (north, -z) turned by the rotation.
    let direction = [-2.0 * (x * z + w * y), -(1.0 - 2.0 * (x * x + y * y))];
    let size = direction[0].hypot(direction[1]);
    // Pointing straight up or down: no direction on the map.
    let direction = if size > 0.001 {
        [direction[0] / size, direction[1] / size]
    } else {
        [0.0, 0.0]
    };
    Some(MapNode {
        x: east,
        z: south,
        direction,
        backward_item,
        forward_item,
    })
}

/// Leaves out the roads of map DLCs that aren't installed: the base map holds the roads of states
/// and countries whose DLC isn't installed too (ATS's `base_map.scs` has Utah's, Colorado's …), and
/// the game shows those regions empty. Each road goes with the city whose area is nearest it (any
/// city on the map); a road is kept when that city is one the installed game has
/// (`installed_cities`, by token). Nothing is left out when the list is empty (no game data).
pub fn keep_installed_roads(map: &mut MapContents, installed_cities: &HashSet<String>) {
    if installed_cities.is_empty() {
        return;
    }
    let centres: Vec<(f32, f32, bool)> = map
        .places
        .iter()
        .filter(|place| place.kind == MapPlaceKind::CityArea)
        .map(|area| (area.x, area.z, installed_cities.contains(&area.city)))
        .collect();
    if centres.is_empty() {
        return;
    }
    map.roads.retain(|road| {
        centres
            .iter()
            .map(|(x, z, is_installed)| {
                let (dx, dz) = (road.x - x, road.z - z);
                (dx * dx + dz * dz, *is_installed)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .is_some_and(|(_, is_installed)| is_installed)
    });
    // The lines of the roads left out go too.
    let kept: HashSet<u64> = map.roads.iter().map(|road| road.id).collect();
    map.lines.retain(|line| kept.contains(&line.id));
}

/// The cities with a recruitment agency: the city area each agency's building (a prefab whose
/// model is one of `agency_models`, by token) stands in, the smallest when areas overlap, else the
/// nearest area within `AGENCY_REACH` (a building can stand just past its city's area: checked on
/// ETS2's map, where Graz's is 135 m out). Buildings farther from every city are left out.
pub fn agency_cities(
    map: &MapContents,
    agency_models: &HashSet<String>,
) -> std::collections::BTreeSet<String> {
    let areas: Vec<&MapPlace> = map
        .places
        .iter()
        .filter(|place| place.kind == MapPlaceKind::CityArea)
        .collect();
    map.prefabs
        .iter()
        .filter(|prefab| {
            decode_token(prefab.model).is_some_and(|model| agency_models.contains(&model))
        })
        .filter_map(|prefab| {
            areas
                .iter()
                .filter(|area| {
                    (prefab.x - area.x).abs() <= area.width / 2.0
                        && (prefab.z - area.z).abs() <= area.depth / 2.0
                })
                .min_by(|a, b| (a.width * a.depth).total_cmp(&(b.width * b.depth)))
                .or_else(|| {
                    areas
                        .iter()
                        .map(|area| (distance_to_area(prefab, area), area))
                        .filter(|(distance, _)| *distance <= AGENCY_REACH)
                        .min_by(|a, b| a.0.total_cmp(&b.0))
                        .map(|(_, area)| area)
                })
                .map(|area| area.city.clone())
        })
        .collect()
}

/// How far an agency's building may stand outside its city's area, in metres.
const AGENCY_REACH: f32 = 250.0;

/// How far a point is from an area's box, in metres (0 inside it).
fn distance_to_area(prefab: &MapPrefab, area: &MapPlace) -> f32 {
    let dx = ((prefab.x - area.x).abs() - area.width / 2.0).max(0.0);
    let dz = ((prefab.z - area.z).abs() - area.depth / 2.0).max(0.0);
    (dx * dx + dz * dz).sqrt()
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

    /// One made-up node: where it is (east, south), the quarter turns it's turned to the left
    /// from north, and the items behind and ahead of it.
    fn node(
        id: u64,
        east: f32,
        south: f32,
        quarter_turns: f32,
        behind: u64,
        ahead: u64,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_le_bytes());
        for metres in [east, 0.0, south] {
            bytes.extend(((metres * NODE_UNITS_PER_METRE) as i32).to_le_bytes());
        }
        // A turn about the up axis: (w, x, y, z) = (cos half, 0, sin half, 0).
        let half = quarter_turns * std::f32::consts::FRAC_PI_4;
        for part in [half.cos(), 0.0, half.sin(), 0.0] {
            bytes.extend(part.to_le_bytes());
        }
        bytes.extend(behind.to_le_bytes());
        bytes.extend(ahead.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes
    }

    #[test]
    fn a_road_runs_between_its_nodes_and_a_junction_between_its_ends() {
        let mut bytes = sector(&[
            item(ROAD, 101, [0.0, 0.0, -100.0], [1.0, 1.0, 0.0]),
            item(PREFAB, 103, [-5.0, 0.0, -140.0], [5.0, 1.0, -100.0]),
        ]);
        // The road runs north from (0, 0) to (0, -100), into the junction, which ends at
        // (0, -140).
        bytes.extend(node(1, 0.0, 0.0, 0.0, 0, 101));
        bytes.extend(node(2, 0.0, -100.0, 0.0, 101, 103));
        bytes.extend(node(3, 0.0, -140.0, 0.0, 103, 0));
        let mut map = sector_contents(&bytes);
        map.lines = lines_of(&map);

        let road = map
            .lines
            .iter()
            .find(|line| line.id == 101)
            .expect("the road's line");
        assert_eq!((road.from, road.to), ([0.0, 0.0], [0.0, -100.0]));
        // Pointing north at both ends: straight up the map, so halfway is halfway.
        let middle = road.point(0.5);
        assert!(middle[0].abs() < 0.01 && (middle[1] + 50.0).abs() < 0.01);

        let junction = map
            .lines
            .iter()
            .find(|line| line.id == 103)
            .expect("the junction's line");
        assert_eq!((junction.from, junction.to), ([0.0, -100.0], [0.0, -140.0]));
        assert!(junction.from_direction[1] < -0.99 && junction.to_direction[1] < -0.99);
    }

    #[test]
    fn a_node_turned_left_points_west() {
        let bytes = node(1, 0.0, 0.0, 1.0, 0, 7);
        let found = node_at(&bytes, 0).expect("a node");
        assert!(found.direction[0] < -0.99 && found.direction[1].abs() < 0.01);
        // Bytes that aren't a rotation aren't a node.
        assert_eq!(node_at(&[0u8; NODE_LENGTH], 0), None);
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
