//! The environment attributes the particle engine reads: what the dimension's clock and the biomes
//! at a place say (`EnvironmentAttributeSystem.getValue`), asked of whoever owns the data packs.

use super::super::ClientWorld;

/// `AmbientParticle`: a particle a biome lets drift about, and the chance each pick makes one.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbientParticle {
    /// The type's name (`minecraft:crimson_spore`).
    pub particle: String,
    pub probability: f32,
}

/// What the environment attributes say, as the particle engine asks for them.
pub trait Ambience: std::fmt::Debug {
    /// `EnvironmentAttributes.CREAKING_ACTIVE` of the dimension: the creaking heart's idle sound.
    fn creaking_active(&self) -> bool {
        false
    }

    /// `EnvironmentAttributes.FIREFLY_BUSH_SOUNDS` of the dimension.
    fn firefly_bush_sounds(&self) -> bool {
        false
    }

    /// `EnvironmentAttributes.AMBIENT_PARTICLES` at a block: the particles the biome there lets
    /// drift.
    fn ambient_particles(
        &self,
        _world: &ClientWorld,
        _pos: (i32, i32, i32),
    ) -> Vec<AmbientParticle> {
        Vec::new()
    }
}

/// A level whose environment says nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAmbience;

impl Ambience for NoAmbience {}
