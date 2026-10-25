//! Cases for `client-world/entity-shared-flags`, `client-world/entity-air-supply`, `client-world/entity-pose`, `client-world/entity-ticks-frozen` and more.

use serde_json::{Value, json};

use crate::case::case;

fn entity_shared_flags() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-shared-flags",
        "math",
        "client_world_entity_shared_flags",
        json!({
            "entity_id": 37,
            "entity_type_id": 120,
            "operations": [
                {"label": "all-defined-flags", "metadata_hex": "0000fbff"},
                {"label": "air-supply-only-preserves-flags", "metadata_hex": "010107ff"},
                {"label": "clear-shared-flags", "metadata_hex": "000000ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-air-supply",
        "math",
        "client_world_entity_air_supply",
        json!({
            "entity_id": 37,
            "entity_type_id": 120,
            "operations": [
                {"label": "set-positive-air-supply", "metadata_hex": "0101fa01ff"},
                {"label": "combined-signed-air-supply", "metadata_hex": "0000400101ffffffff0fff"},
                {"label": "flags-only-preserves-air-supply", "metadata_hex": "000000ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-pose",
        "math",
        "client_world_entity_pose",
        json!({
            "entity_id": 37,
            "entity_type_id": 120,
            "operations": [
                {"label": "set-crouching", "metadata_hex": "061405ff"},
                {"label": "set-sitting-with-glowing", "metadata_hex": "00004006140aff"},
                {"label": "unknown-pose-falls-back", "metadata_hex": "061463ff"},
            ],
        }),
    ));
    out
}

fn entity_ticks_frozen() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-ticks-frozen",
        "math",
        "client_world_entity_ticks_frozen",
        json!({
            "entity_id": 37,
            "entity_type_id": 120,
            "operations": [
                {"label": "set-positive-freeze", "metadata_hex": "070146ff"},
                {"label": "set-full-freeze", "metadata_hex": "07018c01ff"},
                {"label": "combined-shared-flags-and-negative-freeze", "metadata_hex": "0000400701ffffffff0fff"},
                {"label": "clear-freeze", "metadata_hex": "070100ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-custom-name",
        "math",
        "client_world_entity_custom_name",
        json!({
            "entity_id": 37,
            "entity_type_id": 120,
            "operations": [
                {"label": "set-custom-name", "metadata_hex": "02060108000c446973706c6179206e616d65ff"},
                {"label": "set-visible-only", "metadata_hex": "030801ff"},
                {"label": "clear-custom-name", "metadata_hex": "020600ff"},
                {
                    "label": "restore-hidden-custom-name",
                    "metadata_hex": "02060108000c446973706c6179206e616d65030800ff",
                },
            ],
        }),
    ));
    out
}

fn entity_living_flags() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-living-flags",
        "math",
        "client_world_entity_living_flags",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-all-living-flags", "metadata_hex": "080007ff"},
                {"label": "combined-shared-air-and-living", "metadata_hex": "000040010107080001ff"},
                {"label": "clear-living-flags", "metadata_hex": "080000ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-health",
        "math",
        "client_world_entity_health",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-health-after-living-flags", "metadata_hex": "080000090340f00000ff"},
                {"label": "set-zero-health", "metadata_hex": "090300000000ff"},
                {"label": "no-gravity-preserves-health", "metadata_hex": "050801ff"},
                {"label": "set-negative-health", "metadata_hex": "0903bf800000ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-mob-flags",
        "math",
        "client_world_entity_mob_flags",
        json!({
            "entity_id": 37,
            "entity_type_id": 70,
            "operations": [
                {"label": "set-all-mob-flags", "metadata_hex": "0f0007ff"},
                {"label": "combined-shared-air-and-mob", "metadata_hex": "0000400101070f0002ff"},
                {"label": "no-gravity-preserves-mob", "metadata_hex": "050801ff"},
                {"label": "clear-mob-flags", "metadata_hex": "0f0000ff"},
            ],
        }),
    ));
    out
}

fn entity_arrow_count() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-arrow-count",
        "math",
        "client_world_entity_arrow_count",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-positive-arrow-count", "metadata_hex": "0c0107ff"},
                {"label": "combined-living-and-negative-arrow-count", "metadata_hex": "0800010c01ffffffff0fff"},
                {"label": "no-gravity-preserves-arrow-count", "metadata_hex": "050801ff"},
                {"label": "clear-arrow-count", "metadata_hex": "0c0100ff"},
            ],
        }),
    ));
    out.push(case(
        "client-world/entity-effect-ambience",
        "math",
        "client_world_entity_effect_ambience",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-effect-ambience", "metadata_hex": "0b0801ff"},
                {"label": "combined-living-and-effect-ambience", "metadata_hex": "0800010b0801ff"},
                {"label": "empty-particles-preserves-effect-ambience", "metadata_hex": "0a11000b0801ff"},
                {"label": "no-gravity-preserves-effect-ambience", "metadata_hex": "050801ff"},
                {"label": "clear-effect-ambience", "metadata_hex": "0b0800ff"},
            ],
        }),
    ));
    out
}

fn entity_effect_particles() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-effect-particles",
        "math",
        "client_world_entity_effect_particles",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-one-effect-particle", "metadata_hex": "0a11011c11223344ff"},
                {
                    "label": "set-two-effect-particles-with-living-flag",
                    "metadata_hex": "0800010a11021c112233441c55667788ff",
                },
                {"label": "no-gravity-preserves-effect-particles", "metadata_hex": "050801ff"},
                // An effect's own simple particle (oozing's `item_slime`, id 59) beside a swirl.
                {"label": "swirl-and-simple-particle", "metadata_hex": "0a11021c112233443bff"},
                {"label": "clear-effect-particles", "metadata_hex": "0a1100ff"},
            ],
        }),
    ));
    // `tickEffects` with a seeded source: how often, which particle and where in the box.
    let spawns = |label: &str, particles: Value, (ambient, invisible): (bool, bool), seed: i64| {
        case(
            format!("client-world/entity-effect-particle-spawns-{label}"),
            "math",
            "client_world_entity_effect_particles",
            json!({"spawn": {"particles": particles, "ambient": ambient, "invisible": invisible,
                "seed": seed, "ticks": 200}}),
        )
    };
    let swirl = json!([{"type_id": 28, "color": 0x7F33_EBFF_u32 as i32}]);
    let mixed = json!([{"type_id": 28, "color": -1}, {"type_id": 59}, {"type_id": 28, "color": 0x2603_0201}]);
    out.push(spawns("swirl", swirl.clone(), (false, false), 7));
    out.push(spawns("ambient", swirl.clone(), (true, false), 11));
    out.push(spawns("invisible", swirl.clone(), (false, true), 13));
    out.push(spawns("ambient-invisible", swirl, (true, true), 17));
    out.push(spawns("mixed", mixed, (false, false), 19));
    out.push(spawns("none", json!([]), (false, false), 23));
    out.push(case(
        "client-world/entity-sleeping-position",
        "math",
        "client_world_entity_sleeping_position",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-positive-sleeping-position", "metadata_hex": "0e0b01000003bffffe905aff"},
                {
                    "label": "combined-living-and-negative-sleeping-position",
                    "metadata_hex": "0800010e0b01ffff0000007fffc0ff",
                },
                {"label": "no-gravity-preserves-sleeping-position", "metadata_hex": "050801ff"},
                {"label": "clear-sleeping-position", "metadata_hex": "0e0b00ff"},
            ],
        }),
    ));
    out
}

fn entity_stinger_count() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "client-world/entity-stinger-count",
        "math",
        "client_world_entity_stinger_count",
        json!({
            "entity_id": 37,
            "entity_type_id": 54,
            "operations": [
                {"label": "set-positive-stinger-count", "metadata_hex": "0d0102ff"},
                {"label": "combined-living-and-negative-stinger-count", "metadata_hex": "0800010d01ffffffff0fff"},
                {"label": "no-gravity-preserves-stinger-count", "metadata_hex": "050801ff"},
                {"label": "clear-stinger-count", "metadata_hex": "0d0100ff"},
            ],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        entity_shared_flags(),
        entity_ticks_frozen(),
        entity_living_flags(),
        entity_arrow_count(),
        entity_effect_particles(),
        entity_stinger_count(),
    ]
    .concat()
}
