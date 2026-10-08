//! The client's particles: `Particle` and the classes built on it, which the client
//! level ticks each tick and draws as quads facing the camera.
//!
//! A particle starts from random draws of its own (its life, its size, its first motion): the
//! reference's `Particle` makes its random source when it is built, seeded from the clock. Here
//! the source is handed in.

mod colour;
mod engine;
mod flame;
mod group;
mod light;
mod particle;
mod resources;
mod rising;
mod smoke;
mod sprites;
mod terrain;
mod water;

pub use engine::{AddView, LevelParticles, ParticleData, ParticleEngine, ParticleStatus, Spawn};
pub use particle::{Particle, ParticleAtlas};
pub use resources::ParticleResources;
pub use sprites::SpriteSet;
pub use terrain::TerrainMaterials;
