//! Colours between the save's bytes (sRGB, as the game's saved colours and screens show them) and
//! GPUI's colours, for colour pickers and swatches.

use gpui_kit::{Hsla, Rgba};

/// A colour on screen from sRGB bytes.
pub fn to_hsla([r, g, b]: save::Rgb8) -> Hsla {
    Rgba {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: 1.0,
    }
    .into()
}

/// A colour picked on screen as sRGB bytes (any transparency is dropped: paint is opaque).
pub fn to_rgb8(colour: Hsla) -> save::Rgb8 {
    let rgba = Rgba::from(colour);
    [rgba.r, rgba.g, rgba.b].map(|part| (part.clamp(0.0, 1.0) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_round_trip() {
        for colour in [[0u8, 0, 0], [255, 255, 255], [18, 52, 86], [200, 10, 99]] {
            assert_eq!(to_rgb8(to_hsla(colour)), colour);
        }
    }
}
