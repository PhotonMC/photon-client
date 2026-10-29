//! The Java protocol's wire format: connection phases, framing and the packet codecs. No
//! sockets and no client state live here; `photon-net` drives connections with these.

pub mod common;
pub mod configuration;
pub mod cookie;
pub mod crypto;
pub mod frame;
pub mod handshake;
pub mod login;
pub mod phase;
pub mod play;
pub mod registry_data;
pub mod resources;
pub mod server_links;
pub mod status;
pub mod versions;
