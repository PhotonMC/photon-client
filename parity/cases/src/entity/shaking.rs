//! Ravagers, striders, and the shaking a renderer adds to a freezing or
//! converting mob.

use serde_json::{Value, json};

use super::pose::{Field, Fields, LOOK_WALK, entity, fields, layer_row, row, table};
use crate::bits::f32_bits;
use crate::line::float_text;

fn shake_row(kind: &str, label: &str, (ticks, partial): (i64, f64), set: Fields) -> Value {
    let defaults = fields!(
        dimension = "minecraft:overworld",
        ticks_frozen = 0,
        immune = false,
        no_ai = false,
        suffocating = false
    );
    let mut row = row(&format!("{kind}-{label}-{ticks}"), defaults, set);
    row["kind"] = json!(kind);
    row["ticks"] = json!(ticks);
    row["partial"] = json!(f32_bits(partial));
    row["body_rot"] = json!(f32_bits(37.0));
    row
}

/// What `setupRotations` leaves a mob at while it shakes: frozen through
/// (any mob), a hoglin that is converting (not immune, awake, in a
/// dimension where piglins zombify) and a strider that suffocates, over
/// several ticks.
pub fn shaking() -> Vec<Value> {
    let nether = "minecraft:the_nether";
    let mut rows = Vec::new();
    for time in [(0, 0.0), (5, 0.0), (6, 0.75), (123, 0.4), (4001, 0.9)] {
        let mobs: [(&str, &str, Fields); 11] = [
            ("cow", "warm", fields!()),
            ("cow", "almost-frozen", fields!(ticks_frozen = 139)),
            ("cow", "frozen", fields!(ticks_frozen = 140)),
            ("hoglin", "overworld", fields!()),
            ("hoglin", "overworld-immune", fields!(immune = true)),
            ("hoglin", "overworld-no-ai", fields!(no_ai = true)),
            ("hoglin", "nether", fields!(dimension = nether)),
            ("hoglin", "nether-frozen", fields!(dimension = nether, ticks_frozen = 200)),
            ("strider", "lava", fields!()),
            ("strider", "suffocating", fields!(suffocating = true)),
            ("strider", "frozen", fields!(ticks_frozen = 140)),
        ];
        rows.extend(mobs.iter().map(|(kind, label, set)| shake_row(kind, label, time, set)));
    }
    vec![entity("shaking-rotations", "shaking_rotations", rows)]
}

/// The ravager posed by the reference's own `RavagerModel.setupAnim`: the
/// walk and look, the attack swing (the neck's thrust and the mouth's
/// snap), the stun's sway and the roar.
pub fn ravager() -> Vec<Value> {
    let defaults: Vec<(&str, Field)> = [&LOOK_WALK[..], fields!(stunned = 0.0, attack = 0.0, roar = 0.0)].concat();
    let pose = |label: String, set: Fields| {
        let mut row = row(&label, &defaults, set);
        row["layer"] = json!("minecraft:ravager#main");
        row
    };
    let mut rows = vec![
        pose("standing".to_owned(), fields!()),
        pose("looking".to_owned(), fields!(head_y_rot = -35.5, x_rot = 48.0)),
        pose("walking".to_owned(), fields!(walk_position = 7.3, walk_speed = 0.83)),
    ];
    for ticks in [9.5, 8.0, 6.5, 5.0, 4.2, 2.5, 1.0, 0.3] {
        let set = fields!(attack = ticks, head_y_rot = 6.0, walk_position = 2.2, walk_speed = 0.3);
        rows.push(pose(format!("attacking-{}", float_text(ticks)), set));
    }
    for ticks in [39.5, 30.0, 12.25, 1.5] {
        rows.push(pose(format!("stunned-{}", float_text(ticks)), fields!(stunned = ticks, x_rot = -8.0)));
    }
    for amount in [0.05, 0.3, 0.5, 0.75, 0.95] {
        rows.push(pose(format!("roaring-{}", float_text(amount)), fields!(roar = amount, head_y_rot = -12.0)));
    }
    rows.push(pose("attacking-while-stunned".to_owned(), fields!(attack = 7.0, stunned = 20.0)));
    vec![entity("ravager-model", "ravager_model", rows)]
}

/// The strider (grown, baby and the saddle's model) posed by the
/// reference's own `setupAnim`: the body turned by the look unless ridden,
/// rocking with the walk, the legs, the bristles; and its textures.
pub fn strider() -> Vec<Value> {
    let defaults: Vec<(&str, Field)> = [&LOOK_WALK[..], fields!(age_in_ticks = 37.5, ridden = false)].concat();
    let poses: [(&str, Fields); 6] = [
        ("standing", fields!()),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0)),
        ("walking", fields!(walk_position = 7.3, walk_speed = 0.2)),
        ("running", fields!(walk_position = 11.9, walk_speed = 0.83, age_in_ticks = 91.3)),
        ("ridden", fields!(head_y_rot = 20.0, x_rot = -15.0, walk_position = 3.1, walk_speed = 0.1, ridden = true)),
        ("ridden-and-fast", fields!(walk_position = 5.5, walk_speed = 0.9, ridden = true, age_in_ticks = 5.0)),
    ];
    let rows = ["minecraft:strider#main", "minecraft:strider_baby#main", "minecraft:strider#saddle"]
        .iter()
        .flat_map(|layer| poses.iter().map(|(label, set)| layer_row(layer, label, &defaults, set)))
        .collect();
    vec![entity("strider-model", "strider_model", rows), table("strider-textures", "strider_textures")]
}
