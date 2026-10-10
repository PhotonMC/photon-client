//! The quads laid over the view from inside (`ScreenEffectRenderer`): the block the eyes are in,
//! the water over the eyes and the fire the player burns in. They are in the hand's view space,
//! a half block in front of the camera.

use crate::joml::Matrix4;
use crate::particles::color_from_float;

/// One corner of a screen effect's quad: where, which texel and which colour (ARGB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: i32,
}

/// The colour of the block the eyes are in (`-15132391`).
pub const BLOCK_COLOR: i32 = -15_132_391;

/// The colour of the fire quads (`-436207617`).
const FIRE_COLOR: i32 = -436_207_617;

/// `ScreenEffectRenderer.buildQuad`: corners `(x0, y0)`, `(x1, y0)`, `(x1, y1)`, `(x0, y1)` at `z`.
fn quad(
    pose: &Matrix4,
    [x0, y0, x1, y1]: [f32; 4],
    z: f32,
    [u0, v0, u1, v1]: [f32; 4],
    color: i32,
) -> [EffectVertex; 4] {
    let corner = |x: f32, y: f32, u: f32, v: f32| EffectVertex {
        position: pose.transform_position(x, y, z),
        uv: [u, v],
        color,
    };
    [
        corner(x0, y0, u0, v0),
        corner(x1, y0, u1, v0),
        corner(x1, y1, u1, v1),
        corner(x0, y1, u0, v1),
    ]
}

/// `buildSpriteQuad`: the sprite's corners (`u0`, `u1`, `v0`, `v1`) laid from the far one.
fn sprite_quad(
    pose: &Matrix4,
    corners: [f32; 4],
    sprite: [f32; 4],
    color: i32,
) -> [EffectVertex; 4] {
    let [u0, u1, v0, v1] = sprite;
    quad(pose, corners, -0.5, [u1, v1, u0, v0], color)
}

/// `submitBlockSprite`: the sprite over the whole view.
pub fn block_quad(sprite: [f32; 4], color: i32) -> [EffectVertex; 4] {
    sprite_quad(&Matrix4::identity(), [-1.0, -1.0, 1.0, 1.0], sprite, color)
}

/// `submitWater`: the water texture over the whole view, moving with where the player looks,
/// as bright as the light at the eyes.
pub fn water_quad(y_rot: f32, x_rot: f32, brightness: f32) -> [EffectVertex; 4] {
    let color = color_from_float([brightness, brightness, brightness, 0.1]);
    let u0 = -y_rot / 64.0;
    let v0 = x_rot / 64.0;
    quad(
        &Matrix4::identity(),
        [-1.0, -1.0, 1.0, 1.0],
        -0.5,
        [u0 + 4.0, v0 + 4.0, u0, v0],
        color,
    )
}

/// `submitFire`: two quads of the fire sprite, low on either side and turned a little inwards.
pub fn fire_quads(sprite: [f32; 4]) -> [EffectVertex; 8] {
    let tilt = (std::f64::consts::PI / 18.0) as f32;
    let side = |x: f32, angle: f32| {
        let mut pose = Matrix4::identity();
        pose.translate(x, -0.3, 0.0);
        pose.rotate_y(angle);
        sprite_quad(&pose, [-0.5, -0.5, 0.5, 0.5], sprite, FIRE_COLOR)
    };
    let (right, left) = (side(0.24, -tilt), side(-0.24, tilt));
    std::array::from_fn(|i| if i < 4 { right[i] } else { left[i - 4] })
}
