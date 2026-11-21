//! One of the game's own icons (a skill, an ADR placard), drawn at the exact number of pixels it
//! covers on screen so it stays crisp at any display scaling, in colours that suit the theme.

use std::collections::HashMap;
use std::sync::Arc;

use gamedata::GameIcon;
use gpui_kit::{
    App, Global, Hsla, IntoElement, ParentElement as _, RenderImage, RenderOnce, Rgba, Styled as _,
    Window, div, img, rems,
};

/// Drawn icons, by what was drawn: the icon, its size in pixels, and its colours.
#[derive(Default)]
struct Drawn(HashMap<Key, Arc<RenderImage>>);

impl Global for Drawn {}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Key {
    name: String,
    width: u32,
    height: u32,
    colours: [Option<[u8; 4]>; 4],
    backdrop: Option<[u8; 4]>,
}

#[derive(IntoElement)]
pub struct GameIconView {
    icon: Arc<GameIcon>,
    /// Height, in rems; the width follows the icon's shape.
    size: f32,
    colours: [Option<Hsla>; 4],
    backdrop: Option<Hsla>,
    opacity: f32,
    /// Corner radius, for pictures (faces); icons keep their own shape.
    corner_radius: Option<gpui_kit::Pixels>,
}

impl GameIconView {
    pub fn new(icon: Arc<GameIcon>) -> Self {
        Self {
            icon,
            size: 2.,
            colours: [None; 4],
            backdrop: None,
            opacity: 1.,
            corner_radius: None,
        }
    }

    /// Height in rems; the width follows the icon's shape (square icons are this wide too).
    pub fn size(mut self, rems: f32) -> Self {
        self.size = rems;
        self
    }

    /// Draws one of the texture's channels (`gamedata::RED` …) in this colour instead of the
    /// game's.
    pub fn channel(mut self, channel: usize, colour: Hsla) -> Self {
        if let Some(slot) = self.colours.get_mut(channel) {
            *slot = Some(colour);
        }
        self
    }

    /// Paints the dark panel the game draws this icon on, in the icon's shape.
    pub fn backdrop(mut self, colour: Hsla) -> Self {
        self.backdrop = Some(colour);
        self
    }

    /// Rounds the picture's corners.
    pub fn rounded(mut self, radius: gpui_kit::Pixels) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }
}

fn to_rgba(colour: Hsla) -> [u8; 4] {
    let rgba = Rgba::from(colour);
    [rgba.r, rgba.g, rgba.b, rgba.a].map(|value| (value.clamp(0., 1.) * 255.).round() as u8)
}

impl RenderOnce for GameIconView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let height = rems(self.size);
        let width = rems(self.size * self.icon.aspect());
        let to_device = |length: gpui_kit::Rems| {
            (f32::from(length.to_pixels(window.rem_size())) * window.scale_factor())
                .round()
                .max(1.) as u32
        };
        let key = Key {
            name: self.icon.name.clone(),
            width: to_device(width),
            height: to_device(height),
            colours: self.colours.map(|colour| colour.map(to_rgba)),
            backdrop: self.backdrop.map(to_rgba),
        };
        if !cx.has_global::<Drawn>() {
            cx.set_global(Drawn::default());
        }
        let cached = cx.global::<Drawn>().0.get(&key).cloned();
        let image = match cached {
            Some(image) => Some(image),
            None => {
                // Small (a few thousand pixels), so drawing it here doesn't hold up the frame.
                let bgra = self
                    .icon
                    .render(key.width, key.height, key.colours, key.backdrop);
                let image = image::RgbaImage::from_raw(key.width, key.height, bgra)
                    .map(|buffer| Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])));
                if let Some(image) = &image {
                    cx.global_mut::<Drawn>().0.insert(key, image.clone());
                }
                image
            }
        };
        let frame = div()
            .w(width)
            .h(height)
            .flex_shrink_0()
            .opacity(self.opacity);
        match image {
            Some(image) => {
                let image = img(image).size_full();
                let image = match self.corner_radius {
                    Some(radius) => image.rounded(radius),
                    None => image,
                };
                frame.child(image)
            }
            None => frame,
        }
    }
}
