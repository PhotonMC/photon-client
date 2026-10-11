//! The particle engine of a level.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use super::layouts::*;
use crate::movement::world::cat;

fn animating_1() -> Vec<Value> {
    vec![
        // Blocks about the player animate: picks by the level's random, then each torch's flame and smoke (a
        // wall torch's out from its wall), a lit redstone torch's dust; lava under open air pops, still and
        // falling water has specks, flowing water murmurs (ambient sounds, not compared); fluids over air drip
        // nowhere but still draw for it.
        engine_row("animate-torches", vec![animate_tick(0, (0, 64, 0), 200, 77), animate_tick(1, (2, 63, -1), 150, 78)]).blocks((-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| torch_at(x, z)).collect::<Vec<_>>()).ticks(3).seeds(200).describe(12).row(),
        engine_row("animate-fluids", vec![animate_tick(0, (0, 56, 0), 300, 79), animate_tick(1, (0, 50, 0), 300, 80)]).blocks(cat![plane(60, "minecraft:lava", None, 15), plane(56, "minecraft:water", None, 15), plane(52, "minecraft:water", Some(&[("level", "3")]), 15), plane(48, "minecraft:water", Some(&[("level", "8")]), 15)]).ticks(4).seeds(300).describe(12).row(),
        // Drops go through their stages in the engine: a hanging drop lets a falling one go, which lands
        // (a splash, a landed drop, a dripstone's or honey's sound); a fluid with a block and air beneath it
        // drips through.
        engine_row("drip-stages", vec![add(0, "dripping_water", (3.5, 66.5, 3.5)), add(0, "dripping_lava", (-2.5, 66.5, 1.5)), add(1, "falling_honey", (1.5, 67.5, -2.5)), add(1, "falling_dripstone_lava", (0.5, 67.0, 0.5)), add(2, "falling_dripstone_water", (-1.5, 68.0, -1.5))]).blocks(plane(65, "minecraft:stone", None, 6)).ticks(60).seeds(20).describe(8).row(),
        engine_row("animate-drips", vec![animate_tick(0, (0, 55, 0), 400, 81)]).blocks(cat![plane(56, "minecraft:water", None, 15), plane(55, "minecraft:stone", None, 15), plane(60, "minecraft:lava", None, 15), plane(59, "minecraft:glass", None, 15)]).ticks(30).seeds(200).describe(10).row(),
        engine_row("animate-hearths", vec![animate_tick(0, (0, 64, 0), 250, 82), animate_tick(1, (1, 64, 2), 250, 83)]).blocks((-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| hearth_at(x, z)).collect::<Vec<_>>()).ticks(3).seeds(400).describe(12).row(),
        // Fire on netherrack rises through itself; fire hung in the air between planks smokes off the burning
        // faces (and off nothing where a face does not burn); soul fire never floats.
        // The lever's speck, the ore's dust off each open face, the repeater's, redstone dust along its arms in
        // the colour of its power, the wet sponge's drops, bubble columns (and their sounds), the ender chest's
        // portal specks, the sculk sensors' sparks while they are active.
        engine_row("animate-redstone", vec![animate_tick(0, (0, 64, 0), 300, 150), animate_tick(1, (1, 64, -2), 300, 151)]).blocks(cat![(-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| redstone_at(x, z)).collect::<Vec<_>>(), plane(62, "minecraft:stone", None, 15)]).ticks(3).seeds(1500).describe(16).row(),
        engine_row("animate-fire", vec![animate_tick(0, (0, 66, 0), 400, 84), animate_tick(1, (0, 70, 0), 400, 85)]).blocks(cat![plane(64, "minecraft:netherrack", None, 15), plane(65, "minecraft:fire", None, 15), (-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 70, "z": z, "block": if (x + z).rem_euclid(2) != 0 { "minecraft:fire" } else { "minecraft:oak_planks" }})).collect::<Vec<_>>(), (-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).filter(|&(x, z)| (x * z).rem_euclid(3) == 0).map(|(x, z)| json!({"x": x, "y": 72, "z": z, "block": "minecraft:soul_fire"})).collect::<Vec<_>>()]).ticks(3).seeds(900).describe(12).row(),
        // Falling blocks over air sift dust (sand, gravel, concrete powder, anvils, a dragon egg), sand under
        // open sky draws for the desert's sound; dust asked for directly takes a block's tint or map colour,
        // and a block drawn as nothing makes none.
        engine_row("animate-falling", vec![animate_tick(0, (0, 70, 0), 400, 86), animate_tick(1, (0, 70, 0), 400, 87)]).blocks(cat![(-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 70, "z": z, "block": format!("{}{}", "minecraft:", (["sand", "red_sand", "gravel", "white_concrete_powder", "anvil", "dragon_egg", "lime_concrete_powder"][((x * 3) + z).rem_euclid(7) as usize])), "properties": {}})).collect::<Vec<_>>(), plane(62, "minecraft:stone", None, 15)]).ticks(40).seeds(400).describe(12).row(),
    ]
}

fn animating_2() -> Vec<Value> {
    vec![
        engine_row("falling-dust-add", ["stone", "grass_block", "spruce_leaves", "barrier", "water", "air", "blue_concrete_powder", "damaged_anvil"].into_iter().map(|name| with_fields(add(0, "falling_dust", (1.5, 66.0, 0.5)), json!({"block": format!("{}{}", "minecraft:", name)}))).collect::<Vec<_>>()).blocks(plane(63, "minecraft:stone", None, 4)).ticks(50).seeds(10).describe(8).row(),
        // A nether portal hums and draws specks out of its faces (a wall along x, one along z); spore blossoms
        // drop spores and fill the air below them; mycelium puffs; crying obsidian weeps out of uncovered
        // faces; a charged respawn anchor sheds reverse portal specks; end rods glint along their length.
        engine_row("animate-misc", vec![animate_tick(0, (0, 64, 0), 400, 88), animate_tick(1, (2, 62, -3), 400, 89)]).blocks(cat![(-12_i64..13).flat_map(|x| (64_i64..72).map(move |y| (x, y))).map(|(x, y)| json!({"x": x, "y": y, "z": 0, "block": "minecraft:nether_portal", "properties": {"axis": "x"}})).collect::<Vec<_>>(), (2_i64..14).flat_map(|z| (64_i64..72).map(move |y| (z, y))).map(|(z, y)| json!({"x": 0, "y": y, "z": z, "block": "minecraft:nether_portal", "properties": {"axis": "z"}})).collect::<Vec<_>>(), (-14_i64..15).step_by(4).flat_map(|x| (-14_i64..15).step_by(4).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 76, "z": z, "block": "minecraft:spore_blossom", "properties": {}})).collect::<Vec<_>>(), (-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 58, "z": z, "block": format!("{}{}", "minecraft:", (["mycelium", "crying_obsidian", "stone"][(x + (2 * z)).rem_euclid(3) as usize])), "properties": if (x + (2 * z)).rem_euclid(3) == 0 { json!({"snowy": "false"}) } else { json!({}) }})).collect::<Vec<_>>(), (-15_i64..16).step_by(2).flat_map(|x| (-15_i64..16).step_by(3).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 60, "z": z, "block": "minecraft:respawn_anchor", "properties": {"charges": ((x + z).rem_euclid(5)).to_string()}})).collect::<Vec<_>>(), (-14_i64..15).step_by(2).flat_map(|x| (-14_i64..15).step_by(2).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 62, "z": z, "block": "minecraft:end_rod", "properties": {"facing": (["up", "down", "north", "south", "west", "east"][((x * 5) + z).rem_euclid(6) as usize])}})).collect::<Vec<_>>()]).ticks(4).seeds(1500).describe(14).row(),
        engine_row("animate-leaves", (0_i64..16_i64).map(|tick| animate_tick(tick, (0, 70, 0), 667, 90 + tick)).collect::<Vec<_>>()).blocks(cat![(-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 70, "z": z, "block": format!("{}{}", "minecraft:", (["spruce_leaves", "birch_leaves", "cherry_leaves", "pale_oak_leaves", "azalea_leaves", "flowering_azalea_leaves"][(x + (2 * z)).rem_euclid(6) as usize])), "properties": {"distance": "1", "persistent": "true", "waterlogged": "false"}})).collect::<Vec<_>>(), (-15_i64..16).flat_map(|x| (-15_i64..16).map(move |z| (x, z))).filter(|&(x, z)| (x * z).rem_euclid(4) == 0).map(|(x, z)| json!({"x": x, "y": 69, "z": z, "block": "minecraft:stone"})).collect::<Vec<_>>(), plane(60, "minecraft:stone", None, 15)]).ticks(60).seeds(400).describe(12).row(),
        // An explosion: its sound (pitched by the level's random) and its particle at once, then on the level's
        // next tick its block particles about it where there is air, picked by block count, at most 512.
        engine_row("explosions", vec![explode(0, (2.5, 64.5, 1.5), 4.0, 40, "explosion", vec![("poof", 0.5, 1.0, 1), ("smoke", 1.0, 1.0, 1)]), explode(0, (-3.0, 65.0, -2.0), 2.5, 9, "explosion_emitter", vec![("poof", 1.0, 0.5, 3)]), explode(2, (0.5, 64.0, 0.5), 7.0, 900, "explosion", vec![("poof", 0.5, 1.0, 2), ("smoke", 1.0, 1.0, 1)]), explode(3, (8.0, 64.0, 8.0), 3.0, 5, "explosion", vec![])]).blocks(cat![plane(63, "minecraft:stone", None, 8), vec![json!({"x": 1, "y": 64, "z": 0, "block": "minecraft:stone"})]]).ticks(6).seeds(1500).describe(12).row(),
        // More level events: the composter filling (or not), a splash potion's shards, specks and sound (and
        // an instant one's), an ender eye's shards and ring, a spawner's flames, bone meal's sparkles in a
        // block and in the air (a mace's smash, 2013, needs the block models: its dust is the terrain rows').
        engine_row("level-event-more", vec![level_event(0, 1500, (1, 64, 1), 1), level_event(0, 1500, (2, 64, 1), 0), level_event(0, 2002, (-2, 64, 2), 4159204), level_event(1, 2007, (3, 64, -2), 16262179), level_event(1, 2003, (0, 64, 0), 0), level_event(2, 2004, (-3, 65, -3), 0), level_event(2, 2011, (4, 64, 4), 15), level_event(2, 2012, (4, 70, 4), 6)]).blocks(cat![plane(63, "minecraft:stone", None, 8), vec![json!({"x": 1, "y": 64, "z": 1, "block": "minecraft:composter", "properties": {"level": "3"}}), json!({"x": 2, "y": 64, "z": 1, "block": "minecraft:composter", "properties": {"level": "0"}}), json!({"x": 4, "y": 64, "z": 4, "block": "minecraft:oak_sapling", "properties": {"stage": "0"}})]]).ticks(6).seeds(600).describe(14).row(),
        engine_row("level-event-smoke", vec![level_event(0, 1501, (2, 63, 2), 0), level_event(0, 1502, (-3, 64, 1), 0), level_event(1, 1503, (0, 64, -4), 0), level_event(1, 2008, (4, 64, 4), 0), level_event(2, 2009, (-2, 64, -2), 0)]).ticks(6).seeds(45).describe(12).row(),
        engine_row("level-event-decreased", vec![level_event(0, 1501, (2, 63, 2), 0), level_event(0, 1502, (-3, 64, 1), 0)]).status("DECREASED").ticks(4).seeds(15).describe(6).row(),
        engine_row("level-event-shoot", cat![(0_i64..6_i64).map(|face| level_event(0, 2000, (1, 64, 1), face)).collect::<Vec<_>>(), vec![level_event(1, 2010, (-1, 64, -1), 9), level_event(1, 2000, (0, 64, 3), -4)]]).ticks(4).seeds(80).describe(16).row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [animating_1(), animating_2()].concat()
}
