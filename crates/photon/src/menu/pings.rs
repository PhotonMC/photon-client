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
    photon_net::connection::query_status_and_ping(address, Duration::from_secs(30))
        .ok()
        .and_then(|trace| {
            let status = ServerStatus::parse(&trace.status_response)?;
            if let Some((_, protocol)) = &status.version {
                crate::updates::note_server_protocol(*protocol);
            }
            Some(Pinged::answered(&status, trace.round_trip_ms))
        })
        .unwrap_or_else(|| Pinged::unreachable(false))
}

/// The address a server's `ip` names (`ServerAddress.parseString`, then
/// `ServerNameResolver.resolveAddress` with its SRV redirect).
pub(super) fn resolve(ip: &str) -> Option<std::net::SocketAddr> {
    let with_port = address_with_port(ip);
    let (host, port) = with_port.rsplit_once(':')?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let resolved = photon_net::resolver::resolve_server(host, port.parse().ok()?);
    if resolved.is_none() {
        log::warn!("cannot resolve {ip}");
    }
    resolved
}
