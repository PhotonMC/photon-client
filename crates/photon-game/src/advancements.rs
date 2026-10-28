//! The advancements the server tells of: the value types live in `photon-data`.

pub use photon_data::advancement::*;

mod client;
pub use client::{AdvancementProgress, ClientAdvancements, Event, Updated};
