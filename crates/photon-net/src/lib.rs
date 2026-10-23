//! The live connection: phases driven over sockets, the play session and the local probes.
//! The wire format is in `photon-protocol`; its modules are re-exported here for the existing
//! callers.

pub mod account;
pub mod connection;
pub mod resolver;
pub mod server_packs;

pub use photon_protocol::phase as protocol;
pub use photon_protocol::{
    common, configuration, cookie, frame, handshake, login, registry_data, resources, status,
};
