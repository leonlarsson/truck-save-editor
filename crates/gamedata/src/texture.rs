//! Turning the game's textures into pixels, and its distance-field icons into crisp shapes.

use hashfs::{Texture, TextureFormat};

/// A texture's pixels as red, green, blue, alpha bytes, row by row. Single-channel formats (R8,
/// BC4) fill red only: their one channel is a shape of its own, like each channel of an icon.
pub fn decode_rgba(texture: &Texture) -> Option<Vec<u8>> {
    let width = texture.width as usize;
    let height = texture.height as usize;
    let mut rgba = vec![0u8; width * height * 4];
    match texture.format {
        TextureFormat::Rgba8 => {
            // Uncompressed textures are stored blue, green, red, alpha (Direct3D's B8G8R8A8):
            // read as red-first, faces came out blue and oranges sky blue.
            let data = texture.data.get(..width * height * 4)?;
            for (pixel, stored) in rgba
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(data.as_chunks::<4>().0)
            {
                *pixel = [stored[2], stored[1], stored[0], stored[3]];
            }
        }
        TextureFormat::R8 => {
            for (pixel, value) in rgba.as_chunks_mut::<4>().0.iter_mut().zip(&texture.data) {
                pixel.copy_from_slice(&[*value, 0, 0, 255]);
            }
        }
        format => decode_blocks(texture, format, &mut rgba)?,
    }
    Some(rgba)
}

/// BC formats: 4×4 blocks, each decoded into its place in the image.
fn decode_blocks(texture: &Texture, format: TextureFormat, rgba: &mut [u8]) -> Option<()> {
    let width = texture.width as usize;
    let height = texture.height as usize;
    let (_, block_bytes) = format.block();
    let columns = width.div_ceil(4);
    let rows = height.div_ceil(4);
    let mut block_pixels = [0u8; 4 * 4 * 4];
    let mut single = [0u8; 4 * 4];
    let mut pair = [0u8; 4 * 4 * 2];
    for row in 0..rows {
        for column in 0..columns {
            let at = (row * columns + column) * block_bytes;
            let block = texture.data.get(at..at + block_bytes)?;
            match format {
                TextureFormat::Bc1 => bcdec_rs::bc1(block, &mut block_pixels, 16),
                TextureFormat::Bc7 => bcdec_rs::bc7(block, &mut block_pixels, 16),
                TextureFormat::Bc4 => {
                    bcdec_rs::bc4(block, &mut single, 4, false);
                    for (pixel, value) in block_pixels.as_chunks_mut::<4>().0.iter_mut().zip(single)
                    {
                        pixel.copy_from_slice(&[value, 0, 0, 255]);
                    }
                }
                TextureFormat::Bc5 => {
                    bcdec_rs::bc5(block, &mut pair, 8, false);
                    for (pixel, rg) in block_pixels
                        .as_chunks_mut::<4>()
                        .0
                        .iter_mut()
                        .zip(pair.as_chunks::<2>().0)
                    {
                        pixel.copy_from_slice(&[rg[0], rg[1], 0, 255]);
                    }
                }
                TextureFormat::Rgba8 | TextureFormat::R8 => return None,
            }
            // Copy the block's 4×4 pixels into the image, clipped at its right and bottom edges.
            for y in 0..4 {
                let image_y = row * 4 + y;
                if image_y >= height {
                    break;
                }
                for x in 0..4 {
                    let image_x = column * 4 + x;
                    if image_x >= width {
                        break;
                    }
                    let from = (y * 4 + x) * 4;
                    let to = (image_y * width + image_x) * 4;
                    rgba[to..to + 4].copy_from_slice(&block_pixels[from..from + 4]);
                }
            }
        }
    }
    Some(())
}

/// A distance-field icon (the game's `ui.sdf.rfx` material) as coverage, 0–255 per pixel: the
/// shape is where the distance crosses the middle, with a soft edge a little wider than a pixel.
pub fn distance_field_coverage(texture: &Texture, channel: usize) -> Option<Vec<u8>> {
    let rgba = decode_rgba(texture)?;
    Some(
        rgba.as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| {
                let distance = f32::from(pixel[channel.min(3)]) / 255.0;
                let edge = smoothstep(0.44, 0.56, distance);
                (edge * 255.0).round() as u8
            })
            .collect(),
    )
}

fn smoothstep(from: f32, to: f32, value: f32) -> f32 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_formats() {
        let texture = Texture {
            width: 2,
            height: 1,
            format: TextureFormat::R8,
            data: vec![0, 200],
        };
        assert_eq!(
            decode_rgba(&texture),
            Some(vec![0, 0, 0, 255, 200, 0, 0, 255])
        );
    }

    #[test]
    fn turns_distances_into_a_crisp_shape() {
        let texture = Texture {
            width: 3,
            height: 1,
            format: TextureFormat::R8,
            data: vec![0, 128, 255],
        };
        let coverage = distance_field_coverage(&texture, 0).expect("decodes");
        assert_eq!(coverage[0], 0);
        assert!(coverage[1] > 100 && coverage[1] < 155);
        assert_eq!(coverage[2], 255);
    }
}
