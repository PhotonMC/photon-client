//! The mod host (ADR 0006): a mod is a WebAssembly component that gets only the imports its
//! manifest was granted, a fixed memory budget, and a deadline that a clock thread enforces.

mod host;

pub use host::{Host, HostError, Mod};
