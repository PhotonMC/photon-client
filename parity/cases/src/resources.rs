//! The `resources` family: asset indexes, pack stacks, reloads and the
//! known-pack metadata and codecs.

mod known_packs;

use serde_json::{Value, json};

use crate::case::case;

const HASH_A: &str = "00112233445566778899aabbccddeeff00112233";
const HASH_B: &str = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
const HASH_C: &str = "102030405060708090a0b0c0d0e0f00010203040";
const HASH_MISSING: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn resources(id: &str, operation: &str, fields: Value) -> Value {
    case(format!("resources/{id}"), "resources", operation, fields)
}

fn entries(list: &[(&str, &str)]) -> Vec<Value> {
    list.iter().map(|(path, hash)| json!({"path": path, "hash": hash})).collect()
}

fn objects(list: &[(&str, &str)]) -> Vec<Value> {
    list.iter().map(|(hash, hex)| json!({"hash": hash, "hex": hex})).collect()
}

/// A pack (or root) of `(resource id, hex contents)`.
fn pack(id: &str, list: &[(&str, &str)]) -> Value {
    let resources: Vec<Value> = list.iter().map(|(id, hex)| json!({"id": id, "hex": hex})).collect();
    json!({"id": id, "resources": resources})
}

fn index_cases() -> Vec<Value> {
    let basic = json!({
        "index": "32",
        "entries": entries(&[
            ("minecraft/textures/a.png", HASH_A),
            ("minecraft/lang/en_us.json", HASH_B),
            ("realms/icons/realm.png", HASH_C),
        ]),
        "objects": objects(&[(HASH_A, "89504e470d0a"), (HASH_B, "7b226c616e67223a7d"), (HASH_C, "5245414c4d53")]),
        "queries": [
            "minecraft/textures/a.png", "minecraft/lang/en_us.json",
            "realms/icons/realm.png", "minecraft/textures", "minecraft",
            "minecraft/textures/missing.png", "unknown",
        ],
    });
    let collision = json!({
        "index": "collision",
        "entries": entries(&[("foo", HASH_A), ("foo/bar", HASH_B), ("foo/bar/baz", HASH_C), ("foo/qux", HASH_MISSING)]),
        "objects": objects(&[(HASH_A, "41"), (HASH_B, "42"), (HASH_C, "4343")]),
        "queries": ["foo", "foo/bar", "foo/bar/baz", "foo/qux", "bar"],
    });
    let missing = json!({
        "index": "missing",
        "entries": entries(&[("minecraft/data/missing.bin", HASH_MISSING)]),
        "objects": [],
        "queries": ["minecraft/data/missing.bin", "minecraft/data", "minecraft/data/other.bin"],
    });
    let empty = json!({"index": "empty", "entries": [], "objects": [], "queries": ["minecraft", "minecraft/data"]});
    vec![
        resources("index/basic-mappings", "index", basic),
        resources("index/file-directory-collision", "index", collision),
        resources("index/missing-object", "index", missing),
        resources("index/empty", "index", empty),
    ]
}

fn pack_cases() -> Vec<Value> {
    let stack = json!({
        "packs": [
            pack("base", &[("minecraft:textures/shared.png", "42415345"), ("minecraft:textures/base.png", "42415345")]),
            pack("overlay", &[("minecraft:textures/shared.png", "4f564552"), ("minecraft:textures/overlay.png", "4f564552")]),
        ],
        "queries": [
            "minecraft:textures/shared.png", "minecraft:textures/base.png",
            "minecraft:textures/overlay.png", "minecraft:textures/missing.png",
        ],
    });
    let value = "minecraft:config/value.json";
    let layers = json!({
        "packs": [
            pack("low", &[(value, "4c4f57")]),
            pack("middle", &[(value, "4d4944")]),
            pack("high", &[(value, "48494748")]),
        ],
        "queries": [value],
    });
    vec![
        resources("packs/priority-and-stack", "packs", stack),
        resources("packs/three-layer-precedence", "packs", layers),
    ]
}

fn reload_cases() -> Vec<Value> {
    let shared = "minecraft:textures/shared.png";
    let queries = json!([shared, "minecraft:textures/legacy.png", "custom:config/value.json", "other:value.json"]);
    let replaces = json!({
        "initial_queries": [shared, "other:value.json"],
        "rounds": [
            {
                "packs": [
                    pack("base-r1", &[(shared, "4f4c44"), ("minecraft:textures/legacy.png", "4c4547414359")]),
                    pack("overlay-r1", &[(shared, "4f5631"), ("custom:config/value.json", "435553544f4d")]),
                ],
                "queries": queries,
            },
            {
                "packs": [pack("replacement-r2", &[(shared, "4e4557"), ("other:value.json", "5245504c41434544")])],
                "queries": queries,
            },
        ],
    });
    let value = "minecraft:config/value.json";
    let empty_round = json!({"packs": [], "queries": [value]});
    let readd = json!({
        "initial_queries": [value],
        "rounds": [
            empty_round,
            {"packs": [pack("fresh", &[(value, "4652455348")])], "queries": [value]},
            empty_round,
        ],
    });
    vec![
        resources("reload/replaces-active-stack", "reload", replaces),
        resources("reload/empty-remove-readd", "reload", readd),
    ]
}

fn compatibility() -> Value {
    let probes: Vec<Value> = [
        (2147483647, 0, 2147483647, 0),
        (0, 0, 87, 0),
        (87, 5, 87, 99),
        (87, 0, 88, 0),
        (88, 0, 88, 0),
        (88, 1, 88, 2),
        (89, 0, 89, 0),
        (87, 99, 88, 0),
    ]
    .iter()
    .map(|(min_major, min_minor, max_major, max_minor)| {
        json!({"min_major": min_major, "min_minor": min_minor, "max_major": max_major, "max_minor": max_minor})
    })
    .collect();
    resources("metadata/compatibility-boundaries", "compatibility", json!({ "probes": probes }))
}

fn vanilla_cases() -> Vec<Value> {
    let shared = "minecraft:textures/shared.png";
    let external = json!({
        "roots": [
            pack("jar", &[(shared, "4a4152"), ("minecraft:textures/jar-only.png", "4a4152")]),
            pack("external", &[(shared, "455854"), ("minecraft:textures/external-only.png", "455854")]),
        ],
        "namespaces": ["minecraft"],
        "queries": [
            shared, "minecraft:textures/jar-only.png",
            "minecraft:textures/external-only.png", "minecraft:textures/missing.png",
        ],
    });
    let explicit = json!({
        "roots": [pack("assets", &[
            ("minecraft:lang/en_us.json", "4d494e"),
            ("realms:icons/realm.png", "5245414c4d53"),
            ("custom:data/value.json", "435553544f4d"),
        ])],
        "namespaces": ["realms", "minecraft"],
        "queries": [
            "minecraft:lang/en_us.json", "realms:icons/realm.png",
            "custom:data/value.json", "unknown:data/value.json",
        ],
    });
    vec![
        resources("vanilla/external-root-precedence", "vanilla", external),
        resources("vanilla/explicit-namespaces", "vanilla", explicit),
    ]
}

pub fn cases() -> Vec<Value> {
    let mut cases = [index_cases(), pack_cases(), reload_cases()].concat();
    cases.push(compatibility());
    cases.extend(vanilla_cases());
    cases.extend(known_packs::cases());
    cases
}
