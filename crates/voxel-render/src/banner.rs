//! The banner block entity (`BannerRenderer`): the pose its wall or rotation gives, the sprites
//! of its layers and the phase its flag waves by.

use voxel_core::direction::Direction;

use crate::joml::{Matrix4, Quaternion};
use crate::skull::segment_degrees;

/// `BannerRenderer.SIZE`.
const SIZE: f32 = 0.666_666_7;

/// The model layer of the pole and bar, and of the flag, of a standing or a wall banner.
pub const fn layers(wall: bool) -> (&'static str, &'static str) {
    if wall {
        ("minecraft:wall_banner#main", "minecraft:wall_banner#flag")
    } else {
        (
            "minecraft:standing_banner#main",
            "minecraft:standing_banner#flag",
        )
    }
}

/// `Sheets.BANNER_BASE`: the texture under `textures/` the pole, bar and flag wear.
pub const BASE_TEXTURE: &str = "entity/banner/banner_base";

/// `Sheets.BANNER_PATTERN_BASE`: the flag's plain layer, tinted by the banner's colour.
pub const PATTERN_BASE_TEXTURE: &str = "entity/banner/base";

/// `Sheets.getBannerSprite`: the texture of a pattern (a registry id, `minecraft:stripe_bottom`),
/// whose asset is its own id in the vanilla data pack.
pub fn pattern_texture(pattern: &str) -> String {
    format!(
        "entity/banner/{}",
        pattern.strip_prefix("minecraft:").unwrap_or(pattern)
    )
}

/// The most layers drawn (`MAX_PATTERNS`).
pub const MAX_PATTERNS: usize = 16;

/// `BannerRenderer.modelTransformation`.
fn model_transformation(angle: f32) -> Matrix4 {
    let mut matrix = Matrix4::translation(0.5, 0.0, 0.5);
    matrix.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -angle,
    )));
    matrix.scale(SIZE, -SIZE, -SIZE);
    matrix
}

/// `BannerRenderer.createWallTransformation`.
pub fn wall_transformation(facing: Direction) -> Matrix4 {
    model_transformation(facing.to_y_rot())
}

/// `BannerRenderer.createGroundTransformation`.
pub fn ground_transformation(segment: i32) -> Matrix4 {
    model_transformation(segment_degrees(segment))
}

/// `BannerRenderer.extractRenderState`'s phase of the flag's wave: the block's place and the game
/// time spread the banners out over a hundred ticks.
pub fn flag_phase((x, y, z): (i32, i32, i32), game_time: i64, partial: f32) -> f32 {
    let spread = i64::from(
        x.wrapping_mul(7)
            .wrapping_add(y.wrapping_mul(9))
            .wrapping_add(z.wrapping_mul(13)),
    )
    .wrapping_add(game_time);
    (spread.rem_euclid(100) as f32 + partial) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flag_waves_out_of_step_with_its_neighbours() {
        // 1 * 7 + 2 * 9 + 3 * 13 = 64.
        assert_eq!(flag_phase((1, 2, 3), 0, 0.0), 0.64);
        assert_eq!(flag_phase((1, 2, 3), 36, 0.0), 0.0);
        // A negative place wraps up from zero (`Math.floorMod`).
        assert_eq!(flag_phase((-1, 0, 0), 0, 0.0), 0.93);
    }

    #[test]
    fn a_pattern_wears_its_own_mask() {
        assert_eq!(
            pattern_texture("minecraft:stripe_bottom"),
            "entity/banner/stripe_bottom"
        );
    }
}
