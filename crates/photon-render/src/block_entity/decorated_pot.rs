//! The decorated pot (`DecoratedPotRenderer`): its two model layers, the picture each side wears,
//! and the pose it is drawn from as it turns to its facing and wobbles.

use photon_core::direction::Direction;
use photon_core::math::mth_sin_f64;

use crate::joml::{Matrix4, Pose, Quaternion};

/// The model layer of the neck and the top and bottom planes.
pub const BASE_LAYER: &str = "minecraft:decorated_pot_base#main";
/// The model layer of the four sides.
pub const SIDES_LAYER: &str = "minecraft:decorated_pot_sides#main";
/// The parts of the base layer, in the order they are drawn.
pub const BASE_PARTS: [&str; 3] = ["neck", "top", "bottom"];
/// The parts of the sides layer, in the order they are drawn.
pub const SIDE_PARTS: [&str; 4] = ["front", "back", "left", "right"];

/// The texture under `textures/` the base wears (`Sheets.DECORATED_POT_BASE`).
pub const BASE_TEXTURE: &str = "entity/decorated_pot/decorated_pot_base";
/// The texture of a side with no sherd (`Sheets.DECORATED_POT_SIDE`).
pub const SIDE_TEXTURE: &str = "entity/decorated_pot/decorated_pot_side";

/// The patterns of the pottery sherds, by the start of the item's name (`DecoratedPotPatterns`).
pub const PATTERNS: [&str; 23] = [
    "angler",
    "archer",
    "arms_up",
    "blade",
    "brewer",
    "burn",
    "danger",
    "explorer",
    "flow",
    "friend",
    "guster",
    "heart",
    "heartbreak",
    "howl",
    "miner",
    "mourner",
    "plenty",
    "prize",
    "scrape",
    "sheaf",
    "shelter",
    "skull",
    "snort",
];

/// The texture a side wears for the item set in it (`DecoratedPotRenderer.getSideSprite`): the
/// pattern of a pottery sherd, or the plain side for a brick or an item with no pattern.
pub fn side_texture(item: Option<&str>) -> String {
    let pattern = item
        .and_then(|name| name.strip_prefix("minecraft:"))
        .and_then(|name| name.strip_suffix("_pottery_sherd"))
        .filter(|name| PATTERNS.contains(name));
    match pattern {
        Some(name) => format!("entity/decorated_pot/{name}_pottery_pattern"),
        None => SIDE_TEXTURE.to_owned(),
    }
}

/// Every texture a pot may wear.
pub fn textures() -> impl Iterator<Item = String> {
    std::iter::once(BASE_TEXTURE.to_owned())
        .chain(std::iter::once(SIDE_TEXTURE.to_owned()))
        .chain(
            PATTERNS
                .iter()
                .map(|name| format!("entity/decorated_pot/{name}_pottery_pattern")),
        )
}

/// `DecoratedPotRenderer.modelTransformation`: a turn about the block's middle to the facing.
pub fn model_transformation(facing: Direction) -> Matrix4 {
    let degrees = 180.0 - facing.to_y_rot();
    let mut matrix = Matrix4::rotation_around(
        &Quaternion::rotation_y(Quaternion::degrees_to_radians(degrees)),
        (0.5, 0.5, 0.5),
    );
    matrix.determine_properties();
    matrix
}

/// The pose a pot is drawn from, with `base` the pose of the block's corner: turned to its facing,
/// and (while it wobbles, `progress` of the way through) turned about its vertical middle by a
/// swing that dies down. The reference's render state never records which style the wobble is,
/// so every wobble is drawn as this turn.
pub fn pot_pose(base: &Pose, facing: Direction, progress: f32) -> Pose {
    let mut pose = *base;
    pose.mul_matrix(&model_transformation(facing));
    if (0.0..=1.0).contains(&progress) {
        let turn = mth_sin_f64(f64::from(-progress * 3.0 * std::f32::consts::PI)) * 0.125;
        let decay = 1.0 - progress;
        pose.rotate_around(&Quaternion::rotation_y(turn * decay), (0.5, 0.0, 0.5));
    }
    pose
}
