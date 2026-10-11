//! Entity culling scenarios: how far each kind of entity is drawn and the box its renderer tests.

use crate::movement::entities::prelude::*;

fn culling_1() -> Vec<Value> {
    vec![
        // The plain rule (the box's mean side times 64 and the view scale), a living renderer's
        // (half a block more round one wearing a dragon head) and the renderers that grow the box
        // (the illusioner sideways, the sniffer, the trident; the happy ghast downwards; a minecart
        // up to the block it shows).
        cull("pig", "pig", vec![]).row(),
        cull(
            "zombie-dragon-head",
            "zombie",
            vec![json!({"tick": 1, "type": "equipment", "slot": "head", "item": "minecraft:dragon_head"})],
        )
        .row(),
        cull("illusioner", "illusioner", vec![]).row(),
        cull("sniffer", "sniffer", vec![]).row(),
        cull("happy-ghast", "happy_ghast", vec![]).row(),
        cull("trident", "trident", vec![]).row(),
        cull("minecart", "minecart", vec![]).row(),
        cull("chest-minecart", "chest_minecart", vec![]).row(),
    ]
}

fn culling_2() -> Vec<Value> {
    vec![
        cull(
            "minecart-shows-a-block",
            "minecart",
            vec![json!({"tick": 1, "type": "minecart_data", "display": 1, "offset": 10})],
        )
        .row(),
        // Ten boxes for an arrow; four after the first tick near the camera for what is thrown and
        // for fireballs and wind charges, four from the start for the rest of what is shot; an armor
        // stand; the fixed and scaled reaches.
        cull("arrow", "arrow", vec![]).row(),
        cull("snowball", "snowball", vec![]).row(),
        cull("eye-of-ender", "eye_of_ender", vec![]).row(),
        cull("fireball", "fireball", vec![]).row(),
        cull("small-fireball", "small_fireball", vec![]).row(),
        cull("wind-charge", "wind_charge", vec![]).row(),
        cull("breeze-wind-charge", "breeze_wind_charge", vec![]).row(),
    ]
}

fn culling_3() -> Vec<Value> {
    vec![
        cull("dragon-fireball", "dragon_fireball", vec![]).row(),
        cull("wither-skull", "wither_skull", vec![]).row(),
        cull("shulker-bullet", "shulker_bullet", vec![]).row(),
        cull("armor-stand", "armor_stand", vec![]).row(),
        cull("leash-knot", "leash_knot", vec![]).row(),
        // An item frame's and a painting's box hang off the block they are attached to, by
        // their facing direction (`HangingEntity.calculateBoundingBox`); west and north here.
        cull("item-frame", "item_frame", vec![]).data(4).row(),
        cull("glow-item-frame", "glow_item_frame", vec![]).data(2).row(),
        cull("phantom", "phantom", vec![]).row(),
    ]
}

fn culling_4() -> Vec<Value> {
    vec![
        cull("firework", "firework_rocket", vec![]).row(),
        cull("experience-orb", "experience_orb", vec![]).row(),
        cull("tnt", "tnt", vec![]).row(),
        // An end crystal with a beam is drawn from anywhere; the bolt, the dragon and a display with
        // no size are not culled; a display with a size stands its box on its position.
        cull("end-crystal", "end_crystal", vec![]).row(),
        cull(
            "end-crystal-beam",
            "end_crystal",
            vec![json!({"tick": 1, "type": "crystal_data", "target": [3, 70, -4]})],
        )
        .row(),
        cull("lightning-bolt", "lightning_bolt", vec![]).row(),
        cull("ender-dragon", "ender_dragon", vec![]).row(),
        cull("display-no-size", "block_display", vec![]).row(),
    ]
}

fn culling_5() -> Vec<Value> {
    vec![
        cull(
            "display-sized",
            "block_display",
            vec![display_data(1, json!({"block_state": 1, "view_range": 0.5, "width": 1.5, "height": 2.25}))],
        )
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [culling_1(), culling_2(), culling_3(), culling_4(), culling_5()].concat()
}
