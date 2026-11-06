//! Java-protocol connections: the transport, the login and configuration phases, and the play
//! session. Wire codecs are in `photon-protocol`; the live probes are in `photon-dev`.

mod encrypt;
pub mod entities;
mod join;
mod legacy_status;
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

pub use join::prove_identity;
pub use legacy_status::{LegacyStatus, query_legacy_status};
pub use open::{configure_connection, open_play_connection};
pub use photon_protocol::play::*;
pub use play::{
    ChartRequests, CommandBlockSettings, CommandBlockState, DebugQuery, DebugQueryAnswer,
    GameTestHighlight, GlobalBlock, JigsawBlockSettings, LoadingReason, PlaySession, ProfileSkin,
    SampleLog, StructureBlockSettings, StructureFlags, SwitchKind,
};
pub use record::{NoRecorder, Recorder};
pub use session::{
    connect_stream, frame_packet, read_packet, send_configuration_start, send_handshake_intention,
    send_handshake_to, send_packet,
};
pub use state::{ConnectionState, LoginStage, stage};
pub use status::{
    ServerStatus, query_status_and_ping, query_status_and_ping_for, query_status_and_ping_recorded,
    query_status_text,
};
pub use transport::Transport;
pub use version::{development_version, remember_protocol, select_version};

/// The wire ids a test server sends: the constants number 26.2's packets, the newest version
/// numbers them its own way.
#[cfg(test)]
mod test_wire {
    use photon_core::version::Version;
    use photon_protocol::versions::packet_ids;

    pub(super) fn play(id: i32) -> i32 {
        packet_ids::clientbound_play_wire_id(Version::NEWEST, id)
    }

    pub(super) fn config(id: i32) -> i32 {
        packet_ids::clientbound_config_wire_id(Version::NEWEST, id)
    }
}
