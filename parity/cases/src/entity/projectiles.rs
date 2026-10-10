//! Text displays, maps, arrows, thrown items, charges, small projectiles,
//! leashes, fishing hooks, primed tnt, lightning, paintings and screen
//! effects.

use serde_json::{Value, json};

use super::objects::{FAR, HERE, placed};
use super::pose::entity;
use crate::bits::{f32_bits, f64_bits};
use crate::case::merge;
use crate::line::float_text;

/// What the reference's own text display renderer submits for the lines a
/// text split into: the background box, each line's place by the
/// alignment, the shadow, the opacity in the colour, and the light.
pub fn text_display() -> Vec<Value> {
    let text =
        |label: &str, widths: &[i64], (flags, opacity, background): (i64, i64, i64), light: i64, at: [f64; 3]| {
            let row = json!({
                "label": format!("text-{label}"), "widths": widths, "flags": flags, "opacity": opacity,
                "background": background, "light": light,
            });
            placed(row, at)
        };
    let (usual, full) = ((0, -1, 0x4000_0000), 15728880);
    let at = [10.5, 64.25, -3.5];
    let lines = [40, 95, 17];
    let rows = vec![
        text("one-line", &[37], usual, full, at),
        text("three-lines-centred", &lines, usual, full, at),
        text("left", &lines, (8, -1, 0x4000_0000), full, at),
        text("right", &lines, (16, -1, 0x4000_0000), full, at),
        text("both-align-flags", &lines, (24, -1, 0x4000_0000), full, at),
        text("shadow-and-see-through", &[20, 30], (3, -1, 0x4000_0000), full, at),
        text("default-background", &[20, 30], (4, -1, 0x1234_5678), full, at),
        text("clear-background", &[20, 30], (0, -1, 0), full, at),
        text("half-opacity", &[20, 30], (0, -128, 0x4000_0000), full, at),
        text("dim-opacity", &[20, 30], (0, 40, 0x8010_2030 - (1 << 32)), 240, at),
        text("no-lines", &[], usual, full, at),
        text("far", &[64], usual, full, FAR),
    ];
    vec![entity("text-display-submit", "text_display_submit", rows)]
}

fn decoration(x: i64, y: i64, rot: i64, on_frame: bool, sprite: [f64; 4]) -> Value {
    json!({
        "x": x, "y": y, "rot": rot, "on_frame": on_frame,
        "sprite": {"u0": f32_bits(sprite[0]), "v0": f32_bits(sprite[1]), "u1": f32_bits(sprite[2]), "v1": f32_bits(sprite[3])},
    })
}

/// What the reference's own map renderer draws: the map's quad and each
/// decoration, turned by sixteenths, one in front of the next.
pub fn map_render() -> Vec<Value> {
    let usual = [0.25, 0.5, 0.375, 0.625];
    let many = vec![
        decoration(0, 0, 0, true, usual),
        decoration(-128, 127, 4, false, [0.5, 0.0, 0.625, 0.125]),
        decoration(63, -64, 15, true, [0.0, 0.25, 0.125, 0.375]),
        decoration(-20, 33, 8, false, usual),
    ];
    let map = |label: &str, decorations: &[Value], only_frame: bool, light: i64, at: [f64; 3]| {
        let row = json!({
            "label": format!("map-{label}"), "decorations": decorations, "only_frame": only_frame, "light": light,
        });
        placed(row, at)
    };
    let at = [10.5, 64.25, -3.5];
    let rows = vec![
        map("bare", &[], false, 15728850, at),
        map("decorated", &many, false, 15728850, at),
        map("only-a-frames", &many, true, 15728850, at),
        map("none-shown", &[decoration(1, 2, 3, false, usual)], true, 15728850, at),
        map("turned-far", &many, false, 0x00F0_00F0, FAR),
    ];
    vec![entity("map-render", "map_render", rows)]
}

fn model(label: &str, layer: &str) -> Value {
    json!({"label": label, "layer": layer})
}

/// The arrow's model posed by the reference's own `ArrowModel.setupAnim`
/// and where the arrow renderer draws it from the turns it was sent; the
/// trident alike.
pub fn arrow() -> Vec<Value> {
    let shakes = [0.0, 0.5, 1.0, 2.5, 4.0, 6.5, 7.0, -0.25]
        .iter()
        .map(|shake| {
            json!({"label": format!("arrow-shake-{}", float_text(*shake)), "layer": "minecraft:arrow#main", "shake": f32_bits(*shake)})
        })
        .collect();
    let pose = |label: &str, y_rot: f64, x_rot: f64, at: [f64; 3]| {
        let row = json!({"label": format!("arrow-{label}"), "y_rot": f32_bits(y_rot), "x_rot": f32_bits(x_rot), "shake": f32_bits(0.0)});
        placed(row, at)
    };
    let poses = vec![
        pose("still", 0.0, 0.0, HERE),
        pose("south-east", 45.0, 0.0, HERE),
        pose("west", -90.0, 0.0, HERE),
        pose("up", 30.0, 60.0, HERE),
        pose("down", 150.0, -75.0, HERE),
        pose("wrapped", 400.0, 10.0, HERE),
        pose("offset", 12.5, -33.3, FAR),
    ];
    let tridents = poses
        .iter()
        .map(|row| {
            let mut copy = row.clone();
            let label = row["label"].as_str().unwrap_or_default().replacen("arrow-", "trident-", 1);
            copy["label"] = json!(label);
            copy["trident"] = json!(true);
            copy
        })
        .collect();
    vec![
        entity("arrow-model", "airborne_model", shakes),
        entity("arrow-pose", "arrow_pose", poses),
        // The trident: its model (nothing moves it) and where its renderer
        // draws it from the turns it was sent.
        entity("trident-model", "airborne_model", vec![model("trident", "minecraft:trident#main")]),
        entity("trident-pose", "arrow_pose", tridents),
    ]
}

fn seen(label: String, camera: (f64, f64), at: [f64; 3], extra: Value) -> Value {
    let mut row = json!({"label": label, "camera_y_rot": f32_bits(camera.0), "camera_x_rot": f32_bits(camera.1)});
    merge(&mut row, extra);
    placed(row, at)
}

/// Where the reference's own thrown item renderer (and its firework
/// rocket's) puts an item: scaled by the type's, turned to face a camera
/// that looks along each way and slantwise, and a rocket shot at an angle.
pub fn thrown_item() -> Vec<Value> {
    let thrown = |label: &str, camera: (f64, f64), scale: f64, (firework, angled): (bool, bool), at: [f64; 3]| {
        let extra = json!({"scale": f32_bits(scale), "firework": firework, "shot_at_angle": angled});
        seen(format!("thrown-{label}"), camera, at, extra)
    };
    let plain = (false, false);
    let rocket = (true, false);
    let angled = (true, true);
    let rows = vec![
        thrown("south", (0.0, 0.0), 1.0, plain, HERE),
        thrown("west", (90.0, 0.0), 1.0, plain, HERE),
        thrown("north", (180.0, 0.0), 1.0, plain, HERE),
        thrown("down-on-it", (0.0, 90.0), 1.0, plain, HERE),
        thrown("up-at-it", (45.0, -60.0), 1.0, plain, HERE),
        thrown("slantwise", (-130.0, 33.5), 1.0, plain, HERE),
        thrown("fireball", (20.0, 10.0), 3.0, plain, HERE),
        thrown("small-fireball", (20.0, 10.0), 0.75, plain, HERE),
        thrown("far-off", (30.0, 5.0), 1.0, plain, FAR),
        thrown("rocket", (20.0, 10.0), 1.0, rocket, HERE),
        thrown("rocket-at-an-angle", (20.0, 10.0), 1.0, angled, HERE),
        thrown("rocket-at-an-angle-looked-down-on", (-70.0, 80.0), 1.0, angled, HERE),
    ];
    vec![entity("thrown-item-pose", "thrown_item_pose", rows)]
}

/// The wind charge's model posed by the reference's own
/// `WindChargeModel.setupAnim` and the quad the dragon fireball renderer
/// submits, seen by a camera that looks along each way and slantwise.
pub fn charge() -> Vec<Value> {
    let charges = [0.0, 0.5, 1.0, 3.25, 11.75, 22.5, 100.1]
        .iter()
        .map(|age| {
            json!({
                "label": format!("wind-charge-age-{}", float_text(*age)), "layer": "minecraft:wind_charge#main",
                "age_in_ticks": f32_bits(*age),
            })
        })
        .collect();
    let fireball = |label: &str, camera: (f64, f64), at: [f64; 3], light: i64| {
        seen(format!("fireball-{label}"), camera, at, json!({ "light": light }))
    };
    let lit = 0x00F0_00F0;
    let rows = vec![
        fireball("south", (0.0, 0.0), HERE, lit),
        fireball("west", (90.0, 0.0), HERE, lit),
        fireball("north", (180.0, 0.0), HERE, lit),
        fireball("down-on-it", (0.0, 90.0), HERE, lit),
        fireball("up-at-it", (45.0, -60.0), HERE, lit),
        fireball("slantwise", (-130.0, 33.5), HERE, lit),
        fireball("far-off", (30.0, 5.0), FAR, 0x00F0_0000),
    ];
    vec![
        entity("wind-charge-model", "airborne_model", charges),
        entity("dragon-fireball-quad", "dragon_fireball_quad", rows),
    ]
}

fn small(kind: &str, label: &str, at: [f64; 3], values: &[(&str, f64)]) -> Value {
    let mut row = json!({"label": format!("{kind}-{label}"), "kind": kind});
    for (key, value) in values {
        row[*key] = json!(f32_bits(*value));
    }
    placed(row, at)
}

/// The shulker bullet's, llama spit's and evoker fangs' models posed by the
/// reference's own model code and where their own renderers draw them from.
pub fn small_projectile() -> Vec<Value> {
    let bullets = [("still", 0.0, 0.0), ("turned", 45.0, 30.0), ("wrapped", 400.0, -75.5), ("down", -120.0, 90.0)]
        .iter()
        .map(|(label, y, x)| {
            json!({"label": format!("bullet-{label}"), "layer": "minecraft:shulker_bullet#main", "y_rot": f32_bits(*y), "x_rot": f32_bits(*x)})
        })
        .collect();
    let fangs = [0.0, 0.05, 0.25, 0.5, 0.6, 0.85, 0.9, 0.93, 0.99, 1.0]
        .iter()
        .map(|progress| {
            json!({
                "label": format!("fangs-{}", float_text(*progress)), "layer": "minecraft:evoker_fangs#main",
                "bite_progress": f32_bits(*progress),
            })
        })
        .collect();
    let mut rows: Vec<Value> = [0.0, 1.5, 7.25, 31.0, 99.75]
        .iter()
        .map(|age| small("shulker_bullet", &format!("age-{}", float_text(*age)), HERE, &[("age_in_ticks", *age)]))
        .collect();
    rows.push(small("shulker_bullet", "far-off", FAR, &[("age_in_ticks", 12.5)]));
    for (label, y, x) in [("south", 0.0, 0.0), ("west", -90.0, 0.0), ("up", 30.0, 60.0), ("wrapped", 400.0, 10.0)] {
        rows.push(small("llama_spit", label, HERE, &[("y_rot", y), ("x_rot", x)]));
    }
    for (label, y, progress) in [
        ("waiting", 40.0, 0.0),
        ("biting", 40.0, 0.4),
        ("late", -130.0, 0.95),
        ("done", 0.0, 1.0),
        ("turned", 400.0, 0.3),
    ] {
        rows.push(small("evoker_fangs", label, HERE, &[("y_rot", y), ("bite_progress", progress)]));
    }
    rows.push(small("leash_knot", "fence", HERE, &[]));
    rows.push(small("leash_knot", "far-off", FAR, &[]));
    vec![
        entity("shulker-bullet-model", "airborne_model", bullets),
        entity("llama-spit-model", "airborne_model", vec![model("spit", "minecraft:llama_spit#main")]),
        entity("evoker-fangs-model", "airborne_model", fangs),
        entity("leash-knot-model", "airborne_model", vec![model("knot", "minecraft:leash_knot#main")]),
        entity("small-projectile-pose", "small_projectile_pose", rows),
    ]
}

fn vector(values: [f64; 3]) -> Vec<String> {
    values.into_iter().map(f64_bits).collect()
}

struct Strip {
    offset: [f64; 3],
    slack: bool,
    lights: [i64; 4],
    at: [f64; 3],
}

const STRIP: Strip = Strip { offset: [0.0, 1.0, 0.4], slack: true, lights: [15; 4], at: HERE };

fn strip(label: &str, start: [f64; 3], end: [f64; 3], strip: Strip) -> Value {
    let row = json!({
        "label": format!("leash-{label}"), "offset": vector(strip.offset), "start": vector(start), "end": vector(end),
        "start_block": strip.lights[0], "end_block": strip.lights[1], "start_sky": strip.lights[2],
        "end_sky": strip.lights[3], "slack": strip.slack,
    });
    placed(row, strip.at)
}

/// A leash's strip as the reference's own `LeashFeatureRenderer` builds it:
/// taut and slack, a hand above and below the entity, along each axis and
/// slantwise, near and far, lit alike and differently, tied at an offset.
pub fn leash() -> Vec<Value> {
    let (hand, up, down) = ([10.5, 65.0, -3.1], [12.5, 66.2, -1.0], [12.5, 63.5, -1.0]);
    let knot = [0.0, 1.0, 0.0];
    let taut = Strip { slack: false, ..STRIP };
    let rows = vec![
        strip("up-slack", hand, up, STRIP),
        strip("down-slack", hand, down, STRIP),
        strip("up-taut", hand, up, Strip { ..taut }),
        strip("down-taut", hand, down, taut),
        strip("level-slack", hand, [13.5, 65.0, -3.1], STRIP),
        strip("east", knot, [4.0, 1.5, 0.0], STRIP),
        strip("west", knot, [-4.0, 0.5, 0.0], STRIP),
        strip("south", knot, [0.0, 1.5, 4.0], STRIP),
        strip("north", knot, [0.0, 0.5, -4.0], STRIP),
        strip("slantwise", knot, [-3.25, 2.75, 5.5], STRIP),
        strip("straight-up", knot, [0.0, 4.0, 0.0], STRIP),
        strip("short", knot, [0.25, 1.1, 0.25], STRIP),
        strip("far", knot, [20.0, 3.0, -12.0], Strip { at: FAR, ..STRIP }),
        strip("lit-differently", knot, [4.0, 2.0, 1.0], Strip { lights: [0, 15, 15, 0], ..STRIP }),
        strip("dark-to-bright", knot, [4.0, 2.0, 1.0], Strip { lights: [3, 12, 1, 14], ..STRIP }),
        strip("no-offset", knot, [4.0, 2.0, 1.0], Strip { offset: [0.0; 3], ..STRIP }),
        strip("odd-offset", [0.3, 1.7, -0.2], [4.0, 2.0, 1.0], Strip { offset: [0.3, 1.7, -0.2], ..STRIP }),
        strip("on-the-spot", knot, knot, STRIP),
    ];
    vec![entity("leash-strip", "leash_strip", rows)]
}
