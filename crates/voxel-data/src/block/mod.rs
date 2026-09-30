//! Per block and per block state, in registry order.

pub mod blocks;
pub mod classes;
pub mod collision;
pub mod definition;
pub mod offset;
pub mod outline;
pub mod properties;
pub mod registry;
pub mod shape;
pub mod tints;

/// A per-block list holds either one value shared by every state or one value
/// per state: the value for the state at `offset` in its block.
pub fn per_state<T: Copy>(values: &[T], offset: u32) -> Option<T> {
    if values.len() == 1 {
        Some(values[0])
    } else {
        values.get(offset as usize).copied()
    }
}
