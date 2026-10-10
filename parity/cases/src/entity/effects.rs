//! Fishing hooks, primed tnt, lightning, paintings and screen effects.

use serde_json::{Value, json};

use super::objects::{FAR, HERE, placed};
use super::pose::entity;
use crate::bits::f32_bits;
use crate::line::float_text;

/// A fishing bobber as the reference's own renderer submits it: the quad
/// facing a camera that looks along each way and slantwise, and the line's
/// sixteen segments toward a hand in each direction, near and far.
pub fn fishing_hook() -> Vec<Value> {
    let hook = |label: &str, camera: (f64, f64), offset: [f64; 3], at: [f64; 3], light: i64| {
        let row = json!({
            "label": format!("hook-{label}"), "camera_y_rot": f32_bits(camera.0), "camera_x_rot": f32_bits(camera.1),
            "offset_x": f32_bits(offset[0]), "offset_y": f32_bits(offset[1]), "offset_z": f32_bits(offset[2]),
            "light": light,
        });
        placed(row, at)
    };
    let (lit, near) = (0x00F0_00F0, [0.0, 1.0, 2.0]);
    let rows = vec![
        hook("south", (0.0, 0.0), near, HERE, lit),
        hook("west", (90.0, 0.0), [-3.5, 0.5, 0.25], HERE, lit),
        hook("north", (180.0, 0.0), [2.0, -0.3, -4.75], HERE, lit),
        hook("down-on-it", (0.0, 90.0), [0.1, 3.0, 0.1], HERE, lit),
        hook("up-at-it", (45.0, -60.0), [-1.25, 2.5, 7.0], HERE, lit),
        hook("slantwise", (-130.0, 33.5), [5.5, -2.25, -6.125], HERE, lit),
        hook("far-off", (30.0, 5.0), [0.0, 0.0, 0.5], FAR, 0x00F0_0000),
        hook("hand-on-the-bobber", (0.0, 0.0), [0.0; 3], HERE, lit),
    ];
    vec![entity("fishing-hook", "fishing_hook", rows)]
}

/// Where the reference's own tnt renderer draws the block of a primed tnt
/// from the fuse it has left, and the overlay it flashes with.
pub fn primed_tnt() -> Vec<Value> {
    let tnt = |label: String, fuse: f64, at: [f64; 3]| placed(json!({"label": label, "fuse": f32_bits(fuse)}), at);
    let fuses = [81.0, 80.5, 40.0, 11.0, 10.0, 9.5, 9.0, 7.25, 5.0, 4.99, 3.0, 1.5, 1.0, 0.5, 0.0, -0.5];
    let mut rows: Vec<Value> =
        fuses.iter().map(|fuse| tnt(format!("tnt-fuse-{}", float_text(*fuse)), *fuse, HERE)).collect();
    rows.push(tnt("tnt-far-off".to_owned(), 6.0, FAR));
    vec![entity("primed-tnt-pose", "primed_tnt_pose", rows)]
}

/// The vertices the reference's own lightning renderer hands the vertex
/// consumer for several seeds and places.
pub fn lightning_bolt() -> Vec<Value> {
    let bolt = |label: &str, seed: i64, at: [f64; 3]| {
        placed(json!({"label": format!("bolt-{label}"), "seed": seed.to_string()}), at)
    };
    let rows = vec![
        bolt("zero", 0, HERE),
        bolt("one", 1, HERE),
        bolt("ordinary", 123456789, HERE),
        bolt("negative", -987654321, HERE),
        bolt("large", i64::MAX, HERE),
        bolt("small", i64::MIN, HERE),
        bolt("far-off", 42, FAR),
    ];
    vec![entity("lightning-bolt", "lightning_bolt", rows)]
}

const SPRITES: [(&str, f64); 8] = [
    ("front_u0", 0.125),
    ("front_u1", 0.875),
    ("front_v0", 0.0625),
    ("front_v1", 0.5625),
    ("back_u0", 0.5),
    ("back_u1", 0.75),
    ("back_v0", 0.25),
    ("back_v1", 0.375),
];

/// The vertices the reference's own painting renderer hands the vertex
/// consumer: paintings of several sizes facing each way, each block lit
/// differently.
pub fn painting_geometry() -> Vec<Value> {
    let painting = |label: &str, (width, height): (i64, i64), direction: i64, at: [f64; 3]| {
        let lights: Vec<i64> =
            (0..width * height).map(|index| (0x00F0_0000 | ((index * 16 + 3) << 4)) & 0x00F0_00F0).collect();
        let mut row = json!({
            "label": format!("painting-{label}"), "width": width, "height": height, "direction_2d": direction,
            "lights": lights,
        });
        for (key, value) in SPRITES {
            row[key] = json!(f32_bits(value));
        }
        placed(row, at)
    };
    let at = [10.5, 64.5, -3.5];
    let rows = vec![
        painting("one-south", (1, 1), 0, at),
        painting("one-west", (1, 1), 1, at),
        painting("one-north", (1, 1), 2, at),
        painting("one-east", (1, 1), 3, at),
        painting("wide", (3, 1), 0, at),
        painting("tall", (1, 2), 1, at),
        painting("two-by-two", (2, 2), 2, at),
        painting("big", (4, 3), 3, at),
        painting("backyard", (3, 4), 0, at),
        painting("far-off", (2, 1), 1, FAR),
    ];
    vec![entity("painting-geometry", "painting_geometry", rows)]
}

/// The quads the screen effects lay over the view, as the reference's own
/// `ScreenEffectRenderer` builds them: the block the eyes are in and the
/// fire from stand-in sprite corners, and the water from where the player
/// looks and how bright it is at the eyes.
pub fn screen_effect() -> Vec<Value> {
    let corners = |u0: f64, u1: f64, v0: f64, v1: f64| json!({"u0": f32_bits(u0), "u1": f32_bits(u1), "v0": f32_bits(v0), "v1": f32_bits(v1)});
    let quad = |label: &str, kind: &str, uv: &Value, color: Option<i64>| {
        let mut row = json!({"label": label, "kind": kind});
        crate::case::merge(&mut row, uv.clone());
        if let Some(color) = color {
            row["color"] = json!(color);
        }
        row
    };
    let (sprite, whole) = (corners(0.25, 0.3125, 0.5, 0.5625), corners(0.0, 1.0, 0.0, 1.0));
    let mut rows = vec![
        quad("block", "block", &sprite, Some(-15132391)),
        quad("block-whole-texture", "block", &whole, Some(-15132391)),
        quad("fire", "fire", &sprite, None),
        quad("fire-whole-texture", "fire", &whole, None),
    ];
    for (label, y_rot, x_rot, brightness) in [
        ("ahead", 0.0, 0.0, 1.0),
        ("turned", 123.5, -30.25, 0.75),
        ("looking-down", -200.0, 90.0, 0.2),
        ("dark", 10.0, 5.0, 0.0),
    ] {
        rows.push(json!({
            "label": format!("water-{label}"), "kind": "water", "y_rot": f32_bits(y_rot), "x_rot": f32_bits(x_rot),
            "brightness": f32_bits(brightness),
        }));
    }
    vec![entity("screen-effect-quads", "screen_effect_quads", rows)]
}
