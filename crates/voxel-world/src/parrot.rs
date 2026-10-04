//! What a parrot does of its own on the client (`Parrot.aiStep`): how far it
//! has beaten its wings, and the textures its variants wear.

use voxel_core::math::mth_lerp_f32;

/// The textures (under `textures/`) of the five variants, by id (`ParrotRenderer`):
/// red and blue, blue, green, yellow and blue, grey.
pub const VARIANT_TEXTURES: [&str; 5] = [
    "entity/parrot/parrot_red_blue",
    "entity/parrot/parrot_blue",
    "entity/parrot/parrot_green",
    "entity/parrot/parrot_yellow_blue",
    "entity/parrot/parrot_grey",
];

/// `Parrot.Variant.byId`: an id outside the variants is clamped to the nearest.
pub fn variant_texture(id: i32) -> &'static str {
    VARIANT_TEXTURES[id.clamp(0, 4) as usize]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParrotState {
    flap: f32,
    flap_old: f32,
    flap_speed: f32,
    flap_speed_old: f32,
    flapping: f32,
}

impl Default for ParrotState {
    fn default() -> Self {
        Self {
            flap: 0.0,
            flap_old: 0.0,
            flap_speed: 0.0,
            flap_speed_old: 0.0,
            flapping: 1.0,
        }
    }
}

/// The beats are finite, so the comparison is a total one.
impl Eq for ParrotState {}

impl ParrotState {
    /// `Parrot.calculateFlapping`, after `super.aiStep`.
    pub(crate) fn tick(&mut self, on_ground: bool, passenger: bool) {
        self.flap_old = self.flap;
        self.flap_speed_old = self.flap_speed;
        let step = if !on_ground && !passenger { 4.0 } else { -1.0 };
        self.flap_speed = (self.flap_speed + step * 0.3).clamp(0.0, 1.0);
        if !on_ground && self.flapping < 1.0 {
            self.flapping = 1.0;
        }
        self.flapping *= 0.9;
        self.flap += self.flapping * 2.0;
    }

    /// `(Mth.sin(flap) + 1) * flapSpeed` between two ticks
    /// (`ParrotRenderer.extractRenderState`).
    pub fn flap_angle(&self, partial_tick: f32) -> f32 {
        let flap = mth_lerp_f32(partial_tick, self.flap_old, self.flap);
        let speed = mth_lerp_f32(partial_tick, self.flap_speed_old, self.flap_speed);
        (voxel_core::math::mth_sin_f64(f64::from(flap)) + 1.0) * speed
    }
}
