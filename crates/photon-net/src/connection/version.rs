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

/// How long the choice may wait for the server's status: a server that does not answer in time
/// is taken as the newest version, so it never holds a login back for long.
const STATUS_WAIT: Duration = Duration::from_secs(3);

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
    if let Some(version) = development_version() {
        return version;
    }
    hint.or_else(|| answered_protocol(address, host, timeout.min(STATUS_WAIT)))
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
