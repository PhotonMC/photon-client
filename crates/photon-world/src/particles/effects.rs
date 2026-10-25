//! The particles a living entity's effects give off (`LivingEntity.tickEffects` on a client).

use photon_core::math::Vec3;
use photon_core::random::RandomSource;

use super::LevelParticles;
use crate::EffectParticle;

/// `LivingEntity.tickEffects` on a client: one tick in four (fifteen while invisible, five times
/// rarer for ambient effects), one of the entity data's effect particles at a random place in the
/// entity's box, flying as `addParticle(..., 1, 1, 1)` sends it.
pub fn effect_particle(
    particles: &[EffectParticle],
    (ambient, invisible): (bool, bool),
    (position, (width, height)): (Vec3, (f32, f32)),
    random: &mut RandomSource,
) -> Option<LevelParticles> {
    if particles.is_empty() {
        return None;
    }
    let bound = if invisible { 15 } else { 4 };
    let ambient_factor = if ambient { 5 } else { 1 };
    if random.next_int_bound(bound * ambient_factor).ok()? != 0 {
        return None;
    }
    // `Util.getRandom`, then `getRandomX(0.5)`, `getRandomY()` and `getRandomZ(0.5)` in that order.
    let index = random.next_int_bound(particles.len() as i32).ok()?;
    let particle = particles[index as usize];
    let x = position.x + f64::from(width) * ((2.0 * random.next_double() - 1.0) * 0.5);
    let y = position.y + f64::from(height) * random.next_double();
    let z = position.z + f64::from(width) * ((2.0 * random.next_double() - 1.0) * 0.5);
    // No count: one particle moving at the given speed, as a direct `addParticle` is.
    Some(LevelParticles {
        type_id: particle.type_id,
        data: particle.data(),
        override_limiter: false,
        always_show: false,
        at: (x, y, z),
        spread: (1.0, 1.0, 1.0),
        max_speed: 1.0,
        count: 0,
    })
}
