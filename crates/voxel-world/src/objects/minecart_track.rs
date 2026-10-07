//! A minecart under the new (experimental) behaviour, on the client (`NewMinecartBehavior`): the
//! server sends the steps it took along the track, and the client plays them back over three
//! ticks, weighted by how far each went, instead of easing to a place.

use std::cell::Cell;

use voxel_core::math::{mth_lerp_f64, mth_rot_lerp_f32};

/// The ticks a batch of steps is played over (`NewMinecartBehavior.POS_ROT_LERP_TICKS`).
const LERP_TICKS: i32 = 3;

/// `NewMinecartBehavior.MinecartStep`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartStep {
    pub position: (f64, f64, f64),
    pub movement: (f64, f64, f64),
    pub y_rot: f32,
    pub x_rot: f32,
    pub weight: f32,
}

// A step is made of finite numbers: never NaN.
impl Eq for MinecartStep {}

impl MinecartStep {
    /// `MinecartStep.ZERO`.
    pub const ZERO: Self = Self {
        position: (0.0, 0.0, 0.0),
        movement: (0.0, 0.0, 0.0),
        y_rot: 0.0,
        x_rot: 0.0,
        weight: 0.0,
    };
}

/// Where the playback puts the minecart (the part of a step that is lerped).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackPlace {
    pub position: (f64, f64, f64),
    pub movement: (f64, f64, f64),
    pub x_rot: f32,
    pub y_rot: f32,
}

/// `NewMinecartBehavior.StepPartialTicks`: the step a moment falls in, how far into it, and the one
/// before.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StepPartial {
    partial_in_step: f32,
    current: MinecartStep,
    previous: MinecartStep,
}

/// `getCurrentLerpStep`'s one-entry memory: the answer for a part tick at a delay (it is not
/// forgotten when the steps are replaced, so a delay that comes round again at the same part tick
/// gets the old answer).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Cached {
    partial_bits: u32,
    lerp_delay: i32,
    step: StepPartial,
}

/// `NewMinecartBehavior`'s client fields.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackLerp {
    lerp_delay: i32,
    lerp_steps: Vec<MinecartStep>,
    current_steps: Vec<MinecartStep>,
    current_total_weight: f64,
    old_lerp: Option<MinecartStep>,
    cache: Cell<Option<Cached>>,
}

// The cache holds finite numbers: never NaN.
impl Eq for Cached {}
impl Eq for StepPartial {}
impl Eq for TrackLerp {}

impl TrackLerp {
    /// `handleMinecartAlongTrack`: the steps are kept for the next tick that plays some.
    pub(crate) fn push_steps(&mut self, steps: &[MinecartStep]) {
        self.lerp_steps.extend_from_slice(steps);
    }

    /// `lerpClientPositionAndRotation`: the steps sent are taken up when the last batch has been
    /// played; `now` is where the minecart is (what the playback starts from). Returns where the
    /// playback puts it, if it is playing.
    pub(crate) fn tick(&mut self, now: MinecartStep) -> Option<TrackPlace> {
        self.lerp_delay -= 1;
        if self.lerp_delay <= 0 {
            self.old_lerp = Some(now);
            self.current_steps.clear();
            if !self.lerp_steps.is_empty() {
                self.current_steps.append(&mut self.lerp_steps);
                self.current_total_weight = 0.0;
                for step in &self.current_steps {
                    self.current_total_weight += f64::from(step.weight);
                }
                self.lerp_delay = if self.current_total_weight == 0.0 {
                    0
                } else {
                    LERP_TICKS
                };
            }
        }
        self.place(1.0)
    }

    /// `cartHasPosRotLerp`.
    pub fn is_playing(&self) -> bool {
        !self.current_steps.is_empty()
    }

    /// `getCartLerpPosition`, `getCartLerpMovements`, `getCartLerpXRot` and `getCartLerpYRot` at a
    /// part tick, if it is playing.
    pub fn place(&self, partial: f32) -> Option<TrackPlace> {
        if !self.is_playing() {
            return None;
        }
        let step = self.current_step(partial);
        let alpha = step.partial_in_step;
        let (from, to) = (step.previous, step.current);
        let lerp3 = |a: (f64, f64, f64), b: (f64, f64, f64)| {
            let alpha = f64::from(alpha);
            (
                mth_lerp_f64(alpha, a.0, b.0),
                mth_lerp_f64(alpha, a.1, b.1),
                mth_lerp_f64(alpha, a.2, b.2),
            )
        };
        Some(TrackPlace {
            position: lerp3(from.position, to.position),
            movement: lerp3(from.movement, to.movement),
            x_rot: mth_rot_lerp_f32(alpha, from.x_rot, to.x_rot),
            y_rot: mth_rot_lerp_f32(alpha, from.y_rot, to.y_rot),
        })
    }

    /// `getCurrentLerpStep`.
    fn current_step(&self, partial: f32) -> StepPartial {
        if let Some(cached) = self.cache.get()
            && cached.partial_bits == partial.to_bits()
            && cached.lerp_delay == self.lerp_delay
        {
            return cached.step;
        }
        let alpha = (LERP_TICKS - self.lerp_delay) as f32 + partial;
        let alpha = alpha / 3.0;
        let mut count_up = 0.0_f32;
        let mut indexed_partial = 1.0_f32;
        let mut found = None;
        for (index, step) in self.current_steps.iter().enumerate() {
            let weight = step.weight;
            // `!(weight <= 0.0F)`: a weight that is not a number counts.
            if weight > 0.0 || weight.is_nan() {
                count_up += weight;
                if f64::from(count_up) >= self.current_total_weight * f64::from(alpha) {
                    let current = count_up - weight;
                    indexed_partial = ((f64::from(alpha) * self.current_total_weight
                        - f64::from(current))
                        / f64::from(weight)) as f32;
                    found = Some(index);
                    break;
                }
            }
        }
        let index = found.unwrap_or(self.current_steps.len() - 1);
        let step = StepPartial {
            partial_in_step: indexed_partial,
            current: self.current_steps[index],
            previous: if index > 0 {
                self.current_steps[index - 1]
            } else {
                self.old_lerp.unwrap_or(MinecartStep::ZERO)
            },
        };
        self.cache.set(Some(Cached {
            partial_bits: partial.to_bits(),
            lerp_delay: self.lerp_delay,
            step,
        }));
        step
    }
}
