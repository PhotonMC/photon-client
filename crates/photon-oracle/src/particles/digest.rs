//! The digest of a particle group the particle rows compare.

use super::*;

/// The fold the oracle digests a group with: FNV-1a over 64-bit words.
fn fold(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
}

/// The digest of every particle in the group, and a description of the first `limit` of them.
pub(super) fn digest_group(
    engine: &photon_world::particles::ParticleEngine,
    (world, partials): (&photon_world::ClientWorld, &[f32]),
    limit: usize,
) -> (String, Vec<Value>) {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    let mut first = Vec::new();
    for particle in engine.particles() {
        let (position, delta) = (particle.position(), particle.delta());
        for value in [
            position.x, position.y, position.z, delta.x, delta.y, delta.z,
        ] {
            digest = fold(digest, value.to_bits());
        }
        digest = fold(digest, i64::from(particle.age()) as u64);
        digest = fold(digest, i64::from(particle.lifetime()) as u64);
        digest = fold(digest, particle.sprite() as u64);
        if first.len() < limit {
            first.push(describe(particle, world, partials));
        }
    }
    (format!("{digest:x}"), first)
}
