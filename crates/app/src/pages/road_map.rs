//! The map's roads: a dot for every road and junction on the game's map (read from its files, see
//! `gamedata::map_roads`), the ones driven in this save in the accent colour, and where the
//! player's truck is. The mouse wheel zooms around the pointer, dragging moves the map, and
//! buttons do both from the keyboard.
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
    button::{Button, ButtonGroup},
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, BorderStyle, Bounds, Context, Corners, Entity, Hsla, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render, RenderImage, Rgba,
    Styled as _, Task, Window, canvas, div, point, px, quad, rems, size,
};

use crate::game_data;
use crate::i18n::t;
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

/// One tile: its zoom level and place in that level's grid.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct TileKey {
    level: i32,
    x: i32,
    y: i32,
}

/// What tiles are drawn for: new tiles when any of it changes.
#[derive(Clone, Debug, PartialEq)]
struct Look {
    driven: usize,
    is_dark: bool,
    scale: f32,
}

/// The colours tiles are drawn in, as blue, green, red, alpha (GPUI's image order).
#[derive(Clone, Copy)]
struct Palette {
    road: [u8; 4],
    driven: [u8; 4],
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

pub struct RoadMap {
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
    _work: Vec<Task<()>>,
}

impl RoadMap {
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

        // A new look (roads driven, light or dark, screen resolution): tiles drawn again.
        let document = self.session.read(cx).preview();
        let look = Look {
            driven: save::discovered_roads(&document),
            is_dark: cx.theme().is_dark(),
            scale: window.scale_factor(),
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
            road: bgra(theme.muted_foreground.into(), 0.4),
            driven: bgra(theme.primary.into(), 1.0),
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

impl Render for RoadMap {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.prepare(window, cx);
        let paint = self.tiles_to_paint();
        let theme = cx.theme();
        let swatch = |color: Hsla, label: gpui_kit::SharedString| {
            h_flex()
                .gap_1p5()
                .items_center()
                .child(div().size(rems(0.6)).rounded_full().bg(color))
                .child(label)
        };
        let legend = h_flex()
            .gap_4()
            .flex_wrap()
            .items_center()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(swatch(theme.primary, t!(cx, "road-map-driven")))
            .child(swatch(
                theme.muted_foreground.opacity(0.5),
                t!(cx, "road-map-not-driven"),
            ))
            .child(swatch(theme.danger, t!(cx, "road-map-truck")))
            .child(div().flex_1())
            .child(t!(cx, "road-map-hint"))
            .child(
                ButtonGroup::new("road-map-zoom")
                    .outline()
                    .xsmall()
                    .child(
                        Button::new("zoom-in")
                            .label("+")
                            .tooltip(t!(cx, "road-map-zoom-in"))
                            .accessibility_label(t!(cx, "road-map-zoom-in")),
                    )
                    .child(
                        Button::new("zoom-out")
                            .label("−")
                            .tooltip(t!(cx, "road-map-zoom-out"))
                            .accessibility_label(t!(cx, "road-map-zoom-out")),
                    )
                    .child(Button::new("zoom-fit").label(t!(cx, "road-map-fit")))
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
            Some(t!(cx, "road-map-needs-game"))
        } else if paint.is_empty() && is_getting_ready {
            Some(t!(cx, "road-map-drawing"))
        } else {
            None
        };
        // The truck, drawn over the tiles at one size whatever the zoom.
        let truck = save::position(&self.session.read(cx).preview())
            .and_then(|position| placement_xz(&position.placement));
        let view = self.view;
        let marker = (theme.danger, theme.background);
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
                if let (Some((x, z)), Some(view)) = (truck, view) {
                    let middle = point(
                        bounds.center().x + px((x - view.x) / view.metres_per_pixel),
                        bounds.center().y + px((z - view.z) / view.metres_per_pixel),
                    );
                    if bounds.contains(&middle) {
                        let radius = px(MARKER);
                        window.paint_quad(quad(
                            Bounds::new(
                                point(middle.x - radius, middle.y - radius),
                                size(radius * 2.0, radius * 2.0),
                            ),
                            radius,
                            marker.0,
                            px(2.0),
                            marker.1,
                            BorderStyle::Solid,
                        ));
                    }
                }
            },
        )
        .size_full();

        v_flex().size_full().gap_3().child(legend).child(
            div()
                .id("road-map")
                .relative()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .cursor_grab()
                .on_scroll(
                    cx.listener(|this, event: &gpui_kit::ScrollEvent, window, cx| {
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
                        let Some((from, start)) = this.drag else {
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
                    cx.listener(|this, _: &gpui_kit::MouseUpEvent, _, _| this.drag = None),
                )
                .child(map)
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
            road: [1, 1, 1, 255],
            driven: [2, 2, 2, 255],
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
