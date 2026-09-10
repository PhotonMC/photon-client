//! Connection phases and packet directions from the pinned Java protocol.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl ConnectionProtocol {
    pub const VALUES: [Self; 5] = [
        Self::Handshaking,
        Self::Play,
        Self::Status,
        Self::Login,
        Self::Configuration,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Handshaking => "handshake",
            Self::Play => "play",
            Self::Status => "status",
            Self::Login => "login",
            Self::Configuration => "configuration",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Handshaking => "HANDSHAKING",
            Self::Play => "PLAY",
            Self::Status => "STATUS",
            Self::Login => "LOGIN",
            Self::Configuration => "CONFIGURATION",
        }
    }

    pub const fn ordinal(self) -> usize {
        match self {
            Self::Handshaking => 0,
            Self::Play => 1,
            Self::Status => 2,
            Self::Login => 3,
            Self::Configuration => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFlow {
    Serverbound,
    Clientbound,
}

impl PacketFlow {
    pub const VALUES: [Self; 2] = [Self::Serverbound, Self::Clientbound];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Serverbound => "serverbound",
            Self::Clientbound => "clientbound",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Serverbound => "SERVERBOUND",
            Self::Clientbound => "CLIENTBOUND",
        }
    }

    pub const fn ordinal(self) -> usize {
        match self {
            Self::Serverbound => 0,
            Self::Clientbound => 1,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Serverbound => Self::Clientbound,
            Self::Clientbound => Self::Serverbound,
        }
    }
}
