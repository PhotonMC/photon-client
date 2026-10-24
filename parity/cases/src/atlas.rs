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
    list.iter().map(|(name, width, height)| sprite(name, *width, *height)).collect()
}

struct Atlas {
    max_width: u32,
    max_height: u32,
    mip_level: u32,
    anisotropy_bit: u32,
}

const DEFAULT: Atlas = Atlas { max_width: 256, max_height: 256, mip_level: 0, anisotropy_bit: 0 };

fn row(label: &str, sprites: &[Value], atlas: &Atlas) -> Value {
    json!({
        "label": label, "sprites": sprites,
        "max_width": atlas.max_width, "max_height": atlas.max_height,
        "mip_level": atlas.mip_level, "anisotropy_bit": atlas.anisotropy_bit,
    })
}

fn numbered(prefix: &str, count: usize) -> Vec<Value> {
    (0..count).map(|index| sprite(&format!("minecraft:block/{prefix}{index:02}"), 16, 16)).collect()
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
    let namespaces =
        sprites(&[("zzz:block/same", 16, 16), ("aaa:block/same", 16, 16), ("minecraft:block/same", 16, 16)]);
    // Path ordering and namespace ordering disagree here: by path `block/a`
    // (namespace zzz) comes first, by namespace `aaa` would. This row pins
    // path-before-namespace rather than just the fallback.
    let path_first = sprites(&[("zzz:block/a", 16, 16), ("aaa:block/b", 16, 16)]);
    let ragged =
        sprites(&[("minecraft:block/odd", 13, 7), ("minecraft:block/odder", 5, 11), ("minecraft:block/oddest", 3, 3)]);
    vec![
        row("single", &sprites(&[("minecraft:block/only", 16, 16)]), &DEFAULT),
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
    let mip = |mip_level| Atlas { mip_level, ..DEFAULT };
    let anisotropy = |anisotropy_bit| Atlas { anisotropy_bit, ..DEFAULT };
    let size = |max_width, max_height| Atlas { max_width, max_height, ..DEFAULT };
    let both = Atlas { mip_level: 2, anisotropy_bit: 2, ..DEFAULT };
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
        row("exact-fit", &sprites(&[("minecraft:block/full", 64, 64)]), &size(64, 64)),
        row("too-large", &sprites(&[("minecraft:block/huge", 128, 128)]), &size(64, 64)),
        row("overflow", &overflow, &size(32, 32)),
        row("power-of-two-growth", &numbered("g", 5), &DEFAULT),
    ]
}

pub fn cases() -> Vec<Value> {
    let rows = [ordering_rows(), limit_rows()].concat();
    // The sprites the game's own definition of the GUI atlas lists, which the stitcher then places.
    let hex = |text: &str| text.bytes().map(|byte| format!("{byte:02x}")).collect::<String>();
    let file = |id: &str, text: &str| json!({"id": id, "hex": hex(text)});
    let definition = |sources: &str| file("minecraft:atlases/gui.json", &format!("{{\"sources\":[{sources}]}}"));
    let texture = |id: &str| file(id, "x");
    let listed = [
        json!({"label": "gui", "atlas": "minecraft:gui"}),
        json!({"label": "blocks", "atlas": "minecraft:blocks"}),
        json!({"label": "items", "atlas": "minecraft:items"}),
        json!({"label": "armor-trims", "atlas": "minecraft:armor_trims"}),
        // A pack adds a folder to the block atlas and permutes a texture of the item atlas; a
        // permutation of a texture that is missing lists nothing.
        json!({"label": "pack-blocks-folder", "atlas": "minecraft:blocks", "pack": [
            file("minecraft:atlases/blocks.json", concat!(
                r#"{"sources":[{"type":"minecraft:directory","source":"custom","prefix":"custom/"},"#,
                r#"{"type":"minecraft:filter","pattern":{"path":"^block/stone$"}}]}"#)),
            texture("minecraft:textures/custom/a.png"),
            texture("foo:textures/custom/sub/b.png"),
        ]}),
        json!({"label": "pack-items-permutations", "atlas": "minecraft:items", "pack": [
            file("minecraft:atlases/items.json", concat!(
                r#"{"sources":[{"type":"minecraft:paletted_permutations","textures":["foo:p/one","foo:p/gone"],"#,
                r#""palette_key":"foo:p/key","permutations":{"red":"foo:p/red","blue":"foo:p/blue"},"separator":"-"}]}"#)),
            texture("foo:textures/p/one.png"),
        ]}),
        // A pack's own sources come after the game's: a directory of another namespace with a
        // prefix, a single file under a new name, and one filter dropping a game sprite.
        json!({"label": "pack-directory-single-filter", "atlas": "minecraft:gui", "pack": [
            definition(concat!(
                r#"{"type":"minecraft:directory","source":"custom","prefix":"extra/"},"#,
                r#"{"type":"minecraft:single","resource":"minecraft:misc/one","sprite":"foo:solo"},"#,
                r#"{"type":"minecraft:filter","pattern":{"path":"^hud/heart/full$"}}"#
            )),
            texture("foo:textures/custom/deep/c.png"),
            texture("minecraft:textures/custom/d.png"),
            texture("minecraft:textures/misc/one.png"),
        ]}),
        // Regions of one texture are sprites; a filter after them drops one, and one before the
        // unstitch source does not.
        json!({"label": "pack-unstitch-filter", "atlas": "minecraft:gui", "pack": [
            definition(concat!(
                r#"{"type":"minecraft:filter","pattern":{"namespace":"^foo$"}},"#,
                r#"{"type":"minecraft:unstitch","resource":"foo:sheet","divisor_x":2.0,"regions":["#,
                r#"{"sprite":"foo:cut/a","x":0,"y":0,"width":1,"height":1},"#,
                r#"{"sprite":"foo:cut/b","x":1,"y":0,"width":1,"height":1}]},"#,
                r#"{"type":"minecraft:filter","pattern":{"path":"cut/b"}}"#
            )),
            texture("foo:textures/sheet.png"),
        ]}),
        // An unstitched texture that is missing lists nothing; a filter matching everything
        // empties the atlas.
        json!({"label": "pack-missing-unstitch-and-clear", "atlas": "minecraft:gui", "pack": [
            definition(concat!(
                r#"{"type":"minecraft:unstitch","resource":"foo:missing","regions":["#,
                r#"{"sprite":"foo:cut/a","x":0,"y":0,"width":1,"height":1}]},"#,
                r#"{"type":"minecraft:filter","pattern":{}}"#
            )),
        ]}),
    ];
    vec![
        json!({"case_id": "atlas/stitch", "family": "atlas", "operation": "stitch", "rows": rows}),
        json!({"case_id": "atlas/sprite-ids", "family": "atlas", "operation": "sprite_ids", "rows": listed}),
    ]
}
