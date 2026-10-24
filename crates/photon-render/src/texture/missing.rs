//! The picture of the missing texture.

use super::png::DecodedImage;

/// `MissingTextureAtlasSprite.generateMissingImage`: 16 by 16, magenta where exactly one of the
/// row and the column is in the first half, black elsewhere.
pub fn missing_image() -> DecodedImage {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let pink = (y < 8) ^ (x < 8);
            rgba.extend_from_slice(if pink {
                &[0xF8, 0x00, 0xF8, 0xFF]
            } else {
                &[0, 0, 0, 0xFF]
            });
        }
    }
    DecodedImage {
        width: 16,
        height: 16,
        rgba,
    }
}
