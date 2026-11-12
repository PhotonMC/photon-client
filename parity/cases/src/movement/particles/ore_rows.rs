//! The dust a redstone ore gives off when it is touched (`RedStoneOreBlock.spawnParticles`): on
//! each face that is not against a solid block, with the level's random source.

use serde_json::Value;

use super::engine::*;
use super::events::*;
use crate::movement::world::{put, put_with};

/// A stone floor with an ore of a kind in the middle of it, one cell up; `extra` goes around.
fn ore_blocks(kind: &str, extra: Vec<Value>) -> Vec<Value> {
    let mut blocks: Vec<Value> =
        (-3..=3_i64).flat_map(|x| (-3..=3_i64).map(move |z| put(x, 63, z, "minecraft:stone"))).collect();
    blocks.push(put_with(0, 64, 0, kind, &[("lit", "false")]));
    blocks.extend(extra);
    blocks
}

fn touched(label: &str, blocks: Vec<Value>, touches: &[i64]) -> Value {
    let events = touches.iter().map(|tick| ore_touch(*tick, (0, 64, 0))).collect();
    engine_row(label, events).blocks(blocks).ticks(4).seeds(2000).describe(12).row()
}

pub fn rows() -> Vec<Value> {
    let around = |block: &str| -> Vec<Value> {
        [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().map(|(x, z)| put(*x, 64, *z, block)).collect()
    };
    vec![
        touched("ore-sparks-open", ore_blocks("minecraft:redstone_ore", vec![]), &[0]),
        // The second touch goes on from the random source where the first left it.
        touched("ore-sparks-twice", ore_blocks("minecraft:redstone_ore", vec![]), &[0, 1]),
        touched("ore-sparks-deepslate", ore_blocks("minecraft:deepslate_redstone_ore", vec![]), &[0]),
        // Faces against solid blocks give nothing and draw nothing.
        touched("ore-sparks-sealed", ore_blocks("minecraft:redstone_ore", around("minecraft:stone")), &[0]),
        // A block that is not solid to the eye lets the dust out.
        touched("ore-sparks-glass", ore_blocks("minecraft:redstone_ore", around("minecraft:glass")), &[0]),
    ]
}

/// A dragon egg touched (`DragonEggBlock.teleport`): a 26.2 client looks for a place to go with the
/// level's random source and shows the specks itself; a 26.3 client leaves it to the server.
pub fn egg_rows() -> Vec<Value> {
    let floor: Vec<Value> =
        (-3..=3_i64).flat_map(|x| (-3..=3_i64).map(move |z| put(x, 63, z, "minecraft:stone"))).collect();
    let mut standing = floor;
    standing.push(put(0, 64, 0, "minecraft:dragon_egg"));
    let alone = vec![put(0, 64, 0, "minecraft:dragon_egg")];
    let touch = |label: &str, blocks: Vec<Value>, events: Vec<Value>| {
        engine_row(label, events).blocks(blocks).ticks(4).seeds(3000).describe(12).row()
    };
    vec![
        touch("egg-touch-floor", standing.clone(), vec![ore_touch(0, (0, 64, 0))]),
        touch("egg-touch-twice", standing, vec![ore_touch(0, (0, 64, 0)), ore_touch(1, (0, 64, 0))]),
        // No place to go: a thousand tries draw from the random source, and the event after shows
        // where it stands.
        touch("egg-touch-no-place", alone, vec![ore_touch(0, (0, 64, 0)), level_event(1, 2009, (1, 64, 1), 0)]),
    ]
}
