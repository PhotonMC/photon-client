//! `ServerNameResolver`: the address a server name connects to. A name on the default port first
//! looks for a `_minecraft._tcp` SRV record, which names the real host and port.
//!
//! The record is asked of the system's name servers (`/etc/resolv.conf`) with a small UDP query,
//! because the standard library cannot look up SRV records. Without a name server, or on any
//! failure, there is no redirect (the reference client ignores a failed lookup the same way).
//! Names outside ASCII are not converted (`IDN.toASCII`) and so get no SRV lookup.

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

const DEFAULT_PORT: u16 = 25565;
const SRV_TYPE: u16 = 33;
const QUERY_TIMEOUT: Duration = Duration::from_secs(1);

/// A host name and port to the socket address to connect to (`ServerNameResolver.resolveAddress`
/// without the blocked address list): the name itself resolves first, then an SRV record for it
/// replaces the host and port when there is one.
pub fn resolve_server(host: &str, port: u16) -> Option<SocketAddr> {
    let mut resolved = lookup(host, port);
    if let Some((srv_host, srv_port)) = redirect(host, port) {
        resolved = lookup(&srv_host, srv_port);
    }
    resolved
}

fn lookup(host: &str, port: u16) -> Option<SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

/// `ServerRedirectHandler.createDnsSrvRedirectHandler`: only the default port is redirected.
fn redirect(host: &str, port: u16) -> Option<(String, u16)> {
    if port != DEFAULT_PORT || !host.is_ascii() || host.is_empty() {
        return None;
    }
    let query = srv_query(0x5048, &format!("_minecraft._tcp.{host}"))?;
    nameservers().into_iter().find_map(|server| {
        let socket = UdpSocket::bind(("0.0.0.0", 0))
            .or_else(|_| UdpSocket::bind(("::", 0)))
            .ok()?;
        socket.set_read_timeout(Some(QUERY_TIMEOUT)).ok()?;
        socket.connect(server).ok()?;
        socket.send(&query).ok()?;
        let mut answer = [0u8; 4096];
        let length = socket.recv(&mut answer).ok()?;
        parse_srv_answer(&answer[..length], 0x5048)
    })
}

fn nameservers() -> Vec<SocketAddr> {
    let Ok(text) = std::fs::read_to_string("/etc/resolv.conf") else {
        return Vec::new();
    };
    nameservers_from(&text)
}

fn nameservers_from(text: &str) -> Vec<SocketAddr> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            (words.next()? == "nameserver").then_some(())?;
            let address = words.next()?.split('%').next()?;
            address
                .parse::<std::net::IpAddr>()
                .ok()
                .map(|ip| SocketAddr::new(ip, 53))
        })
        .collect()
}

/// A recursive SRV question for `name`.
fn srv_query(id: u16, name: &str) -> Option<Vec<u8>> {
    let mut query = Vec::new();
    query.extend_from_slice(&id.to_be_bytes());
    query.extend_from_slice(&[0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0]);
    for label in name.trim_end_matches('.').split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        query.push(label.len() as u8);
        query.extend_from_slice(label.as_bytes());
    }
    query.push(0);
    query.extend_from_slice(&SRV_TYPE.to_be_bytes());
    query.extend_from_slice(&1u16.to_be_bytes());
    (query.len() <= 512).then_some(query)
}

/// The first SRV answer of a response to the question with `id`: its target (without the final
/// dot) and port.
fn parse_srv_answer(message: &[u8], id: u16) -> Option<(String, u16)> {
    if message.len() < 12 || message[0..2] != id.to_be_bytes() {
        return None;
    }
    let flags = u16::from_be_bytes([message[2], message[3]]);
    if flags & 0x8000 == 0 || flags & 0xf != 0 {
        return None;
    }
    let questions = u16::from_be_bytes([message[4], message[5]]);
    let answers = u16::from_be_bytes([message[6], message[7]]);
    let mut at = 12;
    for _ in 0..questions {
        at = skip_name(message, at)? + 4;
    }
    for _ in 0..answers {
        at = skip_name(message, at)?;
        let record = message.get(at..at + 10)?;
        let kind = u16::from_be_bytes([record[0], record[1]]);
        let length = usize::from(u16::from_be_bytes([record[8], record[9]]));
        let data = at + 10;
        if kind == SRV_TYPE {
            let port = u16::from_be_bytes([*message.get(data + 4)?, *message.get(data + 5)?]);
            let target = read_name(message, data + 6)?;
            return Some((target, port));
        }
        at = data + length;
    }
    None
}

/// The offset after the name at `at` (a compression pointer ends it).
fn skip_name(message: &[u8], mut at: usize) -> Option<usize> {
    loop {
        let length = *message.get(at)?;
        match length {
            0 => return Some(at + 1),
            l if l & 0xc0 == 0xc0 => return Some(at + 2),
            l => at += usize::from(l) + 1,
        }
    }
}

/// The dotted name at `at`, following compression pointers.
fn read_name(message: &[u8], mut at: usize) -> Option<String> {
    let mut name = String::new();
    for _ in 0..128 {
        let length = *message.get(at)?;
        if length == 0 {
            return Some(name);
        }
        if length & 0xc0 == 0xc0 {
            at = usize::from(length & 0x3f) << 8 | usize::from(*message.get(at + 1)?);
            continue;
        }
        let label = message.get(at + 1..at + 1 + usize::from(length))?;
        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(std::str::from_utf8(label).ok()?);
        at += 1 + usize::from(length);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(id: u16, rcode: u16, target: &[u8]) -> Vec<u8> {
        let mut m = Vec::new();
        m.extend_from_slice(&id.to_be_bytes());
        m.extend_from_slice(&(0x8180u16 | rcode).to_be_bytes());
        m.extend_from_slice(&[0, 1, 0, 1, 0, 0, 0, 0]);
        let name = b"\x04_foo\x04_tcp\x07example\x03com\x00";
        m.extend_from_slice(name);
        m.extend_from_slice(&[0, 33, 0, 1]);
        m.extend_from_slice(&[0xc0, 12, 0, 33, 0, 1, 0, 0, 0, 60]);
        let mut data = vec![0, 5, 0, 6, 0x63, 0xdd];
        data.extend_from_slice(target);
        m.extend_from_slice(&(data.len() as u16).to_be_bytes());
        m.extend_from_slice(&data);
        m
    }

    #[test]
    fn reads_target_and_port_with_compression() {
        // "play" followed by a pointer to "example.com" inside the question name.
        let target = [4, b'p', b'l', b'a', b'y', 0xc0, 12 + 10];
        let m = response(7, 0, &target);
        assert_eq!(
            parse_srv_answer(&m, 7),
            Some(("play.example.com".to_owned(), 25565))
        );
        let plain = b"\x03mc2\x07example\x03com\x00";
        assert_eq!(
            parse_srv_answer(&response(7, 0, plain), 7),
            Some(("mc2.example.com".to_owned(), 25565))
        );
    }

    #[test]
    fn ignores_errors_and_foreign_answers() {
        let plain = b"\x01a\x00";
        assert_eq!(parse_srv_answer(&response(7, 3, plain), 7), None);
        assert_eq!(parse_srv_answer(&response(8, 0, plain), 7), None);
        assert_eq!(parse_srv_answer(&[0; 5], 7), None);
    }

    #[test]
    fn only_the_default_port_is_redirected() {
        assert_eq!(redirect("example.com", 25566), None);
        assert_eq!(redirect("b\u{fc}cher.de", 25565), None);
    }

    #[test]
    fn query_is_well_formed() {
        let query = srv_query(1, "_minecraft._tcp.example.com").unwrap();
        assert_eq!(&query[..2], &[0, 1]);
        assert_eq!(query[12], 10);
        assert_eq!(&query[query.len() - 4..], &[0, 33, 0, 1]);
        assert!(srv_query(1, "a..b").is_none());
    }

    #[test]
    fn nameservers_are_read_from_resolv_conf() {
        let text = "# c\nsearch lan\nnameserver 10.0.0.1\nnameserver fe80::1%eth0\nnameserver x\n";
        let servers = nameservers_from(text);
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].to_string(), "10.0.0.1:53");
    }
}
