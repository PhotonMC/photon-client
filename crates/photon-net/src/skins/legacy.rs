//! `SkinTextureDownloader.processLegacySkin`: a 64x32 skin is widened to 64x64 with the left
//! limbs mirrored from the right ones, and the parts that must not be see-through are made
//! opaque. Images are RGBA bytes.

const SIZE: usize = 64;

/// A pixel image of 64 by 64, four bytes a pixel.
struct Skin(Vec<u8>);

impl Skin {
    fn at(x: usize, y: usize) -> usize {
        (y * SIZE + x) * 4
    }

    /// `NativeImage.copyRect(x, y, dx, dy, w, h, true, false)`: a rectangle moved by an offset,
    /// mirrored left to right.
    fn copy_mirrored(
        &mut self,
        (x, y): (usize, usize),
        offset: (isize, usize),
        (w, h): (usize, usize),
    ) {
        for row in 0..h {
            for column in 0..w {
                let from = Self::at(x + column, y + row);
                let to = Self::at(
                    x.wrapping_add_signed(offset.0) + (w - 1 - column),
                    offset.1 + y + row,
                );
                let pixel: [u8; 4] = self.0[from..from + 4].try_into().expect("a pixel");
                self.0[to..to + 4].copy_from_slice(&pixel);
            }
        }
    }

    /// `setNoAlpha`: every pixel of the rectangle opaque.
    fn make_opaque(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize)) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.0[Self::at(x, y) + 3] = 255;
            }
        }
    }

    /// `doNotchTransparencyHack`: a rectangle with no see-through pixel has its alpha cleared.
    fn clear_unneeded_alpha(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize)) {
        let any_clear = (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .any(|(x, y)| self.0[Self::at(x, y) + 3] < 128);
        if any_clear {
            return;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                self.0[Self::at(x, y) + 3] = 0;
            }
        }
    }
}

/// The skin a 64x64 or 64x32 image makes; any other size is discarded.
pub fn process_legacy_skin(rgba: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    if width != SIZE || !(height == 32 || height == SIZE) || rgba.len() != width * height * 4 {
        return Err(format!(
            "Discarding incorrectly sized ({width}x{height}) skin texture"
        ));
    }
    let legacy = height == 32;
    let mut skin = Skin(vec![0; SIZE * SIZE * 4]);
    skin.0[..rgba.len()].copy_from_slice(rgba);
    if legacy {
        // The left leg and arm, from the right ones: (x, y), (offset x, target row), (w, h).
        for (from, offset_x, size) in [
            ((4, 16), 16, (4, 4)),
            ((8, 16), 16, (4, 4)),
            ((0, 20), 24, (4, 12)),
            ((4, 20), 16, (4, 12)),
            ((8, 20), 8, (4, 12)),
            ((12, 20), 16, (4, 12)),
            ((44, 16), -8, (4, 4)),
            ((48, 16), -8, (4, 4)),
            ((40, 20), 0, (4, 12)),
            ((44, 20), -8, (4, 12)),
            ((48, 20), -16, (4, 12)),
            ((52, 20), -8, (4, 12)),
        ] {
            skin.copy_mirrored(from, (offset_x, 32), size);
        }
    }
    skin.make_opaque((0, 0), (32, 16));
    if legacy {
        skin.clear_unneeded_alpha((32, 0), (64, 32));
    }
    skin.make_opaque((0, 16), (64, 32));
    skin.make_opaque((16, 48), (48, 64));
    Ok(skin.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(height: usize, fill: impl Fn(usize, usize) -> [u8; 4]) -> Vec<u8> {
        (0..height)
            .flat_map(|y| (0..SIZE).flat_map(move |x| [x, y]))
            .collect::<Vec<_>>()
            .chunks(2)
            .flat_map(|xy| fill(xy[0], xy[1]))
            .collect()
    }

    fn pixel(skin: &[u8], x: usize, y: usize) -> [u8; 4] {
        skin[Skin::at(x, y)..Skin::at(x, y) + 4].try_into().unwrap()
    }

    #[test]
    fn other_sizes_are_discarded() {
        assert!(process_legacy_skin(&[0; 64 * 16 * 4], 64, 16).is_err());
        assert!(process_legacy_skin(&[0; 32 * 32 * 4], 32, 32).is_err());
        assert!(process_legacy_skin(&[0; 10], 64, 64).is_err());
    }

    #[test]
    fn a_modern_skin_is_only_made_opaque_where_it_must_be() {
        let skin = image(64, |_, _| [9, 8, 7, 0]);
        let out = process_legacy_skin(&skin, 64, 64).unwrap();
        // The head base is opaque, the hat layer keeps its alpha.
        assert_eq!(pixel(&out, 5, 5), [9, 8, 7, 255]);
        assert_eq!(pixel(&out, 40, 5), [9, 8, 7, 0]);
        // Body, arms and legs bases are opaque; the second layer of the body is not.
        assert_eq!(pixel(&out, 20, 20), [9, 8, 7, 255]);
        assert_eq!(pixel(&out, 20, 40), [9, 8, 7, 0]);
        assert_eq!(pixel(&out, 20, 55), [9, 8, 7, 255]);
    }

    #[test]
    fn a_legacy_skin_gets_its_left_limbs_mirrored_from_the_right() {
        // Every pixel names its place in red and green.
        let skin = image(32, |x, y| [x as u8, y as u8, 0, 255]);
        let out = process_legacy_skin(&skin, 64, 32).unwrap();
        assert_eq!(out.len(), 64 * 64 * 4);
        // The right leg top (4,16) moves to (20,48), flipped within its width of four.
        assert_eq!(pixel(&out, 20, 48)[..2], [7, 16]);
        assert_eq!(pixel(&out, 23, 48)[..2], [4, 16]);
        // The right arm's front (44,20) goes 8 left of its place on the new rows (36,52).
        assert_eq!(pixel(&out, 36, 52)[..2], [47, 20]);
        // What nothing was copied to stays clear, and the limb bases are opaque.
        assert_eq!(pixel(&out, 0, 40), [0, 0, 0, 0]);
        assert_eq!(pixel(&out, 20, 50)[3], 255);
    }

    #[test]
    fn the_legacy_hat_loses_its_alpha_only_when_it_has_no_see_through_pixel() {
        let opaque = image(32, |_, _| [1, 2, 3, 255]);
        let out = process_legacy_skin(&opaque, 64, 32).unwrap();
        assert_eq!(pixel(&out, 40, 5)[3], 0);
        let holed = image(32, |x, y| {
            [1, 2, 3, if (x, y) == (50, 10) { 0 } else { 255 }]
        });
        let out = process_legacy_skin(&holed, 64, 32).unwrap();
        assert_eq!(pixel(&out, 40, 5)[3], 255);
        assert_eq!(pixel(&out, 50, 10)[3], 0);
    }
}
