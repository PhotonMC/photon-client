//! The rows of the particle engine that came after the first ten: the level events and the block
//! events that need a level of blocks, block entities and players around them.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use crate::bits::f64_bits;
use crate::movement::world::{put, put_with};

const DRIPSTONE: &str = "minecraft:pointed_dripstone";

/// A dripstone column hanging from the root at `root_y` (the tip `length` blocks below it) with
/// the block above the root: the blocks of the column and the tip's position.
fn dripstone_column(x: i64, root_y: i64, length: i64, (block, above): (&str, Value)) -> Vec<Value> {
    let tip_y = root_y - length;
    let mut blocks = vec![put(x, root_y, 0, "minecraft:stone"), above];
    for y in tip_y..root_y {
        let thickness = if y == tip_y { "tip" } else { "middle" };
        blocks.push(put_with(
            x,
            y,
            0,
            block,
            &[("vertical_direction", "down"), ("thickness", thickness), ("waterlogged", "false")],
        ));
    }
    blocks
}

fn engine_rows_11() -> Vec<Value> {
    // What lies above a stalactite's root: the fluids, mud, nothing, a waterlogged slab and a flow.
    let covers = [
        ("minecraft:water", vec![("level", "0")]),
        ("minecraft:lava", vec![("level", "0")]),
        ("minecraft:mud", vec![]),
        ("minecraft:air", vec![]),
        ("minecraft:stone_slab", vec![("type", "bottom"), ("waterlogged", "true")]),
        ("minecraft:water", vec![("level", "3")]),
        ("minecraft:lava", vec![("level", "5")]),
        ("minecraft:oak_planks", vec![]),
    ];
    let mut blocks = Vec::new();
    let mut events = Vec::new();
    for (index, (block, properties)) in covers.iter().enumerate() {
        let x = index as i64 * 3 + 1;
        blocks.extend(dripstone_column(x, 70, 3, (DRIPSTONE, put_with(x, 71, 0, block, properties))));
        events.push(level_event(index as i64 % 3, 1504, (x, 67, 0), 0));
    }
    // A column whose root is ten blocks above the tip is found, one eleven above is not.
    let near = 27;
    blocks.extend(dripstone_column(near, 76, 10, (DRIPSTONE, put(near, 77, 0, "minecraft:water"))));
    events.push(level_event(0, 1504, (near, 66, 0), 0));
    let far = 30;
    blocks.extend(dripstone_column(far, 77, 11, (DRIPSTONE, put(far, 78, 0, "minecraft:water"))));
    events.push(level_event(0, 1504, (far, 66, 0), 0));
    // A column with a stalagmite in its middle, a lone stalagmite and a block that is no dripstone.
    let broken = 33;
    blocks.extend(dripstone_column(broken, 70, 4, (DRIPSTONE, put(broken, 71, 0, "minecraft:water"))));
    blocks.push(put_with(
        broken,
        68,
        0,
        "minecraft:pointed_dripstone",
        &[("vertical_direction", "up"), ("thickness", "middle"), ("waterlogged", "false")],
    ));
    events.push(level_event(1, 1504, (broken, 66, 0), 0));
    let up = 36;
    blocks.push(put_with(
        up,
        66,
        0,
        "minecraft:pointed_dripstone",
        &[("vertical_direction", "up"), ("thickness", "tip"), ("waterlogged", "false")],
    ));
    events.push(level_event(1, 1504, (up, 66, 0), 0));
    // The sulfur spike is a speleothem too, with its own offset.
    let spike = 42;
    blocks.extend(dripstone_column(spike, 70, 3, ("minecraft:sulfur_spike", put(spike, 71, 0, "minecraft:water"))));
    events.push(level_event(0, 1504, (spike, 67, 0), 0));
    blocks.push(put(39, 66, 0, "minecraft:stone"));
    events.push(level_event(2, 1504, (39, 66, 0), 0));
    vec![engine_row("dripstone-drip", events).blocks(blocks).ticks(6).seeds(150).describe(8).row()]
}

/// The vault's lighting: ominous or not, every way it faces, and what is no vault.
fn engine_rows_12() -> Vec<Value> {
    let vaults = [("north", "false", 0), ("south", "true", 1), ("east", "false", 1), ("west", "true", 0)];
    let mut blocks = Vec::new();
    let mut events = Vec::new();
    for (index, (facing, ominous, data)) in vaults.iter().enumerate() {
        let x = index as i64 * 3;
        blocks.push(put_with(
            x,
            64,
            0,
            "minecraft:vault",
            &[("facing", facing), ("ominous", ominous), ("vault_state", "active")],
        ));
        events.push(level_event(index as i64 % 2, 3015, (x, 64, 0), *data));
    }
    // Where there is no vault nothing happens, sound included.
    blocks.push(put(15, 64, 0, "minecraft:stone"));
    events.push(level_event(2, 3015, (15, 64, 0), 0));
    events.push(level_event(2, 3015, (18, 64, 0), 0));
    vec![engine_row("vault-activation", events).blocks(blocks).ticks(6).seeds(400).describe(8).row()]
}

/// The players connected to a vault: the ones in reach, the ones out of it, one the level does not
/// know, a wider reach and no one at all.
fn engine_rows_13() -> Vec<Value> {
    let near = [1, 2, 3, 4];
    let far = [5, 6, 7, 8];
    let missing = [9, 9, 9, 9];
    let vaults = [
        ("north", vec![near, far, missing], None),
        ("south", vec![near, far], Some(100.0)),
        ("east", vec![near], Some(1.0)),
        ("west", vec![], None),
    ];
    let mut blocks = Vec::new();
    let mut events = Vec::new();
    let mut shared = Vec::new();
    for (index, (facing, players, range)) in vaults.iter().enumerate() {
        let x = index as i64 * 3;
        blocks.push(put_with(
            x,
            64,
            0,
            "minecraft:vault",
            &[("facing", facing), ("ominous", "false"), ("vault_state", "active")],
        ));
        events.push(level_event(index as i64 % 2, 3015, (x, 64, 0), 0));
        let mut vault = json!({"x": x, "y": 64, "z": 0, "players": players});
        if let Some(range) = range {
            vault["range"] = json!(f64_bits(*range));
        }
        shared.push(vault);
    }
    let players = vec![
        json!({"uuid": near, "x": f64_bits(1.5), "y": f64_bits(64.0), "z": f64_bits(2.5)}),
        json!({"uuid": far, "x": f64_bits(40.25), "y": f64_bits(64.0), "z": f64_bits(-40.75)}),
    ];
    vec![
        engine_row("vault-connections", events)
            .blocks(blocks)
            .set("vaults", json!(shared))
            .set("players", json!(players))
            .ticks(6)
            .seeds(600)
            .describe(8)
            .row(),
    ]
}

/// Jukeboxes: songs started, replaced and stopped, and the numbers the registry has no song for.
fn engine_rows_14() -> Vec<Value> {
    vec![
        engine_row(
            "jukebox-songs",
            vec![
                level_event(0, 1010, (0, 64, 0), 0),
                level_event(0, 1010, (3, 64, 0), 5),
                level_event(1, 1010, (0, 64, 0), 7),
                level_event(1, 1011, (3, 64, 0), 0),
                level_event(1, 1011, (6, 64, 0), 0),
                level_event(2, 1010, (6, 64, 0), 999),
                level_event(2, 1010, (6, 64, 0), -1),
                level_event(2, 1010, (0, 64, 0), 999),
                level_event(3, 1010, (-4, 70, 9), 17),
                level_event(3, 1011, (0, 64, 0), 3),
            ],
        )
        .blocks(vec![put(0, 64, 0, "minecraft:jukebox")])
        .ticks(5)
        .seeds(8)
        .describe(8)
        .row(),
    ]
}

/// Parrots near a jukebox: the ones within three blocks of it dance to a record and stop when it
/// does, the ones beyond do not.
fn engine_rows_15() -> Vec<Value> {
    let at = |x: f64, y: f64, z: f64| json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)});
    let parrots = vec![
        at(0.5, 64.0, 0.5),
        at(3.4, 64.0, 0.5),
        at(4.6, 64.0, 0.5),
        at(0.5, 67.9, 0.5),
        at(0.5, 69.5, 0.5),
        at(-3.4, 64.0, -3.4),
        at(20.5, 64.0, 20.5),
    ];
    vec![
        engine_row(
            "jukebox-parrots",
            vec![
                level_event(0, 1010, (0, 64, 0), 2),
                level_event(1, 1011, (0, 64, 0), 0),
                level_event(2, 1010, (0, 64, 0), 999),
                level_event(2, 1010, (0, 64, 0), 4),
                level_event(3, 1011, (30, 64, 30), 0),
            ],
        )
        .blocks(vec![put(0, 64, 0, "minecraft:jukebox")])
        .set("parrots", json!(parrots))
        .ticks(5)
        .seeds(8)
        .describe(8)
        .row(),
    ]
}

/// What block events set in the block entities that keep it: a pot's wobble, a gateway's cooldown,
/// a spawner's delay and when a sulfur block erupted; the events and the blocks that take none.
/// An event a block is sent: the tick, the event and its data.
type Sent = (i64, i64, i64);

fn engine_rows_16() -> Vec<Value> {
    // The block, its place along the row, and the events it is sent: tick, event, data.
    let cases: [(&str, [Sent; 2]); 13] = [
        ("minecraft:decorated_pot", [(0, 1, 0), (9, 0, 0)]),
        ("minecraft:decorated_pot", [(0, 1, 1), (9, 0, 0)]),
        ("minecraft:decorated_pot", [(0, 1, 2), (9, 0, 0)]),
        ("minecraft:decorated_pot", [(0, 0, 0), (9, 0, 0)]),
        ("minecraft:decorated_pot", [(0, 1, 1), (2, 1, 0)]),
        ("minecraft:end_gateway", [(1, 1, 0), (9, 0, 0)]),
        ("minecraft:end_gateway", [(1, 2, 0), (9, 0, 0)]),
        ("minecraft:spawner", [(1, 1, 0), (9, 0, 0)]),
        ("minecraft:spawner", [(1, 2, 0), (9, 0, 0)]),
        ("minecraft:potent_sulfur", [(2, 0, 0), (9, 0, 0)]),
        ("minecraft:potent_sulfur", [(9, 0, 0), (9, 0, 0)]),
        ("minecraft:stone", [(0, 1, 0), (9, 0, 0)]),
        ("minecraft:air", [(0, 1, 0), (9, 0, 0)]),
    ];
    let mut blocks = Vec::new();
    let mut events = Vec::new();
    let mut watch = Vec::new();
    for (index, (block, sent)) in cases.iter().enumerate() {
        let x = index as i64 * 3;
        blocks.push(put(x, 64, 0, block));
        watch.push(json!({"x": x, "y": 64, "z": 0}));
        for &(tick, event, data) in sent.iter().filter(|sent| sent.0 < 9) {
            events.push(block_event(tick, (x, 64, 0), event, data));
        }
    }
    vec![
        engine_row("block-entity-events", events)
            .blocks(blocks)
            .set("watch", json!(watch))
            .set("game_time", json!(12345))
            .ticks(4)
            .seeds(8)
            .describe(8)
            .row(),
    ]
}

/// A piston scene: what stands in the row, the pistons' events, and how many rows of blocks there
/// are, each a row of its own along z.
struct PistonScene {
    label: &'static str,
    rows: Vec<Vec<Value>>,
    events: Vec<Value>,
}

/// A block of a row at x.
fn at(x: i64, z: i64, block: &str, properties: &[(&str, &str)]) -> Value {
    put_with(x, 64, z, block, properties)
}

fn piston_scene(scene: PistonScene) -> Value {
    let blocks: Vec<Value> = scene.rows.concat();
    engine_row(scene.label, scene.events)
        .blocks(blocks)
        .set("region", json!([-1, 63, -1, 16, 66, 40]))
        .ticks(2)
        .seeds(600)
        .describe(8)
        .row()
}

/// Pistons pushing and pulling: what a line of blocks does, the limit of twelve, slime and honey
/// holding on to what is beside them, what is broken and what is not moved.
fn engine_rows_17() -> Vec<Value> {
    let piston = |z: i64, facing: &str, extended: &str| {
        at(0, z, "minecraft:piston", &[("facing", facing), ("extended", extended)])
    };
    let sticky = |z: i64, facing: &str, extended: &str| {
        at(0, z, "minecraft:sticky_piston", &[("facing", facing), ("extended", extended)])
    };
    let stone_line = |z: i64, from: i64, count: i64| -> Vec<Value> {
        (from..from + count).map(|x| at(x, z, "minecraft:stone", &[])).collect()
    };
    let head = |z: i64, kind: &str| {
        at(1, z, "minecraft:piston_head", &[("facing", "east"), ("type", kind), ("short", "false")])
    };
    let rows = vec![
        // One block, twelve and thirteen in a line, then obsidian and a flower.
        vec![piston(0, "east", "false")].into_iter().chain(stone_line(0, 1, 1)).collect(),
        vec![piston(4, "east", "false")].into_iter().chain(stone_line(4, 1, 12)).collect(),
        vec![piston(8, "east", "false")].into_iter().chain(stone_line(8, 1, 13)).collect(),
        vec![piston(12, "east", "false"), at(1, 12, "minecraft:obsidian", &[])],
        // Stone with an air gap, and a piston with nothing before it.
        vec![piston(16, "east", "false"), at(2, 16, "minecraft:stone", &[])],
        vec![piston(20, "up", "false"), at(0, 21, "minecraft:stone", &[]), at(0, 20, "minecraft:stone", &[])],
        // Slime with stone and honey beside it.
        vec![
            piston(24, "east", "false"),
            at(1, 24, "minecraft:slime_block", &[]),
            at(1, 25, "minecraft:stone", &[]),
            at(1, 23, "minecraft:slime_block", &[]),
            at(1, 22, "minecraft:honey_block", &[]),
            at(2, 23, "minecraft:stone", &[]),
        ],
        // A sticky piston that has pulled out: it draws a block back (1), lets go (2) or draws a
        // line of two.
        vec![sticky(28, "east", "true"), head(28, "sticky"), at(2, 28, "minecraft:stone", &[])],
        vec![sticky(32, "east", "true"), head(32, "sticky"), at(2, 32, "minecraft:stone", &[])],
        vec![piston(36, "east", "true"), head(36, "normal"), at(2, 36, "minecraft:stone", &[])],
    ];
    let events = vec![
        block_event(0, (0, 64, 0), 0, 5),
        block_event(0, (0, 64, 4), 0, 5),
        block_event(0, (0, 64, 8), 0, 5),
        block_event(0, (0, 64, 12), 0, 5),
        block_event(0, (0, 64, 16), 0, 5),
        block_event(0, (0, 64, 20), 0, 5),
        block_event(0, (0, 64, 24), 0, 5),
        block_event(0, (0, 64, 28), 1, 5),
        block_event(0, (0, 64, 32), 2, 5),
        block_event(0, (0, 64, 36), 1, 5),
    ];
    vec![piston_scene(PistonScene { label: "pistons-push-and-pull", rows, events })]
}

pub fn rows() -> Vec<Value> {
    [
        engine_rows_11(),
        engine_rows_12(),
        engine_rows_13(),
        engine_rows_14(),
        engine_rows_15(),
        engine_rows_16(),
        engine_rows_17(),
    ]
    .concat()
}
