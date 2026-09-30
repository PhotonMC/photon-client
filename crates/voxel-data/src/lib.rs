//! Tables taken from the reference.
//!
//! The reference builds these in code when it starts (block registry, shapes,
//! item defaults, creative tabs...). They are captured by the Java oracle,
//! written here by `tools/generate/*.py` (`tools/generate/all.py` runs them
//! all), and compared whole with the reference on every parity run.
//!
//! A generated file is never edited by hand: it says so in its header and
//! regeneration overwrites it. What is written by hand (the module layout,
//! `block/definition.rs`, `block/shape.rs` and `block/registry.rs`) holds the
//! types the tables share and the lookups on them.

pub mod block;
pub mod item;
