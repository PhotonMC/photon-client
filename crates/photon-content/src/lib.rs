//! The game's own content (ADR 0007): after the account is shown to own the game, the pinned
//! version's client jar and assets are downloaded from Mojang's servers into a user-level,
//! content-addressed cache. Every file is checked against the digest the metadata promises
//! before it is kept; nothing of the game is embedded, packaged or uploaded. Development reads
//! `.data/` instead and never calls this crate.

mod cache;
mod endpoints;
mod error;
mod extract;
mod fetch;
mod hash;
mod install;
mod metadata;
mod pin;

pub use cache::{Cache, Id};
pub use endpoints::Endpoints;
pub use error::ContentError;
pub use hash::Sha1Hex;
pub use install::{Installed, Progress, Stage, install};
pub use pin::{PINNED, Pin};

#[cfg(test)]
mod tests;
