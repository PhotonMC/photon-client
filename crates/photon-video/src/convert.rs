//! 8-bit 4:2:0 planes to RGBA (BT.709, limited range).

/// RGBA of a `width` x `height` picture from its planes and their strides.
pub fn yuv420_to_rgba(
    (y_plane, y_stride): (&[u8], usize),
    (u_plane, v_plane, c_stride): (&[u8], &[u8], usize),
    (width, height): (usize, usize),
) -> Vec<u8> {
    let mut out = Vec::with_capacity(width * height * 4);
    for row in 0..height {
        for column in 0..width {
            let y = f32::from(y_plane[row * y_stride + column]) - 16.0;
            let chroma = (row / 2) * c_stride + column / 2;
            let u = f32::from(u_plane[chroma]) - 128.0;
            let v = f32::from(v_plane[chroma]) - 128.0;
            let y = 1.164 * y;
            let r = y + 1.793 * v;
            let g = y - 0.213 * u - 0.533 * v;
            let b = y + 2.112 * u;
            out.extend_from_slice(&[clamp(r), clamp(g), clamp(b), 255]);
        }
    }
    out
}

fn clamp(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_white_and_grey_come_out_right() {
        let y = [16u8, 235, 126, 126];
        let rgba = yuv420_to_rgba((&y, 2), (&[128], &[128], 1), (2, 2));
        assert_eq!(&rgba[0..4], &[0, 0, 0, 255]);
        assert_eq!(&rgba[4..8], &[255, 255, 255, 255]);
        assert!(rgba[8].abs_diff(128) <= 1);
    }

    #[test]
    fn the_stride_is_skipped() {
        let y = [16u8, 16, 99, 99, 235, 235, 99, 99];
        let rgba = yuv420_to_rgba((&y, 4), (&[128, 0], &[128, 0], 2), (2, 2));
        assert_eq!(&rgba[0..3], &[0, 0, 0]);
        assert_eq!(&rgba[8..11], &[255, 255, 255]);
    }
}
