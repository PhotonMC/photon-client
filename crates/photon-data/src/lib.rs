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

pub mod advancement;
pub mod animation;
pub mod attribute_state;
pub mod block;
pub mod block_entity_type;
pub mod block_sound;
pub mod compost;
pub mod damage_effect;
pub mod dimension_environment;
pub mod entity_model;
pub mod entity_shadow;
pub mod entity_type;
pub mod fire;
pub mod game_mode;
mod generated;
pub mod item;
pub mod item_components;
pub mod item_stack;
pub mod keys;
pub mod leash;
pub mod leaves;
pub mod map;
pub mod menu_type;
pub mod merchant;
mod name;
pub mod particle_options;
pub mod particle_type;
pub mod particle_values;
pub mod sound_event;
pub mod spear;
pub mod stat;
pub mod version_registries;
pub mod villager;
