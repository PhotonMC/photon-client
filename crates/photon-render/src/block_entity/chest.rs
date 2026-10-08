//! The chest block entity (`ChestRenderer`): which model and texture a chest wears, how its lid
//! eases open, and the pose it is drawn from.

use photon_core::direction::Direction;

use crate::joml::{Matrix4, Pose, Quaternion};

/// `ChestType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestKind {
    Single,
    Left,
    Right,
}

impl ChestKind {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "single" => Self::Single,
            "left" => Self::Left,
            "right" => Self::Right,
            _ => return None,
        })
    }

    /// The model layer of this half (`ChestRenderer.LAYERS`).
    pub const fn layer(self) -> &'static str {
        match self {
            Self::Single => "minecraft:chest#main",
            Self::Left => "minecraft:double_chest_left#main",
            Self::Right => "minecraft:double_chest_right#main",
        }
    }
}

/// `ChestRenderState.ChestMaterialType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestMaterial {
    Ender,
    Christmas,
    Trapped,
    CopperUnaffected,
    CopperExposed,
    CopperWeathered,
    CopperOxidized,
    Regular,
}

impl ChestMaterial {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "ENDER_CHEST" => Self::Ender,
            "CHRISTMAS" => Self::Christmas,
            "TRAPPED" => Self::Trapped,
            "COPPER_UNAFFECTED" => Self::CopperUnaffected,
            "COPPER_EXPOSED" => Self::CopperExposed,
            "COPPER_WEATHERED" => Self::CopperWeathered,
            "COPPER_OXIDIZED" => Self::CopperOxidized,
            "REGULAR" => Self::Regular,
            _ => return None,
        })
    }

    /// The texture under `textures/` (`Sheets.chooseSprite`).
    pub fn texture(self, kind: ChestKind) -> String {
        let prefix = match self {
            Self::Ender => return "entity/chest/ender".to_owned(),
            Self::Christmas => "christmas",
            Self::Trapped => "trapped",
            Self::CopperUnaffected => "copper",
            Self::CopperExposed => "copper_exposed",
            Self::CopperWeathered => "copper_weathered",
            Self::CopperOxidized => "copper_oxidized",
            Self::Regular => "normal",
        };
        match kind {
            ChestKind::Single => format!("entity/chest/{prefix}"),
            ChestKind::Left => format!("entity/chest/{prefix}_left"),
            ChestKind::Right => format!("entity/chest/{prefix}_right"),
        }
    }
}

/// How far the lid is open as the model is given it: `open` eased (`1 - (1 - open)^3`).
pub fn eased_open(open: f32) -> f32 {
    let open = 1.0 - open;
    1.0 - open * open * open
}

/// `ChestRenderer.modelTransformation`: a turn about the block's vertical middle, as a matrix.
pub fn model_transformation(facing: Direction) -> Matrix4 {
    let degrees = -facing.to_y_rot();
    let mut matrix = Matrix4::rotation_around(
        &Quaternion::rotation_y(Quaternion::degrees_to_radians(degrees)),
        (0.5, 0.0, 0.5),
    );
    // `PoseStack.Pose.mulPose` asks `MatrixUtil.isPureTranslation`, which works the flags of the
    // transformation's matrix out the first time it is used; a facing that turns nothing is known
    // to be the identity from then on, and is not multiplied in.
    matrix.determine_properties();
    matrix
}

/// The pose a chest is drawn from, with `base` the pose of the block's corner.
pub fn chest_pose(base: &Pose, facing: Direction) -> Pose {
    let mut pose = *base;
    pose.mul_matrix(&model_transformation(facing));
    pose
}

#[cfg(test)]
mod tests {
    use super::eased_open;

    #[test]
    fn the_lid_eases_out_of_a_shut_and_into_an_open_chest() {
        assert_eq!(eased_open(0.0), 0.0);
        assert_eq!(eased_open(1.0), 1.0);
        assert!(eased_open(0.5) > 0.8);
    }
}
