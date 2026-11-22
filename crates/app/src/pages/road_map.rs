//! The map's roads, as a picture: a dot for every road and junction on the game's map (read from
//! its files, see `gamedata::map_roads`), the ones driven in this save in the accent colour, and
//! where the player's truck is. The mouse wheel zooms around the pointer, dragging moves the map,
//! and buttons do both from the keyboard.
//!
//! What's on screen is drawn in the background as one picture of just the area shown, at the
//! screen's own resolution, so dots stay sharp at any zoom: hundreds of thousands of dots are far
//! too many to draw as shapes on every frame. While the map moves, the last picture is stretched
//! and shifted to follow at once, and a sharp one replaces it a moment later.

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonGroup},
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Bounds, Context, Corners, Entity, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, Pixels, Point, Render, RenderImage, Rgba, Styled as _, Task,
    Window, canvas, div, point, px, rems, size,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;

/// A dot's size on screen, in pixels, when the whole map shows; dots grow as it's zoomed in.
const DOT: f32 = 2.0;
/// The largest a dot gets, in pixels.
const LARGEST_DOT: f32 = 7.0;
/// How many metres a dot stands for once zoomed in (about a road piece).
const DOT_METRES: f32 = 40.0;
/// The truck's marker's radius on screen, in pixels.
const MARKER: f32 = 6.0;
/// The closest zoom: metres a screen pixel shows.
const CLOSEST: f32 = 1.5;
/// How long the map waits after moving before drawing it sharp again.
const SETTLE: Duration = Duration::from_millis(80);
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

/// What a picture shows: drawn again when any of it changes.
#[derive(Clone, Debug, PartialEq)]
struct Request {
    view: View,
    /// The area's size on screen, in pixels.
    width: f32,
    height: f32,
    driven: usize,
    position: Option<String>,
    is_dark: bool,
}

/// A picture drawn, and what it shows.
struct Picture {
    image: Arc<RenderImage>,
    request: Request,
}

/// The colours a picture is drawn in, as blue, green, red, alpha (GPUI's image order).
#[derive(Clone, Copy)]
struct Palette {
    road: [u8; 4],
    driven: [u8; 4],
    marker: [u8; 4],
    marker_edge: [u8; 4],
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

pub struct RoadMap {
    session: Entity<Session>,
    view: Option<View>,
    world: Option<World>,
    /// Where the map's area is on screen, as last laid out (set while painting).
    area: Rc<Cell<Option<Bounds<Pixels>>>>,
    picture: Option<Picture>,
    /// What's being drawn (or was last drawn), so the same picture isn't asked for twice.
    requested: Option<Request>,
    /// Dragging: where the pointer went down, and the view then.
    drag: Option<(Point<Pixels>, View)>,
    _drawing: Option<Task<()>>,
}

impl RoadMap {
    pub fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        Self {
            session,
            view: None,
            world: None,
            area: Rc::new(Cell::new(None)),
            picture: None,
            requested: None,
            drag: None,
            _drawing: None,
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
        match (self.world, self.area.get()) {
            (Some(world), Some(area)) => {
                Self::fitted(
                    world,
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
        if let (Some(world), Some(area)) = (self.world, self.area.get()) {
            self.view = Some(Self::fitted(
                world,
                f32::from(area.size.width),
                f32::from(area.size.height),
            ));
            cx.notify();
        }
    }

    /// Asks for a sharp picture of what's shown, once it has settled, if there isn't one.
    fn redraw_if_changed(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(roads) = game_data::map_roads(cx) else {
            return;
        };
        if self.world.is_none() {
            self.world = world_of(&roads);
        }
        let (Some(world), Some(area)) = (self.world, self.area.get()) else {
            return;
        };
        let (width, height) = (f32::from(area.size.width), f32::from(area.size.height));
        let view = *self
            .view
            .get_or_insert_with(|| Self::fitted(world, width, height));
        let document = self.session.read(cx).preview();
        let position = save::position(&document).map(|position| position.placement);
        let request = Request {
            view,
            width,
            height,
            driven: save::discovered_roads(&document),
            position: position.clone(),
            is_dark: cx.theme().is_dark(),
        };
        if self.requested.as_ref() == Some(&request) {
            return;
        }
        self.requested = Some(request.clone());
        let theme = cx.theme();
        let palette = Palette {
            road: bgra(theme.muted_foreground.into(), 0.4),
            driven: bgra(theme.primary.into(), 1.0),
            marker: bgra(theme.danger.into(), 1.0),
            marker_edge: bgra(theme.background.into(), 1.0),
        };
        let scale = window.scale_factor();
        // Moving: wait until it stops before drawing; the last picture follows meanwhile.
        let settle = self.picture.is_some();
        self._drawing = Some(cx.spawn(async move |this, cx| {
            if settle {
                cx.background_executor().timer(SETTLE).await;
            }
            let drawn = request.clone();
            let image = cx
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
                    let truck = position.as_deref().and_then(placement_xz);
                    draw(&roads, &driven, truck, palette, &drawn, scale)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if let Some(image) = image {
                    let old = this.picture.replace(Picture {
                        image: Arc::new(image),
                        request,
                    });
                    // The old picture's copy on the graphics card goes with it.
                    if let Some(old) = old {
                        cx.drop_image(old.image, None);
                    }
                }
                cx.notify();
            });
        }));
    }
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

/// The picture of what `request` shows, at `scale` device pixels per screen pixel: every road a
/// dot, the driven ones on top, then the truck's marker. `None` for an empty area.
fn draw(
    roads: &[gamedata::MapRoad],
    driven: &HashSet<u64>,
    truck: Option<(f32, f32)>,
    palette: Palette,
    request: &Request,
    scale: f32,
) -> Option<RenderImage> {
    let width = (request.width * scale).round() as i64;
    let height = (request.height * scale).round() as i64;
    if width <= 0 || height <= 0 {
        return None;
    }
    let view = request.view;
    // Metres per device pixel, and a dot's size: bigger as the map is zoomed in.
    let metres = view.metres_per_pixel / scale;
    let dot = ((DOT_METRES / view.metres_per_pixel).clamp(DOT, LARGEST_DOT) * scale).round() as i64;
    let to_pixel = |x: f32, z: f32| {
        (
            ((x - view.x) / metres) as i64 + width / 2,
            // The game's `z` runs south: north at the top.
            ((z - view.z) / metres) as i64 + height / 2,
        )
    };
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let mut put = |x: i64, y: i64, color: [u8; 4]| {
        if x >= 0 && y >= 0 && x < width && y < height {
            let at = ((y * width + x) * 4) as usize;
            pixels[at..at + 4].copy_from_slice(&color);
        }
    };
    let paint_roads = |is_driven: bool, color: [u8; 4], put: &mut dyn FnMut(i64, i64, [u8; 4])| {
        for road in roads
            .iter()
            .filter(|road| driven.contains(&road.id) == is_driven)
        {
            let (px, py) = to_pixel(road.x, road.z);
            if px < -dot || py < -dot || px > width + dot || py > height + dot {
                continue;
            }
            for dy in 0..dot {
                for dx in 0..dot {
                    put(px + dx - dot / 2, py + dy - dot / 2, color);
                }
            }
        }
    };
    paint_roads(false, palette.road, &mut put);
    paint_roads(true, palette.driven, &mut put);
    if let Some((x, z)) = truck {
        let (cx, cy) = to_pixel(x, z);
        let inner = (MARKER * scale) as i64;
        let outer = inner + (2.0 * scale) as i64;
        for dy in -outer..=outer {
            for dx in -outer..=outer {
                let distance = dx * dx + dy * dy;
                if distance <= inner * inner {
                    put(cx + dx, cy + dy, palette.marker);
                } else if distance <= outer * outer {
                    put(cx + dx, cy + dy, palette.marker_edge);
                }
            }
        }
    }
    let buffer = image::RgbaImage::from_raw(width as u32, height as u32, pixels)?;
    Some(RenderImage::new(vec![image::Frame::new(buffer)]))
}

/// Where a picture drawn for one view goes on screen for another, in `area`.
fn placed(picture: &Request, view: View, area: Bounds<Pixels>) -> Bounds<Pixels> {
    let ratio = picture.view.metres_per_pixel / view.metres_per_pixel;
    let (width, height) = (picture.width * ratio, picture.height * ratio);
    // The picture's middle, on screen now.
    let middle_x = (picture.view.x - view.x) / view.metres_per_pixel;
    let middle_z = (picture.view.z - view.z) / view.metres_per_pixel;
    Bounds::new(
        point(
            area.center().x + px(middle_x - width / 2.0),
            area.center().y + px(middle_z - height / 2.0),
        ),
        size(px(width), px(height)),
    )
}

impl Render for RoadMap {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.redraw_if_changed(window, cx);
        let theme = cx.theme();
        let swatch = |color: gpui_kit::Hsla, label: gpui_kit::SharedString| {
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
        let message = match (&self.picture, has_roads) {
            (None, true) => Some(t!(cx, "road-map-drawing")),
            (None, false) => Some(t!(cx, "road-map-needs-game")),
            _ => None,
        };
        let area = self.area.clone();
        let entity = cx.entity().downgrade();
        let picture = self
            .picture
            .as_ref()
            .map(|picture| (picture.image.clone(), picture.request.clone()));
        let view = self.view;
        let map = canvas(
            move |bounds, _, cx| {
                // The area's size decides the picture's: a new size, a new picture.
                if area.get() != Some(bounds) {
                    area.set(Some(bounds));
                    let _ = entity.update(cx, |_, cx| cx.notify());
                }
            },
            move |bounds, _, window, _| {
                if let (Some((image, request)), Some(view)) = (picture, view) {
                    let _ = window.paint_image(
                        bounds,
                        placed(&request, view, bounds),
                        Corners::default(),
                        image,
                        0,
                        false,
                    );
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

    #[test]
    fn reads_a_placements_ground_position() {
        assert_eq!(
            placement_xz("(&45a2ec70, &41db0d04, -200) (1; 0, 0, 0)"),
            Some((f32::from_bits(0x45a2ec70), -200.0))
        );
        assert_eq!(placement_xz("nothing"), None);
    }

    fn request(view: View) -> Request {
        Request {
            view,
            width: 200.0,
            height: 100.0,
            driven: 0,
            position: None,
            is_dark: false,
        }
    }

    #[test]
    fn draws_the_area_shown_at_the_screens_resolution() {
        let roads = [
            gamedata::MapRoad {
                id: 1,
                x: 0.0,
                z: 0.0,
            },
            gamedata::MapRoad {
                id: 2,
                x: 100.0,
                z: 50.0,
            },
        ];
        let palette = Palette {
            road: [1, 1, 1, 255],
            driven: [2, 2, 2, 255],
            marker: [3, 3, 3, 255],
            marker_edge: [4, 4, 4, 255],
        };
        let driven: HashSet<u64> = [2].into_iter().collect();
        let world = world_of(&roads).expect("an extent");
        let view = RoadMap::fitted(world, 200.0, 100.0);
        let picture = draw(&roads, &driven, None, palette, &request(view), 2.0).expect("a picture");
        let drawn = picture.size(0);
        // At a scale of 2, twice the area's size.
        assert_eq!((drawn.width.0, drawn.height.0), (400, 200));
    }

    #[test]
    fn places_an_older_picture_where_the_map_has_moved() {
        let view = View {
            x: 0.0,
            z: 0.0,
            metres_per_pixel: 10.0,
        };
        let area = Bounds::new(point(px(0.0), px(0.0)), size(px(200.0), px(100.0)));
        // Same view: right where it was.
        assert_eq!(placed(&request(view), view, area), area);
        // Zoomed in twice as far: twice the size, still centred.
        let closer = View {
            metres_per_pixel: 5.0,
            ..view
        };
        let placed = placed(&request(view), closer, area);
        assert_eq!(placed.size, size(px(400.0), px(200.0)));
        assert_eq!(placed.center(), area.center());
    }
}
