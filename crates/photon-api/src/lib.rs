//! The native API of ADR 0006: what a feature or mod can see and change. Photon's own opt-in
//! features use it first; the WebAssembly world is generated from it later. The crate knows
//! nothing of the game crates, so it can be depended on from all of them. It names the parts of
//! the play loop's schedules ([`FrameSet`], [`TickSet`]) so that a feature can order its systems
//! against them.

mod event;
mod events;
mod owner;
mod registry;
mod sets;
mod update;

pub use event::Event;
pub use events::{ClientEvents, TickEnd, TickStart};
pub use owner::Owner;
pub use registry::{Registries, Registry};
pub use sets::{FrameSet, TickSet};
pub use update::{ServerNeedsNewerVersion, UpdateChanged, UpdateForServer, UpdateState};
