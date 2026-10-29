//! Java-protocol connections: the transport, the login and configuration phases, and the play
//! session. Wire codecs are in `photon-protocol`; the live probes are in `photon-dev`.

mod encrypt;
pub mod entities;
mod join;
mod open;
mod pack_sink;
mod play;
mod record;
mod session;
mod state;
mod status;
mod transport;
mod version;
mod writer;

pub use open::{configure_connection, open_play_connection};
pub use photon_protocol::play::*;
pub use play::{
    CommandBlockSettings, CommandBlockState, GlobalBlock, JigsawBlockSettings, LoadingReason,
    PlaySession, StructureBlockSettings, StructureFlags, SwitchKind,
};
pub use record::{NoRecorder, Recorder};
pub use session::{
    connect_stream, frame_packet, read_packet, send_configuration_start, send_handshake_intention,
    send_handshake_to, send_packet,
};
pub use state::{ConnectionState, LoginStage, stage};
pub use status::{ServerStatus, query_status_and_ping, query_status_and_ping_recorded};
pub use transport::Transport;
pub use version::{development_version, select_version};
