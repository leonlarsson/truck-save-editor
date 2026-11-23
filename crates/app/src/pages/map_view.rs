//! The map's roads: a dot for every road and junction on the game's map (read from its files, see
//! `gamedata::map_roads`), the ones driven in this save in the accent colour, and where the
//! player's truck is. The mouse wheel zooms around the pointer, dragging moves the map, and
//! buttons do both from the keyboard. A click on a city's name, a garage or the truck opens its
//! panel (the list of cities, Garages and Fleet open the same panels from the keyboard).
//!
//! Drawn the way web maps are: in square tiles at fixed zoom levels, each drawn once in the
//! background and kept. Moving the map reuses the tiles already drawn; zooming shows the nearest
//! level's tiles, scaled, with coarser ones underneath, until sharper ones are ready on top, so
//! nothing goes blank or jumps. The truck is drawn over the tiles on every frame, at one size.
//! Hundreds of thousands of dots are far too many to draw as shapes on every frame.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    App, AppContext as _, BorderStyle, Bounds, Context, Corners, Entity, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    RenderImage, Rgba, SharedString, StatefulInteractiveElement as _, Styled as _, Task, Window,
    canvas, div, point, prelude::FluentBuilder as _, px, quad, rems, size,
};

use crate::game_data;
use crate::i18n::t;
use crate::pages::details;
use crate::session::Session;

/// A tile's side, in screen pixels.
const TILE: f32 = 256.0;
/// The deepest zoom level (each level is twice as close as the one before).
const MAX_LEVEL: i32 = 13;
/// How many levels coarser a tile can be to stand in for one not drawn yet.
const BACKING_LEVELS: i32 = 6;
/// How many tiles are kept; the least recently shown go first.
const KEPT_TILES: usize = 160;
/// The side of the cells roads are sorted into, in metres, to find a tile's roads quickly.
const CELL: f32 = 5000.0;
/// A dot's size on screen, in pixels, zoomed out; dots grow as the map is zoomed in.
const DOT: f32 = 2.0;
/// The largest a dot gets, in pixels.
const LARGEST_DOT: f32 = 6.0;
/// How many metres a dot stands for once zoomed in (about a road piece).
const DOT_METRES: f32 = 40.0;
/// The truck's marker's radius on screen, in pixels.
const MARKER: f32 = 6.0;
/// The closest zoom: metres a screen pixel shows.
const CLOSEST: f32 = 1.5;
/// One button press's zoom.
const STEP: f32 = 1.6;
/// A garage's square on the map, in pixels (the headquarters' a little bigger).
const GARAGE: f32 = 7.0;
/// Room a city's name takes, roughly, per character and in height, in pixels: names that would
/// overlap a bigger city's aren't shown.
const NAME_CHARACTER: f32 = 6.5;
const NAME_HEIGHT: f32 = 16.0;
/// How far the pointer can move between pressing and letting go for it to be a click, not a drag,
/// in pixels.
const CLICK_SLOP: f32 = 4.0;
/// How much bigger than its marker a garage or the truck is to the pointer, in pixels on each
/// side: small marks are hard to hit.
const HIT_MARGIN: f32 = 3.0;

/// What the map shows: the point in the middle (in the game's metres, `x` east and `z` south)
/// and how many metres one screen pixel covers.
#[derive(Clone, Copy, Debug, PartialEq)]
struct View {
    x: f32,
    z: f32,
    metres_per_pixel: f32,
}

/// The map's extent: every road inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct World {
    west: f32,
    east: f32,
    north: f32,
    south: f32,
}

/// The roads, sorted into cells so a tile finds its own without looking at all of them.
struct Index {
    roads: Arc<Vec<gamedata::MapRoad>>,
    world: World,
    /// Metres per pixel at level 0, where one tile holds the whole map.
    base: f32,
    cells: HashMap<(i32, i32), Vec<u32>>,
}

impl Index {
    fn new(roads: Arc<Vec<gamedata::MapRoad>>) -> Option<Self> {
        let world = world_of(&roads)?;
        let base = (world.east - world.west).max(world.south - world.north) / TILE;
        let mut cells: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (index, road) in roads.iter().enumerate() {
            cells
                .entry(cell_of(road.x, road.z))
                .or_default()
                .push(index as u32);
        }
        Some(Self {
            roads,
            world,
            base,
            cells,
        })
    }

    fn metres_per_pixel(&self, level: i32) -> f32 {
        self.base / 2f32.powi(level)
    }

    /// A tile's side, in metres.
    fn tile_metres(&self, level: i32) -> f32 {
        TILE * self.metres_per_pixel(level)
    }

    /// The level whose tiles are at least as sharp as the view.
    fn level_for(&self, view: View) -> i32 {
        ((self.base / view.metres_per_pixel).log2().ceil() as i32).clamp(0, MAX_LEVEL)
    }

    /// A tile's extent, in the game's metres: west, north, east, south.
    fn tile_extent(&self, key: TileKey) -> (f32, f32, f32, f32) {
        let side = self.tile_metres(key.level);
        let west = self.world.west + key.x as f32 * side;
        let north = self.world.north + key.y as f32 * side;
        (west, north, west + side, north + side)
    }

    /// Every road in the cells this extent touches.
    fn roads_in(&self, west: f32, north: f32, east: f32, south: f32) -> Vec<&gamedata::MapRoad> {
        let (first, last) = (cell_of(west, north), cell_of(east, south));
        let mut found = Vec::new();
        for x in first.0..=last.0 {
            for z in first.1..=last.1 {
                if let Some(indices) = self.cells.get(&(x, z)) {
                    found.extend(indices.iter().map(|&index| &self.roads[index as usize]));
                }
            }
        }
        found
    }
}

fn cell_of(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (z / CELL).floor() as i32)
}

/// The map's cities and garages, by city: where each city's name goes (the middle of its biggest
/// area) and its size, and where the city's garage is.
struct Places {
    cities: Vec<MapCity>,
    garages: HashMap<String, (f32, f32)>,
}

struct MapCity {
    token: String,
    x: f32,
    z: f32,
    /// Its biggest area, in square metres: bigger cities' names win where names would overlap.
    size: f32,
}

impl Places {
    fn new(places: &[gamedata::MapPlace]) -> Self {
        let mut cities: HashMap<&str, MapCity> = HashMap::new();
        let mut garages = HashMap::new();
        for place in places {
            match place.kind {
                gamedata::MapPlaceKind::CityArea => {
                    let size = place.width * place.depth;
                    let city = cities.entry(&place.city).or_insert(MapCity {
                        token: place.city.clone(),
                        x: place.x,
                        z: place.z,
                        size: 0.0,
                    });
                    if size > city.size {
                        *city = MapCity {
                            token: place.city.clone(),
                            x: place.x,
                            z: place.z,
                            size,
                        };
                    }
                }
                gamedata::MapPlaceKind::Garage => {
                    garages
                        .entry(place.city.clone())
                        .or_insert((place.x, place.z));
                }
            }
        }
        let mut cities: Vec<MapCity> = cities.into_values().collect();
        cities.sort_by(|a, b| b.size.total_cmp(&a.size));
        Self { cities, garages }
    }
}

/// Something on the map that opens a panel when clicked.
#[derive(Clone, Debug, PartialEq)]
enum Target {
    /// A city, by token.
    City(String),
    /// A garage, by its unit's id, and its city's token.
    Garage { id: String, city: String },
    /// The player's truck, by its unit's id.
    Truck(String),
}

/// A city's name on screen: the room it takes, and whether the player has been there.
struct MapName {
    room: Bounds<Pixels>,
    name: SharedString,
    token: String,
    is_visited: bool,
}

/// One of the player's garages on screen.
struct MapGarage {
    at: Point<Pixels>,
    is_headquarters: bool,
    id: String,
    city: String,
}

impl MapGarage {
    fn side(&self) -> Pixels {
        if self.is_headquarters {
            px(GARAGE + 3.0)
        } else {
            px(GARAGE)
        }
    }
}

/// A square of `side` around `at`, grown by `margin` on each side.
fn square_around(at: Point<Pixels>, side: Pixels, margin: f32) -> Bounds<Pixels> {
    let half = side / 2.0 + px(margin);
    Bounds::new(
        point(at.x - half, at.y - half),
        size(half * 2.0, half * 2.0),
    )
}

/// One tile: its zoom level and place in that level's grid.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct TileKey {
    level: i32,
    x: i32,
    y: i32,
}

/// What the map shows: each of the legend's items, on or off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Layers {
    driven: bool,
    not_driven: bool,
    truck: bool,
    garages: bool,
    names: bool,
}

impl Default for Layers {
    fn default() -> Self {
        Self {
            driven: true,
            not_driven: true,
            truck: true,
            garages: true,
            names: true,
        }
    }
}

/// One of the legend's items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    Driven,
    NotDriven,
    Truck,
    Garages,
    Names,
}

impl Layers {
    fn shows(&self, layer: Layer) -> bool {
        match layer {
            Layer::Driven => self.driven,
            Layer::NotDriven => self.not_driven,
            Layer::Truck => self.truck,
            Layer::Garages => self.garages,
            Layer::Names => self.names,
        }
    }

    fn toggle(&mut self, layer: Layer) {
        let shown = match layer {
            Layer::Driven => &mut self.driven,
            Layer::NotDriven => &mut self.not_driven,
            Layer::Truck => &mut self.truck,
            Layer::Garages => &mut self.garages,
            Layer::Names => &mut self.names,
        };
        *shown = !*shown;
    }
}

/// What tiles are drawn for: new tiles when any of it changes.
#[derive(Clone, Debug, PartialEq)]
struct Look {
    driven: usize,
    is_dark: bool,
    scale: f32,
    /// Which roads are drawn: driven, not driven.
    roads: (bool, bool),
}

/// The colours tiles are drawn in, as blue, green, red, alpha (GPUI's image order).
#[derive(Clone, Copy)]
struct Palette {
    /// `None` for roads not shown.
    road: Option<[u8; 4]>,
    driven: Option<[u8; 4]>,
}

fn bgra(color: Rgba, alpha: f32) -> [u8; 4] {
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [
        byte(color.b),
        byte(color.g),
        byte(color.r),
        byte(color.a * alpha),
    ]
}

/// A tile drawn: its picture (`None` where it has no roads), and when it was last shown.
struct Tile {
    image: Option<Arc<RenderImage>>,
    shown: u64,
}

pub struct MapView {
    session: Entity<Session>,
    view: Option<View>,
    /// Where the map's area is on screen, as last laid out (set while painting).
    area: Rc<Cell<Option<Bounds<Pixels>>>>,
    index: Option<Arc<Index>>,
    is_indexing: bool,
    /// What the tiles are drawn for, and the roads driven then.
    look: Option<Look>,
    driven: Option<Arc<HashSet<u64>>>,
    tiles: HashMap<TileKey, Tile>,
    /// Tiles being drawn.
    drawing: HashSet<TileKey>,
    /// Counts frames, to keep the tiles shown most recently.
    frame: u64,
    /// Dragging: where the pointer went down, and the view then.
    drag: Option<(Point<Pixels>, View)>,
    places: Option<Arc<Places>>,
    /// What the map shows; the legend turns each on and off.
    layers: Layers,
    /// What a click opens, where it is on screen, topmost first (as last drawn).
    targets: Vec<(Bounds<Pixels>, Target)>,
    /// Where the pointer is over the map (`None` when it's elsewhere).
    pointer: Option<Point<Pixels>>,
    /// What the pointer is over, worked out again on every frame: the map moves under a still
    /// pointer when zoomed.
    hovered: Option<Target>,
    _work: Vec<Task<()>>,
}

impl MapView {
    pub fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        Self {
            session,
            view: None,
            area: Rc::new(Cell::new(None)),
            index: None,
            is_indexing: false,
            look: None,
            driven: None,
            tiles: HashMap::new(),
            drawing: HashSet::new(),
            frame: 0,
            drag: None,
            places: None,
            layers: Layers::default(),
            targets: Vec::new(),
            pointer: None,
            hovered: None,
            _work: Vec::new(),
        }
    }

    /// The view that shows the whole map in an area of this size.
    fn fitted(world: World, width: f32, height: f32) -> View {
        let metres_per_pixel = ((world.east - world.west) / width.max(1.0))
            .max((world.south - world.north) / height.max(1.0))
            * 1.05;
        View {
            x: (world.west + world.east) / 2.0,
            z: (world.north + world.south) / 2.0,
            metres_per_pixel,
        }
    }

    /// The farthest zoom: a little beyond the whole map.
    fn farthest(&self) -> f32 {
        match (&self.index, self.area.get()) {
            (Some(index), Some(area)) => {
                Self::fitted(
                    index.world,
                    f32::from(area.size.width),
                    f32::from(area.size.height),
                )
                .metres_per_pixel
                    * 1.5
            }
            _ => f32::MAX,
        }
    }

    /// Zooms by `factor` (above 1 zooms out) keeping the point under `at` (on screen) in place;
    /// without one, the middle.
    fn zoom(&mut self, factor: f32, at: Option<Point<Pixels>>, cx: &mut Context<Self>) {
        let (Some(view), Some(area)) = (self.view, self.area.get()) else {
            return;
        };
        let at = at.unwrap_or(area.center());
        let from_middle = (
            f32::from(at.x - area.center().x),
            f32::from(at.y - area.center().y),
        );
        let under = (
            view.x + from_middle.0 * view.metres_per_pixel,
            view.z + from_middle.1 * view.metres_per_pixel,
        );
        let metres_per_pixel = (view.metres_per_pixel * factor).clamp(CLOSEST, self.farthest());
        self.view = Some(View {
            x: under.0 - from_middle.0 * metres_per_pixel,
            z: under.1 - from_middle.1 * metres_per_pixel,
            metres_per_pixel,
        });
        cx.notify();
    }

    fn fit(&mut self, cx: &mut Context<Self>) {
        if let (Some(index), Some(area)) = (&self.index, self.area.get()) {
            self.view = Some(Self::fitted(
                index.world,
                f32::from(area.size.width),
                f32::from(area.size.height),
            ));
            cx.notify();
        }
    }

    /// Gets ready what the next frame needs: the roads sorted, the roads driven, and the tiles
    /// in view (drawn in the background, as many as are missing).
    fn prepare(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(roads) = game_data::map_roads(cx) else {
            return;
        };
        let Some(index) = self.index.clone() else {
            if !self.is_indexing {
                self.is_indexing = true;
                self._work.push(cx.spawn(async move |this, cx| {
                    let index = cx
                        .background_spawn(async move { Index::new(roads).map(Arc::new) })
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        this.index = index;
                        cx.notify();
                    });
                }));
            }
            return;
        };

        if self.places.is_none()
            && let Some(places) = game_data::map_places(cx)
        {
            self.places = Some(Arc::new(Places::new(&places)));
        }

        // A new look (roads driven, light or dark, screen resolution): tiles drawn again.
        let document = self.session.read(cx).preview();
        let look = Look {
            driven: save::discovered_roads(&document),
            is_dark: cx.theme().is_dark(),
            scale: window.scale_factor(),
            roads: (self.layers.driven, self.layers.not_driven),
        };
        if self.look.as_ref() != Some(&look) {
            self.look = Some(look);
            self.driven = None;
            self.drawing.clear();
            for (_, tile) in self.tiles.drain() {
                if let Some(image) = tile.image {
                    cx.drop_image(image, None);
                }
            }
            self._work.push(cx.spawn(async move |this, cx| {
                let driven = cx
                    .background_spawn(async move {
                        let driven: HashSet<u64> = document
                            .first_of_kind("economy")
                            .map(|economy| {
                                economy
                                    .items("discovered_items")
                                    .iter()
                                    .filter_map(|id| id.parse().ok())
                                    .collect()
                            })
                            .unwrap_or_default();
                        Arc::new(driven)
                    })
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.driven = Some(driven);
                    cx.notify();
                });
            }));
        }

        let Some(area) = self.area.get() else {
            return;
        };
        let view = *self.view.get_or_insert_with(|| {
            Self::fitted(
                index.world,
                f32::from(area.size.width),
                f32::from(area.size.height),
            )
        });
        let Some(driven) = self.driven.clone() else {
            return;
        };
        let level = index.level_for(view);
        let missing: Vec<TileKey> = visible_tiles(&index, view, area, level)
            .into_iter()
            .filter(|key| !self.tiles.contains_key(key) && !self.drawing.contains(key))
            .collect();
        if missing.is_empty() {
            return;
        }
        self.drawing.extend(missing.iter().copied());
        let theme = cx.theme();
        let palette = Palette {
            road: self
                .layers
                .not_driven
                .then(|| bgra(theme.muted_foreground.into(), 0.4)),
            driven: self.layers.driven.then(|| bgra(theme.primary.into(), 1.0)),
        };
        let look = self.look.clone();
        let scale = window.scale_factor();
        self._work.retain(|task| !task.is_ready());
        self._work.push(cx.spawn(async move |this, cx| {
            let drawn = cx
                .background_spawn(async move {
                    missing
                        .into_iter()
                        .map(|key| (key, draw_tile(&index, &driven, key, palette, scale)))
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                // Drawn for a look that's gone: of no use.
                if this.look != look {
                    for (_, image) in drawn {
                        if let Some(image) = image {
                            cx.drop_image(Arc::new(image), None);
                        }
                    }
                    return;
                }
                let frame = this.frame;
                for (key, image) in drawn {
                    this.drawing.remove(&key);
                    this.tiles.insert(
                        key,
                        Tile {
                            image: image.map(Arc::new),
                            shown: frame,
                        },
                    );
                }
                this.forget_old_tiles(cx);
                cx.notify();
            });
        }));
    }

    /// Keeps the tiles shown most recently, up to `KEPT_TILES`.
    fn forget_old_tiles(&mut self, cx: &mut Context<Self>) {
        if self.tiles.len() <= KEPT_TILES {
            return;
        }
        let mut by_age: Vec<(TileKey, u64)> = self
            .tiles
            .iter()
            .map(|(key, tile)| (*key, tile.shown))
            .collect();
        by_age.sort_by_key(|(_, shown)| *shown);
        let extra = self.tiles.len() - KEPT_TILES;
        for (key, _) in by_age.into_iter().take(extra) {
            if let Some(Tile {
                image: Some(image), ..
            }) = self.tiles.remove(&key)
            {
                cx.drop_image(image, None);
            }
        }
    }

    /// The cities' names to show, where, and whether the player has been there: the save's
    /// cities in view, biggest first, each only where it doesn't overlap one already placed.
    fn names(&self, cx: &App) -> Vec<MapName> {
        let (Some(places), Some(view), Some(area)) = (&self.places, self.view, self.area.get())
        else {
            return Vec::new();
        };
        if !self.layers.names {
            return Vec::new();
        }
        let visits: HashMap<String, u32> = self
            .session
            .read(cx)
            .cities()
            .into_iter()
            .map(|city| (city.token, city.visits))
            .collect();
        let mut taken: Vec<Bounds<Pixels>> = Vec::new();
        let mut names = Vec::new();
        for city in &places.cities {
            // Only the game's cities (the map holds other areas named like them).
            let Some(&visited) = visits.get(&city.token) else {
                continue;
            };
            let at = to_screen(view, area, city.x, city.z);
            let name: SharedString = game_data::city_name(cx, &city.token).into();
            let width = name.chars().count() as f32 * NAME_CHARACTER + 8.0;
            let room = Bounds::new(
                point(at.x - px(width / 2.0), at.y - px(NAME_HEIGHT / 2.0)),
                size(px(width), px(NAME_HEIGHT)),
            );
            // Whole names only: one cut off at the edge reads as another word.
            let fits = area.contains(&room.origin) && area.contains(&room.bottom_right());
            if !fits || taken.iter().any(|other| other.intersects(&room)) {
                continue;
            }
            taken.push(room);
            names.push(MapName {
                room,
                name,
                token: city.token.clone(),
                is_visited: visited > 0,
            });
        }
        names
    }

    /// The player's garages on screen, the headquarters marked.
    fn garages(&self, cx: &App) -> Vec<MapGarage> {
        let (Some(places), Some(view), Some(area)) = (&self.places, self.view, self.area.get())
        else {
            return Vec::new();
        };
        if !self.layers.garages {
            return Vec::new();
        }
        let session = self.session.read(cx);
        let headquarters = session.career().hq_city.clone();
        session
            .garages()
            .into_iter()
            .filter(|garage| garage.is_owned)
            .filter_map(|garage| {
                let &(x, z) = places.garages.get(&garage.city)?;
                let at = to_screen(view, area, x, z);
                area.contains(&at).then(|| MapGarage {
                    at,
                    is_headquarters: garage.city == headquarters,
                    id: garage.id,
                    city: garage.city,
                })
            })
            .collect()
    }

    /// What's under the pointer, the topmost first.
    fn target_at(&self, at: Point<Pixels>) -> Option<Target> {
        self.targets
            .iter()
            .find(|(bounds, _)| bounds.contains(&at))
            .map(|(_, target)| target.clone())
    }

    /// Opens what was clicked in its panel.
    fn open(&self, target: Target, window: &mut Window, cx: &mut Context<Self>) {
        let session = self.session.clone();
        match target {
            Target::City(token) => details::open_city(session, &token, window, cx),
            Target::Garage { id, .. } => {
                let garage = session
                    .read(cx)
                    .garages()
                    .into_iter()
                    .find(|garage| garage.id == id);
                if let Some(garage) = garage {
                    details::open_garage(session, &garage, window, cx);
                }
            }
            Target::Truck(id) => {
                let truck = session
                    .read(cx)
                    .vehicles()
                    .into_iter()
                    .find(|vehicle| vehicle.id == id);
                if let Some(truck) = truck {
                    details::open_vehicle(session, &truck, window, cx);
                }
            }
        }
    }

    /// What to paint for each tile in view, marking the tiles used as shown: the tile itself
    /// once drawn, otherwise the closest coarser tile drawn, scaled up and cut to the missing
    /// tile's square (never underneath a tile that's ready, where it would show through the gaps
    /// between roads). Each entry: the square to fill, where the picture goes, the picture.
    fn tiles_to_paint(&mut self) -> Vec<TilePaint> {
        let (Some(index), Some(view), Some(area)) = (&self.index, self.view, self.area.get())
        else {
            return Vec::new();
        };
        self.frame += 1;
        let level = index.level_for(view);
        let mut paint = Vec::new();
        for key in visible_tiles(index, view, area, level) {
            let square = tile_bounds(index, key, view, area);
            // The tile, else its closest drawn ancestor (`None` for one with no roads).
            let found = (0..=BACKING_LEVELS.min(level)).find_map(|up| {
                let ancestor = TileKey {
                    level: key.level - up,
                    x: key.x >> up,
                    y: key.y >> up,
                };
                self.tiles.get_mut(&ancestor).map(|tile| {
                    tile.shown = self.frame;
                    (ancestor, tile.image.clone())
                })
            });
            if let Some((ancestor, Some(image))) = found {
                paint.push(TilePaint {
                    square,
                    placed: tile_bounds(index, ancestor, view, area),
                    image,
                });
            }
        }
        paint
    }
}

/// One square of the map to paint: `square` on screen, filled from `image` placed at `placed`
/// (the same for a tile drawn at this zoom; bigger, for a coarser one standing in).
struct TilePaint {
    square: Bounds<Pixels>,
    placed: Bounds<Pixels>,
    image: Arc<RenderImage>,
}

/// Where a point of the map (in the game's metres) is on screen.
fn to_screen(view: View, area: Bounds<Pixels>, x: f32, z: f32) -> Point<Pixels> {
    point(
        area.center().x + px((x - view.x) / view.metres_per_pixel),
        area.center().y + px((z - view.z) / view.metres_per_pixel),
    )
}

/// The tiles of a level that the view shows.
fn visible_tiles(index: &Index, view: View, area: Bounds<Pixels>, level: i32) -> Vec<TileKey> {
    let side = index.tile_metres(level);
    let half_width = f32::from(area.size.width) / 2.0 * view.metres_per_pixel;
    let half_height = f32::from(area.size.height) / 2.0 * view.metres_per_pixel;
    let count = 1 << level;
    let column = |x: f32| (((x - index.world.west) / side).floor() as i32).clamp(0, count - 1);
    let row = |z: f32| (((z - index.world.north) / side).floor() as i32).clamp(0, count - 1);
    let (first_x, last_x) = (column(view.x - half_width), column(view.x + half_width));
    let (first_y, last_y) = (row(view.z - half_height), row(view.z + half_height));
    let mut keys = Vec::new();
    for y in first_y..=last_y {
        for x in first_x..=last_x {
            keys.push(TileKey { level, x, y });
        }
    }
    keys
}

/// Where a tile goes on screen.
fn tile_bounds(index: &Index, key: TileKey, view: View, area: Bounds<Pixels>) -> Bounds<Pixels> {
    let (west, north, east, south) = index.tile_extent(key);
    let to_screen = |x: f32, z: f32| {
        point(
            area.center().x + px((x - view.x) / view.metres_per_pixel),
            area.center().y + px((z - view.z) / view.metres_per_pixel),
        )
    };
    Bounds::from_corners(to_screen(west, north), to_screen(east, south))
}

/// The extent of the roads.
fn world_of(roads: &[gamedata::MapRoad]) -> Option<World> {
    let mut world = World {
        west: f32::MAX,
        east: f32::MIN,
        north: f32::MAX,
        south: f32::MIN,
    };
    for road in roads {
        world.west = world.west.min(road.x);
        world.east = world.east.max(road.x);
        world.north = world.north.min(road.z);
        world.south = world.south.max(road.z);
    }
    (world.east > world.west && world.south > world.north).then_some(world)
}

/// The `x` and `z` of a placement (`(x, y, z) (w; x, y, z)`; numbers may be the game's `&hex`
/// floats).
fn placement_xz(placement: &str) -> Option<(f32, f32)> {
    let inside = placement.trim().strip_prefix('(')?.split(')').next()?;
    let mut numbers = inside.split(',').map(sii::value::parse_f32);
    let x = numbers.next()??;
    let _y = numbers.next()??;
    let z = numbers.next()??;
    Some((x, z))
}

/// One tile's picture, at `scale` device pixels per screen pixel: every road in it a dot, the
/// driven ones on top. `None` when no road touches it.
fn draw_tile(
    index: &Index,
    driven: &HashSet<u64>,
    key: TileKey,
    palette: Palette,
    scale: f32,
) -> Option<RenderImage> {
    let side = (TILE * scale).round() as i64;
    let (west, north, east, south) = index.tile_extent(key);
    let metres = (east - west) / side as f32;
    let dot = ((DOT_METRES / index.metres_per_pixel(key.level)).clamp(DOT, LARGEST_DOT) * scale)
        .round() as i64;
    // Dots just outside the tile reach into it.
    let margin = dot as f32 * metres;
    let roads = index.roads_in(west - margin, north - margin, east + margin, south + margin);
    if roads.is_empty() {
        return None;
    }
    let mut pixels = vec![0u8; (side * side * 4) as usize];
    let mut put = |x: i64, y: i64, color: [u8; 4]| {
        if x >= 0 && y >= 0 && x < side && y < side {
            let at = ((y * side + x) * 4) as usize;
            pixels[at..at + 4].copy_from_slice(&color);
        }
    };
    let mut any = false;
    for is_driven in [false, true] {
        let color = if is_driven {
            palette.driven
        } else {
            palette.road
        };
        let Some(color) = color else {
            continue;
        };
        for road in roads
            .iter()
            .filter(|road| driven.contains(&road.id) == is_driven)
        {
            let x = ((road.x - west) / metres) as i64 - dot / 2;
            let y = ((road.z - north) / metres) as i64 - dot / 2;
            if x + dot < 0 || y + dot < 0 || x >= side || y >= side {
                continue;
            }
            any = true;
            for dy in 0..dot {
                for dx in 0..dot {
                    put(x + dx, y + dy, color);
                }
            }
        }
    }
    if !any {
        return None;
    }
    let buffer = image::RgbaImage::from_raw(side as u32, side as u32, pixels)?;
    Some(RenderImage::new(vec![image::Frame::new(buffer)]))
}

impl Render for MapView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.prepare(window, cx);
        let paint = self.tiles_to_paint();
        let names = self.names(cx);
        let garages = self.garages(cx);
        // A copy: the legend's switches need the context while the theme is in use.
        let theme = cx.theme().clone();
        // The legend is the map's switches: each item shows or hides what it names, and a
        // hidden one is dimmed and struck through (so it doesn't rest on colour alone).
        let layers = self.layers;
        let item = |id: &'static str,
                    layer: Layer,
                    mark: gpui_kit::AnyElement,
                    label: SharedString,
                    cx: &mut Context<Self>| {
            let shown = layers.shows(layer);
            Button::new(id)
                .ghost()
                .xsmall()
                .tooltip(if shown {
                    t!(cx, "map-view-hide")
                } else {
                    t!(cx, "map-view-show")
                })
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .text_xs()
                        .when(!shown, |this| this.opacity(0.45).line_through())
                        .child(mark)
                        .child(label),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.layers.toggle(layer);
                    cx.notify();
                }))
        };
        let dot = |color: Hsla| {
            div()
                .size(rems(0.6))
                .rounded_full()
                .bg(color)
                .into_any_element()
        };
        let square = div()
            .size(rems(0.6))
            .bg(theme.foreground)
            .border_1()
            .border_color(theme.background)
            .into_any_element();
        let driven_label = t!(cx, "map-view-driven");
        let not_driven_label = t!(cx, "map-view-not-driven");
        let truck_label = t!(cx, "map-view-truck");
        let garages_label = t!(cx, "map-view-garages");
        let names_label = t!(cx, "map-view-names");
        let (primary, muted, danger) = (theme.primary, theme.muted_foreground, theme.danger);
        let legend = h_flex()
            .gap_1()
            .flex_wrap()
            .items_center()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(item(
                "layer-driven",
                Layer::Driven,
                dot(primary),
                driven_label,
                cx,
            ))
            .child(item(
                "layer-not-driven",
                Layer::NotDriven,
                dot(muted.opacity(0.5)),
                not_driven_label,
                cx,
            ))
            .child(item(
                "layer-truck",
                Layer::Truck,
                dot(danger),
                truck_label,
                cx,
            ))
            .child(item(
                "layer-garages",
                Layer::Garages,
                square,
                garages_label,
                cx,
            ))
            .child(item(
                "layer-names",
                Layer::Names,
                div()
                    .text_xs()
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .child("Aa")
                    .into_any_element(),
                names_label,
                cx,
            ))
            .child(div().flex_1())
            .child(t!(cx, "map-view-hint"))
            .child(
                ButtonGroup::new("map-zoom")
                    .outline()
                    .xsmall()
                    .child(
                        Button::new("zoom-in")
                            .label("+")
                            .tooltip(t!(cx, "map-view-zoom-in"))
                            .accessibility_label(t!(cx, "map-view-zoom-in")),
                    )
                    .child(
                        Button::new("zoom-out")
                            .label("−")
                            .tooltip(t!(cx, "map-view-zoom-out"))
                            .accessibility_label(t!(cx, "map-view-zoom-out")),
                    )
                    .child(Button::new("zoom-fit").label(t!(cx, "map-view-fit")))
                    .on_click(cx.listener(
                        |this, clicked: &Vec<usize>, _, cx| match clicked.first() {
                            Some(0) => this.zoom(1.0 / STEP, None, cx),
                            Some(1) => this.zoom(STEP, None, cx),
                            Some(2) => this.fit(cx),
                            _ => {}
                        },
                    )),
            );

        let has_roads = game_data::map_roads(cx).is_some();
        // "Drawing" only while something is: an area with no roads (the sea, past the map's
        // edge) just shows nothing.
        let is_getting_ready =
            self.index.is_none() || self.driven.is_none() || !self.drawing.is_empty();
        let message = if !has_roads {
            Some(t!(cx, "map-view-needs-game"))
        } else if paint.is_empty() && is_getting_ready {
            Some(t!(cx, "map-view-drawing"))
        } else {
            None
        };
        // The truck, drawn over the tiles at one size whatever the zoom.
        let truck = save::position(&self.session.read(cx).preview())
            .and_then(|position| placement_xz(&position.placement))
            .filter(|_| self.layers.truck);
        let truck_at = match (truck, self.view, self.area.get()) {
            (Some((x, z)), Some(view), Some(area)) => {
                Some(to_screen(view, area, x, z)).filter(|at| area.contains(at))
            }
            _ => None,
        };

        // What a click opens, in the order they're drawn, topmost first: the names (over the
        // map), the truck, the garages.
        let mut targets = Vec::new();
        for name in &names {
            targets.push((name.room, Target::City(name.token.clone())));
        }
        if let (Some(at), Some(id)) = (truck_at, self.session.read(cx).own_truck()) {
            targets.push((
                square_around(at, px(MARKER * 2.0), HIT_MARGIN),
                Target::Truck(id),
            ));
        }
        for garage in &garages {
            targets.push((
                square_around(garage.at, garage.side(), HIT_MARGIN),
                Target::Garage {
                    id: garage.id.clone(),
                    city: garage.city.clone(),
                },
            ));
        }
        self.targets = targets;
        // Nothing is hovered while the map is dragged.
        self.hovered = match (self.pointer, self.drag) {
            (Some(pointer), None) => self.target_at(pointer),
            _ => None,
        };
        // A garage or the truck hovered says what it is, beside it (a name says it already).
        let hover_label = self.hovered.as_ref().and_then(|hovered| {
            let label: SharedString = match hovered {
                Target::City(_) => return None,
                Target::Garage { city, .. } => t!(
                    cx,
                    "detail-garage-in",
                    city = game_data::city_name(cx, city)
                ),
                Target::Truck(_) => t!(cx, "map-view-truck"),
            };
            let (bounds, _) = self.targets.iter().find(|(_, target)| target == hovered)?;
            Some((bounds.top_right(), bounds.size.height, label))
        });
        let hovered_city = match &self.hovered {
            Some(Target::City(token)) => Some(token.clone()),
            _ => None,
        };
        let marker = (theme.danger, theme.background);
        let garage_colors = (theme.foreground, theme.background, theme.primary);
        let area = self.area.clone();
        let entity = cx.entity().downgrade();
        let map = canvas(
            move |bounds, _, cx| {
                // The area's size decides which tiles show: a new size, a new look.
                if area.get() != Some(bounds) {
                    area.set(Some(bounds));
                    let _ = entity.update(cx, |_, cx| cx.notify());
                }
            },
            move |bounds, _, window, _| {
                for tile in paint {
                    let square = tile.square.intersect(&bounds);
                    let _ = window.paint_image(
                        square,
                        tile.placed,
                        Corners::default(),
                        tile.image,
                        0,
                        false,
                    );
                }
                // The player's garages: squares, the headquarters bigger and ringed.
                for garage in garages {
                    let is_headquarters = garage.is_headquarters;
                    window.paint_quad(quad(
                        square_around(garage.at, garage.side(), 0.0),
                        px(1.0),
                        garage_colors.0,
                        px(if is_headquarters { 2.0 } else { 1.0 }),
                        if is_headquarters {
                            garage_colors.2
                        } else {
                            garage_colors.1
                        },
                        BorderStyle::Solid,
                    ));
                }
                if let Some(middle) = truck_at {
                    let radius = px(MARKER);
                    window.paint_quad(quad(
                        square_around(middle, radius * 2.0, 0.0),
                        radius,
                        marker.0,
                        px(2.0),
                        marker.1,
                        BorderStyle::Solid,
                    ));
                }
            },
        )
        .size_full();

        v_flex().size_full().gap_3().child(legend).child(
            div()
                .id("map-view")
                .relative()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                // A pointer over what a click opens; elsewhere, the map is dragged.
                .map(|this| {
                    if self.hovered.is_some() {
                        this.cursor_pointer()
                    } else {
                        this.cursor_grab()
                    }
                })
                .on_hover(cx.listener(|this, is_hovered: &bool, _, cx| {
                    if !*is_hovered {
                        this.pointer = None;
                        if this.hovered.take().is_some() {
                            cx.notify();
                        }
                    }
                }))
                .on_scroll(
                    cx.listener(|this, event: &gpui_kit::ScrollEvent, window, cx| {
                        this.pointer = Some(event.position);
                        let delta = f32::from(event.delta.pixel_delta(window.line_height()).y);
                        // One wheel notch zooms by about a third.
                        this.zoom((-delta / 70.0).exp(), Some(event.position), cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui_kit::MouseDownEvent, _, _| {
                        if let Some(view) = this.view {
                            this.drag = Some((event.position, view));
                        }
                    }),
                )
                .on_mouse_move(
                    cx.listener(|this, event: &gpui_kit::MouseMoveEvent, _, cx| {
                        this.pointer = Some(event.position);
                        let Some((from, start)) = this.drag else {
                            // Not dragging: drawn again only when what's under the pointer
                            // changes.
                            if this.target_at(event.position) != this.hovered {
                                cx.notify();
                            }
                            return;
                        };
                        if event.pressed_button != Some(MouseButton::Left) {
                            this.drag = None;
                            return;
                        }
                        let moved = event.position - from;
                        this.view = Some(View {
                            x: start.x - f32::from(moved.x) * start.metres_per_pixel,
                            z: start.z - f32::from(moved.y) * start.metres_per_pixel,
                            ..start
                        });
                        cx.notify();
                    }),
                )
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, event: &gpui_kit::MouseUpEvent, window, cx| {
                        let Some((from, _)) = this.drag.take() else {
                            return;
                        };
                        // What's under the pointer counts again now the drag is over.
                        cx.notify();
                        // Let go about where it was pressed: a click, not a drag.
                        let moved = event.position - from;
                        let is_click = f32::from(moved.x).abs() <= CLICK_SLOP
                            && f32::from(moved.y).abs() <= CLICK_SLOP;
                        if is_click && let Some(target) = this.target_at(event.position) {
                            this.open(target, window, cx);
                        }
                    }),
                )
                .child(map)
                // Cities' names over the map: the ones visited in full colour.
                .children(names.into_iter().map(|name| {
                    let offset = self.area.get().map(|area| area.origin).unwrap_or_default();
                    let is_hovered = hovered_city.as_ref() == Some(&name.token);
                    div()
                        .absolute()
                        .left(name.room.origin.x - offset.x)
                        .top(name.room.origin.y - offset.y)
                        .px_1()
                        .rounded(theme.radius)
                        .bg(theme.background.opacity(0.7))
                        .text_xs()
                        .whitespace_nowrap()
                        .text_color(if name.is_visited || is_hovered {
                            theme.foreground
                        } else {
                            theme.muted_foreground
                        })
                        // Hovered, it reads as a link: a click opens the city.
                        .when(is_hovered, |this| this.underline())
                        .child(name.name)
                }))
                .children(hover_label.map(|(top_right, height, label)| {
                    let offset = self.area.get().map(|area| area.origin).unwrap_or_default();
                    div()
                        .absolute()
                        .left(top_right.x - offset.x + px(2.0))
                        .top(top_right.y - offset.y + height / 2.0 - px(NAME_HEIGHT / 2.0))
                        .px_1()
                        .rounded(theme.radius)
                        .bg(theme.background.opacity(0.85))
                        .text_xs()
                        .whitespace_nowrap()
                        .text_color(theme.foreground)
                        .child(label)
                }))
                .children(message.map(|message| {
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(message)
                })),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roads() -> Arc<Vec<gamedata::MapRoad>> {
        Arc::new(vec![
            gamedata::MapRoad {
                id: 1,
                x: 0.0,
                z: 0.0,
            },
            gamedata::MapRoad {
                id: 2,
                x: 25_600.0,
                z: 12_800.0,
            },
        ])
    }

    #[test]
    fn reads_a_placements_ground_position() {
        assert_eq!(
            placement_xz("(&45a2ec70, &41db0d04, -200) (1; 0, 0, 0)"),
            Some((f32::from_bits(0x45a2ec70), -200.0))
        );
        assert_eq!(placement_xz("nothing"), None);
    }

    #[test]
    fn one_tile_holds_the_whole_map_at_level_zero() {
        let index = Index::new(roads()).expect("an index");
        // 25,600 m wide over 256 pixels: 100 m a pixel.
        assert_eq!(index.metres_per_pixel(0), 100.0);
        assert_eq!(index.tile_metres(1), 12_800.0);
        // A view 50 m a pixel needs level 1's tiles; 30 m, level 2's.
        let view = |metres_per_pixel| View {
            x: 0.0,
            z: 0.0,
            metres_per_pixel,
        };
        assert_eq!(index.level_for(view(50.0)), 1);
        assert_eq!(index.level_for(view(30.0)), 2);
        assert_eq!(index.roads_in(-10.0, -10.0, 10.0, 10.0).len(), 1);
    }

    #[test]
    fn draws_a_tile_with_its_roads_and_skips_an_empty_one() {
        let index = Index::new(roads()).expect("an index");
        let palette = Palette {
            road: Some([1, 1, 1, 255]),
            driven: Some([2, 2, 2, 255]),
        };
        let driven: HashSet<u64> = [2].into_iter().collect();
        let whole = TileKey {
            level: 0,
            x: 0,
            y: 0,
        };
        let picture = draw_tile(&index, &driven, whole, palette, 2.0).expect("a picture");
        let drawn = picture.size(0);
        // At a scale of 2, twice a tile's side.
        assert_eq!((drawn.width.0, drawn.height.0), (512, 512));
        // Level 3: the tile at (3, 3) holds neither road.
        let empty = TileKey {
            level: 3,
            x: 3,
            y: 3,
        };
        assert!(draw_tile(&index, &driven, empty, palette, 1.0).is_none());
    }

    #[test]
    fn places_tiles_where_the_view_puts_them() {
        let index = Index::new(roads()).expect("an index");
        let area = Bounds::new(point(px(0.0), px(0.0)), size(px(256.0), px(256.0)));
        let whole = TileKey {
            level: 0,
            x: 0,
            y: 0,
        };
        // The view at the tile's middle, at its own scale: the tile fills the area.
        let view = View {
            x: 12_800.0,
            z: 12_800.0,
            metres_per_pixel: 100.0,
        };
        assert_eq!(tile_bounds(&index, whole, view, area), area);
        assert_eq!(visible_tiles(&index, view, area, 0), [whole]);
    }
}
