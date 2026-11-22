//! The map's roads, as a picture: a dot for every road and junction on the game's map (read from
//! its files, see `gamedata::map_roads`), the ones driven in this save in the accent colour, and
//! where the player's truck is.
//!
//! The picture is drawn in the background, once for what it shows (the roads driven, the truck's
//! place, light or dark), then shown as an image: hundreds of thousands of dots are far too many
//! to draw as shapes on every frame.

use std::collections::HashSet;
use std::sync::Arc;

use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ObjectFit, ParentElement as _, Render,
    RenderImage, Rgba, Styled as _, StyledImage as _, Task, Window, div, img, rems,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;

/// The picture's width, in pixels; its height follows the map's shape.
const WIDTH: u32 = 1600;
/// A dot's size, in pixels.
const DOT: u32 = 2;
/// The truck's marker's radius, in pixels: big, since the picture is shown smaller than it's drawn.
const MARKER: i64 = 16;

/// What a picture shows: drawn again when any of it changes.
#[derive(Clone, PartialEq)]
struct Shown {
    roads: usize,
    driven: usize,
    position: Option<String>,
    is_dark: bool,
}

pub struct RoadMap {
    session: Entity<Session>,
    picture: Option<Arc<RenderImage>>,
    /// What the picture shows, or is being drawn to show.
    shown: Option<Shown>,
    _drawing: Option<Task<()>>,
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

impl RoadMap {
    pub fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        Self {
            session,
            picture: None,
            shown: None,
            _drawing: None,
        }
    }

    /// Starts drawing the picture again if what it would show has changed.
    fn redraw_if_changed(&mut self, cx: &mut Context<Self>) {
        let Some(roads) = game_data::map_roads(cx) else {
            return;
        };
        let document = self.session.read(cx).preview();
        let position = save::position(&document).map(|position| position.placement);
        let shown = Shown {
            roads: roads.len(),
            driven: save::discovered_roads(&document),
            position: position.clone(),
            is_dark: cx.theme().is_dark(),
        };
        if self.shown.as_ref() == Some(&shown) {
            return;
        }
        self.shown = Some(shown);
        let theme = cx.theme();
        let palette = Palette {
            road: bgra(theme.muted_foreground.into(), 0.35),
            driven: bgra(theme.primary.into(), 1.0),
            marker: bgra(theme.danger.into(), 1.0),
            marker_edge: bgra(theme.background.into(), 1.0),
        };
        self._drawing = Some(cx.spawn(async move |this, cx| {
            let picture = cx
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
                    draw(&roads, &driven, truck, palette)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.picture = picture.map(Arc::new);
                cx.notify();
            });
        }));
    }
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

/// The picture: every road a dot, the driven ones on top, then the truck's marker. `None` with
/// no roads.
fn draw(
    roads: &[gamedata::MapRoad],
    driven: &HashSet<u64>,
    truck: Option<(f32, f32)>,
    palette: Palette,
) -> Option<RenderImage> {
    let (mut west, mut east, mut north, mut south) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for road in roads {
        west = west.min(road.x);
        east = east.max(road.x);
        north = north.min(road.z);
        south = south.max(road.z);
    }
    if roads.is_empty() || east <= west || south <= north {
        return None;
    }
    let scale = (WIDTH - DOT) as f32 / (east - west);
    let height = ((south - north) * scale).ceil() as u32 + DOT;
    let mut pixels = vec![0u8; (WIDTH * height * 4) as usize];
    let to_pixel = |x: f32, z: f32| {
        (
            ((x - west) * scale) as i64,
            // The game's `z` runs south: north at the top.
            ((z - north) * scale) as i64,
        )
    };
    let mut put = |x: i64, y: i64, color: [u8; 4]| {
        if x >= 0 && y >= 0 && (x as u32) < WIDTH && (y as u32) < height {
            let at = ((y as u32 * WIDTH + x as u32) * 4) as usize;
            pixels[at..at + 4].copy_from_slice(&color);
        }
    };
    let dot = |x: f32, z: f32, color: [u8; 4], put: &mut dyn FnMut(i64, i64, [u8; 4])| {
        let (px, py) = to_pixel(x, z);
        for dy in 0..DOT as i64 {
            for dx in 0..DOT as i64 {
                put(px + dx, py + dy, color);
            }
        }
    };
    for road in roads.iter().filter(|road| !driven.contains(&road.id)) {
        dot(road.x, road.z, palette.road, &mut put);
    }
    for road in roads.iter().filter(|road| driven.contains(&road.id)) {
        dot(road.x, road.z, palette.driven, &mut put);
    }
    if let Some((x, z)) = truck {
        let (cx, cy) = to_pixel(x, z);
        for dy in -MARKER - 4..=MARKER + 4 {
            for dx in -MARKER - 4..=MARKER + 4 {
                let distance = dx * dx + dy * dy;
                if distance <= MARKER * MARKER {
                    put(cx + dx, cy + dy, palette.marker);
                } else if distance <= (MARKER + 4) * (MARKER + 4) {
                    put(cx + dx, cy + dy, palette.marker_edge);
                }
            }
        }
    }
    let buffer = image::RgbaImage::from_raw(WIDTH, height, pixels)?;
    Some(RenderImage::new(vec![image::Frame::new(buffer)]))
}

impl Render for RoadMap {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.redraw_if_changed(cx);
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
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(swatch(theme.primary, t!(cx, "road-map-driven")))
            .child(swatch(
                theme.muted_foreground.opacity(0.5),
                t!(cx, "road-map-not-driven"),
            ))
            .child(swatch(theme.danger, t!(cx, "road-map-truck")));
        let body = match (&self.picture, game_data::map_roads(cx).is_some()) {
            (Some(picture), _) => img(picture.clone())
                .size_full()
                .object_fit(ObjectFit::Contain)
                .into_any_element(),
            (None, true) => div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(t!(cx, "road-map-drawing"))
                .into_any_element(),
            (None, false) => div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(t!(cx, "road-map-needs-game"))
                .into_any_element(),
        };
        v_flex().size_full().gap_3().child(legend).child(
            div()
                .flex_1()
                .min_h_0()
                .p_2()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .flex()
                .items_center()
                .justify_center()
                .child(body),
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

    #[test]
    fn draws_driven_roads_over_the_rest() {
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
        let picture = draw(&roads, &driven, None, palette).expect("a picture");
        let size = picture.size(0);
        assert_eq!(u32::from(size.width), WIDTH);
        assert!(draw(&[], &driven, None, palette).is_none());
    }
}
