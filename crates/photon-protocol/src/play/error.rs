use super::*;

/// Errors raised while driving the initial real-server client session.
#[derive(Debug)]
pub enum ConnectionError {
    Io(io::Error),
    Frame(FrameError),
    Wire(wire::ReadError),
    Compression(String),
    Codec(&'static str),
    UnsupportedPacket {
        phase: ConnectionProtocol,
        id: i32,
    },
    ServerDisconnected,
    /// The server ended the connection and said why (a login or configuration `Disconnect`).
    Disconnected(Box<photon_game::text::Component>),
    /// The session server did not accept the client's join (`authenticateServer`).
    LoginFailed(LoginFailure),
    InvalidArgument(&'static str),
    /// The server sent the client on to another (`ClientboundTransferPacket`).
    Transfer {
        host: String,
        port: i32,
    },
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) | Self::Frame(_) | Self::Wire(_) | Self::Compression(_) => {
                self.fmt_transport(formatter)
            }
            Self::Codec(name) => write!(formatter, "packet codec failed: {name}"),
            Self::UnsupportedPacket { phase, id } => {
                write!(formatter, "unsupported packet id {id} in {phase:?}")
            }
            Self::ServerDisconnected => formatter.write_str("server disconnected"),
            Self::Disconnected(_) => formatter.write_str("server disconnected"),
            Self::LoginFailed(failure) => write!(formatter, "Failed to log in: {failure}"),
            Self::InvalidArgument(name) => write!(formatter, "invalid argument: {name}"),
            Self::Transfer { host, port } => write!(formatter, "transferred to {host}:{port}"),
        }
    }
}

impl ConnectionError {
    /// The errors that wrap what the connection or the codec reported.
    fn fmt_transport(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Frame(error) => write!(formatter, "frame error: {error:?}"),
            Self::Wire(error) => write!(formatter, "wire error: {error:?}"),
            Self::Compression(error) => write!(formatter, "compression error: {error}"),
            _ => unreachable!("only the wrapping errors are formatted here"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameError> for ConnectionError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<wire::ReadError> for ConnectionError {
    fn from(error: wire::ReadError) -> Self {
        Self::Wire(error)
    }
}

/// Why the session server refused a join, as the client words it (`disconnect.loginFailedInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginFailure {
    ServersUnavailable,
    InvalidSession,
    InsufficientPrivileges,
    UserBanned,
    /// The server checks accounts (online mode), and no account was given.
    AccountRequired,
    /// Any other answer, with the message the session server gave.
    Other(String),
}

impl fmt::Display for LoginFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ServersUnavailable => {
                "The authentication servers are currently not reachable. Please try again."
            }
            Self::InvalidSession => "Invalid session (Try restarting your game and the launcher)",
            Self::InsufficientPrivileges => {
                "Multiplayer is disabled. Please check your Microsoft account settings."
            }
            Self::UserBanned => "You are banned from playing online",
            Self::AccountRequired => "the server needs a signed-in account (online mode)",
            Self::Other(message) => message,
        })
    }
}
