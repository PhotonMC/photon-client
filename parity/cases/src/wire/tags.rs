//! The update-tags packet.

use serde_json::{Value, json};

use super::codec::{string_frame, tags_frame, varint_hex};
use crate::case::case;

type Tag = (&'static str, &'static [i64]);

const BLOCK: &str = "minecraft:block";
const MULTIPLE: &[Tag] = &[
    ("minecraft:stone", &[1, 2]),
    ("custom:ore/rare", &[-1, 300]),
    ("minecraft:logs", &[]),
];
const REORDERED: &[Tag] = &[
    ("custom:ore/rare", &[-1, 300]),
    ("minecraft:logs", &[]),
    ("minecraft:stone", &[1, 2]),
];
const DUPLICATE: &[Tag] = &[("minecraft:stone", &[1]), ("minecraft:stone", &[2, 3])];

fn registry(tags: &[Tag]) -> Value {
    let tags: Vec<Value> = tags
        .iter()
        .map(|(id, values)| json!({"id": id, "values": values}))
        .collect();
    json!([{"registry": BLOCK, "tags": tags}])
}

fn writes() -> Vec<Value> {
    let writes = [
        ("empty", json!([]), "deadbeef"),
        ("registry-empty", registry(&[]), ""),
        ("multiple", registry(MULTIPLE), ""),
        ("reordered", registry(REORDERED), ""),
        ("duplicate-tag", registry(DUPLICATE), "aa55"),
    ];
    writes
        .into_iter()
        .map(|(label, registries, prefix)| {
            let fields =
                json!({"direction": "write", "registries": registries, "prefix_hex": prefix});
            case(
                format!("common/update-tags/write/{label}"),
                "wire",
                "update_tags",
                fields,
            )
        })
        .collect()
}

fn reads() -> Vec<(&'static str, String)> {
    let empty = tags_frame(BLOCK, &[]);
    let stone_head = format!(
        "{}{}{}{}",
        varint_hex(1),
        string_frame(BLOCK),
        varint_hex(1),
        string_frame("minecraft:stone")
    );
    vec![
        ("empty", varint_hex(0)),
        ("registry-empty", empty.clone()),
        ("multiple", tags_frame(BLOCK, MULTIPLE)),
        ("reordered", tags_frame(BLOCK, REORDERED)),
        ("duplicate-tag", tags_frame(BLOCK, DUPLICATE)),
        ("trailing", format!("{empty}aabb")),
        ("registry-prefix-underflow", varint_hex(1)),
        (
            "registry-body-underflow",
            format!("{}{}6d696e656372616674", varint_hex(1), varint_hex(14)),
        ),
        (
            "tag-count-underflow",
            format!("{}{}", varint_hex(1), string_frame(BLOCK)),
        ),
        (
            "tag-prefix-underflow",
            format!("{}01", &empty[..empty.len() - 2]),
        ),
        ("values-prefix-underflow", stone_head.clone()),
        (
            "negative-values-count",
            format!("{stone_head}{}", varint_hex(-1)),
        ),
        (
            "invalid-tag-identifier",
            tags_frame(BLOCK, &[("Bad:tag", &[])]),
        ),
        ("negative-count", varint_hex(-1)),
        ("count-too-big", "808080808001".to_owned()),
    ]
}

pub fn cases() -> Vec<Value> {
    let mut cases = writes();
    cases.extend(reads().into_iter().map(|(label, hex)| {
        let fields = json!({"direction": "read", "hex": hex});
        case(
            format!("common/update-tags/read/{label}"),
            "wire",
            "update_tags",
            fields,
        )
    }));
    cases
}
