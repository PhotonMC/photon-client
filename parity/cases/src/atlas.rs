//! The `atlas` family: where `Stitcher` places each sprite on the atlas.
//!
//! The packer sorts by descending height, then descending width, then sprite
//! name, and grows the atlas by powers of two on whichever axis is behind.
//! The inputs drive the parts that decide a layout: equal-size sprites where
//! only the name breaks the tie, mixed sizes that force subdivision,
//! non-power-of-two sizes rounded up by the mip level, padding that grows
//! with mip level and anisotropy, and sets that cannot fit at all.

use serde_json::{Value, json};

fn sprite(name: &str, width: u32, height: u32) -> Value {
    json!({"name": name, "width": width, "height": height})
}

fn sprites(list: &[(&str, u32, u32)]) -> Vec<Value> {
    list.iter()
        .map(|(name, width, height)| sprite(name, *width, *height))
        .collect()
}

struct Atlas {
    max_width: u32,
    max_height: u32,
    mip_level: u32,
    anisotropy_bit: u32,
}

const DEFAULT: Atlas = Atlas {
    max_width: 256,
    max_height: 256,
    mip_level: 0,
    anisotropy_bit: 0,
};

fn row(label: &str, sprites: &[Value], atlas: &Atlas) -> Value {
    json!({
        "label": label, "sprites": sprites,
        "max_width": atlas.max_width, "max_height": atlas.max_height,
        "mip_level": atlas.mip_level, "anisotropy_bit": atlas.anisotropy_bit,
    })
}

fn numbered(prefix: &str, count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| sprite(&format!("minecraft:block/{prefix}{index:02}"), 16, 16))
        .collect()
}

fn mixed() -> Vec<Value> {
    sprites(&[
        ("minecraft:block/tall", 16, 64),
        ("minecraft:block/wide", 64, 16),
        ("minecraft:block/small", 8, 8),
        ("minecraft:block/big", 32, 32),
        ("minecraft:block/tiny", 4, 4),
    ])
}

/// Layouts decided by sizes and names alone.
fn ordering_rows() -> Vec<Value> {
    // Same size and same path, different namespace: the comparator falls
    // through to the identifier, which orders on path before namespace.
    let namespaces = sprites(&[
        ("zzz:block/same", 16, 16),
        ("aaa:block/same", 16, 16),
        ("minecraft:block/same", 16, 16),
    ]);
    // Path ordering and namespace ordering disagree here: by path `block/a`
    // (namespace zzz) comes first, by namespace `aaa` would. This row pins
    // path-before-namespace rather than just the fallback.
    let path_first = sprites(&[("zzz:block/a", 16, 16), ("aaa:block/b", 16, 16)]);
    let ragged = sprites(&[
        ("minecraft:block/odd", 13, 7),
        ("minecraft:block/odder", 5, 11),
        ("minecraft:block/oddest", 3, 3),
    ]);
    vec![
        row(
            "single",
            &sprites(&[("minecraft:block/only", 16, 16)]),
            &DEFAULT,
        ),
        row("uniform-8", &numbered("b", 8), &DEFAULT),
        row("name-tiebreak", &namespaces, &DEFAULT),
        row("path-before-namespace", &path_first, &DEFAULT),
        row("mixed-sizes", &mixed(), &DEFAULT),
        row("ragged-sizes", &ragged, &DEFAULT),
    ]
}

/// Layouts changed by padding and by the atlas limits.
fn limit_rows() -> Vec<Value> {
    let mixed = mixed();
    let mip = |mip_level| Atlas {
        mip_level,
        ..DEFAULT
    };
    let anisotropy = |anisotropy_bit| Atlas {
        anisotropy_bit,
        ..DEFAULT
    };
    let size = |max_width, max_height| Atlas {
        max_width,
        max_height,
        ..DEFAULT
    };
    let both = Atlas {
        mip_level: 2,
        anisotropy_bit: 2,
        ..DEFAULT
    };
    let overflow = [numbered("b", 8).as_slice(); 4].concat();
    vec![
        row("mip-1", &mixed, &mip(1)),
        row("mip-2", &mixed, &mip(2)),
        row("mip-4", &mixed, &mip(4)),
        row("anisotropy-1", &mixed, &anisotropy(1)),
        row("anisotropy-3", &mixed, &anisotropy(3)),
        row("anisotropy-clamped-9", &mixed, &anisotropy(9)),
        row("mip-2-anisotropy-2", &mixed, &both),
        row("narrow-atlas", &mixed, &size(64, 256)),
        row("short-atlas", &mixed, &size(256, 64)),
        row(
            "exact-fit",
            &sprites(&[("minecraft:block/full", 64, 64)]),
            &size(64, 64),
        ),
        row(
            "too-large",
            &sprites(&[("minecraft:block/huge", 128, 128)]),
            &size(64, 64),
        ),
        row("overflow", &overflow, &size(32, 32)),
        row("power-of-two-growth", &numbered("g", 5), &DEFAULT),
    ]
}

pub fn cases() -> Vec<Value> {
    let rows = [ordering_rows(), limit_rows()].concat();
    vec![json!({"case_id": "atlas/stitch", "family": "atlas", "operation": "stitch", "rows": rows})]
}
