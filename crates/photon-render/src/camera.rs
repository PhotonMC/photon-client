//! The first-person camera's own state: the eye height it eases towards and
//! the field of view it widens and narrows.
//!
//! Neither follows the player directly. Every tick the camera moves half of
//! the remaining way to the player's eye height and field-of-view factor, and
//! each frame blends the last two values by the partial tick, which is why a
//! crouch sinks the view over a few ticks rather than at once.

use photon_world::local_player::LocalPlayer;

#[derive(Debug, Clone, Default)]
pub struct Camera {
    eye_height: f32,
    eye_height_old: f32,
    fov_modifier: f32,
    old_fov_modifier: f32,
    spinning: Spinning,
    /// The nausea blend and the portal intensity the last tick ended with.
    nausea: f32,
    portal: f32,
    /// `GameRenderer.bossOverlayWorldDarkening`.
    pub boss: BossDarkening,
    /// `ScreenEffectRenderer`'s totem animation.
    pub activation: crate::item_activation::ItemActivation,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye_height: 0.0,
            eye_height_old: 0.0,
            fov_modifier: 1.0,
            old_fov_modifier: 1.0,
            spinning: Spinning::default(),
            nausea: 0.0,
            portal: 0.0,
            boss: BossDarkening::default(),
            activation: crate::item_activation::ItemActivation::default(),
        }
    }

    /// One client tick, after the player's.
    pub fn tick(&mut self, player: &LocalPlayer, nausea: f32) {
        // `GameRenderer.tick` runs before the player's, so it reads what the last tick ended with.
        let seen = std::mem::replace(&mut self.nausea, nausea);
        let portal = std::mem::replace(&mut self.portal, player.portal_effect_intensity_now());
        self.spinning.tick(portal, seen);
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

    /// The spinning effect's matrix for the projection, if nausea turns the view.
    pub fn spin(&self, (portal, nausea): (f32, f32), partial_tick: f32) -> Option<[[f32; 4]; 4]> {
        // `screenEffectScale`'s default is one: the option is not kept.
        self.spinning.matrix(portal, nausea, 1.0, partial_tick)
    }

    /// The eye height between the last two ticks.
    pub fn eye_height(&self, partial_tick: f32) -> f32 {
        self.eye_height_old + partial_tick * (self.eye_height - self.eye_height_old)
    }

    /// The vertical field of view in degrees, from the option's, narrowed in water and in lava.
    pub fn fov(&self, option_fov: i32, partial_tick: f32, in_fluid: bool) -> f32 {
        let modifier =
            self.old_fov_modifier + partial_tick * (self.fov_modifier - self.old_fov_modifier);
        let fov = option_fov as f32 * modifier;
        if in_fluid { fov * 0.857_142_87 } else { fov }
    }
}

/// What `GameRenderer.bobHurt` reads of the camera's entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HurtView {
    /// `hurtTime` less the partial tick: negative once the flinch is over.
    pub hurt_time: f32,
    pub hurt_duration: i32,
    /// The direction the damage came from (`getHurtDir`), in degrees.
    pub hurt_dir: f32,
    pub dying: bool,
    /// `deathTime` plus the partial tick.
    pub death_time: f32,
    /// The option `damageTiltStrength`.
    pub tilt_strength: f64,
}

impl HurtView {
    /// An entity that is not flinching or dying.
    pub const fn none() -> Self {
        Self {
            hurt_time: -1.0,
            hurt_duration: 0,
            hurt_dir: 0.0,
            dying: false,
            death_time: 0.0,
            tilt_strength: 1.0,
        }
    }
}

/// `GameRenderer.bobHurt` on a pose: the view rolls over in a faint death, and tilts away from a
/// blow while the flinch lasts.
pub fn apply_hurt(pose: &mut crate::joml::Pose, view: &HurtView) {
    use crate::joml::Quaternion;
    if view.dying {
        let duration = view.death_time.min(20.0);
        let roll = Quaternion::degrees_to_radians(40.0 - 8000.0 / (duration + 200.0));
        pose.rotate(&Quaternion::rotation_z(roll));
    }
    if view.hurt_time < 0.0 {
        return;
    }
    let hurt = view.hurt_time / view.hurt_duration as f32;
    let hurt =
        photon_core::math::mth_sin_f64(f64::from(hurt * hurt * hurt * hurt * std::f32::consts::PI));
    let direction = view.hurt_dir;
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        -direction,
    )));
    let tilt = (f64::from(-hurt) * 14.0 * view.tilt_strength) as f32;
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        tilt,
    )));
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        direction,
    )));
}

/// The matrix `PoseStack` builds of [`apply_hurt`], column by column.
pub fn view_hurt(view: &HurtView) -> [[f32; 4]; 4] {
    let mut pose = crate::joml::Pose::default();
    apply_hurt(&mut pose, view);
    pose.pose.m
}

/// `GameRenderer.bobView`: the sway of a walking player's view, a shift
/// and two small turns applied after the projection's own transform, from
/// the backwards walked distance and the bob.
pub fn view_bob(walked: f32, bob: f32) -> [[f32; 4]; 4] {
    use photon_core::math::{mth_cos_f64, mth_sin_f64};
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
    crate::scene::multiply(crate::scene::multiply(translate, around_z), around_x)
}

/// `GameRenderer.bossOverlayWorldDarkening`: how far the boss bars have darkened the world, now and
/// a tick ago.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BossDarkening {
    pub old: f32,
    pub now: f32,
}

impl BossDarkening {
    /// One client tick (`GameRenderer.tick`): up by a twentieth while a bar darkens the world,
    /// capped at one, and down by an eightieth otherwise, with no floor but the stop at zero.
    pub fn tick(&mut self, darkens: bool) {
        self.old = self.now;
        if darkens {
            self.now += 0.05;
            if self.now > 1.0 {
                self.now = 1.0;
            }
        } else if self.now > 0.0 {
            self.now -= 0.0125;
        }
    }

    /// `bossOverlayWorldDarkening(partial)`.
    pub fn at(&self, partial_tick: f32) -> f32 {
        photon_core::math::mth_lerp_f32(partial_tick, self.old, self.now)
    }
}

/// `GameRenderer`'s spinning effect: the time and speed of the turn a portal
/// or nausea puts on the view.
#[derive(Debug, Clone, Copy, Default)]
pub struct Spinning {
    time: f32,
    speed: f32,
}

impl Spinning {
    /// One client tick (`GameRenderer.tick`): the speed is the two intensities' weighted mean of
    /// twenty and seven degrees a tick, and the time runs on at it.
    pub fn tick(&mut self, portal: f32, nausea: f32) {
        if portal <= 0.0 && nausea <= 0.0 {
            self.speed = 0.0;
        } else {
            self.speed = (portal * 20.0 + nausea * 7.0) / (portal + nausea);
            self.time += self.speed;
        }
    }

    /// What `GameRenderer.renderLevel` multiplies onto the projection after the view bob: a turn
    /// about a slanted axis, a squeeze across it and the turn back. `None` while nothing spins.
    pub fn matrix(
        &self,
        portal: f32,
        nausea: f32,
        screen_effect_scale: f32,
        partial_tick: f32,
    ) -> Option<[[f32; 4]; 4]> {
        let intensity = portal.max(nausea) * (screen_effect_scale * screen_effect_scale);
        if intensity <= 0.0 {
            return None;
        }
        let mut skew = 5.0 / (intensity * intensity + 5.0) - intensity * 0.04;
        skew *= skew;
        let axis = (
            std::f32::consts::SQRT_2 / 2.0,
            std::f32::consts::SQRT_2 / 2.0,
        );
        let angle = (self.time + partial_tick * self.speed) * (std::f64::consts::PI / 180.0) as f32;
        let turn = |angle: f32| rotation(angle, [0.0, axis.0, axis.1]);
        let squeeze = [
            [1.0 / skew, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let multiply = crate::scene::multiply;
        Some(multiply(multiply(turn(angle), squeeze), turn(-angle)))
    }
}

/// `Matrix4f.rotation(angle, axis)` for a unit axis, column by column.
fn rotation(angle: f32, [x, y, z]: [f32; 3]) -> [[f32; 4]; 4] {
    let sin = crate::joml::sin(angle);
    let cos = crate::joml::cos_from_sin(sin, angle);
    let c = 1.0 - cos;
    [
        [
            x * x * c + cos,
            x * y * c + z * sin,
            x * z * c - y * sin,
            0.0,
        ],
        [
            x * y * c - z * sin,
            y * y * c + cos,
            y * z * c + x * sin,
            0.0,
        ],
        [
            x * z * c + y * sin,
            y * z * c - x * sin,
            z * z * c + cos,
            0.0,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bar that darkens the world ramps it up by a twentieth a tick to one; once none does it
    /// fades by an eightieth and stops at the first value that is not above zero, which may be a
    /// little below it.
    #[test]
    fn the_boss_darkening_ramps_up_and_fades_out() {
        let mut darkening = BossDarkening::default();
        for _ in 0..30 {
            darkening.tick(true);
        }
        assert_eq!(darkening.now, 1.0);
        assert_eq!(darkening.old, 1.0);
        darkening.tick(false);
        assert_eq!((darkening.old, darkening.now), (1.0, 0.9875));
        darkening.now = 0.01;
        darkening.tick(false);
        assert!(darkening.now < 0.0);
        let stopped = darkening.now;
        darkening.tick(false);
        assert_eq!(darkening.now, stopped);
        assert!((darkening.at(0.5) - stopped).abs() < 1e-6);
    }

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
