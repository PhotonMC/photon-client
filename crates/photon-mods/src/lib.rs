//! The mod host (ADR 0006): a mod is a WebAssembly component that gets only the imports its
//! manifest was granted, a fixed memory budget, and a deadline that a clock thread enforces.

mod host;
mod runtime;
mod tick;

pub use host::{Caps, Effect, Host, HostError, MAX_EFFECTS, MAX_TEXT, Mod};
pub use runtime::{GRANTED, HudLine, Runtime, grant};
pub use tick::TickMods;
