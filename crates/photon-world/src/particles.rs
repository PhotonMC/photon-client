//! The client's particles: `Particle` and the classes built on it, which the client
//! level ticks each tick and draws as quads facing the camera.
//!
//! A particle starts from random draws of its own (its life, its size, its first motion): the
//! reference's `Particle` makes its random source when it is built, seeded from the clock. Here
//! the source is handed in.

mod animate;
mod animate_blocks;
mod animate_falling;
mod animate_misc;
mod animate_redstone;
mod attract;
mod breaking_item;
mod colour;
mod drip;
mod elder_guardian;
mod engine;
mod explosion;
mod falling_dust;
mod firework;
mod flame;
mod group;
mod hit;
mod leaves;
mod level_events;
mod level_events_dripstone;
mod level_events_last;
mod level_events_more;
mod level_events_trial;
mod level_events_vault;
mod light;
mod look;
mod particle;
mod pickup;
mod resources;
mod rising;
mod smoke;
mod sprites;
mod terrain;
mod vibration;
mod water;

pub use elder_guardian::ElderGuardian;
pub use engine::{
    AddView, Emitted, LevelParticles, ParticleData, ParticleEngine, ParticleStatus, PositionSource,
    Spawn,
};
pub use explosion::{Explosion, ExplosionParticle};
pub use firework::{
    Explosion as FireworkExplosion, FireworkLaunch, Shape as FireworkShape, SparkOptions,
};
pub use level_events::LevelEventContext;
pub use level_events_dripstone::DripstoneAttributes;
pub use level_events_vault::PlayerSpot;
pub use particle::{Particle, ParticleAtlas};
pub use pickup::{ItemPickup, PickedUp, pickup_target};
pub use resources::ParticleResources;
pub use sprites::SpriteSet;
pub use terrain::TerrainMaterials;
