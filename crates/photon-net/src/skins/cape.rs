//! A cape's image in the size the player cape model reads: 64 by 32 pixels. The reference
//! reads a cape of any size, as the model's texture coordinates are fractions of it; places of
//! one size hold them here, so a larger cape of a whole multiple is sampled down to it (past 256 by 128).

/// The width and height of a cape the model reads, in pixels.
pub const CAPE_WIDTH: usize = 64;
pub const CAPE_HEIGHT: usize = 32;
/// The width and height of a place for a cape: four times the model's, so that a cape of that size is
/// kept whole (a smaller one is doubled, which draws the same).
pub const PLACE_WIDTH: usize = CAPE_WIDTH * 4;
pub const PLACE_HEIGHT: usize = CAPE_HEIGHT * 4;

/// The pixels of a cape image (RGBA bytes) as the place holds them; an error where its size is
/// not that of a cape or a whole multiple of it.
pub fn fit_cape(rgba: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let scale = width / CAPE_WIDTH;
    if scale == 0
        || width != CAPE_WIDTH * scale
        || height != CAPE_HEIGHT * scale
        || rgba.len() != width * height * 4
    {
        return Err(format!(
            "Discarding incorrectly sized ({width}x{height}) cape texture"
        ));
    }
    let mut out = Vec::with_capacity(PLACE_WIDTH * PLACE_HEIGHT * 4);
    for y in 0..PLACE_HEIGHT {
        for x in 0..PLACE_WIDTH {
            let from = ((y * height / PLACE_HEIGHT) * width + x * width / PLACE_WIDTH) * 4;
            out.extend_from_slice(&rgba[from..from + 4]);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: usize, height: usize) -> Vec<u8> {
        (0..width * height)
            .flat_map(|at| [(at % width) as u8, (at / width) as u8, 0, 255])
            .collect()
    }

    #[test]
    fn a_cape_of_the_model_size_is_quadrupled() {
        let out = fit_cape(&image(64, 32), 64, 32).unwrap();
        assert_eq!(out.len(), 256 * 128 * 4);
        // The pixel at (9, 21) of the place is the one at (2, 5).
        let at = (21 * 256 + 9) * 4;
        assert_eq!(out[at..at + 2], [2, 5]);
    }

    #[test]
    fn a_cape_of_the_place_size_is_kept() {
        let cape = image(256, 128);
        assert_eq!(fit_cape(&cape, 256, 128).unwrap(), cape);
    }

    #[test]
    fn a_larger_cape_is_sampled_down() {
        let out = fit_cape(&image(512, 256), 512, 256).unwrap();
        assert_eq!(out.len(), 256 * 128 * 4);
        // The pixel at (3, 5) is the one at (6, 10) of the larger image.
        let at = (5 * 256 + 3) * 4;
        assert_eq!(out[at..at + 2], [6, 10]);
    }

    #[test]
    fn other_sizes_are_discarded() {
        assert!(fit_cape(&image(22, 17), 22, 17).is_err());
        assert!(fit_cape(&image(64, 64), 64, 64).is_err());
    }
}
