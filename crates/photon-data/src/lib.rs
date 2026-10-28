//! Tables taken from the reference.
//!
//! The reference builds these in code when it starts (block registry, shapes,
//! item defaults, creative tabs...). They were captured by the Java oracle once and are
//! maintained by hand in `tables/`; the parity families compare them whole with the
//! reference on every run. What sits beside them (`block/`, `item/`) holds the types the tables
//! share and the lookups on them.

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
mod tables;
pub mod version_registries;
pub mod villager;
