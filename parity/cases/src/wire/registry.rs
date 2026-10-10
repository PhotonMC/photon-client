//! The registry data packet, with entries carrying network NBT.

use serde_json::{Value, json};

use super::codec::{string_frame, varint_hex};
use crate::bits::hex;
use crate::case::case;

const REGISTRY: &str = "minecraft:dimension_type";
const OVERWORLD: &str = "minecraft:overworld";

const TYPE_IDS: &[&str] = &[
    "end",
    "byte",
    "short",
    "int",
    "long",
    "float",
    "double",
    "byte-array",
    "string",
    "list",
    "compound",
    "int-array",
    "long-array",
];

fn type_id(node: &Value) -> usize {
    let name = node["type"].as_str().unwrap_or_default();
    TYPE_IDS.iter().position(|known| *known == name).expect("a known tag type")
}

fn number(value: &Value) -> i64 {
    value.as_str().and_then(|text| text.parse().ok()).expect("an integer tag value")
}

fn items(node: &Value) -> &[Value] {
    node["value"].as_array().map_or(&[], Vec::as_slice)
}

fn array(node: &Value, width: usize) -> String {
    let values = items(node);
    let body: String = values.iter().map(|value| hex(&number(value).to_be_bytes()[8 - width..])).collect();
    format!("{}{body}", hex(&(values.len() as i32).to_be_bytes()))
}

/// The byte width of an integer tag, or of an integer array's elements.
fn width(name: &str) -> Option<usize> {
    match name {
        "byte" | "byte-array" => Some(1),
        "short" => Some(2),
        "int" | "int-array" => Some(4),
        "long" | "long-array" => Some(8),
        _ => None,
    }
}

/// The payload of `node`, without its type byte.
fn payload(node: &Value) -> String {
    let name = TYPE_IDS[type_id(node)];
    match (name, width(name)) {
        (_, Some(width)) if name.ends_with("-array") => array(node, width),
        (_, Some(width)) => hex(&number(&node["value"]).to_be_bytes()[8 - width..]),
        ("string", _) => {
            let text = node["value"].as_str().unwrap_or_default();
            format!("{}{}", hex(&(text.len() as u16).to_be_bytes()), hex(text.as_bytes()))
        }
        ("list", _) => list(node),
        ("compound", _) => compound(node),
        // Floats and doubles are given as their bit patterns.
        _ => node["value"].as_str().unwrap_or_default().to_owned(),
    }
}

fn list(node: &Value) -> String {
    let values = items(node);
    let element = values.first().map_or(0, type_id);
    let body: String = values.iter().map(payload).collect();
    format!("{element:02x}{}{body}", hex(&(values.len() as i32).to_be_bytes()))
}

fn compound(node: &Value) -> String {
    let mut out = String::new();
    for (key, value) in node["value"].as_object().into_iter().flatten() {
        out.push_str(&format!("{:02x}", type_id(value)));
        out.push_str(&hex(&(key.len() as u16).to_be_bytes()));
        out.push_str(&hex(key.as_bytes()));
        out.push_str(&payload(value));
    }
    out + "00"
}

/// `node` as network NBT: its type byte, then its payload.
fn tree_hex(node: &Value) -> String {
    format!("{:02x}{}", type_id(node), payload(node))
}

fn entry(id: &str, data: Value) -> Value {
    json!({"id": id, "data": data})
}

fn frame(registry: &str, entries: &[Value]) -> String {
    let mut out = format!("{}{}", string_frame(registry), varint_hex(entries.len() as i64));
    for entry in entries {
        out.push_str(&string_frame(entry["id"].as_str().unwrap_or_default()));
        match &entry["data"] {
            Value::Null => out.push_str("00"),
            data => out.push_str(&format!("01{}", tree_hex(data))),
        }
    }
    out
}

fn entry_sets() -> Vec<(&'static str, Vec<Value>)> {
    let compound = json!({"type": "compound", "value": {"name": {"type": "string", "value": "overworld"}}});
    let list = json!({"type": "list", "value": [{"type": "int", "value": "1"}, {"type": "int", "value": "-2"}]});
    vec![
        ("empty", vec![]),
        ("absent-data", vec![entry(OVERWORLD, Value::Null)]),
        ("scalar-data", vec![entry(OVERWORLD, json!({"type": "int", "value": "42"}))]),
        ("compound-data", vec![entry(OVERWORLD, compound.clone())]),
        ("list-data", vec![entry(OVERWORLD, list)]),
        ("multiple", vec![entry(OVERWORLD, Value::Null), entry("minecraft:the_nether", compound)]),
    ]
}

fn writes() -> Vec<Value> {
    let mut sets = entry_sets();
    let mixed = vec![
        entry("custom:one", json!({"type": "byte", "value": "-128"})),
        entry("custom:two", json!({"type": "string", "value": "hi"})),
        entry("custom:three", Value::Null),
    ];
    sets.push(("mixed-tags", mixed));
    sets.into_iter()
        .map(|(label, entries)| {
            let prefix = match label {
                "empty" => "deadbeef",
                "multiple" => "aa55",
                _ => "",
            };
            let fields = json!({
                "direction": "write", "registry": REGISTRY, "entries": entries, "prefix_hex": prefix,
            });
            case(format!("configuration/registry-data/write/{label}"), "wire", "registry_data", fields)
        })
        .collect()
}

fn reads() -> Vec<(&'static str, String)> {
    let mut reads: Vec<(&str, String)> =
        entry_sets().into_iter().map(|(label, entries)| (label, frame(REGISTRY, &entries))).collect();
    let one = format!("{}{}{}", string_frame(REGISTRY), varint_hex(1), string_frame(OVERWORLD));
    let head = format!("{}{}", string_frame(REGISTRY), varint_hex(1));
    reads.extend([
        ("trailing", format!("{}aabb", frame(REGISTRY, &[]))),
        ("noncanonical-present", format!("{one}02{}", tree_hex(&json!({"type": "int", "value": "42"})))),
        ("registry-prefix-underflow", "80".to_owned()),
        ("registry-body-underflow", "01".to_owned()),
        ("entry-count-underflow", string_frame(REGISTRY)),
        ("entry-id-prefix-underflow", format!("{head}80")),
        ("optional-underflow", one.clone()),
        ("tag-empty", format!("{one}01")),
        ("tag-end", format!("{one}0100")),
        ("tag-body-underflow", format!("{one}0103")),
        ("invalid-registry-identifier", format!("{}{}", string_frame("Bad:registry"), varint_hex(0))),
        ("invalid-entry-identifier", format!("{head}{}00", string_frame("Bad:entry"))),
        ("negative-entry-count", format!("{}{}", string_frame(REGISTRY), varint_hex(-1))),
        ("entry-count-too-big", format!("{}808080808001", string_frame(REGISTRY))),
    ]);
    reads
}

pub fn cases() -> Vec<Value> {
    let mut cases = writes();
    cases.extend(reads().into_iter().map(|(label, hex)| {
        let fields = json!({"direction": "read", "hex": hex});
        case(format!("configuration/registry-data/read/{label}"), "wire", "registry_data", fields)
    }));
    cases
}
