//! A cape's image in the size the player cape model reads: 64 by 32 pixels. The reference
//! reads a cape of any size, as the model's texture coordinates are fractions of it; places of
//! one size hold them here, so a larger cape of a whole multiple is sampled down to it.

/// The width and height of a cape, in pixels.
pub const CAPE_WIDTH: usize = 64;
pub const CAPE_HEIGHT: usize = 32;

/// The pixels of a cape image (RGBA bytes) as 64 by 32; an error where its size is not that of
/// a cape or a whole multiple of it.
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
    if scale == 1 {
        return Ok(rgba.to_vec());
    }
    let mut out = Vec::with_capacity(CAPE_WIDTH * CAPE_HEIGHT * 4);
    for y in 0..CAPE_HEIGHT {
        for x in 0..CAPE_WIDTH {
            let from = ((y * scale) * width + x * scale) * 4;
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
    fn a_cape_of_the_size_is_kept() {
        let cape = image(64, 32);
        assert_eq!(fit_cape(&cape, 64, 32).unwrap(), cape);
    }

    #[test]
    fn a_larger_cape_is_sampled_down() {
        let out = fit_cape(&image(128, 64), 128, 64).unwrap();
        assert_eq!(out.len(), 64 * 32 * 4);
        // The pixel at (3, 5) is the one at (6, 10) of the larger image.
        let at = (5 * 64 + 3) * 4;
        assert_eq!(out[at..at + 2], [6, 10]);
    }

    #[test]
    fn other_sizes_are_discarded() {
        assert!(fit_cape(&image(22, 17), 22, 17).is_err());
        assert!(fit_cape(&image(64, 64), 64, 64).is_err());
    }
}
