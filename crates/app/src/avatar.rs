//! The profile's picture: whatever the player picks (PNG, JPEG or BMP) made into the square PNG
//! the game shows for the profile, cut from the middle of the picture.

use std::io::Cursor;

use image::imageops::FilterType;

/// The picture's side, in pixels.
pub const AVATAR_SIZE: u32 = 256;

/// Turns a picture file's bytes into the profile's picture: the largest square from its middle,
/// scaled to `AVATAR_SIZE`, as PNG.
pub fn make_avatar(bytes: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let picture = image::load_from_memory(bytes)?;
    let side = picture.width().min(picture.height());
    let left = (picture.width() - side) / 2;
    let top = (picture.height() - side) / 2;
    let square = picture.crop_imm(left, top, side, side).resize_exact(
        AVATAR_SIZE,
        AVATAR_SIZE,
        FilterType::Lanczos3,
    );
    let mut png = Vec::new();
    square.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_a_square_png_from_the_middle() {
        // A 4 × 2 picture: red on the left and right quarters, blue in the middle half.
        let mut wide = image::RgbImage::new(4, 2);
        for (x, _, pixel) in wide.enumerate_pixels_mut() {
            *pixel = if x == 0 || x == 3 {
                image::Rgb([255, 0, 0])
            } else {
                image::Rgb([0, 0, 255])
            };
        }
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(wide)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("encodes");

        let png = make_avatar(&bytes).expect("makes a picture");
        let made = image::load_from_memory(&png).expect("decodes").to_rgb8();
        assert_eq!(made.dimensions(), (AVATAR_SIZE, AVATAR_SIZE));
        // Only the middle was kept: blue throughout.
        assert_eq!(made.get_pixel(0, 0), &image::Rgb([0, 0, 255]));
        assert_eq!(
            made.get_pixel(AVATAR_SIZE - 1, AVATAR_SIZE - 1),
            &image::Rgb([0, 0, 255])
        );
    }

    #[test]
    fn says_so_when_it_isnt_a_picture() {
        assert!(make_avatar(b"not a picture").is_err());
    }
}
