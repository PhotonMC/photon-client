//! The particles a living entity's effects give off (`LivingEntity.tickEffects` on a client).

use photon_core::binary::BlockPos;
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

/// `LivingEntity.handleEntityEvent(46)`: 128 portal particles strewn along the way from where the
/// entity was last tick to where it is, each in a box twice the entity's width, drifting a
/// little. (The reference draws each from the entity's own unseeded source.)
pub fn teleport_particles(
    random: &mut RandomSource,
    (from, to): (Vec3, Vec3),
    (width, height): (f32, f32),
) -> Vec<LevelParticles> {
    let Some(type_id) = photon_data::particle_type::id("portal") else {
        return Vec::new();
    };
    let lerp = |d: f64, a: f64, b: f64| a + d * (b - a);
    (0..128)
        .map(|i| {
            let d = f64::from(i) / 127.0;
            let mut drift = || (random.next_float() - 0.5) * 0.2;
            let spread = (drift(), drift(), drift());
            let x = lerp(d, from.x, to.x) + (random.next_double() - 0.5) * f64::from(width) * 2.0;
            let y = lerp(d, from.y, to.y) + random.next_double() * f64::from(height);
            let z = lerp(d, from.z, to.z) + (random.next_double() - 0.5) * f64::from(width) * 2.0;
            LevelParticles {
                type_id,
                data: super::ParticleData::Plain,
                override_limiter: false,
                always_show: false,
                at: (x, y, z),
                spread,
                max_speed: 1.0,
                count: 0,
            }
        })
        .collect()
}

/// `Entity.spawnSprintParticle`: a flake of the block the entity stands on, at a random place
/// within its width (kept over that block when it stands on an edge, in `on`'s column), at its
/// feet and a tenth up, thrown back against its motion and up.
pub fn sprint_particle(
    random: &mut RandomSource,
    (state, on, here): (photon_core::ids::BlockStateId, BlockPos, BlockPos),
    (position, width, motion): (Vec3, f32, Vec3),
) -> Option<LevelParticles> {
    let type_id = photon_data::particle_type::id("block")?;
    let width = f64::from(width);
    let mut x = position.x + (random.next_double() - 0.5) * width;
    let mut z = position.z + (random.next_double() - 0.5) * width;
    if here.x != on.x {
        x = x.clamp(f64::from(on.x), f64::from(on.x) + 1.0);
    }
    if here.z != on.z {
        z = z.clamp(f64::from(on.z), f64::from(on.z) + 1.0);
    }
    Some(LevelParticles {
        type_id,
        data: super::ParticleData::Block(state),
        override_limiter: false,
        always_show: false,
        at: (x, position.y + 0.1, z),
        // Count 0 reads the spread as the velocity.
        spread: ((motion.x * -4.0) as f32, 1.5, (motion.z * -4.0) as f32),
        max_speed: 1.0,
        count: 0,
    })
}
