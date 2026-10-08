//! A leash (`LeashFeatureRenderer`): the line from a leashed entity to whoever holds it, a strip
//! of twenty-four steps drawn out and back, two tones of the lead's brown, hanging in a curve
//! when it has slack.

use photon_core::math::{mth_inv_sqrt_f32, mth_lerp_f32};

use crate::objects::fishing::{x_rot, y_rot};

use crate::joml::Pose;

/// How many steps the strip has.
const STEPS: i32 = 24;
/// The strip's width.
const WIDTH: f32 = 0.05;

/// `EntityRenderState.LeashState`, as far as the line is drawn from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeashState {
    /// Where on the entity the line is tied, turned with the entity.
    pub offset: [f64; 3],
    /// The tied end and the held end.
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub start_block_light: i32,
    pub end_block_light: i32,
    pub start_sky_light: i32,
    pub end_sky_light: i32,
    /// Whether the line hangs in a curve.
    pub slack: bool,
}

/// One vertex of the strip as the reference hands it to the vertex consumer: placed, with its
/// colour as bytes and its packed light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeashVertex {
    pub position: [f32; 3],
    pub colour: [u8; 3],
    pub light: i32,
}

/// `Mth.lerp` of floats.
fn lerp(delta: f32, from: f32, to: f32) -> f32 {
    from + delta * (to - from)
}

/// `LeashFeatureRenderer.addVertexPair`.
#[expect(clippy::too_many_arguments, reason = "mirrors the reference's method")]
fn vertex_pair(
    out: &mut Vec<LeashVertex>,
    pose: &Pose,
    [dx, dy, dz]: [f32; 3],
    fudge: f32,
    [dx_off, dz_off]: [f32; 2],
    k: i32,
    backwards: bool,
    state: &LeashState,
) {
    let progress = k as f32 / STEPS as f32;
    let block = lerp(
        progress,
        state.start_block_light as f32,
        state.end_block_light as f32,
    ) as i32;
    let sky = lerp(
        progress,
        state.start_sky_light as f32,
        state.end_sky_light as f32,
    ) as i32;
    // `LightCoordsUtil.pack`.
    let light = block << 4 | sky << 20;
    let modifier: f32 = if k % 2 == i32::from(backwards) {
        0.7
    } else {
        1.0
    };
    let colour =
        [0.5 * modifier, 0.4 * modifier, 0.3 * modifier].map(|channel| (channel * 255.0) as u8);
    let x = dx * progress;
    let y = if state.slack {
        if dy > 0.0 {
            dy * progress * progress
        } else {
            dy - dy * (1.0 - progress) * (1.0 - progress)
        }
    } else {
        dy * progress
    };
    let z = dz * progress;
    out.push(LeashVertex {
        position: pose
            .pose
            .transform_position(x - dx_off, y + fudge, z + dz_off),
        colour,
        light,
    });
    out.push(LeashVertex {
        position: pose
            .pose
            .transform_position(x + dx_off, y + 0.05 - fudge, z - dz_off),
        colour,
        light,
    });
}

/// `LeashFeatureRenderer.prepare` from the pose the entity is drawn at: the strip's vertices in
/// the order the reference submits them (out along one edge, then back along the other).
pub fn leash_vertices(pose: &Pose, state: &LeashState) -> Vec<LeashVertex> {
    let dx = (state.end[0] - state.start[0]) as f32;
    let dy = (state.end[1] - state.start[1]) as f32;
    let dz = (state.end[2] - state.start[2]) as f32;
    let factor = mth_inv_sqrt_f32(dx * dx + dz * dz) * WIDTH / 2.0;
    let offsets = [dz * factor, dx * factor];
    let mut pose = *pose;
    pose.translate(
        state.offset[0] as f32,
        state.offset[1] as f32,
        state.offset[2] as f32,
    );
    let mut out = Vec::new();
    for k in 0..=STEPS {
        vertex_pair(
            &mut out,
            &pose,
            [dx, dy, dz],
            WIDTH,
            offsets,
            k,
            false,
            state,
        );
    }
    for k in (0..=STEPS).rev() {
        vertex_pair(&mut out, &pose, [dx, dy, dz], 0.0, offsets, k, true, state);
    }
    out
}

/// A triangle strip as triangles, in the order the strip goes.
pub fn strip_triangles(vertices: &[LeashVertex]) -> Vec<LeashVertex> {
    let mut out = Vec::new();
    for i in 2..vertices.len() {
        out.extend([vertices[i - 2], vertices[i - 1], vertices[i]]);
    }
    out
}

/// What `Player.getRopeHoldPosition` reads of a player who holds a lead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRope {
    /// `getPosition(partial)`.
    pub position: [f64; 3],
    /// `getBoundingBox().getYsize()`.
    pub box_height: f64,
    pub x_rot: f32,
    pub old_x_rot: f32,
    pub body_rot: f32,
    pub old_body_rot: f32,
    pub right_arm: bool,
    pub crouching: bool,
    /// `isVisuallySwimming`.
    pub swimming: bool,
}

/// `Player.getRopeHoldPosition` for a player who is neither gliding nor spinning with a trident.
pub fn player_rope_hold_position(player: &PlayerRope, partial: f32) -> [f64; 3] {
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    let x_off = 0.22 * if player.right_arm { -1.0 } else { 1.0 };
    let pitch = mth_lerp_f32(partial * 0.5, player.x_rot, player.old_x_rot) * degrees;
    let yaw = mth_lerp_f32(partial, player.old_body_rot, player.body_rot) * degrees;
    let local = if player.swimming {
        y_rot(x_rot([x_off, 0.2, -0.15], -pitch), -yaw)
    } else {
        let y_off = player.box_height - 1.0;
        let z_off = if player.crouching { -0.2 } else { 0.07 };
        y_rot([x_off, y_off, z_off], -yaw)
    };
    [
        player.position[0] + local[0],
        player.position[1] + local[1],
        player.position[2] + local[2],
    ]
}

/// `LeashFenceKnotEntity.getRopeHoldPosition`.
pub fn knot_rope_hold_position(position: [f64; 3]) -> [f64; 3] {
    [position[0], position[1] + 0.2, position[2]]
}

/// `Entity.getRopeHoldPosition`: a little above the middle of the body.
pub fn entity_rope_hold_position(position: [f64; 3], eye_height: f32) -> [f64; 3] {
    [
        position[0],
        position[1] + f64::from(eye_height) * 0.7,
        position[2],
    ]
}
