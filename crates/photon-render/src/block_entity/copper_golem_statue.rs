//! The copper golem statue (`CopperGolemStatueBlockRenderer`): one of four posed models in the
//! texture of how weathered the block is, turned to the way it faces.

use crate::joml::{Pose, Quaternion};
use photon_core::direction::Direction;

/// The model layer of a statue in a pose (`standing`, `running`, `sitting` or `star`).
pub fn layer(pose: &str) -> &'static str {
    match pose {
        "running" => "minecraft:copper_golem_running#main",
        "sitting" => "minecraft:copper_golem_sitting#main",
        "star" => "minecraft:copper_golem_star#main",
        _ => "minecraft:copper_golem#main",
    }
}

/// The texture under `textures/` of a statue as weathered as `weathering` says
/// (`CopperGolemOxidationLevels`): the block's own state, 0 for none to 3 for oxidized.
pub fn texture(weathering: u8) -> &'static str {
    match weathering {
        1 => "entity/copper_golem/copper_golem_exposed",
        2 => "entity/copper_golem/copper_golem_weathered",
        3 => "entity/copper_golem/copper_golem_oxidized",
        _ => "entity/copper_golem/copper_golem",
    }
}

/// `CopperGolemStatueBlockRenderer.modelTransformation`: the pose of a statue facing a way, with
/// `base` the pose of the block's corner.
pub fn placed(base: &Pose, facing: Direction) -> Pose {
    let mut pose = *base;
    pose.translate(0.5, 0.0, 0.5);
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -facing.opposite().to_y_rot(),
    )));
    pose
}
