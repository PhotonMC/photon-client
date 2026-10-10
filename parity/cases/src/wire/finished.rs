//! The login finished packet: the game profile and its properties.

use serde_json::{Value, json};

use super::codec::{utf16_hex, varint_hex};
use crate::bits::hex;
use crate::case::case;

const PROFILE: (&str, &str) = ("0123456789abcdef", "fedcba9876543210");
const SESSION: (&str, &str) = ("0011223344556677", "8899aabbccddeeff");

fn property(name: &str, value: &str, signature: Option<&str>) -> Value {
    json!({
        "name_utf16_hex": utf16_hex(name), "value_utf16_hex": utf16_hex(value),
        "signature_utf16_hex": signature.map(utf16_hex),
    })
}

fn bytes_frame(bytes: &[u8]) -> String {
    format!("{}{}", varint_hex(bytes.len() as i64), hex(bytes))
}

fn property_hex(name: &[u8], value: &[u8], signature: Option<&[u8]>) -> String {
    let fields = format!("{}{}", bytes_frame(name), bytes_frame(value));
    match signature {
        None => format!("{fields}00"),
        Some(signature) => format!("{fields}01{}", bytes_frame(signature)),
    }
}

fn finished_hex(name: &[u8], properties: &[String]) -> String {
    format!(
        "{}{}{}{}{}{}{}",
        PROFILE.0,
        PROFILE.1,
        bytes_frame(name),
        varint_hex(properties.len() as i64),
        properties.concat(),
        SESSION.0,
        SESSION.1
    )
}

fn write_sets() -> Vec<(&'static str, String, Vec<Value>, &'static str)> {
    let numbered = |count: usize, name: &dyn Fn(usize) -> String| -> Vec<Value> {
        (0..count).map(|index| property(&name(index), &format!("v{index}"), None)).collect()
    };
    let player = || "Player".to_owned();
    let signed = vec![property("textures", "payload", Some("signature"))];
    vec![
        ("basic", player(), vec![], ""),
        ("empty", String::new(), vec![], "aa55"),
        ("unicode-signed", "\u{1f600}Alex".to_owned(), signed, ""),
        ("maximum-name", "a".repeat(16), numbered(16, &|_| "p".to_owned()), ""),
        ("too-many-properties", player(), numbered(17, &|index| format!("p{index}")), ""),
        ("too-long-name", "a".repeat(17), vec![], ""),
        ("property-name-too-long", player(), vec![property(&"n".repeat(65), "v", None)], ""),
        ("property-value-too-long", player(), vec![property("n", &"v".repeat(32768), None)], ""),
        ("signature-too-long", player(), vec![property("n", "v", Some(&"s".repeat(1025)))], ""),
    ]
}

fn writes() -> Vec<Value> {
    write_sets()
        .into_iter()
        .map(|(label, name, properties, prefix)| {
            let fields = json!({
                "direction": "write", "profile_most_hex": PROFILE.0, "profile_least_hex": PROFILE.1,
                "name_utf16_hex": utf16_hex(&name), "properties": properties,
                "session_most_hex": SESSION.0, "session_least_hex": SESSION.1, "prefix_hex": prefix,
            });
            case(format!("login/finished/write/{label}"), "wire", "login_finished", fields)
        })
        .collect()
}

fn reads() -> Vec<(&'static str, String)> {
    let empty = finished_hex(b"", &[]);
    // The profile and an empty name, without the property count and session.
    let head = &empty[..empty.len() - 34];
    let player = finished_hex(b"Player", &[]);
    let unsigned = property_hex(b"n", b"v", None);
    vec![
        ("basic", player.clone()),
        ("signed-property", finished_hex(b"Player", &[property_hex(b"textures", b"payload", Some(b"signature"))])),
        ("trailing", format!("{player}aabb")),
        ("profile-first-underflow", "00".repeat(7)),
        ("profile-second-underflow", "00".repeat(15)),
        ("name-underflow", format!("{}{}01", PROFILE.0, PROFILE.1)),
        ("name-too-long", format!("{}{}{}", PROFILE.0, PROFILE.1, varint_hex(49))),
        ("property-count-too-large", format!("{}{}", &player[..player.len() - 34], varint_hex(17))),
        ("property-count-negative", format!("{head}{}{}{}", varint_hex(-1), SESSION.0, SESSION.1)),
        ("property-name-underflow", format!("{head}{}", varint_hex(1))),
        ("property-value-underflow", format!("{head}{}{}{}01", varint_hex(1), varint_hex(1), hex(b"n"))),
        ("signature-underflow", format!("{head}{}{}0101", varint_hex(1), &unsigned[..unsigned.len() - 2])),
        ("session-first-underflow", format!("{}{}", &empty[..empty.len() - 32], "00".repeat(7))),
        ("session-second-underflow", format!("{}{}", &empty[..empty.len() - 32], "00".repeat(15))),
    ]
}

pub fn cases() -> Vec<Value> {
    let mut cases = writes();
    cases.extend(reads().into_iter().map(|(label, hex)| {
        let fields = json!({"direction": "read", "hex": hex});
        case(format!("login/finished/read/{label}"), "wire", "login_finished", fields)
    }));
    cases
}
