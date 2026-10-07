//! The skulls a head wears (`CustomHeadLayer`'s `wornHeadType`, `SkullBlockRenderer.submitSkull`): which
//! item is which skull, its model and texture, and where on the head it sits.

use voxel_core::direction::Direction;

use crate::entity::player_model::Part;
use crate::joml::{Matrix4, Pose, Quaternion};

/// `SkullBlock.Types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkullKind {
    Skeleton,
    WitherSkeleton,
    Player,
    Zombie,
    Creeper,
    Piglin,
    Dragon,
}

impl SkullKind {
    pub const ALL: [Self; 7] = [
        Self::Skeleton,
        Self::WitherSkeleton,
        Self::Player,
        Self::Zombie,
        Self::Creeper,
        Self::Piglin,
        Self::Dragon,
    ];

    /// The skull a head item shows (`AbstractSkullBlock.getType` of its block), by item name.
    pub fn of_item(name: &str) -> Option<Self> {
        Some(match name {
            "minecraft:skeleton_skull" => Self::Skeleton,
            "minecraft:wither_skeleton_skull" => Self::WitherSkeleton,
            "minecraft:player_head" => Self::Player,
            "minecraft:zombie_head" => Self::Zombie,
            "minecraft:creeper_head" => Self::Creeper,
            "minecraft:piglin_head" => Self::Piglin,
            "minecraft:dragon_head" => Self::Dragon,
            _ => return None,
        })
    }

    /// `SkullBlockRenderer.createModel`'s layer.
    pub fn layer(self) -> &'static str {
        match self {
            Self::Skeleton => "minecraft:skeleton_skull#main",
            Self::WitherSkeleton => "minecraft:wither_skeleton_skull#main",
            Self::Player => "minecraft:player_head#main",
            Self::Zombie => "minecraft:zombie_head#main",
            Self::Creeper => "minecraft:creeper_head#main",
            Self::Piglin => "minecraft:piglin_head#main",
            Self::Dragon => "minecraft:dragon_skull#main",
        }
    }

    /// `SkullBlockRenderer.SKIN_BY_TYPE`: the texture's path under `textures/`; a player head
    /// without a profile wears the default skin.
    pub fn texture(self) -> &'static str {
        match self {
            Self::Skeleton => "entity/skeleton/skeleton",
            Self::WitherSkeleton => "entity/skeleton/wither_skeleton",
            Self::Player => crate::entity::player_renderer::skin_at(6).texture,
            Self::Zombie => "entity/zombie/zombie",
            Self::Creeper => "entity/creeper/creeper",
            Self::Piglin => "entity/piglin/piglin",
            Self::Dragon => "entity/enderdragon/dragon",
        }
    }
}

/// `CustomHeadLayer.submit` for a skull: across by the renderer's horizontal scale, to the head,
/// then up by its skull offset and out to the skull's size (`1.1875`).
pub fn skull_pose(
    base: &Pose,
    root: &Part,
    head: &Part,
    (skull_y_offset, horizontal_scale): (f32, f32),
) -> Pose {
    let mut pose = *base;
    pose.scale(horizontal_scale, 1.0, horizontal_scale);
    root.translate_and_rotate(&mut pose);
    head.translate_and_rotate(&mut pose);
    pose.translate(0.0, skull_y_offset, 0.0);
    pose.scale(1.1875, 1.1875, 1.1875);
    pose
}

/// `SkullBlockRenderer.createWallTransformation`: against the wall it hangs on, turned to face
/// away from it.
pub fn wall_transformation(facing: Direction) -> Matrix4 {
    let (step_x, _, step_z) = facing.step();
    let mut matrix =
        Matrix4::translation(0.5 - step_x as f32 * 0.25, 0.25, 0.5 - step_z as f32 * 0.25);
    matrix.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -facing.opposite().to_y_rot(),
    )));
    matrix.scale(-1.0, -1.0, 1.0);
    matrix
}

/// `RotationSegment.convertToDegrees` for the sixteen segments.
pub(crate) fn segment_degrees(segment: i32) -> f32 {
    let degrees = (segment & 15) as f32 * 22.5;
    if degrees >= 180.0 {
        degrees - 360.0
    } else {
        degrees
    }
}

/// `SkullBlockRenderer.createGroundTransformation`: on the floor, turned by one of the sixteen
/// segments.
pub fn ground_transformation(segment: i32) -> Matrix4 {
    let mut matrix = Matrix4::translation(0.5, 0.0, 0.5);
    matrix.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -segment_degrees(segment),
    )));
    matrix.scale(-1.0, -1.0, 1.0);
    matrix
}

/// The pose a block entity is drawn from (a skull, a banner), with `base` the pose of the block's
/// corner and `transformation` the block's `Transformation`.
pub fn block_pose(base: &Pose, transformation: &Matrix4) -> Pose {
    let mut pose = *base;
    let mut transformation = *transformation;
    // `MatrixUtil.isPureTranslation` works the flags of a transformation's matrix out the first
    // time it is used.
    transformation.determine_properties();
    pose.mul_matrix(&transformation);
    pose
}
