//! Cases for `client-world/section-read-and-coordinate-lookup`, `client-world/chunk-cache-lifecycle`, `client-world/block-update-section-counters`.

use serde_json::{Value, json};

use crate::case::case;
use crate::math::client_world::sections::*;

fn air_section() -> String {
    section_hex((0, 0), (0, vec![0], vec![0; 4096]), (0, vec![1], vec![0; 64]))
}

fn linear_section() -> String {
    section_hex(
        (4, 0),
        (4, vec![0, 1, 2, 3], sparse_values(4096, &[(0, 1), (1, 2), (17, 3), (255, 1)])),
        (2, vec![0, 1, 2, 3], sparse_values(64, &[(0, 1), (3, 2), (15, 3), (63, 1)])),
    )
}

fn hash_map_section() -> String {
    section_hex(
        (4, 0),
        (5, vec![0, 1, 2, 3, 4], sparse_values(4096, &[(10, 1), (511, 2), (1023, 3), (4095, 4)])),
        (3, vec![0, 1, 2, 3, 4], sparse_values(64, &[(0, 1), (15, 2), (16, 3), (63, 4)])),
    )
}

fn global_section() -> String {
    section_hex(
        (6, 0),
        (15, vec![], sparse_values(4096, &[(6, 1), (7, 2), (8, 3), (63, 1), (64, 2), (4095, 4)])),
        (7, vec![], sparse_values(64, &[(3, 1), (4, 2), (15, 3), (16, 4), (63, 4)])),
    )
}

/// A section whose first block cell holds a solid state.
fn solid_section() -> String {
    let mut values = vec![0; 4096];
    values[0] = 1;
    section_hex((1, 0), (1, vec![0, 1], values), (0, vec![1], vec![0; 64]))
}

/// A section whose last block cell holds a state with fluid.
fn fluid_section() -> String {
    let mut values = vec![0; 4096];
    values[4095] = 1;
    section_hex((1, 1), (1, vec![0, 1], values), (0, vec![1], vec![0; 64]))
}

fn probe(label: &str, x: i64, y: i64, z: i64) -> Value {
    json!({"label": label, "x": x, "y": y, "z": z})
}

fn section_lookup() -> Value {
    case(
        "client-world/section-read-and-coordinate-lookup",
        "math",
        "client_world_section_lookup",
        json!({
            "dimension_min_y": -64,
            "dimension_height": 384,
            "tracks": [{
                "label": "negative-chunk-and-overworld-floor",
                "chunk_x": -2,
                "chunk_z": 3,
                "sections": [air_section(), linear_section(), hash_map_section(), global_section()],
                "queries": lookup_queries(),
            }],
        }),
    )
}

fn lookup_queries() -> Vec<Value> {
    vec![
        probe("air-at-world-floor", -32, -64, 48),
        probe("linear-first-entry", -32, -48, 48),
        probe("linear-local-boundary", -17, -47, 63),
        probe("hash-map-entry", -22, -32, 48),
        probe("hash-map-last-row", -17, -29, 63),
        probe("global-cross-word-start", -17, -16, 51),
        probe("global-cross-word-next", -32, -16, 52),
        probe("global-last-entry", -17, -1, 63),
        probe("outside-chunk", -16, -48, 48),
        probe("outside-loaded-sections", -32, 0, 48),
    ]
}

fn cells(points: &[(i64, i64)]) -> Vec<Value> {
    points.iter().map(|(x, z)| json!({"x": x, "z": z})).collect()
}

fn replace_chunk(chunk_x: i64, chunk_z: i64, sections: &[&String], checks: &[(i64, i64)]) -> Value {
    json!({
        "action": "replace",
        "chunk_x": chunk_x,
        "chunk_z": chunk_z,
        "sections": sections,
        "checks": cells(checks),
    })
}

fn drop_chunk(chunk_x: i64, chunk_z: i64) -> Value {
    json!({"action": "drop", "chunk_x": chunk_x, "chunk_z": chunk_z, "checks": []})
}

fn flip() -> Value {
    json!({"action": "flip", "checks": []})
}

fn center(chunk_x: i64, chunk_z: i64, checks: &[(i64, i64)]) -> Value {
    json!({"action": "center", "chunk_x": chunk_x, "chunk_z": chunk_z, "checks": cells(checks)})
}

fn radius(server_view_radius: i64, checks: &[(i64, i64)]) -> Value {
    json!({"action": "radius", "server_view_radius": server_view_radius, "checks": cells(checks)})
}

fn section_emptiness(section_x: i64, section_y: i64, section_z: i64, empty: bool) -> Value {
    json!({
        "action": "section_emptiness",
        "section_x": section_x,
        "section_y": section_y,
        "section_z": section_z,
        "empty": empty,
        "checks": [],
    })
}

fn cache_operations() -> Vec<Value> {
    let (air, solid, fluid) = (air_section(), solid_section(), fluid_section());
    vec![
        replace_chunk(0, 0, &[&air, &solid], &[(-5, -5), (-6, 0), (5, 5)]),
        replace_chunk(0, 0, &[&solid, &air], &[(0, 0)]),
        flip(),
        center(8, 0, &[(8, 0), (2, 0), (14, 0)]),
        replace_chunk(11, 0, &[&air, &fluid], &[(11, 0), (0, 0)]),
        drop_chunk(0, 0),
        drop_chunk(11, 0),
        replace_chunk(8, 1, &[&air], &[(8, 1)]),
        radius(1, &[]),
        radius(6, &[(-1, 0), (17, 1), (18, 1)]),
        section_emptiness(8, -4, 1, false),
        section_emptiness(99, 0, 99, true),
        flip(),
        drop_chunk(18, 1),
    ]
}

fn chunk_cache_lifecycle() -> Value {
    case(
        "client-world/chunk-cache-lifecycle",
        "math",
        "client_world_chunk_cache",
        json!({
            "dimension_min_y": -64,
            "server_view_radius": 0,
            "operations": cache_operations(),
        }),
    )
}

fn block_update(label: &str, at: (i64, i64, i64), state_id: i64, queries: Vec<Value>) -> Value {
    json!({
        "label": label,
        "x": at.0,
        "y": at.1,
        "z": at.2,
        "state_id": state_id,
        "queries": queries,
    })
}

fn update_probe(label: &str, at: (i64, i64, i64)) -> Vec<Value> {
    vec![probe(label, at.0, at.1, at.2)]
}

fn block_updates() -> Vec<Value> {
    const FLOOR: (i64, i64, i64) = (-32, -64, 48);
    const EDGE: (i64, i64, i64) = (-17, -49, 63);
    let both = vec![probe("updated-cell", -32, -64, 48), probe("section-edge", -17, -49, 63)];
    vec![
        block_update("same-air-empty-rejected", FLOOR, 0, update_probe("unchanged-cell", FLOOR)),
        block_update("air-to-stone", FLOOR, 1, both),
        block_update("stone-to-water", FLOOR, 86, update_probe("updated-cell", FLOOR)),
        block_update("water-to-air", FLOOR, 0, update_probe("updated-cell", FLOOR)),
        block_update("air-to-stone-at-negative-edge", EDGE, 1, update_probe("updated-cell", EDGE)),
        block_update("same-state-stone", EDGE, 1, update_probe("updated-cell", EDGE)),
        block_update("missing-section-rejected", (-32, -48, 48), 1, update_probe("missing-section", (-32, -48, 48))),
        block_update("outside-chunk-rejected", (-16, -64, 48), 1, update_probe("outside-chunk", (-16, -64, 48))),
    ]
}

fn block_update_section_counters() -> Value {
    let section = section_hex((0, 0), (1, vec![0, 1], vec![0; 4096]), (0, vec![1], vec![0; 64]));
    case(
        "client-world/block-update-section-counters",
        "math",
        "client_world_block_update",
        json!({
            "dimension_min_y": -64,
            "dimension_height": 384,
            "chunk_x": -2,
            "chunk_z": 3,
            "sections": [section],
            "updates": block_updates(),
        }),
    )
}

pub fn cases() -> Vec<Value> {
    vec![section_lookup(), chunk_cache_lifecycle(), block_update_section_counters()]
}
