//! Cases for `client-world/light-update-queue-and-publication`, `client-world/light-unload-removal`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::client_world::sections::light_layer_hex;

fn light_probe(label: &str, layer: &str, x: i64, y: i64) -> Value {
    json!({"label": label, "layer": layer, "x": x, "y": y, "z": 48})
}

/// The sky and block probes at the first cell of the test chunk.
fn first_cell(sky: &str, block: &str) -> Vec<Value> {
    vec![light_probe(sky, "sky", -32, -64), light_probe(block, "block", -32, -64)]
}

fn apply_light(label: &str, empty_mask: &[i64], sky: u8, block: u8, queries: Vec<Value>) -> Value {
    json!({
        "action": "apply",
        "label": label,
        "chunk_x": -2,
        "chunk_z": 3,
        "sky_mask": [1],
        "sky_empty_mask": empty_mask,
        "sky_updates": [light_layer_hex(sky)],
        "block_mask": [1],
        "block_empty_mask": empty_mask,
        "block_updates": [light_layer_hex(block)],
        "queries": queries,
    })
}

fn load_chunk(label: &str, queries: Vec<Value>) -> Value {
    json!({
        "action": "load",
        "label": label,
        "chunk_x": -2,
        "chunk_z": 3,
        "section_count": 1,
        "non_empty_sections": [0],
        "queries": queries,
    })
}

fn unload_chunk(label: &str, queries: Vec<Value>) -> Value {
    json!({"action": "unload", "label": label, "chunk_x": -2, "chunk_z": 3, "queries": queries})
}

fn publication_probes() -> Vec<Value> {
    vec![
        light_probe("sky-low", "sky", -32, -64),
        light_probe("sky-high-nibble", "sky", -31, -64),
        light_probe("block-low", "block", -32, -64),
        light_probe("block-high-nibble", "block", -31, -64),
        light_probe("empty-sky-section", "sky", -32, -48),
        light_probe("empty-block-section", "block", -32, -48),
    ]
}

fn replaced_probes() -> Vec<Value> {
    vec![
        light_probe("updated-sky-low", "sky", -32, -64),
        light_probe("updated-sky-high-nibble", "sky", -31, -64),
        light_probe("updated-block-low", "block", -32, -64),
        light_probe("updated-block-high-nibble", "block", -31, -64),
    ]
}

fn queue_and_publication() -> Value {
    case(
        "client-world/light-update-queue-and-publication",
        "math",
        "client_world_light",
        json!({
            "dimension_min_y": -64,
            "dimension_height": 384,
            "operations": [
                apply_light("queue-first-unloaded", &[2], 165, 182, vec![]),
                apply_light("queue-second-replaces-first", &[2], 60, 77, vec![]),
                load_chunk("load-and-publish", publication_probes()),
                apply_light("replace-loaded-layer", &[], 46, 127, replaced_probes()),
            ],
        }),
    )
}

fn unload_removal() -> Value {
    case(
        "client-world/light-unload-removal",
        "math",
        "client_world_light",
        json!({
            "dimension_min_y": -64,
            "dimension_height": 384,
            "operations": [
                apply_light("queue-before-unload", &[], 15, 166, vec![]),
                unload_chunk(
                    "forget-clears-queued-light",
                    first_cell("sky-after-forget", "block-after-forget"),
                ),
                load_chunk(
                    "load-after-queued-cleanup",
                    first_cell("sky-not-republished", "block-not-republished"),
                ),
                apply_light(
                    "loaded-light-before-unload",
                    &[],
                    38,
                    55,
                    first_cell("sky-loaded", "block-loaded"),
                ),
                unload_chunk(
                    "forget-clears-loaded-light",
                    first_cell("sky-after-loaded-forget", "block-after-loaded-forget"),
                ),
            ],
        }),
    )
}

pub fn cases() -> Vec<Value> {
    vec![queue_and_publication(), unload_removal()]
}
