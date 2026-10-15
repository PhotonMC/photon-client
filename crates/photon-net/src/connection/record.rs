//! What a connection tells whoever watches it. Production runs with [`NoRecorder`]; the live
//! probes of `photon-dev` write what they hear down.

use photon_protocol::phase::ConnectionProtocol;

/// A watcher of the login and configuration exchange.
pub trait Recorder {
    /// A packet was received, with the length of its body.
    fn packet(&mut self, _phase: ConnectionProtocol, _id: i32, _body_length: usize) {}

    fn login_finished(&mut self) {}

    /// A registry data packet with this many entries.
    fn registry_data(&mut self, _entries: usize) {}

    fn update_tags(&mut self) {}

    fn enabled_features(&mut self, _count: usize) {}

    /// The known packs the server offered, and how many of them the client selected.
    fn known_packs(&mut self, _offered: usize, _selected: usize) {}

    fn configuration_finished(&mut self) {}
}

/// Watches nothing.
pub struct NoRecorder;

impl Recorder for NoRecorder {}
