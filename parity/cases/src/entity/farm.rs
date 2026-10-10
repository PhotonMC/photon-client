//! Farm animals and other quadrupeds: sheep, chickens, rabbits, horses,
//! camels, llamas and goats.

use serde_json::{Value, json};

use super::pose::{Field, Fields, LOOK_WALK, entity, fields, layer_row};
use crate::bits::f32_bits;
use crate::line::float_text;

fn with(base: &[(&'static str, Field)], extra: Fields) -> Vec<(&'static str, Field)> {
    [base, extra].concat()
}

fn float(set: Fields, key: &str) -> Option<f64> {
    set.iter().find(|(name, _)| *name == key).and_then(|(_, value)| match value {
        Field::Float(value) => Some(*value),
        Field::Int(value) => Some(*value as f64),
        _ => None,
    })
}

/// A sheep's models from the layer table (the body, the undercoat and the
/// wool, grown and baby), posed by the reference's `SheepModel`,
/// `BabySheepModel` and `SheepFurModel` `setupAnim`: standing, looking,
/// walking, with the head down and tipped while eating.
pub fn sheep() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(eat_position = 0.0, age_scale = 1.0));
    let mut rows = Vec::new();
    let layers = [
        ("minecraft:sheep#main", 1.0),
        ("minecraft:sheep#wool", 1.0),
        ("minecraft:sheep#wool_undercoat", 1.0),
        ("minecraft:sheep_baby#main", 0.5),
        ("minecraft:sheep_baby#wool", 0.5),
    ];
    for (layer, scale) in layers {
        let poses: [(&str, Fields); 5] = [
            ("standing", fields!()),
            ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
            ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
            ("eating-down", fields!(eat_position = 1.0, eat_angle = 0.62831855)),
            (
                "eating-rising",
                fields!(eat_position = 0.375, eat_angle = 0.5, x_rot = 20.0, walk_position = 2.0, walk_speed = 0.2),
            ),
        ];
        for (label, set) in poses {
            // Not eating, the angle is the pitch.
            let angle = float(set, "eat_angle").unwrap_or(float(set, "x_rot").unwrap_or(0.0) * 0.017453292);
            let mut row = layer_row(layer, label, &defaults, &[set, fields!(age_scale = scale)].concat());
            row["eat_angle"] = json!(f32_bits(angle));
            rows.push(row);
        }
    }
    vec![entity("sheep-model", "sheep_model", rows)]
}

/// A chicken's models from the layer table (grown, cold and baby), posed by
/// the reference's `ChickenModel.setupAnim` with the wings' flap, drawn.
pub fn chicken() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(flap = 0.0, flap_speed = 0.0));
    let mut rows = Vec::new();
    for layer in ["minecraft:chicken#main", "minecraft:cold_chicken#main", "minecraft:chicken_baby#main"] {
        let poses: [(&str, Fields); 5] = [
            ("standing", fields!()),
            ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
            ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
            ("flapping", fields!(flap = 3.7, flap_speed = 1.0)),
            (
                "flapping-slowing",
                fields!(
                    flap = 11.25,
                    flap_speed = 0.4,
                    walk_position = 2.0,
                    walk_speed = 0.2,
                    head_y_rot = 12.0,
                    x_rot = -8.0
                ),
            ),
        ];
        rows.extend(poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)));
    }
    vec![entity("chicken-model", "chicken_model", rows)]
}

/// What a sheep's wool is multiplied by: the reference's table for each of
/// the sixteen dyes, and a `jeb_` sheep's colour along its cycle.
pub fn sheep_colour() -> Vec<Value> {
    let mut rows: Vec<Value> = (0..16).map(|dye| json!({"label": format!("dye-{dye}"), "dye": dye})).collect();
    rows.extend((0..16).map(|dye| json!({"label": format!("texture-dye-{dye}"), "texture_dye": dye})));
    for tick in [0.0, 0.5, 12.25, 24.9, 25.0, 31.5, 100.1, 399.0, 400.0, 401.7, 1234.567] {
        rows.push(json!({"label": format!("jeb-{}", float_text(tick)), "tick": f32_bits(tick)}));
    }
    vec![entity("sheep-colors", "sheep_colors", rows)]
}

/// How far a sheep's head is down and how it is tipped while it eats, from
/// the reference's own `Sheep` with its timer and pitch set.
pub fn sheep_eat() -> Vec<Value> {
    let mut rows = Vec::new();
    for tick in [0, 1, 3, 4, 5, 20, 36, 37, 39, 40] {
        for partial in [0.0, 0.25, 1.0] {
            rows.push(json!({
                "label": format!("eat-{tick}-{}", float_text(partial)), "eat_tick": tick,
                "partial": f32_bits(partial), "x_rot": f32_bits(17.5),
            }));
        }
    }
    rows.push(json!({
        "label": "pitch-down", "eat_tick": 0, "partial": f32_bits(0.5), "x_rot": f32_bits(-45.0),
    }));
    vec![entity("sheep-eat", "sheep_eat", rows)]
}

/// The rabbit, grown and baby, posed by the reference's own
/// `RabbitModel.setupAnim` with its hop and idle head tilt animation states
/// started at different ticks.
pub fn rabbit() -> Vec<Value> {
    let defaults = fields!(head_y_rot = 0.0, x_rot = 0.0, age_in_ticks = 0.0);
    let mut rows = Vec::new();
    for layer in ["minecraft:rabbit#main", "minecraft:rabbit_baby#main"] {
        let poses: [(&str, Fields); 6] = [
            ("standing", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
            ("hopping-early", fields!(head_y_rot = 12.0, x_rot = -8.0, age_in_ticks = 14.5, hop_start = 10)),
            ("hopping-late", fields!(age_in_ticks = 40.0, hop_start = 10)),
            ("idle-tilting", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 60.25, idle_start = 20)),
            ("idle-tilting-late", fields!(age_in_ticks = 120.0, idle_start = 20)),
            ("both", fields!(head_y_rot = 12.0, x_rot = -8.0, age_in_ticks = 50.0, hop_start = 30, idle_start = 20)),
        ];
        rows.extend(poses.iter().map(|(label, set)| layer_row(layer, label, defaults, set)));
    }
    vec![entity("rabbit-model", "rabbit_model", rows)]
}

/// The horse family from the layer table, posed by the reference's own
/// `AbstractEquineModel.setupAnim` (and the baby and donkey overrides):
/// standing, walking, galloping in water, grazing, rearing, the mouth open,
/// the tail swishing, the donkey's chests.
pub fn equine() -> Vec<Value> {
    let defaults = with(
        &LOOK_WALK,
        fields!(
            age_in_ticks = 0.0,
            eat = 0.0,
            stand = 0.0,
            feeding = 0.0,
            animate_tail = false,
            in_water = false,
            has_chest = false
        ),
    );
    let poses: [(&str, Fields); 10] = [
        ("standing", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0, animate_tail = true)),
        ("slow-walk", fields!(walk_position = 2.1, walk_speed = 0.15, age_in_ticks = 12.0)),
        ("swimming", fields!(walk_position = 7.3, walk_speed = 0.6, age_in_ticks = 40.0, in_water = true)),
        ("grazing-half", fields!(eat = 0.5, age_in_ticks = 21.5)),
        ("grazing", fields!(eat = 1.0, head_y_rot = 12.0, x_rot = -8.0, age_in_ticks = 33.0)),
        ("rearing-half", fields!(stand = 0.4, walk_speed = 0.2, age_in_ticks = 9.0)),
        ("rearing", fields!(stand = 1.0, age_in_ticks = 64.0, has_chest = true)),
        ("mouth-open", fields!(feeding = 1.0, age_in_ticks = 17.0, has_chest = true)),
        ("rearing-and-grazing", fields!(stand = 0.5, eat = 0.5, feeding = 0.7, age_in_ticks = 50.0)),
    ];
    let layers = [
        "minecraft:horse#main",
        "minecraft:horse_baby#main",
        "minecraft:donkey#main",
        "minecraft:donkey_baby#main",
        "minecraft:mule#main",
        "minecraft:mule_baby#main",
        "minecraft:zombie_horse#main",
        "minecraft:skeleton_horse_baby#main",
    ];
    let rows = layers
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)))
        .collect();
    vec![entity("equine-model", "equine_model", rows)]
}

/// The camel, grown and baby, posed by the reference's own
/// `CamelModel.setupAnim`: the head within its limits, dipping while the
/// dash cools down, the walk and each animation state.
pub fn camel() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(age_in_ticks = 0.0, jump_cooldown = 0.0));
    let poses: [(&str, Fields); 12] = [
        ("standing", fields!(head_y_rot = -50.0, x_rot = 60.0, age_in_ticks = 3.25)),
        ("looking", fields!(head_y_rot = 12.0, x_rot = -40.0, age_in_ticks = 3.25)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)),
        ("walking-fast", fields!(walk_position = 21.6, walk_speed = 1.0, age_in_ticks = 63.0)),
        ("dash-cooldown", fields!(x_rot = 10.0, jump_cooldown = 30.0, age_in_ticks = 9.0)),
        ("dash-cooldown-long", fields!(x_rot = 60.0, jump_cooldown = 55.0, age_in_ticks = 9.0)),
        ("sitting-down", fields!(age_in_ticks = 30.0, sit = 10)),
        ("sitting", fields!(age_in_ticks = 130.0, sit_pose = 10, walk_position = 3.0, walk_speed = 0.4)),
        ("standing-up", fields!(age_in_ticks = 45.0, sit_up = 20)),
        ("idle", fields!(age_in_ticks = 77.5, idle = 30, head_y_rot = 20.0)),
        ("dashing", fields!(age_in_ticks = 33.0, dash = 12, walk_position = 4.0, walk_speed = 0.9)),
        ("dashing-and-idle", fields!(age_in_ticks = 61.25, dash = 12, idle = 44)),
    ];
    let rows = ["minecraft:camel#main", "minecraft:camel_baby#main"]
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)))
        .collect();
    vec![entity("camel-model", "camel_model", rows)]
}

/// The llama, grown and baby, posed by the reference's own
/// `LlamaModel.setupAnim`: the head, the diagonal stride, the chests.
pub fn llama() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(has_chest = false));
    let poses: [(&str, Fields); 5] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("walking-fast-with-a-chest", fields!(walk_position = 21.6, walk_speed = 1.0, has_chest = true)),
        ("chest", fields!(head_y_rot = 12.0, x_rot = -8.0, has_chest = true)),
    ];
    let rows = ["minecraft:llama#main", "minecraft:llama_baby#main"]
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)))
        .collect();
    vec![entity("llama-model", "llama_model", rows)]
}

/// The goat, grown and baby, posed by the reference's own
/// `GoatModel.setupAnim` (the quadruped pose, the horns it still has, the
/// head lowered to ram) and drawn.
pub fn goat() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(left_horn = true, right_horn = true, ramming = 0.0));
    let poses: [(&str, Fields); 7] = [
        ("standing", fields!()),
        ("walking-and-looking", fields!(head_y_rot = 12.0, x_rot = -8.0, walk_position = 7.3, walk_speed = 0.83)),
        ("no-left-horn", fields!(left_horn = false)),
        ("no-right-horn", fields!(right_horn = false)),
        ("hornless", fields!(left_horn = false, right_horn = false)),
        ("ramming-half", fields!(head_y_rot = -35.5, x_rot = 48.0, ramming = 0.2617994)),
        ("ramming-full", fields!(walk_position = 3.4, walk_speed = 0.6, ramming = 0.5235988)),
    ];
    let rows = ["minecraft:goat#main", "minecraft:goat_baby#main"]
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)))
        .collect();
    vec![entity("goat-model", "goat_model", rows)]
}
