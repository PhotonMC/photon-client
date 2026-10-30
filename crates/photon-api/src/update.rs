//! What the client's updater says, in plain values: a feature or mod reads it without knowing how
//! releases are found, verified or stored.

/// Where the updater stands. Every change is announced as an [`UpdateChanged`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UpdateState {
    /// Nothing is known or under way.
    #[default]
    Idle,
    /// The release host is being asked.
    Checking,
    /// A newer release exists and has not been downloaded.
    Available {
        version: String,
        notes: String,
        size: u64,
    },
    /// The archive is being downloaded.
    Downloading { done: u64, total: u64 },
    /// A newer release is verified and stored; the next start applies it.
    Ready { version: String },
    /// The last check or download did not succeed; the running install is unchanged.
    Failed { reason: String },
}

/// The updater's state changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateChanged(pub UpdateState);

/// Whether a newer release can join a server the running one cannot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateForServer {
    /// A release that supports the server's protocol is stored and applies at the next start.
    Staged { version: String },
    /// A release that supports it exists but is not downloaded.
    Available { version: String },
    /// No known release names the server's protocol.
    None,
}

/// The server speaks a newer protocol than this client supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerNeedsNewerVersion {
    pub server_protocol: u32,
    pub update: UpdateForServer,
}
