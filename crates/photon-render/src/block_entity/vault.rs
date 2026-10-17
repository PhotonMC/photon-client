//! The item a vault shows (`VaultRenderer`): turning above its keyhole while it is active.

use crate::joml::{Pose, Quaternion};
use crate::objects::item::{ModelExtent, cluster_poses, rendered_amount, seed_for_stack};
use photon_core::ids::ItemId;
use photon_core::math::mth_rot_lerp_f32;

/// `VaultRenderer.extractRenderState`'s spin: between the last two ticks', in degrees.
pub fn spin(partial: f32, (previous, current): (i32, i32)) -> f32 {
    mth_rot_lerp_f32(partial, previous as f32, current as f32)
}

/// `VaultRenderer.submit`: the pose of each copy of the display item of `count`, with `base` the
/// pose of the block's corner and `spin` the degrees it is turned.
pub fn item_poses(
    base: &Pose,
    spin: f32,
    (item, count): (ItemId, i32),
    extent: ModelExtent,
) -> Vec<Pose> {
    let mut pose = *base;
    pose.translate(0.5, 0.4, 0.5);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        spin,
    )));
    cluster_poses(
        pose,
        rendered_amount(count),
        seed_for_stack(item, 0),
        extent,
    )
}
