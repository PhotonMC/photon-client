//! What an axolotl does of its own on the client (`Axolotl.baseTick`): the
//! eased factors an adult's pose blends by, and the animation states a baby's
//! keyframe animations run on.

use photon_core::math::{mth_cos_f64, mth_lerp_f32};

use crate::animation_state::AnimationState;

/// `BinaryAnimator` eased by `EasingType.IN_OUT_SINE`: a factor that rises to
/// one over `length` ticks while active and falls back while not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryAnimator {
    length: i32,
    ticks: i32,
    ticks_old: i32,
}

impl BinaryAnimator {
    pub(crate) fn new(length: i32) -> Self {
        Self {
            length,
            ticks: 0,
            ticks_old: 0,
        }
    }

    pub(crate) fn tick(&mut self, active: bool) {
        self.ticks_old = self.ticks;
        if active {
            if self.ticks < self.length {
                self.ticks += 1;
            }
        } else if self.ticks > 0 {
            self.ticks -= 1;
        }
    }

    /// `getFactor`.
    pub fn factor(&self, partial_tick: f32) -> f32 {
        let factor = mth_lerp_f32(partial_tick, self.ticks_old as f32, self.ticks as f32)
            / self.length as f32;
        // `Ease.inOutSine`.
        let pi = std::f64::consts::PI as f32;
        -(mth_cos_f64(f64::from(pi * factor)) - 1.0) / 2.0
    }
}

/// The animators and animation states of one axolotl.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxolotlState {
    pub playing_dead: BinaryAnimator,
    pub in_water: BinaryAnimator,
    pub on_ground: BinaryAnimator,
    pub moving: BinaryAnimator,
    pub swim: AnimationState,
    pub walk: AnimationState,
    pub walk_under_water: AnimationState,
    pub idle_under_water: AnimationState,
    pub idle_under_water_on_ground: AnimationState,
    pub idle_on_ground: AnimationState,
    pub play_dead: AnimationState,
}

impl Default for AxolotlState {
    fn default() -> Self {
        Self::new()
    }
}

/// What an axolotl's tick reads of it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AxolotlInput {
    pub baby: bool,
    pub playing_dead: bool,
    pub in_water: bool,
    pub on_ground: bool,
    pub moving: bool,
    pub tick_count: i32,
}

impl AxolotlState {
    pub fn new() -> Self {
        let animator = BinaryAnimator::new(10);
        Self {
            playing_dead: animator,
            in_water: animator,
            on_ground: animator,
            moving: animator,
            swim: AnimationState::default(),
            walk: AnimationState::default(),
            walk_under_water: AnimationState::default(),
            idle_under_water: AnimationState::default(),
            idle_under_water_on_ground: AnimationState::default(),
            idle_on_ground: AnimationState::default(),
            play_dead: AnimationState::default(),
        }
    }

    /// `Axolotl.tickBabyAnimations` and `tickAdultAnimations`.
    pub(crate) fn tick(&mut self, input: AxolotlInput) {
        let AxolotlInput {
            baby,
            playing_dead,
            in_water,
            on_ground,
            moving,
            tick_count,
        } = input;
        if baby {
            self.moving.tick(moving);
            let chosen = if playing_dead {
                0
            } else if moving {
                if in_water && !on_ground {
                    1
                } else if !in_water && on_ground {
                    2
                } else {
                    3
                }
            } else if in_water && !on_ground {
                4
            } else if in_water && on_ground {
                5
            } else {
                6
            };
            let states = [
                &mut self.play_dead,
                &mut self.swim,
                &mut self.walk,
                &mut self.walk_under_water,
                &mut self.idle_under_water,
                &mut self.idle_under_water_on_ground,
                &mut self.idle_on_ground,
            ];
            for (index, state) in states.into_iter().enumerate() {
                if index == chosen {
                    state.start_if_stopped(tick_count);
                } else {
                    state.stop();
                }
            }
        } else {
            self.playing_dead.tick(playing_dead);
            self.in_water.tick(!playing_dead && in_water);
            self.on_ground.tick(!playing_dead && !in_water && on_ground);
            self.moving.tick(moving);
        }
    }
}
