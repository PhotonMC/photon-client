//! Configuration packets: unit packets, the code of conduct, client
//! information, enabled features and tags.

use serde_json::{Value, json};

use super::codec::{feature_frame, flag, string_frame, utf16_hex, varint_hex};
use crate::bits::hex;
use crate::case::case;

/// A 4-byte emoji and two 2-byte letters.
const RULES: &str = "Rules: \u{1f600} / caf\u{e9} na\u{ef}ve";
const LANGUAGE: &str = "fr_FR\u{1f600}";
const URL: &str = "https://example.invalid/rules";

fn wire(id: String, operation: &str, fields: Value) -> Value {
    case(id, "wire", operation, fields)
}

fn misc(id: String, direction: &str, packet: &str, fields: Value) -> Value {
    let mut base = json!({"direction": direction, "packet": packet});
    crate::case::merge(&mut base, fields);
    wire(id, "configuration_misc", base)
}

const PREFIXES: [(&str, &str); 2] = [("empty", ""), ("with-prefix", "deadbeef")];
const UNIT_READS: [(&str, &str); 3] = [("empty", ""), ("trailing", "aabbcc"), ("zero", "00")];

fn units() -> Vec<Value> {
    let mut cases = Vec::new();
    for packet in ["server-finish", "client-finish", "accept-code-of-conduct"] {
        for (label, prefix) in PREFIXES {
            let fields = json!({"direction": "write", "packet": packet, "prefix_hex": prefix});
            let id = format!("configuration/unit/{packet}/write/{label}");
            cases.push(wire(id, "configuration_unit", fields));
        }
        for (label, hex) in UNIT_READS {
            let fields = json!({"direction": "read", "packet": packet, "hex": hex});
            let id = format!("configuration/unit/{packet}/read/{label}");
            cases.push(wire(id, "configuration_unit", fields));
        }
    }
    cases
}

fn code_of_conduct() -> Vec<Value> {
    let writes = [
        ("empty", String::new(), ""),
        ("url", URL.to_owned(), "deadbeef"),
        ("unicode", RULES.to_owned(), ""),
        ("encode-limit", "a".repeat(32767), ""),
        ("encode-too-long", "a".repeat(32768), ""),
    ];
    let mut cases: Vec<Value> = writes
        .iter()
        .map(|(label, value, prefix)| {
            let fields =
                json!({"code_of_conduct_utf16_hex": utf16_hex(value), "prefix_hex": prefix});
            let id = format!("configuration/code-of-conduct/write/{label}");
            misc(id, "write", "code-of-conduct", fields)
        })
        .collect();
    let reads = [
        ("empty", string_frame("")),
        ("url", string_frame(URL)),
        ("unicode", string_frame(RULES)),
        ("trailing", format!("{}aabb", string_frame("ok"))),
        ("body-underflow", "01".to_owned()),
        ("malformed-utf8", "01ff".to_owned()),
        ("encoded-too-big", "808080808000".to_owned()),
        (
            "decoded-too-long",
            format!("{}{}", varint_hex(32768), "61".repeat(32768)),
        ),
    ];
    cases.extend(reads.iter().map(|(label, hex)| {
        let id = format!("configuration/code-of-conduct/read/{label}");
        misc(id, "read", "code-of-conduct", json!({ "hex": hex }))
    }));
    for (label, prefix) in PREFIXES {
        let id = format!("configuration/reset-chat/write/{label}");
        cases.push(misc(
            id,
            "write",
            "reset-chat",
            json!({ "prefix_hex": prefix }),
        ));
    }
    for (label, hex) in UNIT_READS {
        let id = format!("configuration/reset-chat/read/{label}");
        cases.push(misc(id, "read", "reset-chat", json!({ "hex": hex })));
    }
    cases
}

fn protocol_metadata() -> Vec<Value> {
    let connections = ["HANDSHAKING", "PLAY", "STATUS", "LOGIN", "CONFIGURATION"]
        .map(|name| ("connection", name));
    let flows = ["SERVERBOUND", "CLIENTBOUND"].map(|name| ("flow", name));
    connections
        .into_iter()
        .chain(flows)
        .map(|(kind, name)| {
            let id = format!("protocol/{kind}/{}", name.to_lowercase());
            wire(id, "protocol_metadata", json!({"kind": kind, "name": name}))
        })
        .collect()
}

/// The fields of a client information packet, in wire order.
struct Information<'a> {
    language: &'a [u8],
    view_distance: i64,
    chat: i64,
    colours: bool,
    model: i64,
    hand: i64,
    filtering: bool,
    listing: bool,
    particles: i64,
}

impl Information<'_> {
    fn hex(&self) -> String {
        let byte = |value: i64| format!("{:02x}", value & 0xff);
        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            varint_hex(self.language.len() as i64),
            hex(self.language),
            byte(self.view_distance),
            varint_hex(self.chat),
            flag(self.colours),
            byte(self.model),
            varint_hex(self.hand),
            flag(self.filtering),
            flag(self.listing),
            varint_hex(self.particles)
        )
    }
}

const BASIC: Information = Information {
    language: b"en_us",
    view_distance: 2,
    chat: 0,
    colours: true,
    model: 0,
    hand: 1,
    filtering: false,
    listing: false,
    particles: 0,
};

fn information_writes() -> Vec<(&'static str, Value)> {
    let write = |language: &str, view: i64, chat: &str, colours: bool, model: i64| {
        json!({
            "direction": "write", "language_utf16_hex": utf16_hex(language), "view_distance": view,
            "chat_visibility": chat, "chat_colors": colours, "model_customisation": model,
        })
    };
    let tail = |hand: &str, filtering: bool, listing: bool, particles: &str, prefix: &str| {
        json!({
            "main_hand": hand, "text_filtering_enabled": filtering, "allows_listing": listing,
            "particle_status": particles, "prefix_hex": prefix,
        })
    };
    let long = "a".repeat(32768);
    let rows = [
        (
            "basic",
            write("en_us", 2, "full", true, 0),
            tail("right", false, false, "all", ""),
        ),
        (
            "empty",
            write("", 0, "hidden", false, 255),
            tail("left", true, true, "minimal", "aa55"),
        ),
        (
            "unicode",
            write(LANGUAGE, -1, "system", true, 3),
            tail("right", true, false, "decreased", ""),
        ),
        (
            "view-overflow",
            write("en_us", 256, "full", false, 0),
            tail("left", false, false, "all", ""),
        ),
        (
            "model-overflow",
            write("en_us", 2, "full", false, 511),
            tail("right", false, false, "all", ""),
        ),
        (
            "language-encode-limit",
            write(&long, 2, "full", true, 0),
            tail("right", false, false, "all", ""),
        ),
    ];
    rows.into_iter()
        .map(|(label, mut fields, tail)| {
            crate::case::merge(&mut fields, tail);
            (label, fields)
        })
        .collect()
}

fn client_information() -> Vec<Value> {
    let mut cases: Vec<Value> = information_writes()
        .into_iter()
        .map(|(label, fields)| {
            let id = format!("configuration/client-information/write/{label}");
            wire(id, "client_information", fields)
        })
        .collect();
    let unicode = Information {
        language: LANGUAGE.as_bytes(),
        view_distance: -1,
        chat: 1,
        model: 3,
        filtering: true,
        particles: 1,
        ..BASIC
    };
    let reads = [
        ("basic", BASIC.hex()),
        ("trailing", format!("{}aabb", BASIC.hex())),
        ("unicode", unicode.hex()),
        ("language-underflow", "01".to_owned()),
        ("language-encoded-too-long", varint_hex(49)),
        (
            "language-decoded-too-long",
            format!("{}{}", varint_hex(17), "61".repeat(17)),
        ),
        ("view-underflow", "00".to_owned()),
        ("chat-prefix-underflow", "000080".to_owned()),
        ("chat-invalid", "000003".to_owned()),
        ("chat-colors-underflow", "000000".to_owned()),
        ("model-underflow", "00000000".to_owned()),
        ("arm-invalid", "000000000003".to_owned()),
        ("text-filter-underflow", "000000000001".to_owned()),
        ("allows-underflow", "00000000000100".to_owned()),
        ("particle-invalid", "000000000001000003".to_owned()),
        ("particle-underflow", "000000000001000080".to_owned()),
        ("boolean-noncanonical", "000000020001020300".to_owned()),
    ];
    cases.extend(reads.iter().map(|(label, hex)| {
        let fields = json!({"direction": "read", "hex": hex});
        wire(
            format!("configuration/client-information/read/{label}"),
            "client_information",
            fields,
        )
    }));
    cases
}

const FEATURE_SETS: &[(&str, &[&str])] = &[
    ("single", &["minecraft:vanilla"]),
    ("default-namespace", &["vanilla"]),
    (
        "multiple",
        &[
            "minecraft:vanilla",
            "minecraft:trade_rebalance",
            "custom:feature/path",
        ],
    ),
    (
        "reordered",
        &[
            "custom:feature/path",
            "minecraft:trade_rebalance",
            "minecraft:vanilla",
        ],
    ),
    (
        "duplicate",
        &[
            "minecraft:vanilla",
            "minecraft:vanilla",
            "custom:feature/path",
        ],
    ),
];

fn feature_writes() -> Vec<(&'static str, Vec<String>, &'static str)> {
    let mut writes: Vec<(&str, Vec<String>, &str)> = vec![("empty", vec![], "deadbeef")];
    for (label, features) in FEATURE_SETS {
        let prefix = if *label == "duplicate" { "aa55" } else { "" };
        let features = features
            .iter()
            .map(|feature| (*feature).to_owned())
            .collect();
        writes.push((label, features, prefix));
    }
    let oversized = vec![format!("minecraft:{}", "a".repeat(32768))];
    writes.push(("feature-encode-too-long", oversized, ""));
    writes
}

fn feature_reads() -> Vec<(&'static str, String)> {
    let mut reads = vec![("empty", feature_frame(&[]))];
    reads.extend(
        FEATURE_SETS
            .iter()
            .map(|(label, features)| (*label, feature_frame(features))),
    );
    let oversized = "a".repeat(32768);
    reads.extend([
        (
            "trailing",
            format!("{}aabb", feature_frame(&["minecraft:vanilla"])),
        ),
        ("feature-prefix-underflow", varint_hex(1)),
        (
            "feature-body-underflow",
            format!("{}{}6162", varint_hex(1), varint_hex(4)),
        ),
        ("invalid-namespace", feature_frame(&["Bad:feature"])),
        ("invalid-path", feature_frame(&["minecraft:bad?feature"])),
        (
            "malformed-utf8",
            format!("{}{}ff", varint_hex(1), varint_hex(1)),
        ),
        (
            "encoded-too-long",
            format!("{}{}", varint_hex(1), varint_hex(32767 * 3 + 1)),
        ),
        ("decoded-too-long", feature_frame(&[&oversized])),
        ("negative-count", varint_hex(-1)),
        ("count-too-big", "808080808001".to_owned()),
    ]);
    reads
}

fn enabled_features() -> Vec<Value> {
    let mut cases: Vec<Value> = feature_writes()
        .iter()
        .map(|(label, features, prefix)| {
            let fields = json!({"direction": "write", "features": features, "prefix_hex": prefix});
            let id = format!("configuration/enabled-features/write/{label}");
            wire(id, "enabled_features", fields)
        })
        .collect();
    cases.extend(feature_reads().iter().map(|(label, hex)| {
        let fields = json!({"direction": "read", "hex": hex});
        let id = format!("configuration/enabled-features/read/{label}");
        wire(id, "enabled_features", fields)
    }));
    cases
}

pub fn cases() -> Vec<Value> {
    let mut cases = units();
    cases.extend(code_of_conduct());
    cases.extend(protocol_metadata());
    cases.extend(client_information());
    cases.extend(enabled_features());
    cases.extend(super::tags::cases());
    cases.extend(super::registry::cases());
    cases
}
