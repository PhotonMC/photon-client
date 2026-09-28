//! The first-person camera's own state: the eye height it eases towards and
//! the field of view it widens and narrows.
//!
//! Neither follows the player directly. Every tick the camera moves half of
//! the remaining way to the player's eye height and field-of-view factor, and
//! each frame blends the last two values by the partial tick, which is why a
//! crouch sinks the view over a few ticks rather than at once.

use crate::world::local_player::LocalPlayer;

/// The option's default field of view, in degrees.
pub const DEFAULT_FOV: f32 = 70.0;

#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    eye_height: f32,
    eye_height_old: f32,
    fov_modifier: f32,
    old_fov_modifier: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye_height: 0.0,
            eye_height_old: 0.0,
            fov_modifier: 1.0,
            old_fov_modifier: 1.0,
        }
    }

    /// One client tick, after the player's.
    pub fn tick(&mut self, player: &LocalPlayer) {
        self.eye_height_old = self.eye_height;
        self.eye_height += (player.eye_height() - self.eye_height) * 0.5;

        // The field-of-view effect scale is left at its default, one, so the
        // target is the player's modifier itself.
        let abilities = player.abilities();
        let mut target = 1.0_f32;
        if abilities.flying() {
            target *= 1.1;
        }
        let walking_speed = abilities.walking_speed();
        if walking_speed != 0.0 {
            let speed_factor = player.movement_speed() / walking_speed;
            target *= (speed_factor + 1.0) / 2.0;
        }
        self.old_fov_modifier = self.fov_modifier;
        self.fov_modifier += (target - self.fov_modifier) * 0.5;
        self.fov_modifier = self.fov_modifier.clamp(0.1, 1.5);
    }

    /// The eye height between the last two ticks.
    pub fn eye_height(&self, partial_tick: f32) -> f32 {
        self.eye_height_old + partial_tick * (self.eye_height - self.eye_height_old)
    }

    /// The vertical field of view in degrees, narrowed under water.
    pub fn fov(&self, partial_tick: f32, eye_in_water: bool) -> f32 {
        let modifier =
            self.old_fov_modifier + partial_tick * (self.fov_modifier - self.old_fov_modifier);
        let fov = DEFAULT_FOV * modifier;
        if eye_in_water {
            fov * 0.857_142_87
        } else {
            fov
        }
    }
}

/// `GameRenderer.bobView`: the sway of a walking player's view, a shift
/// and two small turns applied after the projection's own transform, from
/// the backwards walked distance and the bob.
pub fn view_bob(walked: f32, bob: f32) -> [[f32; 4]; 4] {
    use crate::math::{mth_cos_f64, mth_sin_f64};
    let angle = walked * std::f32::consts::PI;
    let sin = mth_sin_f64(f64::from(angle));
    let cos = mth_cos_f64(f64::from(angle));
    let shift = [sin * bob * 0.5, -(cos * bob).abs()];
    let roll = (sin * bob * 3.0).to_radians();
    let pitch = ((mth_cos_f64(f64::from(angle - 0.2)) * bob).abs() * 5.0).to_radians();
    let translate = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [shift[0], shift[1], 0.0, 1.0],
    ];
    let around_z = [
        [roll.cos(), roll.sin(), 0.0, 0.0],
        [-roll.sin(), roll.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let around_x = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, pitch.cos(), pitch.sin(), 0.0],
        [0.0, -pitch.sin(), pitch.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    crate::render::scene::multiply(
        crate::render::scene::multiply(translate, around_z),
        around_x,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// JOML's matrices for the same inputs, built as `GameRenderer.bobView`
    /// builds them, printed column by column.
    #[test]
    fn view_bob_matches_the_reference_matrices() {
        let cases: [(f32, f32, [f32; 16]); 2] = [
            (
                -1.37,
                0.08,
                [
                    0.99999255,
                    0.003844246,
                    0.0,
                    0.0,
                    -0.0038442418,
                    0.9999915,
                    0.0014445999,
                    0.0,
                    5.553398E-6,
                    -0.0014445892,
                    0.99999887,
                    0.0,
                    0.03670994,
                    -0.031772956,
                    0.0,
                    1.0,
                ],
            ),
            (
                -12.61,
                0.1,
                [
                    0.99998784,
                    -0.004926502,
                    0.0,
                    0.0,
                    0.0049264515,
                    0.9999776,
                    0.00452793,
                    0.0,
                    -2.2306856E-5,
                    -0.004527875,
                    0.99998975,
                    0.0,
                    -0.047044817,
                    -0.033869464,
                    0.0,
                    1.0,
                ],
            ),
        ];
        for (walked, bob, expected) in cases {
            let matrix = view_bob(walked, bob);
            for (index, value) in expected.iter().enumerate() {
                let actual = matrix[index / 4][index % 4];
                assert!(
                    (actual - value).abs() < 1.0e-6,
                    "{walked} {bob} [{index}]: {actual} vs {value}"
                );
            }
        }
    }
}
