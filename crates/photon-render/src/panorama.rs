//! The title screen's panorama (`Panorama`, `CubeMap`): the cube map's layers, the spin and the
//! matrix the cube is seen through.

// Matrix entries are addressed by column and row, as the library writes them.
#![expect(
    clippy::needless_range_loop,
    reason = "index loops that follow JOML's element-wise arithmetic"
)]

use crate::scene::{multiply, perspective};
use crate::texture::png::DecodedImage;

/// `PanoramaRenderer`'s constants: the cube map's projection and its fixed pitch.
const FOV_DEGREES: f32 = 85.0;
const Z_NEAR: f32 = 0.05;
const Z_FAR: f32 = 10.0;
pub const PITCH_DEGREES: f32 = 10.0;

/// The file suffix each cube layer reads (`CubeMapTexture.SUFFIXES`): +x, -x, +y, -y, +z, -z.
pub const LAYER_FILES: [usize; 6] = [1, 3, 5, 4, 0, 2];

/// The six faces `panorama_0` to `panorama_5` as the cube map's layers: in the layers' order, each
/// with its rows reversed (`copyRect` into the stacked image mirrors vertically).
pub fn layers(faces: &[DecodedImage; 6]) -> [DecodedImage; 6] {
    std::array::from_fn(|layer| {
        let face = &faces[LAYER_FILES[layer]];
        let row = face.width as usize * 4;
        DecodedImage {
            width: face.width,
            height: face.height,
            rgba: face
                .rgba
                .chunks_exact(row)
                .rev()
                .flatten()
                .copied()
                .collect(),
        }
    })
}

/// `Panorama.extractRenderState`'s spin after a frame of `real_delta_ticks` at the option's speed.
pub fn spin_after(spin: f32, real_delta_ticks: f32, speed: f64) -> f32 {
    let delta = (f64::from(real_delta_ticks) * speed) as f32;
    photon_core::math::wrap_degrees_f32(spin + delta * 0.1)
}

/// `Timer.getRealtimeDeltaTicks`: the frame's seconds in ticks, or half a tick after a stall.
pub fn real_delta_ticks(elapsed_seconds: f32) -> f32 {
    let ticks = elapsed_seconds * 1000.0 / 50.0;
    if ticks > 7.0 { 0.5 } else { ticks }
}

/// `Matrix4f.rotationX`.
fn rotation_x(angle: f32) -> [[f32; 4]; 4] {
    let sin = crate::joml::sin(angle);
    let cos = crate::joml::cos_from_sin(sin, angle);
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, cos, sin, 0.0],
        [0.0, -sin, cos, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `Matrix4f.rotateX`: the matrix turned further about x, each product and sum rounded as the
/// library does (columns 1 and 2 mix).
fn rotate_x(mut matrix: [[f32; 4]; 4], angle: f32) -> [[f32; 4]; 4] {
    let sin = crate::joml::sin(angle);
    let cos = crate::joml::cos_from_sin(sin, angle);
    for row in 0..4 {
        let (m1, m2) = (matrix[1][row], matrix[2][row]);
        matrix[1][row] = m1 * cos + m2 * sin;
        matrix[2][row] = m1 * -sin + m2 * cos;
    }
    matrix
}

/// `Matrix4f.rotateY`: columns 0 and 2 mix.
fn rotate_y(mut matrix: [[f32; 4]; 4], angle: f32) -> [[f32; 4]; 4] {
    let sin = crate::joml::sin(angle);
    let cos = crate::joml::cos_from_sin(sin, angle);
    for row in 0..4 {
        let (m0, m2) = (matrix[0][row], matrix[2][row]);
        matrix[0][row] = m0 * cos + m2 * -sin;
        matrix[2][row] = m0 * sin + m2 * cos;
    }
    matrix
}

/// `CubeMap.render`'s model-view: turned half a turn about x, then by the pitch about x and by the
/// render state's spin (`-spin`) about y.
pub fn model_view(rot_x_degrees: f32, rot_y_degrees: f32) -> [[f32; 4]; 4] {
    let radians = |degrees: f32| degrees * (std::f64::consts::PI / 180.0) as f32;
    let pitched = rotate_x(rotation_x(std::f32::consts::PI), radians(rot_x_degrees));
    rotate_y(pitched, radians(rot_y_degrees))
}

/// The cube's view-projection for a window of `width` by `height` pixels at a render-state spin.
pub fn view_projection(spin: f32, width: u32, height: u32) -> [[f32; 4]; 4] {
    let projection = perspective(
        FOV_DEGREES * (std::f64::consts::PI / 180.0) as f32,
        width as f32 / height.max(1) as f32,
        Z_NEAR,
        Z_FAR,
    );
    multiply(projection, model_view(PITCH_DEGREES, spin))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(tag: u8) -> DecodedImage {
        DecodedImage {
            width: 1,
            height: 2,
            rgba: vec![tag, 0, 0, 255, tag, 1, 0, 255],
        }
    }

    #[test]
    fn layers_follow_the_cube_map_order_with_rows_reversed() {
        let faces: [DecodedImage; 6] = std::array::from_fn(|i| face(i as u8));
        let layers = layers(&faces);
        let tags: Vec<u8> = layers.iter().map(|layer| layer.rgba[0]).collect();
        assert_eq!(tags, [1, 3, 5, 4, 0, 2]);
        // The second row comes first.
        assert_eq!(layers[0].rgba[1], 1);
        assert_eq!(layers[0].rgba[5], 0);
    }

    #[test]
    fn the_spin_wraps_to_half_turns() {
        assert_eq!(spin_after(179.0, 10.0, 1.0), -180.0);
        assert!((spin_after(0.0, 1.0, 1.0) - 0.1).abs() < 1e-6);
    }
}
