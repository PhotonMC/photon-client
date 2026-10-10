//! The `model` family: block models, blockstates and the geometry baked
//! from them.

mod geometry;
mod slots;
mod states;
mod texts;

use serde_json::{Value, json};

use crate::bits::hex;
use crate::case::case;

const BLOCKSTATES: &[(&str, &str)] = &[
    // The plain shape: one key, one model.
    (
        "single",
        r#"{"variants": {"": {"model": "minecraft:block/stone"}}}"#,
    ),
    (
        "property-key",
        r#"{"variants": {"facing=north": {"model": "minecraft:block/furnace"}}}"#,
    ),
    (
        "rotation",
        r#"{"variants": {"": {"model": "minecraft:block/stairs", "x": 90, "y": 180, "z": 270, "uvlock": true}}}"#,
    ),
    // Angles are taken modulo a full turn before the quadrant is chosen.
    (
        "rotation-wrapped",
        r#"{"variants": {"": {"model": "minecraft:block/a", "x": 450, "y": -90}}}"#,
    ),
    (
        "rotation-bad",
        r#"{"variants": {"": {"model": "minecraft:block/a", "x": 45}}}"#,
    ),
    // A list is a weighted choice; the weight defaults to one.
    (
        "weighted",
        r#"{"variants": {"": [{"model": "minecraft:block/a"}, {"model": "minecraft:block/b", "weight": 3}]}}"#,
    ),
    (
        "weighted-zero",
        r#"{"variants": {"": [{"model": "minecraft:block/a", "weight": 0}]}}"#,
    ),
    (
        "weighted-negative",
        r#"{"variants": {"": [{"model": "minecraft:block/a", "weight": -1}]}}"#,
    ),
    ("weighted-empty", r#"{"variants": {"": []}}"#),
    (
        "several-keys",
        r#"{"variants": {"half=bottom": {"model": "minecraft:block/a"}, "half=top": {"model": "minecraft:block/b"}}}"#,
    ),
    // A model name is required and must be a valid identifier.
    ("no-model", r#"{"variants": {"": {}}}"#),
    ("bad-model", r#"{"variants": {"": {"model": "Not An Id"}}}"#),
    ("empty-variants", r#"{"variants": {}}"#),
    ("neither", "{}"),
    (
        "both",
        r#"{"variants": {"": {"model": "minecraft:block/a"}}, "multipart": [{"apply": {"model": "minecraft:block/b"}}]}"#,
    ),
];

/// Multipart: a condition is optional, and its two forms nest.
const MULTIPARTS: &[(&str, &str)] = &[
    (
        "multipart-plain",
        r#"{"multipart": [{"apply": {"model": "minecraft:block/a"}}]}"#,
    ),
    ("multipart-when", r#"{"north": "true"}"#),
    ("multipart-multi-term", r#"{"facing": "north|south"}"#),
    ("multipart-negated", r#"{"facing": "!north"}"#),
    (
        "multipart-or",
        r#"{"OR": [{"north": "true"}, {"south": "true"}]}"#,
    ),
    (
        "multipart-and",
        r#"{"AND": [{"north": "true"}, {"south": "true"}]}"#,
    ),
    (
        "multipart-nested",
        r#"{"OR": [{"north": "true"}, {"AND": [{"south": "true"}, {"east": "false"}]}]}"#,
    ),
    // A number or a boolean in a term is taken as its text.
    ("multipart-boolean", r#"{"north": true}"#),
    ("multipart-number", r#"{"level": 3}"#),
    ("multipart-float", r#"{"level": 3.5}"#),
    // Empty terms and empty maps are refused.
    ("multipart-empty-term", r#"{"facing": ""}"#),
    ("multipart-empty-piece", r#"{"facing": "north|"}"#),
    ("multipart-empty-when", "{}"),
    ("multipart-empty", r#"{"multipart": []}"#),
    (
        "multipart-no-apply",
        r#"{"multipart": [{"when": {"north": "true"}}]}"#,
    ),
    // A combiner holds exactly one operation.
    (
        "multipart-two-operations",
        r#"{"OR": [{"north": "true"}], "AND": [{"south": "true"}]}"#,
    ),
    (
        "multipart-lowercase-operation",
        r#"{"or": [{"north": "true"}]}"#,
    ),
    // An operation alongside a property: does the combiner take the one key
    // it knows and drop the rest, or refuse the whole condition?
    (
        "multipart-operation-and-property",
        r#"{"OR": [{"north": "true"}], "north": "true"}"#,
    ),
    ("multipart-empty-operation-list", r#"{"AND": []}"#),
    // The selector string is not validated while parsing, only later when
    // it meets a state definition.
    (
        "nonsense-selector",
        r#"{"variants": {"=,,=nope": {"model": "minecraft:block/a"}}}"#,
    ),
    (
        "multipart-weighted",
        r#"{"multipart": [{"apply": [{"model": "minecraft:block/a"}, {"model": "minecraft:block/b", "weight": 2}]}]}"#,
    ),
    ("not-an-object", "[]"),
];

/// A whole blockstate text, or a multipart `when` condition wrapped into one.
fn blockstate_text(text: &str) -> String {
    if text.starts_with(r#"{"multipart""#) || text.starts_with(r#"{"variants""#) || text == "[]" {
        return text.to_owned();
    }
    format!(r#"{{"multipart": [{{"when": {text}, "apply": {{"model": "minecraft:block/a"}}}}]}}"#)
}

fn blockstates() -> Vec<Value> {
    let mut rows: Vec<Value> = BLOCKSTATES
        .iter()
        .map(|(label, text)| json!({"label": label, "json": text}))
        .collect();
    rows.extend(
        MULTIPARTS
            .iter()
            .map(|(label, text)| json!({"label": label, "json": blockstate_text(text)})),
    );
    rows
}

fn parse_model() -> Vec<Value> {
    texts::accepted_models()
        .into_iter()
        .chain(texts::rejected_models())
        .map(|(label, text)| json!({"label": label, "text": hex(text.as_bytes())}))
        .collect()
}

fn model(label: &str, operation: &str, rows: Vec<Value>) -> Value {
    case(
        format!("model/{label}"),
        "model",
        operation,
        json!({ "rows": rows }),
    )
}

pub fn cases() -> Vec<Value> {
    vec![
        model(
            "nearest-direction",
            "nearest_direction",
            geometry::nearest_direction(),
        ),
        model("cull-rotation", "cull_rotation", states::cull_rotation()),
        model("uv-lock", "uv_lock", states::uv_lock()),
        model("blockstate-parse", "blockstate_parse", blockstates()),
        model("model-state", "model_state", states::model_state()),
        model("parse-model", "parse_model", parse_model()),
        model(
            "face-uv-rotation",
            "face_uv_rotation",
            geometry::face_uv_rotation(),
        ),
        model(
            "euler-rotation",
            "euler_rotation",
            geometry::euler_rotation(),
        ),
        model("quad-winding", "quad_winding", geometry::quad_winding()),
        model(
            "element-rotation",
            "element_rotation",
            geometry::element_rotation(),
        ),
        model("face-geometry", "face_geometry", geometry::face_geometry()),
        model("texture-slots", "texture_slots", slots::rows()),
    ]
}
