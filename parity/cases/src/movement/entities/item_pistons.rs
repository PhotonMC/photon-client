//! A dropped item in the way of a piston's arm: the level's block entities tick after the
//! entities and push what stands in their way.

use crate::movement::entities::prelude::*;

/// A piston of a facing, not yet extended.
fn piston(at: (i64, i64, i64), facing: &str) -> Value {
    put_with(at.0, at.1, at.2, "minecraft:piston", &[("facing", facing), ("extended", "false")])
}

/// A block event at a tick: the piston at a place extends towards a facing id.
fn extend(tick: i64, at: (i64, i64, i64), facing: i64) -> Value {
    json!({"tick": tick, "x": at.0, "y": at.1, "z": at.2, "b0": 0, "b1": facing})
}

pub fn rows() -> Vec<Value> {
    [arm_rows(), slime_rows(), edge_rows(), stand_rows(), other_rows()].concat()
}

/// Other entities the server added: the client moves them by the arm too.
fn other_rows() -> Vec<Value> {
    let mob = |label: &str, kind: &'static str, blocks: Vec<Value>, at: (f64, f64, f64), event: Value| {
        row(label, cat![floor(), blocks], at, (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .entity_type(kind)
            .block_events(vec![event])
            .row()
    };
    vec![
        mob(
            "pig-pushed-east",
            "minecraft:pig",
            vec![piston((0, 64, 4), "east")],
            (1.5, 64.0, 4.5),
            extend(1, (0, 64, 4), 5),
        ),
        mob("cow-lifted", "minecraft:cow", vec![piston((6, 64, 2), "up")], (6.5, 65.0, 2.5), extend(1, (6, 64, 2), 1)),
        // The arm pushes the pig against a slab: a mob steps up it.
        mob(
            "pig-pushed-onto-slab",
            "minecraft:pig",
            vec![piston((0, 64, 6), "east"), put_with(2, 64, 6, "minecraft:stone_slab", &[("type", "bottom")])],
            (1.5, 64.0, 6.5),
            extend(1, (0, 64, 6), 5),
        ),
        // The arm pushes the pig into a bubble column: the effects of the block it ends in apply.
        mob(
            "pig-pushed-into-bubbles",
            "minecraft:pig",
            vec![piston((0, 64, 14), "east"), put_with(2, 64, 14, "minecraft:bubble_column", &[("drag", "false")])],
            (1.5, 64.0, 14.5),
            extend(1, (0, 64, 14), 5),
        ),
        // An entity without physics is moved by the arm with no limit and no collision.
        mob(
            "bullet-pushed-east",
            "minecraft:shulker_bullet",
            vec![piston((0, 64, 10), "east")],
            (1.5, 64.0, 10.5),
            extend(1, (0, 64, 10), 5),
        ),
        mob(
            "dragon-pushed-east",
            "minecraft:ender_dragon",
            vec![piston((0, 64, 12), "east")],
            (1.5, 64.0, 12.5),
            extend(1, (0, 64, 12), 5),
        ),
        // Honey carries what stands on it along; the pig is on the honey block the arm pushes.
        mob(
            "pig-carried-on-honey",
            "minecraft:pig",
            vec![piston((0, 64, 8), "east"), put(1, 64, 8, "minecraft:honey_block")],
            (1.5, 65.0, 8.5),
            extend(1, (0, 64, 8), 5),
        ),
    ]
}

/// An armor stand, which the client does not move itself, in the way of the arm.
fn stand_rows() -> Vec<Value> {
    let stand = |label: &str, blocks: Vec<Value>, at: (f64, f64, f64), event: Value| {
        row(label, cat![floor(), blocks], at, (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .entity_type("minecraft:armor_stand")
            .block_events(vec![event])
            .row()
    };
    vec![
        stand("stand-pushed-east", vec![piston((0, 64, -6), "east")], (1.5, 64.0, -5.5), extend(1, (0, 64, -6), 5)),
        stand("stand-lifted", vec![piston((6, 64, -4), "up")], (6.5, 65.0, -3.5), extend(1, (6, 64, -4), 1)),
        stand(
            "stand-slime-pushed",
            vec![piston((0, 64, -2), "east"), put(1, 64, -2, "minecraft:slime_block")],
            (2.5, 64.0, -1.5),
            extend(1, (0, 64, -2), 5),
        ),
    ]
}

/// The arm reaches the item, or a block pushed along its way.
fn arm_rows() -> Vec<Value> {
    vec![
        // The arm reaches up under the item.
        row("lifted", cat![floor(), vec![piston((6, 64, 6), "up")]], (6.5, 65.0, 6.5), (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .block_events(vec![extend(1, (6, 64, 6), 1)])
            .row(),
        // The arm reaches into the item's side.
        row("pushed-east", cat![floor(), vec![piston((0, 64, 0), "east")]], (1.5, 64.0, 0.5), (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .block_events(vec![extend(1, (0, 64, 0), 5)])
            .row(),
        // A block pushed along the item's way.
        row(
            "block-pushed-east",
            cat![floor(), vec![piston((0, 64, 14), "east"), put(1, 64, 14, "minecraft:stone")]],
            (2.5, 64.0, 14.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .on_ground(true)
        .block_events(vec![extend(1, (0, 64, 14), 5)])
        .row(),
    ]
}

/// A slime block sends the item along its way at full speed.
fn slime_rows() -> Vec<Value> {
    vec![
        // A slime block sends the item along its way at full speed.
        row(
            "slime-pushed-east",
            cat![floor(), vec![piston((0, 64, 18), "east"), put(1, 64, 18, "minecraft:slime_block")]],
            (2.5, 64.0, 18.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .on_ground(true)
        .block_events(vec![extend(1, (0, 64, 18), 5)])
        .row(),
        // A slime block lifting the item resting on it.
        row(
            "slime-lifted",
            cat![floor(), vec![piston((6, 64, 22), "up"), put(6, 65, 22, "minecraft:slime_block")]],
            (6.5, 66.0, 22.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .on_ground(true)
        .block_events(vec![extend(1, (6, 64, 22), 1)])
        .row(),
    ]
}

/// The arm in other directions, against a wall, and drawn back.
fn edge_rows() -> Vec<Value> {
    vec![
        // The arm pushes the item towards the north.
        row("pushed-north", cat![floor(), vec![piston((10, 64, 8), "north")]], (10.5, 64.0, 7.5), (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .block_events(vec![extend(1, (10, 64, 8), 2)])
            .row(),
        // The arm pushes the item against a wall: the push stops at the stone.
        row(
            "pushed-into-wall",
            cat![floor(), vec![piston((0, 64, 26), "east"), put(3, 64, 26, "minecraft:stone")]],
            (1.5, 64.0, 26.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .on_ground(true)
        .block_events(vec![extend(1, (0, 64, 26), 5)])
        .row(),
        // A sticky piston draws the block back, and the item beside it stays.
        row(
            "sticky-retracts",
            cat![
                floor(),
                vec![
                    put_with(0, 64, 30, "minecraft:sticky_piston", &[("facing", "east"), ("extended", "true")]),
                    put_with(1, 64, 30, "minecraft:piston_head", &[("facing", "east"), ("type", "sticky")]),
                ]
            ],
            (1.5, 65.0, 30.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .on_ground(true)
        .block_events(vec![json!({"tick": 1, "x": 0, "y": 64, "z": 30, "b0": 1, "b1": 5})])
        .row(),
    ]
}
