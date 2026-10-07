//! The client's particles: `Particle` and the classes built on it, which the client
//! level ticks each tick and draws as quads facing the camera.
//!
//! A particle starts from random draws of its own (its life, its size, its first motion): the
//! reference's `Particle` makes its random source when it is built, seeded from the clock. Here
//! the source is handed in.

mod flame;
mod light;
mod particle;
mod smoke;
mod sprites;

pub use particle::Particle;
pub use sprites::SpriteSet;
