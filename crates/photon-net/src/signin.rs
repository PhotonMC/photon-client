//! The Microsoft sign-in the launcher does for the game: a device code, then the Xbox Live,
//! XSTS and Minecraft services tokens that make an [`Account`](crate::account::Account).

mod device_code;
mod http;
mod login;
mod minecraft;

pub use device_code::{DeviceCode, MicrosoftTokens, poll_for_token, refresh, request_device_code};
pub use login::{SecretStore, finish_sign_in, resume, sign_out};
pub use minecraft::{MinecraftSession, minecraft_session};

use std::fmt;
use std::time::Duration;

/// Which Microsoft endpoints an application signs in through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// The identity platform (`login.microsoftonline.com`), for an application registered in Azure.
    Azure,
    /// The consumer service (`login.live.com`) that the official launcher's application uses.
    Live,
}

/// The application the sign-in is made for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientId {
    id: String,
    flow: Flow,
}

/// The application of the official launcher, which the project owner chose to sign in with until
/// Photon has a registration of its own. It signs in through the consumer service.
const LAUNCHER_CLIENT_ID: &str = "00000000402b5328";

impl ClientId {
    /// An application registered in Azure.
    pub fn new(id: String) -> Self {
        Self {
            id,
            flow: Flow::Azure,
        }
    }

    /// The application used when none is named.
    pub fn launcher() -> Self {
        Self {
            id: LAUNCHER_CLIENT_ID.to_owned(),
            flow: Flow::Live,
        }
    }

    /// The development variable names an Azure application; without it, the launcher's.
    pub fn from_environment() -> Self {
        std::env::var("PHOTON_MS_CLIENT_ID")
            .ok()
            .filter(|id| !id.trim().is_empty())
            .map_or_else(Self::launcher, |id| Self::new(id.trim().to_owned()))
    }

    pub(crate) fn flow(&self) -> Flow {
        self.flow
    }

    fn as_str(&self) -> &str {
        &self.id
    }
}

/// Where each service of the chain is reached. Only a test or a development run changes these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hosts {
    /// The Microsoft identity platform (device code and token endpoints).
    pub login: String,
    /// The consumer service of the launcher's application (device code and token endpoints).
    pub live: String,
    /// Xbox Live user authentication.
    pub xbox_user: String,
    /// Xbox Secure Token Service.
    pub xsts: String,
    /// The Minecraft services (login, entitlements, profile).
    pub services: String,
}

impl Default for Hosts {
    fn default() -> Self {
        Self {
            login: "https://login.microsoftonline.com".to_owned(),
            live: "https://login.live.com".to_owned(),
            xbox_user: "https://user.auth.xboxlive.com".to_owned(),
            xsts: "https://xsts.auth.xboxlive.com".to_owned(),
            services: "https://api.minecraftservices.com".to_owned(),
        }
    }
}

/// Why a sign-in step did not give what it was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignInError {
    /// The service could not be reached or answered with a server error.
    Unavailable,
    /// The player turned the request down on the Microsoft page.
    Declined,
    /// The code or the refresh token ran out before it was used.
    Expired,
    /// The service refused with a reason it worded itself.
    Rejected(String),
    /// Xbox Live refused the account, with the reason worded for the player.
    Xbox(String),
    /// The account does not own Minecraft: Java Edition.
    NotOwned,
    /// The account owns the game but has not made a profile (a player name) yet.
    NoProfile,
    /// The place the refresh token is kept failed; the words never carry the token.
    Store(String),
    /// The host is neither `https` nor this machine, so no secret goes to it.
    InsecureHost,
}

impl fmt::Display for SignInError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("the sign-in service is unavailable"),
            Self::Declined => formatter.write_str("the sign-in was declined"),
            Self::Expired => formatter.write_str("the sign-in code expired, start again"),
            Self::Rejected(reason) => write!(formatter, "the sign-in was refused: {reason}"),
            Self::Xbox(reason) => write!(formatter, "Xbox Live refused the account: {reason}"),
            Self::NotOwned => {
                formatter.write_str("this account does not own Minecraft: Java Edition")
            }
            Self::NoProfile => formatter.write_str("this account has no Minecraft profile yet"),
            Self::Store(reason) => write!(formatter, "the credential store failed: {reason}"),
            Self::InsecureHost => formatter.write_str("the sign-in host needs https"),
        }
    }
}

impl std::error::Error for SignInError {}

/// How long one request may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
