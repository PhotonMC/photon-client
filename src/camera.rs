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
