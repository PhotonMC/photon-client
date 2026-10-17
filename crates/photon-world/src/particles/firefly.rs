//! `FireflyParticle`: a speck of light that drifts about a firefly bush at night.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// How the light a firefly gives fades in and out of its life.
const LIGHT_FADE: (f32, f32) = (0.1, 0.3);
/// How the firefly itself fades.
const ALPHA_FADE: (f32, f32) = (0.3, 0.5);

/// `FireflyParticle.getFadeAmount`: in over the first `fade_out` of its life, out over the last
/// `fade_in`.
fn fade_amount(progress: f32, (fade_in, fade_out): (f32, f32)) -> f32 {
    if progress >= 1.0 - fade_in {
        (1.0 - progress) / fade_in
    } else if progress <= fade_out {
        progress / fade_out
    } else {
        1.0
    }
}

impl Particle {
    /// `FireflyParticle.FireflyProvider`: it starts invisible, a little larger than most, and
    /// lives ten to fifteen seconds.
    pub fn firefly(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        engine_random: &mut RandomSource,
    ) -> Self {
        let xa = 0.5 - engine_random.next_double();
        let ya = if engine_random.next_boolean() {
            push.1
        } else {
            -push.1
        };
        let za = 0.5 - engine_random.next_double();
        let sprite = sprites.from_random(engine_random);
        let mut particle =
            Self::thrown(random, at, (xa, ya, za), (sprites, sprite), Kind::Firefly).quad();
        particle.speed_up_when_y_motion_is_blocked = true;
        particle.friction = 0.96;
        particle.quad_size *= 0.75;
        particle.xd *= f64::from(0.8_f32);
        particle.yd *= f64::from(0.8_f32);
        particle.zd *= f64::from(0.8_f32);
        particle.lifetime = engine_random
            .next_int_between_inclusive(200, 300)
            .expect("the bounds are in order");
        particle.scale(1.5);
        particle.alpha = 0.0;
        particle
    }

    /// `FireflyParticle.getLifetimeProgress`.
    fn lifetime_progress(&self, age: f32) -> f32 {
        (age / self.lifetime as f32).clamp(0.0, 1.0)
    }

    /// `FireflyParticle.getLightCoords`: lit by how far through its life it is, not by the level.
    pub fn firefly_light(&self, partial: f32) -> i32 {
        let progress = self.lifetime_progress(self.age as f32 + partial);
        (255.0 * fade_amount(progress, LIGHT_FADE)) as i32
    }

    /// `FireflyParticle.tick`: it goes out in anything but air, fades in and out, and now and then
    /// (and at its first tick) takes a new heading.
    pub(super) fn firefly_tick(&mut self, world: &ClientWorld) {
        self.base_tick(world);
        let cell = [self.x, self.y, self.z].map(photon_core::math::floor_f64);
        let state = world.block_state_or_air(cell[0], cell[1], cell[2]);
        if !super::terrain::is_air(super::terrain::name_of(state)) {
            self.removed = true;
            return;
        }
        self.alpha = fade_amount(self.lifetime_progress(self.age as f32), ALPHA_FADE);
        if self.random.next_float() > 0.95 || self.age == 1 {
            let mut heading = || f64::from(-0.05 + 0.1 * self.random.next_float());
            let (xd, yd, zd) = (heading(), heading(), heading());
            (self.xd, self.yd, self.zd) = (xd, yd, zd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fade_amount;

    #[test]
    fn a_firefly_fades_in_over_its_start_and_out_over_its_end() {
        let alpha = (0.3, 0.5);
        assert_eq!(fade_amount(0.0, alpha), 0.0);
        assert_eq!(fade_amount(0.25, alpha), 0.5);
        assert_eq!(fade_amount(0.5, alpha), 1.0);
        assert_eq!(fade_amount(0.6, alpha), 1.0);
        assert_eq!(fade_amount(1.0, alpha), 0.0);
        // Over the last tenth of the glow's life it dims (`(1 - progress) / fade_in`).
        let glow = (0.1, 0.3);
        assert_eq!(fade_amount(0.95, glow), (1.0_f32 - 0.95) / 0.1);
        assert_eq!(fade_amount(0.3, glow), 1.0);
    }
}
