//! `ClientboundLevelParticlesPacket` on the client: how a count of particles is spread about the
//! packet's place.

use photon_core::random::RandomSource;
use photon_data::particle_values::Randomization;

use super::super::ClientWorld;
use super::engine::{AddView, LevelParticles, ParticleEngine, Spawn};

/// Where one particle of a packet goes and how fast, as `handleParticleEvent` draws them.
fn scattered(
    packet: &LevelParticles,
    random: &mut RandomSource,
) -> ((f64, f64, f64), (f64, f64, f64)) {
    let (dx, dy, dz) = (
        f64::from(packet.spread.0),
        f64::from(packet.spread.1),
        f64::from(packet.spread.2),
    );
    let (sx, sy, sz) = (
        f64::from(packet.max_speed.0),
        f64::from(packet.max_speed.1),
        f64::from(packet.max_speed.2),
    );
    let (x, y, z) = packet.at;
    if packet.randomization == Randomization::Default {
        let x_variance = random.next_gaussian() * dx;
        let y_variance = random.next_gaussian() * dy;
        let z_variance = random.next_gaussian() * dz;
        let xa = random.next_gaussian() * sx;
        let ya = random.next_gaussian() * sy;
        let za = random.next_gaussian() * sz;
        return (
            (x + x_variance, y + y_variance, z + z_variance),
            (xa, ya, za),
        );
    }
    let x_variance = random.next_double() * dx;
    let y_variance = random.next_double() * dy;
    let z_variance = random.next_double() * dz;
    let mut push = (sx, sy, sz);
    if packet.randomization == Randomization::AlternativeWithSpeed {
        push.0 *= random.next_double();
        push.1 *= random.next_double();
        push.2 *= random.next_double();
    }
    ((x + x_variance, y + y_variance, z + z_variance), push)
}

impl ParticleEngine {
    /// `ClientPacketListener.handleParticleEvent`: no count is one particle moving as told; a
    /// count is that many, each spread and sped by the listener's random source as the packet's
    /// randomization says.
    pub fn level_particles(
        &mut self,
        world: &ClientWorld,
        packet: &LevelParticles,
        view: &AddView,
        (listener_random, level_random): (&mut RandomSource, &mut RandomSource),
        mut particle_random: impl FnMut() -> RandomSource,
    ) {
        let spawn = |at: (f64, f64, f64), push: (f64, f64, f64)| Spawn {
            type_id: packet.type_id,
            data: packet.data,
            override_limiter: packet.override_limiter,
            always_show: packet.always_show,
            at,
            push,
        };
        if packet.count == 0 {
            let push = (
                f64::from(packet.max_speed.0 * packet.spread.0),
                f64::from(packet.max_speed.1 * packet.spread.1),
                f64::from(packet.max_speed.2 * packet.spread.2),
            );
            let one = spawn(packet.at, push);
            self.add_particle(world, &one, view, level_random, &mut particle_random);
            return;
        }
        for _ in 0..packet.count {
            let (at, push) = scattered(packet, listener_random);
            let one = spawn(at, push);
            self.add_particle(world, &one, view, level_random, &mut particle_random);
        }
    }
}
