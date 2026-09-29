//! Tables taken from the reference.
//!
//! The reference builds these in code when it starts (block registry, shapes,
//! item defaults, creative tabs...). They are captured by the Java oracle,
//! written here by `tools/generate/*.py`, and compared whole with the reference
//! on every parity run. Nothing in this crate is written by hand but the
//! module layout and the few lookups on the block registry.

pub mod block;
pub mod item;
pub mod model;
