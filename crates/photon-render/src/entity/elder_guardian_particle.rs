//! How an elder guardian particle is posed over the screen
//! (`ElderGuardianParticleGroup.ElderGuardianParticleRenderState.fromParticle`). The camera's own
//! rotation is [`super::name_tag::camera_orientation`].

use crate::joml::{Pose, Quaternion};

/// The texture the model is drawn in (`ElderGuardianRenderer.GUARDIAN_ELDER_LOCATION`).
pub const TEXTURE: &str = "entity/guardian/guardian_elder";
/// The model layer, baked as the particle's own model.
pub const LAYER: &str = "minecraft:elder_guardian#main";
/// The packed light the group submits the model with: full bright.
pub const LIGHT: i32 = 15_728_880;

/// What the group submits for a particle of this `age` and `lifetime`, `partial` of a tick on: the
/// pose, from the camera's rotation, and the colour the model is multiplied by (white, with an
/// opacity that swells and fades).
pub struct Placed {
    pub pose: Pose,
    /// `ARGB.colorFromFloat(alpha, 1, 1, 1)`.
    pub color: u32,
}

/// `base` is where the camera is in the space the frame draws in (the identity when that space
/// is the camera's own).
pub fn place(
    base: &Pose,
    camera: &Quaternion,
    (age, lifetime): (i32, i32),
    partial: f32,
) -> Placed {
    let age_scale = (age as f32 + partial) / lifetime as f32;
    let alpha =
        0.05 + 0.5 * photon_core::math::mth_sin_f64(f64::from(age_scale * std::f32::consts::PI));
    let mut pose = *base;
    pose.rotate(camera);
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        60.0 - 150.0 * age_scale,
    )));
    let scale = 0.425_531_92;
    pose.scale(scale, -scale, -scale);
    pose.translate(0.0, -0.56, 3.5);
    Placed {
        pose,
        color: color_from_float(alpha),
    }
}

/// `ARGB.colorFromFloat(alpha, 1.0F, 1.0F, 1.0F)`: each channel is `Mth.floor(value * 255.0F)`.
fn color_from_float(alpha: f32) -> u32 {
    let channel = |value: f32| ((value * 255.0).floor() as i32 as u32) & 0xFF;
    channel(alpha) << 24 | 0x00FF_FFFF
}
