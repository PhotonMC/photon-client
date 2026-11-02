//! `LegacyServerPinger`: the ping of the servers before the status protocol, which the server
//! list tries when the modern ping failed. Such a server is listed as incompatible.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// What a legacy server answered: its protocol, version name, text and player counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyStatus {
    pub protocol: i32,
    pub version: String,
    pub motd: String,
    pub players: i32,
    pub max_players: i32,
}

/// `LegacyProtocolUtils.writeLegacyString`: the length in characters, then UTF-16BE.
fn write_legacy_string(out: &mut Vec<u8>, text: &str) {
    let units: Vec<u16> = text.encode_utf16().collect();
    out.extend_from_slice(&(units.len() as u16).to_be_bytes());
    for unit in units {
        out.extend_from_slice(&unit.to_be_bytes());
    }
}

/// The `0xFE 0x01 0xFA MC|PingHost` request for the host and port the player typed.
fn request(host: &str, port: u16) -> Vec<u8> {
    let mut out = vec![254, 1, 250];
    write_legacy_string(&mut out, "MC|PingHost");
    let mut payload = vec![127];
    write_legacy_string(&mut payload, host);
    payload.extend_from_slice(&i32::from(port).to_be_bytes());
    out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    out.extend_from_slice(&payload);
    out
}

/// `Mth.getInt`: the number a text holds, or the default.
fn int_or(text: &str, default: i32) -> i32 {
    text.parse().unwrap_or(default)
}

/// The answer's `section-sign 1` fields (`LegacyServerPinger.channelRead0`); none for any other answer.
fn parse(text: &str) -> Option<LegacyStatus> {
    let mut fields = text.splitn(6, '\0');
    if fields.next()? != "\u{a7}1" {
        return None;
    }
    let mut next = || fields.next();
    Some(LegacyStatus {
        protocol: int_or(next()?, 0),
        version: next()?.to_owned(),
        motd: next()?.to_owned(),
        players: int_or(next()?, -1),
        max_players: int_or(next()?, -1),
    })
}

/// Ping a server the old way: none when it does not answer or answers with something else.
pub fn query_legacy_status(
    address: SocketAddr,
    (host, port): (&str, u16),
    timeout: Duration,
) -> Option<LegacyStatus> {
    let mut stream = TcpStream::connect_timeout(&address, timeout).ok()?;
    stream.set_read_timeout(Some(timeout)).ok()?;
    stream.set_write_timeout(Some(timeout)).ok()?;
    stream.write_all(&request(host, port)).ok()?;
    let mut head = [0u8; 3];
    stream.read_exact(&mut head).ok()?;
    if head[0] != 255 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([head[1], head[2]]));
    let mut body = vec![0u8; length * 2];
    stream.read_exact(&mut body).ok()?;
    let units: Vec<u16> = body
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_be_bytes(*pair))
        .collect();
    parse(&String::from_utf16(&units).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_names_the_host_and_port() {
        let bytes = request("a.b", 25565);
        assert_eq!(&bytes[..3], [254, 1, 250]);
        // "MC|PingHost" is 11 characters of two bytes.
        assert_eq!(&bytes[3..5], [0, 11]);
        let tail = &bytes[5 + 22..];
        // payload size: 1 + 2 + 3 * 2 + 4.
        assert_eq!(&tail[..2], [0, 13]);
        assert_eq!(tail[2], 127);
        assert_eq!(&tail[tail.len() - 4..], 25565_i32.to_be_bytes());
    }

    #[test]
    fn an_answer_reads_by_its_nul_separated_fields() {
        let status = parse("\u{a7}1\u{0}47\u{0}1.8.8\u{0}Old server\u{0}3\u{0}20").expect("legacy");
        assert_eq!(
            status,
            LegacyStatus {
                protocol: 47,
                version: "1.8.8".to_owned(),
                motd: "Old server".to_owned(),
                players: 3,
                max_players: 20,
            }
        );
        assert_eq!(parse("other"), None);
    }

    #[test]
    fn an_old_server_that_answers_is_read_over_the_wire() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut asked = [0u8; 3];
            stream.read_exact(&mut asked).expect("request");
            assert_eq!(asked, [254, 1, 250]);
            let text = "\u{a7}1\u{0}47\u{0}1.8.8\u{0}Old\u{0}1\u{0}20";
            let mut out = vec![255];
            write_legacy_string(&mut out, text);
            stream.write_all(&out).expect("answer");
        });
        let status = query_legacy_status(address, ("localhost", 25565), Duration::from_secs(5));
        server.join().expect("server");
        assert_eq!(
            status.map(|status| status.version),
            Some("1.8.8".to_owned())
        );
    }
}
