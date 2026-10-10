//! Handshake and login packets.

use serde_json::{Value, json};

use super::codec::{flag, utf16_hex, varint_hex};
use crate::bits::hex;
use crate::case::case;

const JAPANESE_HOST: &str = "\u{4f8b}\u{3048}.\u{30c6}\u{30b9}\u{30c8}";
const CHINESE_ID: &str = "\u{670d}\u{52a1}\u{5668}";
const EMOJI_NAME: &str = "\u{1f600}Alex";
const PROFILE: (&str, &str) = ("0123456789abcdef", "fedcba9876543210");

fn wire(id: String, operation: &str, fields: Value) -> Value {
    case(id, "wire", operation, fields)
}

fn read(id: &str, operation: &str, hex: &str) -> Value {
    wire(id.to_owned(), operation, json!({"direction": "read", "hex": hex}))
}

fn bytes_frame(bytes: &[u8]) -> String {
    format!("{}{}", varint_hex(bytes.len() as i64), hex(bytes))
}

fn intention_hex(protocol: i64, host: &[u8], port: u16, intention: i64) -> String {
    let port = hex(&port.to_be_bytes());
    format!("{}{}{port}{}", varint_hex(protocol), bytes_frame(host), varint_hex(intention))
}

fn handshake() -> Vec<Value> {
    let long = |count| "a".repeat(count);
    let writes = [
        ("basic", 776, "localhost".to_owned(), 25565, "status", "deadbeef"),
        ("unicode-host", 776, JAPANESE_HOST.to_owned(), 25565, "login", ""),
        ("transfer", 776, "transfer.example".to_owned(), 0, "transfer", ""),
        ("empty-host", 0, String::new(), 65535, "login", "aa"),
        ("negative-protocol", -1, "legacy".to_owned(), 25565, "status", ""),
        ("port-wrap", 776, "localhost".to_owned(), -1, "status", ""),
        ("host-over-decode-limit", 776, long(256), 25565, "login", ""),
        ("host-encode-limit", 776, long(32768), 25565, "login", ""),
    ];
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, protocol, host, port, intention, prefix)| {
            let fields = json!({
                "direction": "write", "protocol_version": protocol, "host_utf16_hex": utf16_hex(host),
                "port": port, "intention": intention, "prefix_hex": prefix,
            });
            wire(format!("handshake/intention/write/{label}"), "handshake_intention", fields)
        })
        .collect();
    cases.extend(
        handshake_reads()
            .iter()
            .map(|(label, hex)| read(&format!("handshake/intention/read/{label}"), "handshake_intention", hex)),
    );
    cases
}

fn handshake_reads() -> Vec<(&'static str, String)> {
    let basic = intention_hex(776, b"localhost", 25565, 1);
    vec![
        ("basic", basic.clone()),
        ("unicode-host", intention_hex(776, JAPANESE_HOST.as_bytes(), 25565, 2)),
        ("trailing", format!("{basic}aabb")),
        ("protocol-underflow", String::new()),
        ("host-prefix-underflow", format!("{}80", varint_hex(776))),
        ("port-underflow", format!("{}0000", varint_hex(776))),
        ("unknown-intent", intention_hex(776, b"localhost", 25565, 4)),
        ("host-encoded-too-long", format!("{}{}", varint_hex(776), varint_hex(766))),
        ("host-decoded-too-long", intention_hex(776, &[b'a'; 256], 25565, 2)),
        ("protocol-too-big", "808080808001".to_owned()),
    ]
}

fn hello_hex(name: &[u8], most: &str, least: &str) -> String {
    format!("{}{most}{least}", bytes_frame(name))
}

fn hello() -> Vec<Value> {
    let writes = [
        ("basic", "Player".to_owned(), PROFILE.0, PROFILE.1, ""),
        ("empty-name", String::new(), "0000000000000000", "0000000000000000", "aa55"),
        ("unicode-name", EMOJI_NAME.to_owned(), "8000000000000001", "7fffffffffffffff", ""),
        ("maximum-name", "a".repeat(16), "ffffffffffffffff", "0000000000000001", ""),
        ("too-long-name", "a".repeat(17), PROFILE.0, PROFILE.1, ""),
    ];
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, name, most, least, prefix)| {
            let fields = json!({
                "direction": "write", "name_utf16_hex": utf16_hex(name), "profile_most_hex": most,
                "profile_least_hex": least, "prefix_hex": prefix,
            });
            wire(format!("login/hello/write/{label}"), "login_hello", fields)
        })
        .collect();
    let reads = hello_reads();
    cases.extend(reads.iter().map(|(label, hex)| read(&format!("login/hello/read/{label}"), "login_hello", hex)));
    cases
}

fn hello_reads() -> Vec<(&'static str, String)> {
    let basic = hello_hex(b"Player", PROFILE.0, PROFILE.1);
    vec![
        ("basic", basic.clone()),
        ("trailing", format!("{basic}aabb")),
        ("empty-name", hello_hex(b"", "0000000000000000", "0000000000000000")),
        ("name-underflow", "01".to_owned()),
        ("name-encoded-too-long", "31".to_owned()),
        ("name-decoded-too-long", hello_hex(&[b'a'; 17], PROFILE.0, PROFILE.1)),
        ("uuid-first-half-underflow", "00".repeat(8)),
        ("uuid-second-half-underflow", "00".repeat(16)),
        ("name-too-big", "808080808001".to_owned()),
    ]
}

fn compression() -> Vec<Value> {
    let writes = [
        ("zero", 0, ""),
        ("one", 1, ""),
        ("single-byte-limit", 127, ""),
        ("two-byte-start", 128, ""),
        ("negative", -1, ""),
        ("maximum", i32::MAX, ""),
        ("with-prefix", 256, "deadbeef"),
    ];
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, threshold, prefix)| {
            let fields = json!({"direction": "write", "threshold": threshold, "prefix_hex": prefix});
            wire(format!("login/compression/write/{label}"), "login_compression", fields)
        })
        .collect();
    let reads = [
        ("zero", varint_hex(0)),
        ("negative", varint_hex(-1)),
        ("trailing", format!("{}aabb", varint_hex(128))),
        ("noncanonical-zero", "8000".to_owned()),
        ("empty", String::new()),
        ("truncated", "80".to_owned()),
        ("too-big", "808080808001".to_owned()),
    ];
    cases.extend(
        reads.iter().map(|(label, hex)| read(&format!("login/compression/read/{label}"), "login_compression", hex)),
    );
    cases
}

fn server_hello_hex(server: &[u8], public_key: &str, challenge: &str, authenticate: bool) -> String {
    let field = |hex: &str| format!("{}{hex}", varint_hex(hex.len() as i64 / 2));
    format!("{}{}{}{}", bytes_frame(server), field(public_key), field(challenge), flag(authenticate))
}

fn server_hello() -> Vec<Value> {
    let writes = [
        ("basic", "server".to_owned(), "000102", "aabb", true, "dead"),
        ("empty", String::new(), "", "", false, ""),
        ("unicode-server-id", CHINESE_ID.to_owned(), "ff00", "01020304", true, ""),
        ("maximum-read-name", "a".repeat(20), "", "", false, ""),
        ("over-read-limit", "a".repeat(21), "00", "00", true, ""),
    ];
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, server, public_key, challenge, authenticate, prefix)| {
            let fields = json!({
                "direction": "write", "server_id_utf16_hex": utf16_hex(server),
                "public_key_hex": public_key, "challenge_hex": challenge,
                "should_authenticate": authenticate, "prefix_hex": prefix,
            });
            wire(format!("login/server-hello/write/{label}"), "login_server_hello", fields)
        })
        .collect();
    let basic = server_hello_hex(b"server", "000102", "aabb", true);
    let reads = [
        ("basic", basic.clone()),
        ("trailing", format!("{basic}aabb")),
        ("empty", server_hello_hex(b"", "", "", false)),
        ("server-id-underflow", "01".to_owned()),
        ("server-id-encoded-too-long", varint_hex(61)),
        ("server-id-decoded-too-long", server_hello_hex(&[b'a'; 21], "00", "00", true)),
        ("public-key-underflow", "0003aabb".to_owned()),
        ("challenge-underflow", "00010002aa".to_owned()),
        ("boolean-underflow", "000000".to_owned()),
        ("boolean-noncanonical", "00000002".to_owned()),
        ("negative-array-size", format!("00{}", varint_hex(-1))),
        ("server-id-too-big", "808080808001".to_owned()),
    ];
    cases.extend(
        reads.iter().map(|(label, hex)| read(&format!("login/server-hello/read/{label}"), "login_server_hello", hex)),
    );
    cases
}

fn acknowledged() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, prefix) in [("empty", ""), ("with-prefix", "deadbeef")] {
        let fields = json!({"direction": "write", "prefix_hex": prefix});
        cases.push(wire(format!("login/acknowledged/write/{label}"), "login_acknowledged", fields));
    }
    for (label, hex) in [("empty", ""), ("trailing", "aabbcc"), ("zero", "00")] {
        cases.push(read(&format!("login/acknowledged/read/{label}"), "login_acknowledged", hex));
    }
    cases
}

pub fn cases() -> Vec<Value> {
    let mut cases = handshake();
    cases.extend(hello());
    cases.extend(compression());
    cases.extend(server_hello());
    cases.extend(acknowledged());
    cases.extend(super::finished::cases());
    cases
}
