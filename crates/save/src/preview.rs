//! A slot's preview picture (`preview.tga`): the snapshot the game's load menu shows. The game
//! writes a 256×128, 24-bit, uncompressed TGA; this also reads run-length compressed and 32-bit
//! ones, in either row order.

/// Pixels in blue, green, red, alpha order (what TGA stores, and what GPUI draws).
#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Decodes a true-colour TGA. `None` for anything else (or a damaged file).
pub fn decode_tga(bytes: &[u8]) -> Option<Picture> {
    let header = bytes.get(..18)?;
    let id_length = usize::from(header[0]);
    let has_colour_map = header[1] != 0;
    let image_type = header[2];
    let width = u32::from(u16::from_le_bytes([header[12], header[13]]));
    let height = u32::from(u16::from_le_bytes([header[14], header[15]]));
    let bits = header[16];
    let starts_at_top = header[17] & 0x20 != 0;
    let is_compressed = match image_type {
        2 => false,
        10 => true,
        _ => return None,
    };
    if has_colour_map || !(bits == 24 || bits == 32) || width == 0 || height == 0 {
        return None;
    }
    // A preview is small; refuse anything absurd before allocating.
    if width > 8192 || height > 8192 {
        return None;
    }

    let bytes_per_pixel = usize::from(bits / 8);
    let pixels = (width * height) as usize;
    let mut data = bytes.get(18 + id_length..)?;
    let mut bgra = Vec::with_capacity(pixels * 4);
    let mut push = |pixel: &[u8]| {
        bgra.extend_from_slice(&pixel[..3]);
        bgra.push(if bytes_per_pixel == 4 { pixel[3] } else { 255 });
    };

    if is_compressed {
        let mut done = 0;
        while done < pixels {
            let (&packet, rest) = data.split_first()?;
            data = rest;
            let count = usize::from(packet & 0x7F) + 1;
            if packet & 0x80 != 0 {
                let pixel = data.get(..bytes_per_pixel)?;
                for _ in 0..count {
                    push(pixel);
                }
                data = &data[bytes_per_pixel..];
            } else {
                let run = data.get(..count * bytes_per_pixel)?;
                for pixel in run.chunks_exact(bytes_per_pixel) {
                    push(pixel);
                }
                data = &data[count * bytes_per_pixel..];
            }
            done += count;
        }
        bgra.truncate(pixels * 4);
    } else {
        let raw = data.get(..pixels * bytes_per_pixel)?;
        for pixel in raw.chunks_exact(bytes_per_pixel) {
            push(pixel);
        }
    }

    // TGA rows run bottom-up unless the header says otherwise.
    if !starts_at_top {
        let row = (width * 4) as usize;
        let flipped: Vec<u8> = bgra.chunks_exact(row).rev().flatten().copied().collect();
        bgra = flipped;
    }
    Some(Picture {
        width,
        height,
        bgra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(image_type: u8, width: u16, height: u16, bits: u8, top: bool) -> Vec<u8> {
        let mut header = vec![0u8; 18];
        header[2] = image_type;
        header[12..14].copy_from_slice(&width.to_le_bytes());
        header[14..16].copy_from_slice(&height.to_le_bytes());
        header[16] = bits;
        header[17] = if top { 0x20 } else { 0 };
        header
    }

    #[test]
    fn reads_an_uncompressed_picture_bottom_up() {
        // 1×2, bottom row first: blue pixel at the bottom, red on top.
        let mut bytes = header(2, 1, 2, 24, false);
        bytes.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
        let picture = decode_tga(&bytes).expect("decodes");
        assert_eq!((picture.width, picture.height), (1, 2));
        assert_eq!(picture.bgra, [0, 0, 255, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn reads_a_compressed_picture() {
        let mut bytes = header(10, 3, 1, 32, true);
        // Three repeats of one pixel, as a run.
        bytes.extend_from_slice(&[0x82, 1, 2, 3, 4]);
        let picture = decode_tga(&bytes).expect("decodes");
        assert_eq!(picture.bgra, [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn refuses_what_it_cant_read() {
        assert_eq!(decode_tga(&[]), None);
        assert_eq!(decode_tga(&header(1, 1, 1, 8, false)), None);
        let mut short = header(2, 4, 4, 24, false);
        short.extend_from_slice(&[0; 5]);
        assert_eq!(decode_tga(&short), None);
    }
}
