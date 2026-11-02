//! The pings of the server list: one thread for each saved server, which asks it for its status.

use std::time::Duration;

use photon_game::server_status::{Pinged, ServerStatus};
use photon_game::servers::{ServerList, address_with_port};

/// `ServerSelectionList.OnlineServerEntry`'s pings, each on a thread of its own: what each saved
/// server answers, by its address.
pub(super) fn ping_all(servers: &ServerList) -> std::sync::mpsc::Receiver<(String, Pinged)> {
    let (send, receive) = std::sync::mpsc::channel();
    for server in &servers.servers {
        let (ip, send) = (server.ip.clone(), send.clone());
        let _ = std::thread::Builder::new()
            .name("Server Pinger".to_owned())
            .spawn(move || {
                let pinged = ping(&ip);
                let _ = send.send((ip, pinged));
            });
    }
    receive
}

/// `ServerStatusPinger.pingServer`: an unknown name, a server that does not answer or answers in a
/// way that does not read is unreachable.
fn ping(ip: &str) -> Pinged {
    let Some(address) = resolve(ip) else {
        return Pinged::unreachable(true);
    };
    modern_ping(address)
        .or_else(|| legacy_ping(ip, address))
        .unwrap_or_else(|| Pinged::unreachable(false))
}

/// The status protocol's ping: its answer and the pong's time.
fn modern_ping(address: std::net::SocketAddr) -> Option<Pinged> {
    let trace =
        photon_net::connection::query_status_and_ping(address, Duration::from_secs(30)).ok()?;
    let status = ServerStatus::parse(&trace.status_response)?;
    if let Some((_, protocol)) = &status.version {
        crate::updates::note_server_protocol(*protocol);
        photon_net::connection::remember_protocol(address, *protocol);
    }
    Some(Pinged::answered(&status, trace.round_trip_ms))
}

/// `ServerStatusPinger.pingLegacyServer`, which the list tries when the modern ping failed.
fn legacy_ping(ip: &str, address: std::net::SocketAddr) -> Option<Pinged> {
    let with_port = address_with_port(ip);
    let (host, port) = with_port.rsplit_once(':')?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let answer = photon_net::connection::query_legacy_status(
        address,
        (host, port.parse().ok()?),
        Duration::from_secs(30),
    )?;
    Some(Pinged::legacy(
        &answer.version,
        &answer.motd,
        (answer.players, answer.max_players),
    ))
}

/// The address a server's `ip` names (`ServerAddress.parseString`, then
/// `ServerNameResolver.resolveAddress` with its SRV redirect).
pub(crate) fn resolve(ip: &str) -> Option<std::net::SocketAddr> {
    let with_port = address_with_port(ip);
    let (host, port) = with_port.rsplit_once(':')?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let resolved = photon_net::resolver::resolve_server(host, port.parse().ok()?);
    if resolved.is_none() {
        log::warn!("cannot resolve {ip}");
    }
    resolved
}
