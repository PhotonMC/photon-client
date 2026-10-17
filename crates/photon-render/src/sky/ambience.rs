//! The environment attributes the particle engine reads, from the dimension's sky data.

use photon_world::ClientWorld;
use photon_world::particles::{Ambience, AmbientParticle};

use super::{Attribute, Environment};

/// What the dimension's data says at this tick: the flags the clock sets, and the biomes'
/// ambient particles.
#[derive(Debug, Clone, Copy)]
pub struct SkyAmbience<'a> {
    environment: &'a Environment,
    creaking_active: bool,
    firefly_bush_sounds: bool,
}

impl<'a> SkyAmbience<'a> {
    /// The attributes at the clocks' current ticks.
    pub fn new(environment: &'a Environment, clock_ticks: impl Fn(&str) -> i64) -> Self {
        let flag = |attribute| environment.dimension_value(attribute, &clock_ticks).bool();
        Self {
            environment,
            creaking_active: flag(Attribute::CreakingActive),
            firefly_bush_sounds: flag(Attribute::FireflyBushSounds),
        }
    }
}

impl Ambience for SkyAmbience<'_> {
    fn creaking_active(&self) -> bool {
        self.creaking_active
    }

    fn firefly_bush_sounds(&self) -> bool {
        self.firefly_bush_sounds
    }

    fn ambient_particles(
        &self,
        world: &ClientWorld,
        (x, y, z): (i32, i32, i32),
    ) -> Vec<AmbientParticle> {
        self.environment
            .ambient_particles(world.biome_at(x, y, z))
            .to_vec()
    }
}
