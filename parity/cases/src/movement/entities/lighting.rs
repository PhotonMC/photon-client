//! How a block's quads are lit: the occlusion shade and light each corner
//! takes from the blocks around it, smooth or flat, whole or partial faces.

use crate::movement::entities::prelude::*;

const DIRECTIONS: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

/// A face's corners in the model's own order, the face plane at `plane` and
/// the other two axes running from `low` to `high`.
fn corners(direction: &str, plane: f64, low: f64, high: f64) -> Vec<Value> {
    let (a, b) = (low, high);
    let points: [[f64; 3]; 4] = match direction {
        "down" => [[a, plane, b], [a, plane, a], [b, plane, a], [b, plane, b]],
        "up" => [[a, plane, a], [a, plane, b], [b, plane, b], [b, plane, a]],
        "north" => [[b, b, plane], [b, a, plane], [a, a, plane], [a, b, plane]],
        "south" => [[a, b, plane], [a, a, plane], [b, a, plane], [b, b, plane]],
        "west" => [[plane, b, a], [plane, a, a], [plane, a, b], [plane, b, b]],
        _ => [[plane, b, b], [plane, a, b], [plane, a, a], [plane, b, a]],
    };
    points.iter().map(|point| json!(point.map(f32_bits))).collect()
}

/// The plane a face sits on in a whole block.
fn edge(direction: &str) -> f64 {
    if matches!(direction, "down" | "north" | "west") { 0.0 } else { 1.0 }
}

fn quad(label: String, at: [i64; 3], direction: &str, corners: Vec<Value>, smooth: bool, shade: bool) -> Value {
    json!({
        "label": label, "x": at[0], "y": at[1], "z": at[2], "direction": direction,
        "corners": corners, "smooth": smooth, "shade": shade, "cull": Value::Null,
    })
}

/// Every face of the block at `at`: whole, partial, set back inside the
/// block, unshaded, and flat with and without a side it hides behind.
fn quads_of(at: [i64; 3]) -> Vec<Value> {
    let mut quads = Vec::new();
    for direction in DIRECTIONS {
        let whole = corners(direction, edge(direction), 0.0, 1.0);
        let partial = corners(direction, edge(direction), 0.25, 0.75);
        let inside = corners(direction, 0.5, 0.0, 1.0);
        let lopsided = corners(direction, edge(direction), 0.0, 0.5);
        let variants = [
            ("whole", whole.clone(), true, true),
            ("whole-unshaded", whole.clone(), true, false),
            ("partial", partial.clone(), true, true),
            ("inside", inside.clone(), true, true),
            ("lopsided", lopsided, true, true),
            ("flat-whole", whole.clone(), false, true),
            ("flat-partial", partial, false, true),
            ("flat-inside", inside, false, true),
        ];
        for (name, shape, smooth, shade) in variants {
            quads.push(quad(format!("{direction}-{name}"), at, direction, shape, smooth, shade));
        }
        let mut culled = quad(format!("{direction}-flat-culled"), at, direction, whole, false, true);
        culled["cull"] = json!(direction);
        quads.push(culled);
    }
    quads
}

fn row(label: &str, blocks: Vec<Value>, at: [i64; 3]) -> Value {
    json!({"label": label, "blocks": blocks, "chunk_radius": 1, "quads": quads_of(at)})
}

/// Stone scattered about the origin so that faces see walls, ledges and
/// corners at different heights.
fn rubble() -> Vec<Value> {
    [
        (1, 64, 0),
        (1, 65, 0),
        (0, 64, 1),
        (-1, 65, 0),
        (0, 65, -1),
        (1, 64, 1),
        (-1, 64, -1),
        (1, 66, 1),
        (0, 66, 0),
        (-1, 64, 1),
    ]
    .iter()
    .map(|&(x, y, z)| put(x, y, z, "minecraft:stone"))
    .collect()
}

fn lit_rubble() -> Vec<Value> {
    cat![rubble(), vec![put(2, 65, 0, "minecraft:glowstone"), put(0, 67, 2, "minecraft:torch")]]
}

fn roofed() -> Vec<Value> {
    (-8..9).flat_map(|x| (-8..9).map(move |z| put(x, 70, z, "minecraft:stone"))).collect()
}

/// A row's cardinal light type: the dimension type's `cardinal_light`.
fn in_nether(mut row: Value) -> Value {
    row["label"] = json!(format!("nether-{}", row["label"].as_str().unwrap_or_default()));
    row["cardinal"] = json!("nether");
    row
}

pub fn rows() -> Vec<Value> {
    let overworld = overworld_rows();
    let nether: Vec<Value> = overworld.iter().take(3).cloned().map(in_nether).collect();
    cat![overworld, nether]
}

fn overworld_rows() -> Vec<Value> {
    let ground = floor();
    let centre = [0, 64, 0];
    let on = |block: Value| cat![ground.clone(), vec![block]];
    let around = |block: Value, extra: Vec<Value>| cat![ground.clone(), vec![block], extra];
    vec![
        row("stone-open", on(put(0, 64, 0, "minecraft:stone")), centre),
        row("stone-in-rubble", around(put(0, 64, 0, "minecraft:stone"), rubble()), centre),
        row("stone-by-light", around(put(0, 64, 0, "minecraft:stone"), lit_rubble()), centre),
        row("stone-under-roof", around(put(0, 64, 0, "minecraft:stone"), cat![lit_rubble(), roofed()]), centre),
        row(
            "slab-in-rubble",
            around(put_with(0, 64, 0, "minecraft:stone_slab", &[("type", "bottom")]), rubble()),
            centre,
        ),
        row(
            "top-slab-in-rubble",
            around(put_with(0, 64, 0, "minecraft:stone_slab", &[("type", "top")]), rubble()),
            centre,
        ),
        row("glass-in-rubble", around(put(0, 64, 0, "minecraft:glass"), rubble()), centre),
        row("leaves-in-rubble", around(put(0, 64, 0, "minecraft:oak_leaves"), rubble()), centre),
        row(
            "glass-neighbours",
            around(
                put(0, 64, 0, "minecraft:stone"),
                vec![
                    put(1, 65, 0, "minecraft:glass"),
                    put(0, 65, 1, "minecraft:glass"),
                    put(-1, 64, 0, "minecraft:oak_leaves"),
                    put(0, 64, -1, "minecraft:stone"),
                    put(1, 64, 1, "minecraft:stone"),
                ],
            ),
            centre,
        ),
        row("glowing-block", around(put(0, 64, 0, "minecraft:glowstone"), rubble()), centre),
        row(
            "stairs-in-rubble",
            around(put_with(0, 64, 0, "minecraft:oak_stairs", &[("facing", "east")]), rubble()),
            centre,
        ),
        row(
            "buried",
            around(put(0, 64, 0, "minecraft:stone"), cat![rubble(), vec![put(0, 65, 0, "minecraft:stone")]]),
            centre,
        ),
    ]
}

/// Which lighting a block takes: the `ao` option, light emission and the
/// parts' own say, the first of which decides for all.
pub fn gate_rows() -> Vec<Value> {
    let rows = [
        ("stone-option-on", "minecraft:stone", true, vec![true]),
        ("stone-option-off", "minecraft:stone", false, vec![true]),
        ("stone-model-opts-out", "minecraft:stone", true, vec![false]),
        ("stone-first-part-opts-out", "minecraft:stone", true, vec![false, true]),
        ("stone-first-part-opts-in", "minecraft:stone", true, vec![true, false]),
        ("stone-no-parts-opt-out", "minecraft:stone", true, vec![false, false]),
        ("glowstone-option-on", "minecraft:glowstone", true, vec![true]),
        ("glowstone-option-off", "minecraft:glowstone", false, vec![true]),
        ("torch-glows", "minecraft:torch", true, vec![true]),
    ];
    rows.into_iter()
        .map(|(label, block, option, parts)| {
            json!({
                "label": label, "option": option, "parts": parts, "chunk_radius": 1,
                "blocks": cat![floor(), rubble(), vec![put(0, 64, 0, block)]],
            })
        })
        .collect()
}

/// What a dimension type's `cardinal_light` field decodes to: each preset, the
/// field left out, and values the codec has no preset for.
pub fn dimension_rows() -> Vec<Value> {
    let rows = [
        ("absent", json!({"kind": "absent"})),
        ("default", json!({"kind": "string", "value": "default"})),
        ("nether", json!({"kind": "string", "value": "nether"})),
        ("unknown-name", json!({"kind": "string", "value": "end"})),
        ("capitalised", json!({"kind": "string", "value": "Nether"})),
        ("empty-name", json!({"kind": "string", "value": ""})),
        ("namespaced", json!({"kind": "string", "value": "minecraft:nether"})),
        ("int", json!({"kind": "int", "value": 1})),
        ("byte", json!({"kind": "byte", "value": 1})),
        ("list", json!({"kind": "list"})),
        ("compound", json!({"kind": "compound"})),
    ];
    rows.into_iter().map(|(label, cardinal)| json!({"label": label, "cardinal": cardinal})).collect()
}

/// The vertices of a lava or water block, with the default and the Nether's
/// cardinal light: open, under a solid block, under a ceiling, beside a wall
/// and flowing.
pub fn fluid_rows() -> Vec<Value> {
    let at = [2, 64, 3];
    let ceiling = (1..=3).flat_map(|x| (2..=4).map(move |z| put(x, 65, z, "minecraft:stone"))).collect::<Vec<_>>();
    let glass = (1..=3).flat_map(|x| (2..=4).map(move |z| put(x, 65, z, "minecraft:glass"))).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for fluid in ["lava", "water"] {
        let id = format!("minecraft:{fluid}");
        let source = || put(2, 64, 3, &id);
        let scenes: [(&str, Vec<Value>); 6] = [
            ("open", vec![source()]),
            ("covered", vec![source(), put(2, 65, 3, "minecraft:stone")]),
            ("ceiled", cat![vec![source()], ceiling.clone()]),
            ("glass-ceiled", cat![vec![source()], glass.clone()]),
            (
                "walled",
                vec![source(), put(3, 64, 3, "minecraft:stone"), put(2, 64, 2, "minecraft:stone"), put(2, 64, 4, &id)],
            ),
            ("flowing", vec![put_with(2, 64, 3, &id, &[("level", "3")])]),
        ];
        for cardinal in [None, Some("nether")] {
            for (label, blocks) in &scenes {
                let mut row = json!({
                    "label": format!("{fluid}-{}{label}", if cardinal.is_some() { "nether-" } else { "" }),
                    "blocks": cat![floor(), blocks.clone()],
                    "chunk_radius": 1,
                    "x": at[0], "y": at[1], "z": at[2],
                    "sprites": if fluid == "water" { water_alpha() } else { json!({}) },
                });
                if let Some(kind) = cardinal {
                    row["cardinal"] = json!(kind);
                }
                rows.push(row);
            }
        }
    }
    rows
}

/// Water's sprites as the game ships them: every one has partly transparent texels.
fn water_alpha() -> Value {
    json!({"water_still": "translucent", "water_flow": "translucent", "water_overlay": "translucent"})
}

/// The layer a fluid is drawn in, from how much of its sprites is see-through:
/// the still, the flowing and (for water) the overlay sprite each decide.
pub fn fluid_layer_rows() -> Vec<Value> {
    let alphas: [(&str, &str, &str); 9] = [
        ("lava-opaque", "lava", "{}"),
        ("lava-still-translucent", "lava", r#"{"lava_still": "translucent"}"#),
        ("lava-flow-translucent", "lava", r#"{"lava_flow": "translucent"}"#),
        ("lava-still-transparent", "lava", r#"{"lava_still": "transparent"}"#),
        ("lava-water-sprite-translucent", "lava", r#"{"water_overlay": "translucent", "water_still": "translucent"}"#),
        ("water-opaque", "water", "{}"),
        ("water-still-translucent", "water", r#"{"water_still": "translucent"}"#),
        ("water-flow-translucent", "water", r#"{"water_flow": "translucent"}"#),
        ("water-overlay-translucent", "water", r#"{"water_overlay": "translucent"}"#),
    ];
    alphas
        .into_iter()
        .map(|(label, fluid, sprites)| {
            json!({
                "label": label,
                "blocks": cat![floor(), vec![put(2, 64, 3, &format!("minecraft:{fluid}"))]],
                "chunk_radius": 1,
                "x": 2, "y": 64, "z": 3,
                "sprites": serde_json::from_str::<Value>(sprites).unwrap_or_default(),
            })
        })
        .collect()
}

/// A named sprite: sizes, base alpha, patch rectangle and patch alpha.
type ProbeSprite = (&'static str, [i64; 4], i64, [i64; 4], i64);

/// The sprites the `face_layer` rows name: size, size of one frame, the alpha
/// of every pixel but those in a rectangle (`x0, x1, y0, y1`), and the alpha
/// inside it.
const PROBE_SPRITES: [ProbeSprite; 9] = [
    ("opaque", [16, 16, 16, 16], 255, [0, 0, 0, 0], 255),
    ("translucent", [16, 16, 16, 16], 128, [0, 0, 0, 0], 128),
    ("ring", [16, 16, 16, 16], 100, [4, 12, 4, 12], 255),
    ("column-7", [16, 16, 16, 16], 255, [7, 8, 0, 16], 100),
    ("column-8", [16, 16, 16, 16], 255, [8, 9, 0, 16], 100),
    ("hole", [16, 16, 16, 16], 255, [0, 1, 0, 1], 0),
    ("strip", [16, 48, 16, 16], 255, [4, 8, 36, 40], 100),
    ("side-by-side", [32, 16, 16, 16], 255, [16, 32, 0, 16], 100),
    ("wide", [32, 32, 32, 32], 255, [20, 24, 20, 24], 100),
];

/// One `face_layer` row: label, sprite, frames played, the face's rectangle in
/// sixteenths and whether the material forces translucency.
type FaceProbe = (&'static str, &'static str, &'static [i64], [f32; 4], bool);

/// A sprite's pixels (width, height, one alpha each) and the size of one
/// frame, by the name the `face_layer` rows give.
fn probe_sprite(kind: &str) -> ((i64, i64, Vec<i64>), (i64, i64)) {
    let Some(&(_, [width, height, frame_width, frame_height], base, [x0, x1, y0, y1], patch)) =
        PROBE_SPRITES.iter().find(|sprite| sprite.0 == kind)
    else {
        panic!("no probe sprite {kind}");
    };
    let inside = |x: i64, y: i64| (x0..x1).contains(&x) && (y0..y1).contains(&y);
    let pixels = (0..height).flat_map(|y| (0..width).map(move |x| (x, y)));
    let alphas = pixels.map(|(x, y)| if inside(x, y) { patch } else { base }).collect();
    ((width, height, alphas), (frame_width, frame_height))
}

/// The layer a face bakes to: the whole sprite's alpha, or just the part of
/// it the face shows, over every frame an animation plays.
pub fn face_layer_rows() -> Vec<Value> {
    let full = [0.0, 0.0, 16.0, 16.0];
    let probes: [FaceProbe; 22] = [
        ("opaque", "opaque", &[], full, false),
        ("translucent", "translucent", &[], full, false),
        ("ring-centre-solid", "ring", &[], [4.0, 4.0, 12.0, 12.0], false),
        ("ring-edge-translucent", "ring", &[], [3.0, 3.0, 12.0, 12.0], false),
        ("ring-full-translucent", "ring", &[], full, false),
        ("ring-reversed-solid", "ring", &[], [12.0, 12.0, 4.0, 4.0], false),
        ("half-texel-floor-low", "column-7", &[], [7.5, 0.0, 16.0, 16.0], false),
        ("half-texel-low-clear", "column-7", &[], [8.0, 0.0, 16.0, 16.0], false),
        ("half-texel-ceil-high", "column-8", &[], [0.0, 0.0, 8.5, 16.0], false),
        ("half-texel-high-clear", "column-8", &[], [0.0, 0.0, 8.0, 16.0], false),
        ("force-translucent", "opaque", &[], full, true),
        ("hole-cutout", "hole", &[], full, false),
        ("hole-missed-solid", "hole", &[], [1.0, 1.0, 16.0, 16.0], false),
        ("strip-later-frame-translucent", "strip", &[0, 1, 2, 2, 0], [4.0, 4.0, 8.0, 8.0], false),
        ("strip-other-part-solid", "strip", &[0, 1, 2], [8.0, 8.0, 16.0, 16.0], false),
        ("strip-unused-frame-solid", "strip", &[0, 1], [4.0, 4.0, 8.0, 8.0], false),
        ("strip-unused-frame-full-translucent", "strip", &[0, 1], full, false),
        ("strip-one-frame-solid", "strip", &[0], [4.0, 4.0, 8.0, 8.0], false),
        ("strip-across-translucent", "side-by-side", &[0, 1], [1.0, 1.0, 15.0, 15.0], false),
        ("wide-sprite-translucent", "wide", &[], [8.0, 8.0, 12.0, 12.0], false),
        ("wide-sprite-solid", "wide", &[], [0.0, 0.0, 8.0, 8.0], false),
        ("out-of-bounds", "column-7", &[], [0.0, 0.0, 20.0, 16.0], false),
    ];
    probes
        .into_iter()
        .map(|(label, kind, playback, uvs, force)| {
            let ((width, height, alphas), (frame_width, frame_height)) = probe_sprite(kind);
            let mut row = json!({
                "label": label, "width": width, "height": height, "alphas": alphas,
                "frame_width": frame_width, "frame_height": frame_height,
                "uvs": uvs.map(f32_bits), "force_translucent": force,
            });
            if !playback.is_empty() {
                row["playback"] = json!(playback);
            }
            row
        })
        .collect()
}
