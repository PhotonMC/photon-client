//! `ElderGuardianParticle`: the elder guardian that flashes over the screen. It has no position
//! the picture uses and no motion, only an age; the `ELDER_GUARDIANS` group draws it.

/// `ElderGuardianParticle.lifetime`.
const LIFETIME: i32 = 30;

/// An elder guardian particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElderGuardian {
    age: i32,
    lifetime: i32,
}

impl ElderGuardian {
    pub(super) fn new() -> Self {
        Self {
            age: 0,
            lifetime: LIFETIME,
        }
    }

    pub fn age(&self) -> i32 {
        self.age
    }

    pub fn lifetime(&self) -> i32 {
        self.lifetime
    }

    /// `Particle.tick`: the age counts on, and a particle whose age had reached its lifetime is
    /// removed.
    pub(super) fn tick(&mut self) -> bool {
        let alive = self.age < self.lifetime;
        self.age += 1;
        alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_makes_one_for_its_type_and_it_goes_after_thirty_ticks() {
        use crate::{ClientWorld, DimensionBounds};
        use photon_core::math::Vec3;
        use photon_core::random::RandomSource;

        let world = ClientWorld::new(DimensionBounds::default());
        let mut engine =
            super::super::ParticleEngine::new(RandomSource::create(0), Default::default());
        let view = super::super::AddView {
            camera: Vec3::new(0.0, 0.0, 0.0),
            status: super::super::ParticleStatus::All,
        };
        let type_id = photon_data::particle_type::id("elder_guardian").unwrap();
        let spawn = super::super::Spawn {
            type_id,
            data: super::super::ParticleData::Plain,
            override_limiter: true,
            always_show: false,
            at: (0.0, 0.0, 0.0),
            push: (0.0, 0.0, 0.0),
        };
        let mut level_random = RandomSource::create(1);
        let mut random = || RandomSource::create(2);
        engine.add_particle(&world, &spawn, &view, &mut level_random, &mut random);
        // It joins the group at the end of the next tick.
        for _ in 0..2 {
            engine.tick(&world, &view, &mut level_random, &mut random);
        }
        assert_eq!(engine.elder_guardians().count(), 1);
        for _ in 0..30 {
            engine.tick(&world, &view, &mut level_random, &mut random);
        }
        assert_eq!(engine.elder_guardians().count(), 0);
    }

    #[test]
    fn it_ticks_thirty_times_and_goes_on_the_next() {
        let mut particle = ElderGuardian::new();
        for _ in 0..30 {
            assert!(particle.tick());
        }
        assert!(!particle.tick());
    }
}
