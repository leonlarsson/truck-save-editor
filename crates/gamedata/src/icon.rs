//! The game's UI icons (skills, ADR placards). Each is a material (`.mat`, effect `ui.sdf.rfx`)
//! over a texture whose four channels are separate shapes, one per colour of the material:
//! red, green, blue and alpha take `aux[1]` to `aux[4]`. In skill icons green is the highlight
//! and blue the base; in ADR placards alpha is the placard's fill and the others its lines,
//! number and symbol. Each channel is a narrow distance field, which reads well as coverage.

use hashfs::Archive;

use crate::texture::decode_rgba;

/// One of the game's icons: four coverage masks at the texture's size, and the material's colours.
#[derive(Clone, Debug, PartialEq)]
pub struct GameIcon {
    /// The material's path without `.mat` (`/material/ui/skills/adr_1`): unique per icon.
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Coverage 0–255 per pixel, for the red, green, blue and alpha channels.
    pub masks: [Vec<u8>; 4],
    /// The material's colour for each channel, as sRGB red, green, blue, alpha (0–255).
    pub colours: [[u8; 4]; 4],
    /// A plain picture (`ui.rfx`, a driver's face), not shapes to colour: its channels are its
    /// own red, green, blue and alpha.
    pub is_picture: bool,
}

/// A channel of an icon, named for what it is in the game's textures.
pub const RED: usize = 0;
pub const GREEN: usize = 1;
pub const BLUE: usize = 2;
pub const ALPHA: usize = 3;

/// Back to front: the placard fill (alpha) first, the outlines and text (red) last.
const DRAW_ORDER: [usize; 4] = [ALPHA, BLUE, GREEN, RED];

impl GameIcon {
    /// Width over height (brand logos are wide; skill icons square).
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Reads `<folder>/<name>.mat` and its texture. `None` if either is missing or unreadable.
    pub fn load(archive: &mut Archive, folder: &str, name: &str) -> Option<Self> {
        let material_path = format!("{folder}/{name}.mat");
        let material = archive.read(&material_path).ok().flatten()?;
        let material = String::from_utf8_lossy(&material);
        let is_picture = !material.contains("ui.sdf.rfx");
        let (source, colours) = parse_material(&material)?;
        let texture_path = if source.starts_with('/') {
            source
        } else {
            format!("{folder}/{source}")
        };
        let texture = archive.read_texture(&texture_path).ok().flatten()?;
        let rgba = decode_rgba(&texture)?;
        let mut masks: [Vec<u8>; 4] = Default::default();
        for (channel, mask) in masks.iter_mut().enumerate() {
            *mask = rgba
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[channel])
                .collect();
        }
        Some(GameIcon {
            name: format!("{folder}/{name}"),
            width: texture.width,
            height: texture.height,
            masks,
            colours,
            is_picture,
        })
    }

    /// A copy shrunk to fit `size` pixels (keeping its shape), to keep many pictures in memory
    /// cheaply. Not grown when smaller already.
    pub fn shrunk(&self, size: u32) -> GameIcon {
        let scale = (size as f32 / self.width.max(self.height) as f32).min(1.0);
        let width = ((self.width as f32 * scale).round() as u32).max(1);
        let height = ((self.height as f32 * scale).round() as u32).max(1);
        GameIcon {
            name: self.name.clone(),
            width,
            height,
            masks: self
                .masks
                .each_ref()
                .map(|mask| self.shrink(mask, width, height)),
            colours: self.colours,
            is_picture: self.is_picture,
        }
    }

    /// The icon at `width`×`height` pixels, as blue, green, red, alpha bytes (what GPUI draws).
    /// `colours` replaces the material's colour of a channel where it's `Some`; a channel whose
    /// colour is fully transparent isn't drawn.
    ///
    /// The game draws its icons over dark panels, and some let that panel show through on
    /// purpose (the black half of the corrosive placard). `backdrop` paints that panel first, in
    /// the icon's own shape: every channel that isn't a solid square.
    pub fn render(
        &self,
        width: u32,
        height: u32,
        colours: [Option<[u8; 4]>; 4],
        backdrop: Option<[u8; 4]>,
    ) -> Vec<u8> {
        let (width, height) = (width.max(1), height.max(1));
        let masks = self
            .masks
            .each_ref()
            .map(|mask| self.shrink(mask, width, height));
        let mut bgra = vec![0u8; (width * height * 4) as usize];
        if self.is_picture {
            for (index, pixel) in bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                *pixel = [
                    masks[BLUE][index],
                    masks[GREEN][index],
                    masks[RED][index],
                    masks[ALPHA][index],
                ];
            }
            return bgra;
        }
        if let Some(backdrop) = backdrop {
            let shapes: Vec<&Vec<u8>> = (0..4)
                .filter(|channel| !self.masks[*channel].iter().all(|coverage| *coverage > 250))
                .map(|channel| &masks[channel])
                .collect();
            for (index, pixel) in bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                let coverage = shapes.iter().map(|mask| mask[index]).max().unwrap_or(0);
                let alpha = f32::from(coverage) / 255.0 * f32::from(backdrop[3]) / 255.0;
                blend_over(pixel, backdrop, alpha);
            }
        }
        for channel in DRAW_ORDER {
            let colour = colours[channel].unwrap_or(self.colours[channel]);
            if colour[3] == 0 {
                continue;
            }
            for (pixel, coverage) in bgra.as_chunks_mut::<4>().0.iter_mut().zip(&masks[channel]) {
                let alpha = f32::from(*coverage) / 255.0 * f32::from(colour[3]) / 255.0;
                blend_over(pixel, colour, alpha);
            }
        }
        bgra
    }

    /// Averages a mask down to `out_width`×`out_height` (a box filter, so small sizes stay
    /// smooth).
    fn shrink(&self, mask: &[u8], out_width: u32, out_height: u32) -> Vec<u8> {
        let (width, height) = (self.width as usize, self.height as usize);
        let (out_width, out_height) = (out_width as usize, out_height as usize);
        let mut out = Vec::with_capacity(out_width * out_height);
        for y in 0..out_height {
            let (top, bottom) = span(y, out_height, height);
            for x in 0..out_width {
                let (left, right) = span(x, out_width, width);
                let mut sum = 0u32;
                for row in top..bottom {
                    for column in left..right {
                        sum += u32::from(mask.get(row * width + column).copied().unwrap_or(0));
                    }
                }
                let count = ((bottom - top) * (right - left)).max(1) as u32;
                out.push((sum / count) as u8);
            }
        }
        out
    }
}

/// The source pixels an output pixel covers: at least one.
fn span(index: usize, out: usize, source: usize) -> (usize, usize) {
    let start = index * source / out;
    let end = ((index + 1) * source / out).max(start + 1).min(source);
    (start.min(source.saturating_sub(1)), end)
}

/// Paints `colour` at `alpha` over a BGRA pixel (straight, not premultiplied, alpha).
fn blend_over(pixel: &mut [u8], colour: [u8; 4], alpha: f32) {
    let below = f32::from(pixel[3]) / 255.0;
    let out = alpha + below * (1.0 - alpha);
    if out <= 0.0 {
        return;
    }
    // BGRA below, RGBA colour.
    for (at, channel) in [(2, 0), (1, 1), (0, 2)] {
        let mixed = (f32::from(colour[channel]) * alpha
            + f32::from(pixel[at]) * below * (1.0 - alpha))
            / out;
        pixel[at] = mixed.round() as u8;
    }
    pixel[3] = (out * 255.0).round() as u8;
}

/// From a material's text: its texture's `source` and colours `aux[1]` to `aux[4]`.
fn parse_material(text: &str) -> Option<(String, [[u8; 4]; 4])> {
    let mut source = None;
    let mut colours = [[0u8; 4]; 4];
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("source") {
            let value = rest
                .trim_start()
                .strip_prefix(':')?
                .trim()
                .trim_matches('"');
            source = Some(value.to_string());
        } else if let Some(rest) = line.strip_prefix("aux[") {
            let (index, rest) = rest.split_once(']')?;
            let Ok(index) = index.parse::<usize>() else {
                continue;
            };
            if !(1..=4).contains(&index) {
                continue;
            }
            let numbers = rest.split_once('{')?.1.split_once('}')?.0;
            let values: Vec<f32> = numbers
                .split(',')
                .filter_map(|number| number.trim().parse().ok())
                .collect();
            if let [red, green, blue, alpha] = values.as_slice() {
                colours[index - 1] = [
                    linear_to_srgb(*red),
                    linear_to_srgb(*green),
                    linear_to_srgb(*blue),
                    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
                ];
            }
        }
    }
    Some((source?, colours))
}

/// The game's material colours are linear; screens (and GPUI images) want sRGB.
fn linear_to_srgb(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let srgb = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (srgb * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATERIAL: &str = "effect : \"ui.sdf.rfx\" {\n\taux[0] : { 64.0, 64.0, 2.0, 0.0 }\n\taux[1] : { 0.0, 0.0, 0.0, 1.0 }\n\taux[2] : { 1.0, 0.21404, 0.0, 1.0 }\n\taux[3] : { 0.0, 0.0, 0.0, 0.0 }\n\taux[4] : { 1.0, 1.0, 1.0, 1.0 }\n\ttexture : \"texture\" {\n\t\tsource : \"icon.tobj\"\n\t}\n}\n";

    #[test]
    fn reads_a_material() {
        let (source, colours) = parse_material(MATERIAL).expect("parses");
        assert_eq!(source, "icon.tobj");
        assert_eq!(colours[0], [0, 0, 0, 255]);
        // Linear 0.214 is sRGB 0.5 (127 of 255).
        assert_eq!(colours[1], [255, 127, 0, 255]);
        assert_eq!(colours[2][3], 0);
        assert_eq!(colours[3], [255, 255, 255, 255]);
    }

    fn icon() -> GameIcon {
        // 2×2: green covers the left column, alpha (the fill) everything.
        GameIcon {
            name: "test".into(),
            width: 2,
            height: 2,
            masks: [vec![0; 4], vec![255, 0, 255, 0], vec![0; 4], vec![255; 4]],
            colours: [
                [0, 0, 0, 255],
                [255, 0, 0, 255],
                [0, 0, 0, 0],
                [0, 0, 255, 255],
            ],
            is_picture: false,
        }
    }

    #[test]
    fn draws_layers_back_to_front() {
        let bgra = icon().render(2, 2, [None; 4], None);
        // Left: green's red colour over the blue fill. Right: the blue fill. (BGRA.)
        assert_eq!(&bgra[0..4], &[0, 0, 255, 255]);
        assert_eq!(&bgra[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn shrinks_by_averaging_and_takes_colour_overrides() {
        let colours = [None, Some([0, 255, 0, 255]), None, Some([0, 0, 0, 0])];
        let bgra = icon().render(1, 1, colours, None);
        // Half covered by green, nothing below: green at half opacity.
        assert_eq!(bgra, vec![0, 255, 0, 127]);
    }

    #[test]
    fn paints_the_backdrop_in_the_icons_shape() {
        let mut icon = icon();
        // Only green is a shape; the solid alpha square isn't, and its colour is transparent.
        icon.colours = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
        let bgra = icon.render(2, 2, [None; 4], Some([10, 20, 30, 255]));
        assert_eq!(&bgra[0..4], &[30, 20, 10, 255]);
        assert_eq!(&bgra[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn draws_pictures_as_they_are() {
        let mut picture = icon();
        picture.is_picture = true;
        let bgra = picture.render(2, 2, [None; 4], None);
        // Pixel 0: red 0, green 255, blue 0, alpha 255.
        assert_eq!(&bgra[0..4], &[0, 255, 0, 255]);
        assert_eq!(picture.shrunk(1).width, 1);
    }
}
