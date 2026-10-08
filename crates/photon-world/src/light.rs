//! The client level's light engine: `LevelLightEngine` with its block and
//! sky engines, their section storage and each chunk's sky light sources.
//!
//! The server lights the world and sends the light with each chunk, but it
//! only sends light changes to players whose view ends at that chunk. Inside
//! the view the client relights on its own: every block update that changes
//! how light passes checks that block, and the engine spreads the change.
//!
//! Light is kept per section as nibble layers. The engine writes an
//! "updating" copy and publishes a "visible" copy after each run, which is
//! what the getters read. A layer may be shared: a section's queued data from
//! the server becomes its stored layer as the very same object, and a write
//! during the run shows in both. Layers therefore live in an arena and the
//! maps hold their ids, so sharing and "is it the same layer" behave as in
//! the reference.

use std::collections::{HashMap, HashSet, VecDeque};

use photon_core::direction::Direction;

mod entry {
    use photon_core::direction::Direction;

    pub const DIRECTIONS_MASK: u64 = 1008;
    const FROM_EMPTY_SHAPE: u64 = 1024;
    const INCREASE_FROM_EMISSION: u64 = 2048;

    fn bit(direction: Direction) -> u64 {
        1 << (direction as u64 + 4)
    }

    fn with_level(entry: u64, level: u8) -> u64 {
        (entry & !15) | u64::from(level & 15)
    }

    pub fn decrease_skip_one_direction(level: u8, skip: Direction) -> u64 {
        with_level(DIRECTIONS_MASK & !bit(skip), level)
    }

    pub fn decrease_all_directions(level: u8) -> u64 {
        with_level(DIRECTIONS_MASK, level)
    }

    pub fn increase_light_from_emission(level: u8, from_empty_shape: bool) -> u64 {
        let mut entry = DIRECTIONS_MASK | INCREASE_FROM_EMISSION;
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry, level)
    }

    pub fn increase_skip_one_direction(level: u8, from_empty_shape: bool, skip: Direction) -> u64 {
        let mut entry = DIRECTIONS_MASK & !bit(skip);
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry, level)
    }

    pub fn increase_only_one_direction(
        level: u8,
        from_empty_shape: bool,
        direction: Direction,
    ) -> u64 {
        let mut entry = 0;
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry | bit(direction), level)
    }

    pub fn increase_sky_source_in_directions(
        down: bool,
        north: bool,
        south: bool,
        west: bool,
        east: bool,
    ) -> u64 {
        let mut entry = with_level(0, 15);
        for (on, direction) in [
            (down, Direction::Down),
            (north, Direction::North),
            (south, Direction::South),
            (west, Direction::West),
            (east, Direction::East),
        ] {
            if on {
                entry |= bit(direction);
            }
        }
        entry
    }

    pub fn from_level(entry: u64) -> u8 {
        (entry & 15) as u8
    }

    pub fn is_from_empty_shape(entry: u64) -> bool {
        entry & FROM_EMPTY_SHAPE != 0
    }

    pub fn is_increase_from_emission(entry: u64) -> bool {
        entry & INCREASE_FROM_EMISSION != 0
    }

    pub fn propagates(entry: u64, direction: Direction) -> bool {
        entry & bit(direction) != 0
    }
}

mod engine;
use engine::*;
mod layer;
pub use layer::*;
mod level_engine;
pub use level_engine::*;
mod shapes;
pub use shapes::*;
mod sky_sources;
pub use sky_sources::*;
mod storage;
pub use storage::*;
