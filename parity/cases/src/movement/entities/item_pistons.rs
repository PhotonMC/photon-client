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
    [arm_rows(), slime_rows(), edge_rows()].concat()
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
