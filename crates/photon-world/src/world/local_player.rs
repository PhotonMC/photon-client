//! `ClientWorld`'s methods for the local player: movement, containers and state.

mod container;
pub(in crate::world) use container::consume_cue;
mod state;
mod travel;
