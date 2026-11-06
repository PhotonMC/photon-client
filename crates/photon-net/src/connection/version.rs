//! The version a connection speaks, chosen for each connection from the protocol number the
//! server answers with ([ADR 0013](../../../../docs/adr/0013-multi-version-content-model.md)).

use std::net::SocketAddr;
use std::time::Duration;

use photon_core::version::Version;
use photon_game::server_status::ServerStatus;

use super::status::query_status_text;

/// Development only: `PHOTON_CONNECT_VERSION` names a supported release (`26.1`) to speak
/// whatever the server says. Read at every connection.
pub fn development_version() -> Option<Version> {
    std::env::var("PHOTON_CONNECT_VERSION")
        .ok()
        .and_then(|name| Version::from_name(&name))
}

/// The version the player picked inside the client for the joins to come; none leaves it to the
/// server's protocol. It is only set between play sessions (title, server list).
static CHOSEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Pick the version of the next joins, or `None` to follow each server's protocol.
pub fn choose_version(version: Option<Version>) {
    let index = version.map_or(usize::MAX, Version::index);
    CHOSEN.store(index, std::sync::atomic::Ordering::Relaxed);
}

/// The version the player picked, if any.
pub fn chosen_version() -> Option<Version> {
    Version::ALL
        .get(CHOSEN.load(std::sync::atomic::Ordering::Relaxed))
        .copied()
}

/// How long the choice may wait for the server's status: a server that does not answer in time
/// is taken as the newest version, so it never holds a login back for long.
const STATUS_WAIT: Duration = Duration::from_secs(3);

/// The protocol each server last answered with, by address: what the server list's ping saw, or
/// an earlier connection, so that joining does not ask the server again before it logs in (the
/// reference client sends no status request before a login).
fn remembered() -> &'static std::sync::Mutex<std::collections::HashMap<SocketAddr, i32>> {
    static SEEN: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<SocketAddr, i32>>> =
        std::sync::OnceLock::new();
    SEEN.get_or_init(Default::default)
}

/// Note the protocol the server at `address` answered with.
pub fn remember_protocol(address: SocketAddr, protocol: i32) {
    if let Ok(mut seen) = remembered().lock() {
        seen.insert(address, protocol);
    }
}

/// The version to speak to the server at `address` (reached under `host` where that is not the
/// address's own): from the protocol the server list last saw (`hint`), else from a status
/// request. A protocol that is not a supported version, or a server that does not answer, gives
/// the newest version and leaves it to the server to refuse it, as the reference client does
/// with its own.
pub fn select_version(
    (address, host): (SocketAddr, Option<&str>),
    timeout: Duration,
    hint: Option<i32>,
) -> Version {
    if let Some(version) = development_version().or_else(chosen_version) {
        return version;
    }
    let seen = || remembered().lock().ok()?.get(&address).copied();
    hint.or_else(seen)
        .or_else(|| {
            let asked = answered_protocol(address, host, timeout.min(STATUS_WAIT))?;
            remember_protocol(address, asked);
            Some(asked)
        })
        .and_then(Version::from_protocol)
        .unwrap_or(Version::NEWEST)
}

/// The protocol number the server's status answer reports.
fn answered_protocol(address: SocketAddr, host: Option<&str>, timeout: Duration) -> Option<i32> {
    let text = query_status_text(address, host, timeout).ok()?;
    ServerStatus::parse(&text)?
        .version
        .map(|(_, protocol)| protocol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nowhere() -> SocketAddr {
        // A port nothing listens on: a hint must decide without asking it.
        SocketAddr::from(([127, 0, 0, 1], 9))
    }

    #[test]
    fn the_protocol_the_server_list_saw_picks_the_version() {
        if development_version().is_some() {
            return;
        }
        let wait = Duration::from_millis(50);
        assert_eq!(
            select_version((nowhere(), None), wait, Some(775)),
            Version::V26_1
        );
        assert_eq!(
            select_version((nowhere(), None), wait, Some(776)),
            Version::V26_2
        );
    }

    #[test]
    fn a_protocol_a_server_answered_with_before_is_not_asked_again() {
        if development_version().is_some() {
            return;
        }
        // A port nothing listens on, so that only the memory can answer.
        let address = SocketAddr::from(([127, 0, 0, 1], 10));
        remember_protocol(address, 775);
        assert_eq!(
            select_version((address, None), Duration::from_millis(50), None),
            Version::V26_1
        );
    }

    #[test]
    fn an_unknown_protocol_or_no_answer_leaves_the_newest_version() {
        if development_version().is_some() {
            return;
        }
        let wait = Duration::from_millis(50);
        assert_eq!(
            select_version((nowhere(), None), wait, Some(-1)),
            Version::NEWEST
        );
        assert_eq!(
            select_version((nowhere(), None), wait, Some(9999)),
            Version::NEWEST
        );
        assert_eq!(
            select_version((nowhere(), None), wait, None),
            Version::NEWEST
        );
    }
}
