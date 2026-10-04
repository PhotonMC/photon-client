//! Where the layers that draw a block on an entity put it: the enderman's
//! carried block, the snow golem's pumpkin and the iron golem's flower
//! (`CarriedBlockLayer`, `SnowGolemHeadLayer`, `IronGolemFlowerLayer`).
//! Each starts from the entity's pose and the part it is held by, and leaves
//! the pose the block's model is drawn from.

use crate::joml::{Pose, Quaternion};
use crate::player_model::Part;

fn degrees(pose: &mut Pose, axis: fn(f32) -> Quaternion, angle: f32) {
    pose.rotate(&axis(Quaternion::degrees_to_radians(angle)));
}

/// `CarriedBlockLayer`: in front of the body, tipped and turned, half size.
pub fn carried_block_pose(entity: &Pose) -> Pose {
    let mut pose = *entity;
    pose.translate(0.0, 0.6875, -0.75);
    degrees(&mut pose, Quaternion::rotation_x, 20.0);
    degrees(&mut pose, Quaternion::rotation_y, 45.0);
    pose.translate(0.25, 0.1875, 0.25);
    pose.scale(-0.5, -0.5, 0.5);
    degrees(&mut pose, Quaternion::rotation_y, 90.0);
    pose
}

/// `SnowGolemHeadLayer`: on top of the head, turned about and flipped, at
/// five eighths of a block.
pub fn snow_golem_head_pose(entity: &Pose, head: &Part) -> Pose {
    let mut pose = *entity;
    head.translate_and_rotate(&mut pose);
    pose.translate(0.0, -0.34375, 0.0);
    degrees(&mut pose, Quaternion::rotation_y, 180.0);
    pose.scale(0.625, -0.625, -0.625);
    pose.translate(-0.5, -0.5, -0.5);
    pose
}

/// `IronGolemFlowerLayer`: in the right hand, half size and stood up.
pub fn iron_golem_flower_pose(entity: &Pose, right_arm: &Part) -> Pose {
    let mut pose = *entity;
    right_arm.translate_and_rotate(&mut pose);
    pose.translate(-1.1875, 1.0625, -0.9375);
    pose.translate(0.5, 0.5, 0.5);
    pose.scale(0.5, 0.5, 0.5);
    degrees(&mut pose, Quaternion::rotation_x, -90.0);
    pose.translate(-0.5, -0.5, -0.5);
    pose
}

/// `SulfurCubeInnerLayer`'s block: turned upside down (the entity pose is
/// already flipped), halved on a small cube, and moved to the block's corner.
pub fn sulfur_cube_block_pose(entity: &Pose, baby: bool) -> Pose {
    let mut pose = *entity;
    degrees(&mut pose, Quaternion::rotation_x, 180.0);
    if baby {
        pose.scale(0.5, 0.5, 0.5);
    }
    pose.translate(-0.5, -0.518, -0.5);
    pose
}

/// `MushroomCowMushroomLayer`: three mushrooms on a grown mooshroom, two on
/// its back and one on its head, each turned its own way.
pub fn mooshroom_mushroom_poses(entity: &Pose, head: &Part) -> [Pose; 3] {
    let mut first = *entity;
    first.translate(0.2, -0.35, 0.5);
    degrees(&mut first, Quaternion::rotation_y, -48.0);
    first.scale(-1.0, -1.0, 1.0);
    first.translate(-0.5, -0.5, -0.5);
    let mut second = *entity;
    second.translate(0.2, -0.35, 0.5);
    degrees(&mut second, Quaternion::rotation_y, 42.0);
    second.translate(0.1, 0.0, -0.6);
    degrees(&mut second, Quaternion::rotation_y, -48.0);
    second.scale(-1.0, -1.0, 1.0);
    second.translate(-0.5, -0.5, -0.5);
    let mut third = *entity;
    head.translate_and_rotate(&mut third);
    third.translate(0.0, -0.7, -0.2);
    degrees(&mut third, Quaternion::rotation_y, -78.0);
    third.scale(-1.0, -1.0, 1.0);
    third.translate(-0.5, -0.5, -0.5);
    [first, second, third]
}

/// `BlockModelResolver.update`'s `blockLightCoords`: a block that renders
/// emissively is lit fully, any other by what it gives off (no sky light).
pub fn block_light_coords(emission: u8, emissive_rendering: bool) -> i32 {
    if emissive_rendering {
        crate::block_light::FULL_BRIGHT
    } else {
        crate::block_light::pack_light(i32::from(emission), 0)
    }
}

/// The light a block is submitted with (`BlockModelRenderState.submit`): the
/// larger, in each of block and sky light, of the entity's and the block's own.
pub fn submit_light(entity_light: i32, block_light: i32) -> i32 {
    crate::block_light::max_light(entity_light, block_light)
}
