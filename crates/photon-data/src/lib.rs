//! Tables taken from the reference.
//!
//! The reference builds these in code when it starts (block registry, shapes,
//! item defaults, creative tabs...). They are captured by the Java oracle,
//! written here by `tools/generate/*.py` (`tools/generate/all.py` runs them
//! all), and compared whole with the reference on every parity run.
//!
//! Everything under `generated/` is written by the generators and never edited by
//! hand; regeneration overwrites it. What is written by hand (`block/` and
//! `item/` outside `generated/`) holds the types the tables share and the
//! lookups on them.

pub mod animation;
pub mod block;
pub mod block_entity_type;
pub mod block_sound;
pub mod damage_effect;
pub mod entity_model;
pub mod entity_type;
pub mod fire;
mod generated;
pub mod item;
pub mod keys;
pub mod leash;
pub mod leaves;
pub mod map;
pub mod menu_type;
mod name;
pub mod particle_type;
pub mod sound_event;
pub mod spear;
pub mod villager;
