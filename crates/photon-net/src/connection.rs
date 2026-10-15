//! Java-protocol connections: the transport, the login and configuration phases, and the play
//! session. Wire codecs are in `photon-protocol`; the live probes are in `photon-dev`.

pub mod entities;
mod open;
mod play;
mod record;
mod session;
mod state;
mod status;

pub use open::{configure_connection, open_play_connection};
pub use photon_protocol::play::*;
pub use play::PlaySession;
pub use record::{NoRecorder, Recorder};
pub use session::{
    connect_stream, read_packet, send_configuration_start, send_handshake_intention,
    send_handshake_to, send_packet,
};
pub use state::ConnectionState;
pub use status::{ServerStatus, query_status_and_ping, query_status_and_ping_recorded};
