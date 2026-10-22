//! The name tags over entities that are not players: which renderer's rules decide, the
//! custom name and its visibility, the crosshair, the viewer, the distance, and where the tag
//! hangs for grown, baby and scaled entities of different kinds.

use serde_json::{Value, json};

use super::locator::{TAG, Tag, tag};
use crate::bits::f64_bits;
use crate::line::float_text;

/// One entity's scenario on top of the player scenario's camera and light.
struct Mob {
    kind: &'static str,
    renderer: &'static str,
    name: Option<&'static str>,
    visible: bool,
    picked: bool,
    baby: bool,
    scale: Option<f64>,
    reach: Option<f64>,
    tag: Tag,
}

const MOB: Mob = Mob {
    kind: "minecraft:zombie",
    renderer: "mob",
    name: Some("Bob"),
    visible: true,
    picked: false,
    baby: false,
    scale: None,
    reach: None,
    tag: TAG,
};

fn mob(label: &str, mob: Mob) -> Value {
    let mut row = tag(label, mob.tag);
    row["type"] = json!(mob.kind);
    row["renderer"] = json!(mob.renderer);
    row["custom_name_visible"] = json!(mob.visible);
    row["picked"] = json!(mob.picked);
    row["baby"] = json!(mob.baby);
    if let Some(name) = mob.name {
        row["custom_name"] = json!(name);
    }
    if let Some(scale) = mob.scale {
        row["scale"] = json!(f64_bits(scale));
    }
    if let Some(reach) = mob.reach {
        row["name_tag_distance"] = json!(f64_bits(reach));
    }
    row
}

fn at(distance: f64) -> Tag {
    Tag { position: [0.5 + distance, 64.0, 0.5], ..TAG }
}

fn visibility_rows() -> Vec<Value> {
    let plain = Mob { kind: "minecraft:minecart", renderer: "entity", ..MOB };
    vec![
        mob("mob-visible", MOB),
        mob("mob-hidden", Mob { visible: false, ..MOB }),
        mob("mob-hidden-picked", Mob { visible: false, picked: true, ..MOB }),
        mob("mob-visible-unnamed", Mob { name: None, ..MOB }),
        mob("mob-unnamed-picked", Mob { name: None, visible: false, picked: true, ..MOB }),
        mob("mob-invisible", Mob { tag: Tag { invisible: true, ..TAG }, ..MOB }),
        mob("mob-invisible-spectating", Mob { tag: Tag { invisible: true, spectator: true, ..TAG }, ..MOB }),
        mob("mob-ridden", Mob { tag: Tag { vehicle: true, ..TAG }, ..MOB }),
        mob("mob-hud-hidden", Mob { tag: Tag { hud_hidden: true, ..TAG }, ..MOB }),
        mob("mob-sneaking", Mob { tag: Tag { shift: true, ..TAG }, ..MOB }),
        mob("entity-visible", Mob { ..plain }),
        mob("entity-hidden", Mob { visible: false, ..plain }),
        mob("entity-hidden-picked", Mob { visible: false, picked: true, ..plain }),
        mob("entity-ridden", Mob { tag: Tag { vehicle: true, ..TAG }, ..plain }),
        mob("entity-invisible", Mob { tag: Tag { invisible: true, ..TAG }, ..plain }),
        mob("entity-hud-hidden", Mob { tag: Tag { hud_hidden: true, ..TAG }, ..plain }),
        mob("dragon-visible", Mob { kind: "minecraft:ender_dragon", renderer: "entity", ..MOB }),
        mob(
            "dragon-ridden",
            Mob { kind: "minecraft:ender_dragon", renderer: "entity", tag: Tag { vehicle: true, ..TAG }, ..MOB },
        ),
        mob(
            "dragon-invisible",
            Mob { kind: "minecraft:ender_dragon", renderer: "entity", tag: Tag { invisible: true, ..TAG }, ..MOB },
        ),
        mob("stand-visible", Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", ..MOB }),
        mob("stand-hidden", Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", visible: false, ..MOB }),
        mob(
            "stand-hidden-picked",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", visible: false, picked: true, ..MOB },
        ),
        mob(
            "stand-ridden",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", tag: Tag { vehicle: true, ..TAG }, ..MOB },
        ),
    ]
}

fn distance_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for distance in [20.0, 31.9, 32.1, 63.9, 64.1] {
        let tag = at(distance);
        rows.push(mob(&format!("mob-{}", float_text(distance)), Mob { tag, ..MOB }));
        rows.push(mob(
            &format!("mob-sneaking-{}", float_text(distance)),
            Mob { tag: Tag { shift: true, ..tag }, ..MOB },
        ));
        rows.push(mob(
            &format!("entity-{}", float_text(distance)),
            Mob { kind: "minecraft:minecart", renderer: "entity", tag, ..MOB },
        ));
    }
    for distance in [8.0, 9.9, 10.1, 100.0, 127.9, 128.1] {
        rows.push(mob(
            &format!("mob-reach-{}", float_text(distance)),
            Mob { tag: at(distance), reach: Some(if distance > 50.0 { 128.0 } else { 10.0 }), ..MOB },
        ));
    }
    rows
}

fn attachment_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in [
        "minecraft:zombie",
        "minecraft:cow",
        "minecraft:villager",
        "minecraft:ghast",
        "minecraft:enderman",
        "minecraft:spider",
        "minecraft:chicken",
        "minecraft:slime",
        "minecraft:iron_golem",
        "minecraft:wolf",
        "minecraft:horse",
        "minecraft:ender_dragon",
        "minecraft:pig",
        "minecraft:squid",
        "minecraft:allay",
    ] {
        // The dragon's renderer is a plain `EntityRenderer`.
        let renderer = if kind == "minecraft:ender_dragon" { "entity" } else { "mob" };
        rows.push(mob(&format!("kind-{kind}"), Mob { kind, renderer, ..MOB }));
    }
    for kind in ["minecraft:zombie", "minecraft:cow", "minecraft:villager", "minecraft:pig", "minecraft:wolf"] {
        rows.push(mob(&format!("baby-{kind}"), Mob { kind, baby: true, ..MOB }));
    }
    for scale in [0.5, 2.0, 7.25] {
        rows.push(mob(&format!("scaled-{}", float_text(scale)), Mob { scale: Some(scale), ..MOB }));
    }
    rows.push(mob("baby-scaled", Mob { baby: true, scale: Some(2.0), ..MOB }));
    for (label, yaw, pitch) in [("look-east", -90.0, 0.0), ("odd-angles", 123.456, 17.25)] {
        rows.push(mob(label, Mob { tag: Tag { camera_rot: (yaw, pitch), ..TAG }, ..MOB }));
    }
    rows.push(mob("turned", Mob { tag: Tag { y_rot: -73.5, ..TAG }, ..MOB }));
    rows.push(mob("dark-cave", Mob { tag: Tag { light: 0, ..TAG }, ..MOB }));
    rows
}

/// The rows for entities that are not players.
pub fn rows() -> Vec<Value> {
    [visibility_rows(), distance_rows(), attachment_rows()].concat()
}
