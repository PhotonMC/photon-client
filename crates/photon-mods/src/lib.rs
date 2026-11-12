//! The mod host (ADR 0006): a mod is a WebAssembly component that gets only the imports its
//! manifest was granted, a fixed memory budget, and a deadline that a clock thread enforces.

mod data;
mod host;
mod runtime;
mod tick;

pub use data::{
    DataError, DataWriter, MAX_KEY, MAX_TOTAL, MAX_VALUE, MAX_WRITES_PER_TICK, ModData, data_path,
};
pub use host::{Caps, Compiled, Effect, Host, HostError, MAX_EFFECTS, MAX_TEXT, Mod};
pub use photon_api::{PlayerView, WorldView};
pub use runtime::{GRANTED, HudLine, Runtime, TICK_BUDGET, TickReport, grant};
pub use tick::TickMods;
