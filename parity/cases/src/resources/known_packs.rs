//! Known-pack values and the codecs of one pack and of a pack list.

use serde_json::{Value, json};

use super::resources;
use crate::bits::{hex, varint as varint_bytes};

/// Non-ASCII fields: a two-byte letter, a four-byte emoji and a combining
/// accent.
const UNICODE: (&str, &str, &str) = ("m\u{ef}necraft", "pack-\u{1f600}", "v1\u{301}");

fn varint(value: i64) -> String {
    hex(&varint_bytes(value))
}

fn string(value: &str) -> String {
    format!("{}{}", varint(value.len() as i64), hex(value.as_bytes()))
}

fn pack_value(namespace: &str, id: &str, version: &str) -> Value {
    json!({"namespace": namespace, "id": id, "version": version})
}

fn pack_frame(namespace: &str, id: &str, version: &str) -> String {
    format!("{}{}{}", string(namespace), string(id), string(version))
}

fn known_pack() -> Value {
    let probes = json!([
        {"kind": "vanilla", "id": "vanilla"},
        {"kind": "vanilla", "id": "custom-pack"},
        {"namespace": "minecraft", "id": "manual", "version": "26.2"},
        {"namespace": "custom", "id": "manual", "version": "v1"},
        {"namespace": "Minecraft", "id": "case-sensitive", "version": "26.2"},
        {"namespace": "", "id": "", "version": ""},
    ]);
    resources(
        "metadata/known-pack-values",
        "known_pack",
        json!({ "probes": probes }),
    )
}

fn codec(mode: &str, label: &str, fields: Value) -> Value {
    let id = format!("metadata/known-pack-codec/{mode}/{label}");
    let mut case = resources(
        &id,
        "known_pack_codec",
        json!({"mode": mode, "prefix_hex": "a55a"}),
    );
    crate::case::merge(&mut case, fields);
    case
}

fn codec_cases() -> Vec<Value> {
    let long = |ch: &str| ch.repeat(32768);
    let encodes = [
        (
            "ascii",
            "minecraft".to_owned(),
            "vanilla".to_owned(),
            "26.2".to_owned(),
        ),
        (
            "unicode",
            UNICODE.0.to_owned(),
            UNICODE.1.to_owned(),
            UNICODE.2.to_owned(),
        ),
        ("empty-fields", String::new(), String::new(), String::new()),
        ("long-namespace", long("a"), "id".to_owned(), "v".to_owned()),
        ("long-id", "minecraft".to_owned(), long("b"), "v".to_owned()),
        (
            "long-version",
            "minecraft".to_owned(),
            "id".to_owned(),
            long("c"),
        ),
    ];
    let mut cases: Vec<Value> = encodes
        .iter()
        .map(|(label, namespace, id, version)| {
            codec(
                "encode",
                label,
                json!({"namespace": namespace, "id": id, "version": version}),
            )
        })
        .collect();
    let decodes = [
        ("ascii", pack_frame("minecraft", "vanilla", "26.2")),
        ("unicode", pack_frame(UNICODE.0, UNICODE.1, UNICODE.2)),
        ("empty-fields", "000000".to_owned()),
        ("trailing", "00000000aa".to_owned()),
        ("malformed-body", "01ff000000".to_owned()),
        ("second-prefix-truncated", "0080".to_owned()),
        ("third-body-missing", "000001".to_owned()),
        ("second-body-short", "000241".to_owned()),
        ("first-prefix-truncated", "80".to_owned()),
        ("first-prefix-too-big", "808080808000".to_owned()),
        ("first-encoded-limit", "feff05".to_owned()),
    ];
    cases.extend(
        decodes
            .iter()
            .map(|(label, framed)| codec("decode", label, json!({ "hex": framed }))),
    );
    cases
}

fn list_codec(mode: &str, label: &str, packet: &str, fields: Value) -> Value {
    let id = format!("metadata/known-pack-list-codec/{mode}/{label}");
    let base = json!({"mode": mode, "packet": packet, "prefix_hex": "a55a"});
    let mut case = resources(&id, "known_pack_list_codec", base);
    crate::case::merge(&mut case, fields);
    case
}

fn list_cases() -> Vec<Value> {
    let basic = vec![
        pack_value("minecraft", "vanilla", "26.2"),
        pack_value("custom", "example", "v1"),
    ];
    let small = pack_value("minecraft", "p", "1");
    let encodes = [
        ("clientbound-empty", "clientbound", vec![]),
        ("clientbound-basic", "clientbound", basic),
        ("serverbound-empty", "serverbound", vec![]),
        ("serverbound-64", "serverbound", vec![small.clone(); 64]),
        ("serverbound-65", "serverbound", vec![small; 65]),
        (
            "serverbound-member-partial",
            "serverbound",
            vec![pack_value("minecraft", &"a".repeat(32768), "1")],
        ),
    ];
    let mut cases: Vec<Value> = encodes
        .into_iter()
        .map(|(label, packet, packs)| {
            list_codec("encode", label, packet, json!({ "packs": packs }))
        })
        .collect();
    let basic_frame = format!(
        "{}{}{}",
        varint(2),
        pack_frame("minecraft", "vanilla", "26.2"),
        pack_frame("custom", "example", "v1")
    );
    let decodes = [
        ("clientbound-basic", "clientbound", basic_frame),
        ("clientbound-trailing", "clientbound", "00aa".to_owned()),
        (
            "serverbound-basic",
            "serverbound",
            varint(1) + &pack_frame("minecraft", "vanilla", "26.2"),
        ),
        (
            "serverbound-64",
            "serverbound",
            varint(64) + &"000000".repeat(64),
        ),
        ("serverbound-65", "serverbound", "41".to_owned()),
        ("count-truncated", "serverbound", "80".to_owned()),
        ("negative-count", "serverbound", "ffffffff0f".to_owned()),
        (
            "member-malformed-body",
            "serverbound",
            "0101ff0000".to_owned(),
        ),
        ("member-truncated", "serverbound", "0200000000".to_owned()),
    ];
    cases.extend(decodes.iter().map(|(label, packet, framed)| {
        list_codec("decode", label, packet, json!({ "hex": framed }))
    }));
    cases
}

pub fn cases() -> Vec<Value> {
    let mut cases = vec![known_pack()];
    cases.extend(codec_cases());
    cases.extend(list_cases());
    cases
}
