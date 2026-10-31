//! The friends service of the Minecraft services (`YggdrasilFriendsService`): the friend list
//! with its requests, the actions on it, the presence of the friends and the answers' mapping to
//! the result codes the screens react to.
//!
//! The service is a plain state machine over a `Transport`, so the answers of the real service
//! are replaced by written ones in tests. A caller serialises the calls, which the reference does
//! with a pending flag.

mod data;
mod service;
mod transport;
mod updates;

pub use data::{
    Friend, FriendData, PresenceEntry, PresenceStatus, ResultCode, parse_friends, parse_presence,
};
pub use service::FriendsService;
pub use transport::{HttpTransport, Method, Reply, Request, Transport, TransportError};
pub use updates::{Applied, FriendListUpdates, Notice, State};

#[cfg(test)]
mod tests;
