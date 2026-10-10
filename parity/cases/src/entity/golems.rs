//! Golems, the enderman, insects, and the layers that hold things on
//! villagers, witches and copper golems.

use serde_json::{Value, json};

use super::pose::{Field, Fields, LOOK_WALK, entity, fields, layer_row, row};
use crate::bits::f32_bits;
use crate::line::float_text;

fn with(base: &[(&'static str, Field)], extra: Fields) -> Vec<(&'static str, Field)> {
    [base, extra].concat()
}

fn rows(defaults: Fields, poses: &[(&str, Fields)]) -> Vec<Value> {
    poses.iter().map(|(label, set)| row(label, defaults, set)).collect()
}

/// The golems and the enderman posed by the reference's own `setupAnim`;
/// the pose the iron golem's renderer adds while it walks and the
/// crackiness level a health fraction gives; the swirl, eyes and block
/// layers.
pub fn golem() -> Vec<Value> {
    let look = with(&LOOK_WALK, fields!(age_in_ticks = 0.0));
    let iron = with(&look, fields!(attack_ticks = 0.0, offer_flower_tick = 0));
    let iron_poses: [(&str, Fields); 9] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83)),
        ("walking-fast", fields!(walk_position = 21.6, walk_speed = 1.0, head_y_rot = 12.0, x_rot = -8.0)),
        ("attacking-early", fields!(attack_ticks = 9.5, walk_position = 7.3, walk_speed = 0.83)),
        ("attacking-late", fields!(attack_ticks = 2.25)),
        ("offering-a-flower", fields!(offer_flower_tick = 400, walk_position = 7.3, walk_speed = 0.83)),
        ("offering-a-flower-late", fields!(offer_flower_tick = 33)),
        ("attacking-while-offering", fields!(attack_ticks = 4.0, offer_flower_tick = 100)),
    ];
    let snow: [(&str, Fields); 3] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("looking-the-other-way", fields!(head_y_rot = 120.0, x_rot = -20.0)),
    ];
    let enderman = with(&look, fields!(attack_time = 0.0, swim_amount = 0.0, creepy = false, carrying = false));
    let enderman_poses: [(&str, Fields); 6] = [
        ("standing", fields!()),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)),
        ("walking-fast-looking", fields!(walk_position = 21.6, walk_speed = 1.0, head_y_rot = 12.0, x_rot = -8.0)),
        ("creepy", fields!(head_y_rot = -35.5, x_rot = 48.0, creepy = true)),
        ("carrying", fields!(walk_position = 7.3, walk_speed = 0.83, carrying = true)),
        ("carrying-and-creepy", fields!(creepy = true, carrying = true)),
    ];
    let mut swirls = Vec::new();
    for age in [0.0, 7.5, 123.25, 1000.5, 99999.75] {
        for powered in [true, false] {
            let state = if powered { "charged" } else { "plain" };
            swirls.push(json!({
                "label": format!("age-{}-{state}", float_text(age)), "age_in_ticks": f32_bits(age), "powered": powered,
            }));
        }
    }
    let eyes = vec![json!({"label": "spider", "layer": "spider"}), json!({"label": "enderman", "layer": "enderman"})];
    vec![
        entity("iron-golem-model", "iron_golem_model", rows(&iron, &iron_poses)),
        entity("snow-golem-model", "snow_golem_model", rows(&look, &snow)),
        entity("enderman-model", "enderman_model", rows(&enderman, &enderman_poses)),
        // What the charged creeper's swirl layer submits, at several ages.
        entity("swirl-layers", "swirl_layers", swirls),
        // What the glowing-eyes layers submit: the draw order, the texture,
        // the pipeline's state, the light and the overlay.
        entity("eyes-layers", "eyes_layers", eyes),
        entity("block-layers", "entity_block_layers", block_layers()),
        entity("iron-golem-render", "iron_golem_render", iron_golem_render()),
    ]
}

/// Where the layers that draw a block on an entity put it: the enderman's
/// carried block, the snow golem's pumpkin (on the head's pose) and the
/// iron golem's flower (on the right arm's pose).
fn block_layers() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(attack_ticks = 0.0, offer_flower_tick = 0));
    let looking = fields!(head_y_rot = -35.5, x_rot = 48.0);
    let layers: [(&str, &str, Fields); 10] = [
        ("carried", "carried", fields!()),
        ("mushrooms-straight", "mushrooms", fields!()),
        ("mushrooms-looking", "mushrooms", looking),
        ("pumpkin-straight", "snow_head", fields!()),
        ("pumpkin-looking", "snow_head", looking),
        ("pumpkin-turned", "snow_head", fields!(head_y_rot = 120.0, x_rot = -20.0)),
        ("flower-offered", "flower", fields!(offer_flower_tick = 400)),
        ("flower-offered-late", "flower", fields!(offer_flower_tick = 33, head_y_rot = 12.0, x_rot = -8.0)),
        ("flower-while-striking", "flower", fields!(attack_ticks = 4.0, offer_flower_tick = 100)),
        ("flower-while-walking", "flower", fields!(offer_flower_tick = 70, walk_position = 7.3, walk_speed = 0.83)),
    ];
    layers
        .iter()
        .map(|(label, layer, set)| {
            let mut row = row(label, &defaults, set);
            row["layer"] = json!(layer);
            row
        })
        .collect()
}

fn iron_golem_render() -> Vec<Value> {
    let render = |label: String, walk_position: f64, walk_speed: f64, body_rot: f64, fraction: f64| {
        json!({
            "label": label, "walk_position": f32_bits(walk_position), "walk_speed": f32_bits(walk_speed),
            "body_rot": f32_bits(body_rot), "fraction": f32_bits(fraction),
        })
    };
    let mut rows = vec![
        render("still".to_owned(), 3.0, 0.0, 30.0, 1.0),
        render("barely-moving".to_owned(), 3.0, 0.009, 30.0, 1.0),
        render("walk-start".to_owned(), 0.0, 0.5, 90.0, 0.8),
        render("walk-mid".to_owned(), 7.3, 0.83, 90.0, 0.75),
        render("walk-late".to_owned(), 21.6, 1.0, -45.0, 0.74),
        render("walk-far".to_owned(), 1234.5, 0.6, 200.0, 0.5),
    ];
    for fraction in [0.0, 0.1, 0.2499, 0.25, 0.4999, 0.5, 0.7499, 0.75, 0.9, 1.0] {
        rows.push(render(format!("health-{}", float_text(fraction)), 5.0, 0.7, 10.0, fraction));
    }
    rows
}

/// Where the layers that hold something on a villager, a wandering trader
/// and a witch put it: the crossed arms' item, the witch's item and potion,
/// and the head item, from the reference's own layer classes.
pub fn villager_held() -> Vec<Value> {
    let defaults = fields!(
        baby = false,
        head_y_rot = 0.0,
        x_rot = 0.0,
        age_in_ticks = 0.0,
        unhappy = false,
        entity_id = 0,
        villager_transforms = true
    );
    let layers: [(&str, &str, Fields); 9] = [
        ("crossed-arms", "crossed", fields!()),
        ("crossed-arms-baby", "crossed", fields!(baby = true)),
        ("witch-item", "witch_item", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 31.5, entity_id = 7)),
        ("witch-potion", "witch_potion", fields!(age_in_ticks = 31.5, entity_id = 7)),
        (
            "witch-potion-looking",
            "witch_potion",
            fields!(head_y_rot = 12.0, x_rot = -8.0, age_in_ticks = 9.0, entity_id = 13),
        ),
        ("head-villager", "villager_head", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 3.25)),
        ("head-villager-unhappy", "villager_head", fields!(unhappy = true, age_in_ticks = 17.5)),
        ("head-baby-villager", "villager_head", fields!(baby = true, head_y_rot = 12.0, x_rot = -8.0)),
        ("head-trader", "villager_head", fields!(villager_transforms = false, head_y_rot = -35.5, x_rot = 48.0)),
    ];
    let rows = layers
        .iter()
        .map(|(label, kind, set)| {
            let mut row = row(label, defaults, set);
            row["kind"] = json!(kind);
            row
        })
        .collect();
    vec![entity("villager-held-layers", "villager_held_layers", rows)]
}

/// The copper golem posed by the reference's own
/// `CopperGolemModel.setupAnim`: the look, the walk (with arms raised while
/// an item is held), the idle spin and the four chest animations, each
/// started at a tick and read at a later age.
pub fn copper_golem() -> Vec<Value> {
    let defaults = with(&LOOK_WALK, fields!(age_in_ticks = 0.0, holds_item = false));
    let poses: [(&str, Fields); 8] = [
        ("standing", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.31, age_in_ticks = 20.5)),
        ("running", fields!(walk_position = 19.75, walk_speed = 0.9, head_y_rot = 12.0, x_rot = -8.0)),
        ("holding-still", fields!(holds_item = true)),
        ("holding-looking", fields!(holds_item = true, head_y_rot = 20.0, x_rot = 15.0)),
        ("holding-walking-slowly", fields!(holds_item = true, walk_position = 3.1, walk_speed = 0.12)),
        ("holding-walking", fields!(holds_item = true, walk_position = 8.9, walk_speed = 0.45)),
        ("holding-running", fields!(holds_item = true, walk_position = 21.4, walk_speed = 1.0)),
    ];
    let mut rows = rows(&defaults, &poses);
    for age in [10.0, 11.5, 14.25, 19.0, 30.0, 55.5, 80.0] {
        rows.push(row(&format!("idle-{}", float_text(age)), &defaults, fields!(age_in_ticks = age, idle = 10)));
    }
    for key in ["get_item", "get_no_item", "drop_item", "drop_no_item"] {
        for age in [5.0, 12.5, 25.0, 41.75, 70.0, 110.0, 190.0] {
            let set = [("age_in_ticks", Field::Float(age)), (key, Field::Int(5))];
            rows.push(row(&format!("{key}-{}", float_text(age)), &defaults, &set));
        }
    }
    rows.push(row(
        "walking-and-idle",
        &defaults,
        fields!(age_in_ticks = 24.0, walk_position = 6.0, walk_speed = 0.5, idle = 10),
    ));
    rows.push(row(
        "holding-and-dropping",
        &defaults,
        fields!(
            age_in_ticks = 33.0,
            holds_item = true,
            walk_position = 2.0,
            walk_speed = 0.2,
            drop_item = 5,
            head_y_rot = -14.0,
            x_rot = 6.0
        ),
    ));
    vec![entity("copper-golem-model", "copper_golem_model", rows)]
}

/// Where the layers that carry things on a copper golem put them, on the
/// model as posed, idle and at a chest, for the look and walk.
pub fn copper_golem_layer() -> Vec<Value> {
    let mut rows = Vec::new();
    for layer in ["hands", "head", "antenna"] {
        for action in ["IDLE", "GETTING_ITEM", "DROPPING_NO_ITEM"] {
            let poses = [
                ("standing", (0.0, 0.0), (0.0, 0.0)),
                ("looking", (-35.5, 48.0), (0.0, 0.0)),
                ("walking", (12.0, -8.0), (7.3, 0.6)),
            ];
            for (label, look, walk) in poses {
                rows.push(json!({
                    "label": format!("{layer}-{}-{label}", action.to_lowercase()), "layer": layer, "action": action,
                    "head_y_rot": f32_bits(look.0), "x_rot": f32_bits(look.1), "walk_position": f32_bits(walk.0),
                    "walk_speed": f32_bits(walk.1), "age_in_ticks": f32_bits(21.5),
                }));
            }
        }
    }
    vec![entity("copper-golem-layers", "copper_golem_layers", rows)]
}

/// The silverfish, the endermite and the bee, posed by the reference's own
/// `setupAnim` (the segments' wave, the bee's wings, legs and bobbing, its
/// roll and its stinger).
pub fn insect() -> Vec<Value> {
    let mut segmented = Vec::new();
    for layer in ["minecraft:silverfish#main", "minecraft:endermite#main"] {
        for (label, age) in [("start", 0.0), ("early", 3.5), ("late", 47.25), ("long", 1234.5)] {
            segmented.push(json!({"label": format!("{layer}-{label}"), "layer": layer, "age_in_ticks": f32_bits(age)}));
        }
    }
    let defaults = fields!(age_in_ticks = 0.0, roll_amount = 0.0, has_stinger = true, on_ground = false, angry = false);
    let poses: [(&str, Fields); 7] = [
        ("flying", fields!(age_in_ticks = 12.5)),
        ("flying-later", fields!(age_in_ticks = 70.25)),
        ("angry", fields!(age_in_ticks = 33.0, angry = true)),
        ("on-the-ground", fields!(age_in_ticks = 33.0, on_ground = true)),
        ("stung", fields!(age_in_ticks = 33.0, has_stinger = false)),
        ("rolling-half", fields!(age_in_ticks = 21.0, roll_amount = 0.5, angry = true)),
        ("rolling-full", fields!(age_in_ticks = 21.0, roll_amount = 1.0, angry = true, has_stinger = false)),
    ];
    let bees = ["minecraft:bee#main", "minecraft:bee_baby#main"]
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, defaults, set)))
        .collect();
    vec![entity("segmented-model", "segmented_model", segmented), entity("bee-model", "bee_model", bees)]
}

/// What the creeper's renderer does to a swelling creeper: the scale it
/// applies to the pose (wobbling as the fuse nears its end) and how far it
/// flashes white.
pub fn creeper_render() -> Vec<Value> {
    let rows = [0.0, 0.05, 0.1, 0.19, 0.25, 0.37, 0.5, 0.64, 0.8, 0.95, 1.0, 1.0714286]
        .iter()
        .map(|value| json!({"label": format!("swelling-{}", float_text(*value)), "swelling": f32_bits(*value)}))
        .collect();
    vec![entity("creeper-render", "creeper_render", rows)]
}
