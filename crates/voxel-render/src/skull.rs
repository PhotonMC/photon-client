//! The skulls a head wears (`CustomHeadLayer`'s `wornHeadType`, `SkullBlockRenderer.submitSkull`): which
//! item is which skull, its model and texture, and where on the head it sits.

use crate::joml::Pose;
use crate::player_model::Part;

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
            Self::Player => crate::player_renderer::skin_at(6).texture,
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
