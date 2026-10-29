//! The developer commands: `setup`, `server`, `rcon`, `play`, `bench` and `shot`.
//!
//! They fill `.data/` with the pinned game from Mojang's distribution, run
//! the test server and drive the client. Nothing here enters Git.

mod args;
mod baseline;
mod bench;
mod decompile;
mod download;
mod java;
mod layout;
mod play;
mod png;
mod rcon;
mod server;
mod setup;
mod shot;

pub use bench::bench;
pub use play::play;
pub use rcon::rcon;
pub use server::server;
pub use setup::setup;
pub use shot::shot;
