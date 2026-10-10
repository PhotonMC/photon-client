//! `TextureSlots`: how a model's texture variables resolve.
//!
//! A model's texture map may name a sprite directly or point at another
//! slot with a leading `#`. The chain is walked child first, but applied
//! parent first, so a child overrides its parent; a reference clears an
//! inherited value and a value clears an inherited reference. References
//! then resolve to a fixed point, and anything still dangling is dropped
//! rather than failing.

use serde_json::{Value, json};

const STONE: &str = "minecraft:block/stone";
const DIRT: &str = "minecraft:block/dirt";
const OAK: &str = "minecraft:block/oak_planks";

fn link(slot: &str, target: &str) -> Value {
    json!({"slot": slot, "raw": format!("#{target}")})
}

fn raw(slot: &str, value: &str) -> Value {
    json!({"slot": slot, "raw": value})
}

fn sprite(slot: &str, sprite: &str, force_translucent: bool) -> Value {
    json!({"slot": slot, "sprite": sprite, "force_translucent": force_translucent})
}

fn chains() -> Vec<(&'static str, Value)> {
    vec![
        ("empty", json!([])),
        ("single-value", json!([[raw("all", STONE)]])),
        ("object-form", json!([[sprite("all", STONE, true)]])),
        // A child naming a sprite wins over the parent's sprite.
        ("child-overrides-parent", json!([[raw("all", DIRT)], [raw("all", STONE)]])),
        // cube_all shape: parent points every face at #all, child fills it in.
        (
            "cube-all",
            json!([
                [raw("all", STONE)],
                [link("top", "all"), link("bottom", "all"), link("side", "all"), link("particle", "all")],
            ]),
        ),
        // The reference target itself arrives from a grandparent.
        (
            "three-level",
            json!([[raw("all", OAK)], [link("side", "all")], [link("north", "side"), link("particle", "side")]]),
        ),
        // A chain of references collapsing to one sprite.
        (
            "reference-chain",
            json!([[raw("all", STONE)], [link("top", "all")], [link("side", "top")], [link("north", "side")]]),
        ),
        // A child reference replaces an inherited concrete value.
        ("reference-clears-value", json!([[link("all", "side")], [raw("all", STONE), raw("side", DIRT)]])),
        // A child value replaces an inherited reference.
        ("value-clears-reference", json!([[raw("all", STONE)], [link("all", "side"), raw("side", DIRT)]])),
    ]
}

fn unresolved_chains() -> Vec<(&'static str, Value)> {
    vec![
        // Nothing to point at: the slot is dropped, not an error.
        ("dangling-reference", json!([[link("top", "nothing")]])),
        ("self-reference", json!([[link("top", "top")]])),
        ("cycle", json!([[link("top", "side"), link("side", "top")]])),
        // A cycle next to a resolvable reference: only the good one survives.
        (
            "cycle-plus-resolvable",
            json!([[raw("all", STONE)], [link("top", "side"), link("side", "top"), link("bottom", "all")]]),
        ),
        ("force-translucent-inherited", json!([[sprite("all", STONE, true)], [link("side", "all")]])),
        (
            "mixed-levels",
            json!([
                [raw("all", STONE), sprite("particle", DIRT, true)],
                [link("top", "all"), raw("bottom", OAK)],
                [link("side", "bottom"), link("north", "missing")],
            ]),
        ),
        // Same slot named at every level: the deepest child still wins.
        ("deep-override", json!([[raw("all", OAK)], [raw("all", DIRT)], [raw("all", STONE)]])),
    ]
}

pub fn rows() -> Vec<Value> {
    chains()
        .into_iter()
        .chain(unresolved_chains())
        .map(|(label, chain)| json!({"label": label, "chain": chain}))
        .collect()
}
