//! The native API of ADR 0006: what a feature or mod can see and change. Photon's own opt-in
//! features use it first; the WebAssembly world is generated from it later. The crate knows
//! nothing of the game crates, so it can be depended on from all of them.

mod event;
mod owner;

pub use event::Event;
pub use owner::Owner;
