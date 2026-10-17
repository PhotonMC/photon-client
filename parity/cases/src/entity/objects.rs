//! Boats, minecarts, item frames, displays, chests, shulker boxes, bells,
//! skulls, banners and the elder guardian particle.

use serde_json::{Value, json};

use super::pose::{Fields, entity, fields, row};
use crate::bits::{f32_bits, f64_bits};
use crate::case::merge;
use crate::line::float_text;

pub const HERE: [f64; 3] = [10.5, 64.0, -3.5];
pub const FAR: [f64; 3] = [-100.25, 70.5, 33.75];
const FULL_LIGHT: i64 = 15728880;
const DIM_LIGHT: i64 = 0xF000F0 - 17;

/// The `feet_*` fields of a place.
pub fn feet(at: [f64; 3]) -> Value {
    json!({"feet_x": f32_bits(at[0]), "feet_y": f32_bits(at[1]), "feet_z": f32_bits(at[2])})
}

/// `row` with the `feet_*` fields of `at`.
pub fn placed(mut row: Value, at: [f64; 3]) -> Value {
    merge(&mut row, feet(at));
    row
}

fn boat_defaults() -> Vec<(&'static str, super::pose::Field)> {
    fields!(
        y_rot = 0.0,
        hurt_dir = 1,
        hurt_time = 0.0,
        damage_time = 0.0,
        bubble_angle = 0.0,
        under_water = false,
        rowing_left = 0.0,
        rowing_right = 0.0
    )
    .to_vec()
}

/// The boat models posed by the reference's own `setupAnim` (each paddle by
/// the time it has rowed), and where `AbstractBoatRenderer.submit` draws a
/// boat: turned, rocked by a hit, shaken by a bubble column (not under
/// water) and flipped.
pub fn boat() -> Vec<Value> {
    let defaults = boat_defaults();
    let mut models = Vec::new();
    for layer in ["boat/oak", "chest_boat/oak", "boat/bamboo", "chest_boat/bamboo", "boat/cherry"] {
        let name = layer.replace('/', "-");
        for (label, left, right) in
            [("still", 0.0, 0.0), ("rowing", 0.7, 1.9), ("rowed-far", 3.2, 5.5), ("rowing-one-side", 2.4, 0.0)]
        {
            let mut row = row(&format!("{name}-{label}"), &defaults, fields!(rowing_left = left, rowing_right = right));
            row["layer"] = json!(format!("minecraft:{layer}#main"));
            models.push(row);
        }
    }
    let poses: [(&str, Fields); 10] = [
        ("still", fields!()),
        ("turned", fields!(y_rot = 37.0)),
        ("turned-far", fields!(y_rot = -170.5)),
        ("hit", fields!(hurt_time = 7.5, damage_time = 12.0, hurt_dir = 1)),
        ("hit-the-other-way", fields!(hurt_time = 3.25, damage_time = 30.0, hurt_dir = -1)),
        ("hit-just-over", fields!(hurt_time = 0.0, damage_time = 12.0)),
        ("shaken-by-bubbles", fields!(bubble_angle = 6.5)),
        ("shaken-under-water", fields!(bubble_angle = 6.5, under_water = true)),
        ("shaken-a-little", fields!(bubble_angle = 0.000_005)),
        ("everything", fields!(y_rot = 90.0, hurt_time = 5.0, damage_time = 20.0, hurt_dir = -1, bubble_angle = -8.0)),
    ];
    let poses = poses.iter().map(|(label, set)| placed(row(label, &defaults, set), [10.5, 64.25, -3.75])).collect();
    vec![entity("boat-model", "boat_model", models), entity("boat-pose", "boat_pose", poses)]
}

type Rail = [[f64; 3]; 3];

/// A minecart: where it is and its rail, turns, id, hurt, block and the
/// display offset.
struct Cart {
    rail: Option<Rail>,
    at: [f64; 3],
    turns: (f64, f64),
    id: i64,
    hurt: (i64, f64, f64),
    block: bool,
    offset: i64,
}

const CART: Cart =
    Cart { rail: None, at: HERE, turns: (0.0, 0.0), id: 7, hurt: (0, 0.0, 0.0), block: false, offset: 6 };

fn vector(values: [f64; 3]) -> Vec<String> {
    values.into_iter().map(f64_bits).collect()
}

fn cart(label: &str, cart: Cart) -> Value {
    let seed = cart.id.wrapping_mul(493286711);
    let seed = seed.wrapping_mul(seed).wrapping_mul(4392167121).wrapping_add(seed.wrapping_mul(98761));
    let (x_rot, y_rot) = cart.turns;
    let mut row = json!({
        "label": format!("minecart-{label}"), "x": f64_bits(cart.at[0]), "y": f64_bits(cart.at[1]),
        "z": f64_bits(cart.at[2]), "x_rot": f32_bits(x_rot), "y_rot": f32_bits(y_rot),
        "offset_seed": seed.to_string(), "hurt_dir": cart.hurt.0, "hurt_time": f32_bits(cart.hurt.1),
        "damage_time": f32_bits(cart.hurt.2), "display_offset": cart.offset, "block": cart.block,
    });
    if let Some([pos, front, back]) = cart.rail {
        row["rail"] = json!({"pos": vector(pos), "front": vector(front), "back": vector(back)});
    }
    placed(row, cart.at)
}

fn tnt(label: &str, fuse: f64, cart_: Cart) -> Value {
    let mut row = cart(label, Cart { block: true, ..cart_ });
    row["tnt_fuse"] = json!(f32_bits(fuse));
    row
}

const SLOPE: Rail = [[10.5, 64.5, -3.5], [10.8, 64.8, -3.4], [10.2, 64.2, -3.6]];

/// Where the reference's own minecart renderer draws a minecart and the
/// block it carries: off a rail, along a rail (flat, along each axis,
/// sloping, at a corner), with the rocking of a hit, with and without a
/// block, and the primed tnt's fuse.
pub fn minecart_pose() -> Vec<Value> {
    let flat = 64.0625;
    let mut rows = vec![
        cart("off-rail", CART),
        cart("off-rail-turned", Cart { turns: (11.0, 123.0), ..CART }),
        cart("flat-along-x", Cart { rail: Some([[10.5, flat, -3.5], [10.8, flat, -3.5], [10.2, flat, -3.5]]), ..CART }),
        cart("flat-along-z", Cart { rail: Some([[10.5, flat, -3.5], [10.5, flat, -3.2], [10.5, flat, -3.8]]), ..CART }),
        cart("sloping-up", Cart { rail: Some([[10.5, 64.5, -3.5], [10.8, 64.8, -3.5], [10.2, 64.2, -3.5]]), ..CART }),
        cart("sloping-down", Cart { rail: Some([[10.5, 64.5, -3.5], [10.2, 64.8, -3.5], [10.8, 64.2, -3.5]]), ..CART }),
        cart("corner", Cart { rail: Some([[10.4, flat, -3.4], [10.7, flat, -3.5], [10.4, flat, -3.1]]), ..CART }),
        cart("ends-meet", Cart { rail: Some([[10.5, flat, -3.5]; 3]), ..CART }),
        cart("hit", Cart { hurt: (1, 7.5, 12.0), ..CART }),
        cart("hit-the-other-way", Cart { hurt: (-1, 3.25, 40.0), ..CART }),
        cart("hit-over", Cart { hurt: (1, -0.5, 12.0), ..CART }),
        cart("another-id", Cart { id: 123456, ..CART }),
        cart("with-a-block", Cart { block: true, ..CART }),
        cart("with-a-block-low", Cart { block: true, offset: 8, ..CART }),
        cart("with-a-block-high", Cart { block: true, offset: 1, ..CART }),
    ];
    for (label, fuse) in [
        ("tnt-unlit", -1.0),
        ("tnt-long-fuse", 40.0),
        ("tnt-fuse-ten", 10.0),
        ("tnt-fuse-nine", 9.0),
        ("tnt-fuse-five-lit", 5.0),
        ("tnt-fuse-four-lit", 4.99),
        ("tnt-fuse-three", 3.0),
        ("tnt-fuse-one", 1.0),
        ("tnt-about-to-blow", 0.25),
        ("tnt-at-zero", 0.0),
    ] {
        rows.push(tnt(label, fuse, CART));
    }
    rows.push(tnt("tnt-lit-on-a-rail", 7.5, Cart { rail: Some(SLOPE), hurt: (1, 4.0, 10.0), ..CART }));
    let everything =
        Cart { rail: Some(SLOPE), hurt: (-1, 5.0, 20.0), block: true, id: 99, turns: (4.0, -50.0), at: FAR, ..CART };
    rows.push(cart("everything", everything));
    // The new behaviour draws from the turns alone, whatever rail there is;
    // a hit and a block still apply.
    let new = [
        cart("new-flat", CART),
        cart("new-turned-and-sloped", Cart { turns: (-37.0, 123.0), ..CART }),
        cart("new-ignores-the-rail", Cart { rail: Some(SLOPE), turns: (22.5, -80.0), ..CART }),
        cart("new-hit-with-a-block", Cart { hurt: (-1, 5.0, 20.0), block: true, id: 99, turns: (10.0, 45.0), ..CART }),
    ];
    for mut row in new {
        row["new_render"] = json!(true);
        rows.push(row);
    }
    vec![entity("minecart-pose", "minecart_pose", rows)]
}

/// Where the reference's own item frame renderer draws a frame's model and
/// the item in it, for each way it faces, with and without the frame, the
/// item turned by each eighth, and a map in the item's place.
pub fn item_frame_pose() -> Vec<Value> {
    let defaults = fields!(invisible = false, rotation = 0, item = true, glow = false, map = false, light = FULL_LIGHT);
    let frame = |label: &str, direction: &str, set: Fields, at: [f64; 3]| {
        let mut row = row(&format!("item-frame-{label}"), defaults, set);
        row["direction"] = json!(direction);
        placed(row, at)
    };
    let at = [10.5, 64.25, -3.5];
    let mut rows: Vec<Value> = ["north", "south", "west", "east", "up", "down"]
        .iter()
        .map(|direction| frame(direction, direction, fields!(), at))
        .collect();
    for turn in 0..8 {
        rows.push(frame(&format!("turned-{turn}"), "east", fields!(rotation = turn), at));
    }
    rows.extend([
        frame("empty", "north", fields!(item = false), at),
        frame("invisible", "south", fields!(invisible = true), at),
        frame("invisible-up", "up", fields!(invisible = true, rotation = 3), at),
        frame("far", "down", fields!(rotation = 5), FAR),
    ]);
    // A frame that holds a map draws the map in the item's place, turned by
    // quarter turns, and lit as the map is.
    let map_light = 0x00A0_00C0;
    for turn in [0, 1, 2, 3, 5] {
        rows.push(frame(&format!("map-{turn}"), "north", fields!(rotation = turn, map = true, light = map_light), at));
    }
    rows.push(frame("map-glow", "up", fields!(map = true, glow = true, light = map_light, invisible = true), at));
    rows.push(frame("map-south", "south", fields!(map = true, rotation = 2), at));
    vec![entity("item-frame-pose", "item_frame_pose", rows)]
}

const QUARTER: [f64; 4] = [0.0, 0.7071068, 0.0, 0.7071068];
const TILTED: [f64; 4] = [0.2418448, 0.0, 0.0, 0.9703133];
const NO_TURN: [f64; 4] = [0.0, 0.0, 0.0, 1.0];

struct Display {
    kind: &'static str,
    billboard: i64,
    translation: [f64; 3],
    scale: [f64; 3],
    left: [f64; 4],
    right: [f64; 4],
    entity: (f64, f64),
    camera: (f64, f64),
    at: [f64; 3],
}

const DISPLAY: Display = Display {
    kind: "block",
    billboard: 0,
    translation: [0.0; 3],
    scale: [1.0; 3],
    left: NO_TURN,
    right: NO_TURN,
    entity: (33.0, -12.5),
    camera: (110.0, 25.0),
    at: [10.5, 64.25, -3.5],
};

fn display(label: &str, display: Display) -> Value {
    let bits = |values: &[f64]| values.iter().map(|value| f32_bits(*value)).collect::<Vec<_>>();
    let row = json!({
        "label": format!("display-{label}"), "kind": display.kind, "billboard": display.billboard,
        "translation": bits(&display.translation), "scale": bits(&display.scale),
        "left_rotation": bits(&display.left), "right_rotation": bits(&display.right),
        "entity_y_rot": f32_bits(display.entity.0), "entity_x_rot": f32_bits(display.entity.1),
        "camera_y_rot": f32_bits(display.camera.0), "camera_x_rot": f32_bits(display.camera.1),
    });
    placed(row, display.at)
}

/// Where the reference's own display renderers draw a block or an item
/// display: each billboard constraint, a transformation that keeps normals,
/// one that does not, and the item's turn about.
pub fn display_pose() -> Vec<Value> {
    let mut rows: Vec<Value> = ["fixed", "vertical", "horizontal", "center"]
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let shape = Display { billboard: index as i64, translation: [0.25, -0.5, 1.0], left: QUARTER, ..DISPLAY };
            display(&format!("billboard-{name}"), shape)
        })
        .collect();
    rows.extend([
        display("uniform-scale", Display { scale: [2.0; 3], left: TILTED, translation: [-1.0, 0.5, 0.0], ..DISPLAY }),
        display(
            "squashed",
            Display { scale: [2.0, 0.5, 1.0], left: QUARTER, right: TILTED, translation: [0.0, 1.0, 0.0], ..DISPLAY },
        ),
        display(
            "squashed-center",
            Display { billboard: 3, scale: [1.0, 3.0, 0.25], left: TILTED, right: QUARTER, ..DISPLAY },
        ),
        display("mirrored", Display { scale: [-1.0, 1.0, 1.0], left: TILTED, ..DISPLAY }),
        display(
            "item",
            Display {
                kind: "item",
                billboard: 3,
                translation: [0.0, 0.5, 0.0],
                scale: [0.5; 3],
                left: TILTED,
                ..DISPLAY
            },
        ),
        display("item-fixed", Display { kind: "item", right: QUARTER, ..DISPLAY }),
        display("no-turns", Display { entity: (0.0, 0.0), camera: (0.0, 0.0), ..DISPLAY }),
        display("far", Display { scale: [1.5; 3], left: QUARTER, at: FAR, ..DISPLAY }),
    ]);
    vec![entity("display-pose", "display_pose", rows)]
}

/// The light the reference's own display renderer uses when the server set
/// a brightness: its block and sky levels, whatever else the number holds.
pub fn display_light() -> Vec<Value> {
    let rows = [
        ("both", (15 << 4) | (10 << 20)),
        ("dark", 0),
        ("block-only", 3 << 4),
        ("sky-only", 7 << 20),
        ("stray-bits", 0x7fff_ffff),
        ("negative", -2),
    ]
    .iter()
    .map(|(label, value)| json!({"label": format!("display-light-{label}"), "brightness": value}))
    .collect();
    vec![entity("display-light", "display_light", rows)]
}

fn chest(label: String, (kind, facing, material): (&str, &str, &str), open: f64, light: i64, at: [f64; 3]) -> Value {
    json!({
        "label": label, "type": kind, "facing": facing, "material": material, "open": f32_bits(open), "light": light,
        "x": f32_bits(at[0]), "y": f32_bits(at[1]), "z": f32_bits(at[2]),
    })
}

/// The chest's three models with the lid shut and at a few openings, and
/// where the real `ChestRenderer` draws a chest of each type, material and
/// facing, at each openness.
pub fn chest_cases() -> Vec<Value> {
    let mut models = Vec::new();
    for layer in ["minecraft:chest#main", "minecraft:double_chest_left#main", "minecraft:double_chest_right#main"] {
        for open in [0.0, 0.25, 0.7, 1.0] {
            models.push(
                json!({"label": format!("{layer} {}", float_text(open)), "layer": layer, "open": f32_bits(open)}),
            );
        }
    }
    let mut poses = Vec::new();
    for facing in ["north", "east", "south", "west"] {
        for kind in ["single", "left", "right"] {
            poses.push(chest(format!("{facing} {kind}"), (kind, facing, "REGULAR"), 0.4, DIM_LIGHT, [3.0, -1.25, 7.5]));
        }
    }
    for repeat in 0..3 {
        poses.push(chest(format!("repeat {repeat}"), ("single", "south", "REGULAR"), 0.4, 240, [1.0; 3]));
    }
    let materials = [
        "ENDER_CHEST",
        "CHRISTMAS",
        "TRAPPED",
        "COPPER_UNAFFECTED",
        "COPPER_EXPOSED",
        "COPPER_WEATHERED",
        "COPPER_OXIDIZED",
    ];
    for material in materials {
        for kind in ["single", "left", "right"] {
            poses.push(chest(format!("{material} {kind}"), (kind, "south", material), 0.0, FULL_LIGHT, [0.0; 3]));
        }
    }
    for (index, open) in [0.0, 0.1, 0.5, 0.9, 1.0].into_iter().enumerate() {
        poses.push(chest(format!("open {index}"), ("single", "north", "TRAPPED"), open, 240, [-4.5, 2.0, -0.5]));
    }
    vec![entity("chest-model", "chest_model", models), entity("chest-pose", "chest_pose", poses)]
}

pub const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// Where the real `ShulkerBoxRenderer` draws a box of each colour, facing
/// and lid progress, and the model with the lid set for it.
pub fn shulker_box() -> Vec<Value> {
    let boxed = |label: String, color: Option<&str>, facing: &str, progress: f64, light: i64, at: [f64; 3]| {
        json!({
            "label": label, "color": color, "facing": facing, "progress": f32_bits(progress), "light": light,
            "x": f32_bits(at[0]), "y": f32_bits(at[1]), "z": f32_bits(at[2]),
        })
    };
    let mut rows = Vec::new();
    for facing in ["down", "up", "north", "south", "west", "east"] {
        for progress in [0.0, 0.3, 1.0] {
            let label = format!("{facing} {}", float_text(progress));
            rows.push(boxed(label, None, facing, progress, DIM_LIGHT, [3.0, -1.25, 7.5]));
        }
    }
    for color in DYES {
        rows.push(boxed(color.to_owned(), Some(color), "up", 0.5, FULL_LIGHT, [0.0; 3]));
    }
    vec![entity("shulker-box-pose", "shulker_box_pose", rows)]
}

/// The real `BellRenderer` model for a bell at rest and swinging from each
/// side, through the fifty ticks of a ring.
pub fn bell() -> Vec<Value> {
    let mut rows = Vec::new();
    for shake in [None, Some("north"), Some("south"), Some("east"), Some("west"), Some("up")] {
        for ticks in [0.0, 0.5, 3.25, 10.0, 24.75, 49.9] {
            rows.push(json!({
                "label": format!("{} {}", shake.unwrap_or("None"), float_text(ticks)), "ticks": f32_bits(ticks),
                "shake": shake, "light": DIM_LIGHT,
            }));
        }
    }
    vec![entity("bell-pose", "bell_pose", rows)]
}

/// The real `EnchantTableRenderer` for a table whose book is at several numbers: its flip either
/// side of the clamps on the two loose pages, shut, half open and open, a turn that wraps at the
/// edge of the circle, and several partial ticks.
pub fn enchant_table() -> Vec<Value> {
    let book = |label: String, [time, flip, o_flip, open, o_open, rot, o_rot, partial]: [f64; 8]| {
        json!({
            "label": label, "time": time as i64, "flip": f32_bits(flip), "o_flip": f32_bits(o_flip),
            "open": f32_bits(open), "o_open": f32_bits(o_open), "rot": f32_bits(rot), "o_rot": f32_bits(o_rot),
            "partial": f32_bits(partial),
        })
    };
    let mut rows = Vec::new();
    for flip in [-1.3, 0.0, 0.1, 0.3, 0.55, 0.9, 2.45, 7.8] {
        rows.push(book(format!("flip {}", float_text(flip)), [17.0, flip, flip - 0.2, 0.7, 0.6, 0.4, 0.3, 0.37]));
    }
    for open in [0.0, 0.25, 0.5, 1.0] {
        rows.push(book(format!("open {}", float_text(open)), [1000.0, 1.7, 1.5, open, open - 0.1, -1.2, -1.0, 0.8]));
    }
    for (rot, o_rot) in [(3.1, -3.1), (-3.1, 3.1), (3.0, 3.1), (-0.2, 0.2), (2.0, -2.5)] {
        for partial in [0.0, 0.5, 1.0] {
            let label = format!("turn {} from {} at {}", float_text(rot), float_text(o_rot), float_text(partial));
            rows.push(book(label, [0.0, 0.0, 0.0, 1.0, 1.0, rot, o_rot, partial]));
        }
    }
    for time in [0.0, 5.0, 31.0, 157.0, 4000.0] {
        rows.push(book(format!("time {}", float_text(time)), [time, 0.45, 0.4, 0.9, 0.8, 1.0, 0.9, 0.25]));
    }
    vec![entity("enchant-table-pose", "enchant_table_pose", rows)]
}

/// The real `LecternRenderer` for a lectern facing each way, with a book and without.
pub fn lectern() -> Vec<Value> {
    let mut rows = Vec::new();
    for facing in ["north", "south", "west", "east"] {
        for has_book in [true, false] {
            let label = format!("{facing} {}", if has_book { "book" } else { "no book" });
            rows.push(json!({"label": label, "facing": facing, "has_book": has_book}));
        }
    }
    vec![entity("lectern-pose", "lectern_pose", rows)]
}

/// The real `SkullBlockRenderer` for each skull standing at each of the
/// sixteen turns and hanging on each wall, with the jaw and ears at several
/// animation times.
pub fn skull_block() -> Vec<Value> {
    let skull = |label: String, kind: &str, wall: Option<&str>, rotation: i64, animation: f64, light: i64| {
        json!({
            "label": label, "kind": kind, "wall": wall, "rotation": rotation, "animation": f32_bits(animation),
            "light": light,
        })
    };
    let mut rows = Vec::new();
    for kind in ["SKELETON", "WITHER_SKELETON", "PLAYER", "ZOMBIE", "CREEPER", "PIGLIN", "DRAGON"] {
        for rotation in 0..16 {
            rows.push(skull(format!("{kind} floor {rotation}"), kind, None, rotation, 0.0, DIM_LIGHT));
        }
        for wall in ["north", "south", "east", "west"] {
            rows.push(skull(format!("{kind} wall {wall}"), kind, Some(wall), 0, 0.0, 240));
        }
    }
    for kind in ["PIGLIN", "DRAGON"] {
        for animation in [0.5, 7.25, 33.0, 120.5] {
            let label = format!("{kind} animated {}", float_text(animation));
            rows.push(skull(label, kind, None, 4, animation, FULL_LIGHT));
        }
    }
    vec![entity("skull-block-pose", "skull_block_pose", rows)]
}

/// The pose and colour of the elder guardian particle over the whole of its
/// life, the camera looking every way.
pub fn elder_guardian_particle() -> Vec<Value> {
    let particle = |label: String, age: i64, (yaw, pitch): (f64, f64), partial: f64| {
        json!({
            "label": label, "age": age, "lifetime": 30, "yaw": f32_bits(yaw), "pitch": f32_bits(pitch),
            "partial": f32_bits(partial),
        })
    };
    let mut rows = Vec::new();
    for age in [0, 1, 7, 15, 23, 29, 30] {
        for (yaw, pitch) in [(0.0, 0.0), (90.0, 0.0), (-135.5, 30.25), (41.0, -77.5)] {
            let label = format!("age {age} yaw {} pitch {}", float_text(yaw), float_text(pitch));
            rows.push(particle(label, age, (yaw, pitch), 0.4));
        }
    }
    for partial in [0.0, 0.25, 0.999] {
        rows.push(particle(format!("partial {}", float_text(partial)), 10, (12.0, -3.0), partial));
    }
    vec![entity("elder-guardian-particle-pose", "elder_guardian_particle_pose", rows)]
}
