//! A piston pushing the player: the block events the server sends make the moving blocks, whose
//! block entities tick after the player and move what stands in their way.

use serde_json::{Value, json};

use crate::movement::player::scene::{Scene, keys};
use crate::movement::world::{cat, floor, put, put_with};

/// A piston of a facing, not yet extended.
fn piston(at: (i64, i64, i64), facing: &str) -> Value {
    put_with(at.0, at.1, at.2, "minecraft:piston", &[("facing", facing), ("extended", "false")])
}

/// A block event at a tick: the piston at a place extends towards a facing id.
fn extend(tick: i64, at: (i64, i64, i64), facing: i64) -> Value {
    json!({"tick": tick, "x": at.0, "y": at.1, "z": at.2, "b0": 0, "b1": facing})
}

fn scene(label: &str, blocks: Vec<Value>, start: (f64, f64, f64), events: Vec<Value>) -> Scene {
    Scene::new(label, cat![floor(), blocks], keys(12, &[])).start(start.0, start.1, start.2).block_events(events)
}

pub fn rows() -> Vec<Value> {
    [pushing_rows(), carrying_rows()].concat()
}

/// The arm and the blocks it pushes into the player.
fn pushing_rows() -> Vec<Value> {
    vec![
        // The arm reaches into the player's side.
        scene("push-east", vec![piston((0, 64, 0), "east")], (1.5, 64.0, 0.5), vec![extend(1, (0, 64, 0), 5)]).row(),
        // The arm reaches up under the player's feet.
        scene("push-up", vec![piston((6, 64, 6), "up")], (6.5, 65.0, 6.5), vec![extend(1, (6, 64, 6), 1)]).row(),
        // Two arms in the same tick, from both sides.
        scene(
            "push-two-sides",
            vec![piston((0, 64, 10), "east"), piston((3, 64, 10), "west")],
            (1.5, 64.0, 10.5),
            vec![extend(1, (0, 64, 10), 5), extend(1, (3, 64, 10), 4)],
        )
        .row(),
        // A block pushed along the player's way.
        scene(
            "push-block-east",
            vec![piston((0, 64, 14), "east"), put(1, 64, 14, "minecraft:stone")],
            (2.5, 64.0, 14.5),
            vec![extend(1, (0, 64, 14), 5)],
        )
        .row(),
    ]
}

/// Blocks that carry, throw or leave the player, and the players a piston does not move.
fn carrying_rows() -> Vec<Value> {
    vec![
        // A slime block throws what it pushes (high up, where the floor is not stuck to it).
        scene(
            "push-slime",
            vec![piston((8, 80, 2), "east"), put(9, 80, 2, "minecraft:slime_block")],
            (10.5, 80.0, 2.5),
            vec![extend(1, (8, 80, 2), 5)],
        )
        .row(),
        // Honey takes along what stands on it.
        scene(
            "carry-honey",
            vec![piston((8, 80, 5), "east"), put(9, 80, 5, "minecraft:honey_block")],
            (9.5, 81.0, 5.5),
            vec![extend(1, (8, 80, 5), 5)],
        )
        .row(),
        // A stone pushed up carries the player standing on it.
        scene(
            "lift-on-stone",
            vec![piston((14, 64, 8), "up"), put(14, 65, 8, "minecraft:stone")],
            (14.5, 66.0, 8.5),
            vec![extend(1, (14, 64, 8), 1)],
        )
        .row(),
        // A sticky piston draws its arm back past the player.
        scene(
            "draw-back",
            vec![
                put_with(0, 64, 3, "minecraft:sticky_piston", &[("facing", "east"), ("extended", "true")]),
                put_with(
                    1,
                    64,
                    3,
                    "minecraft:piston_head",
                    &[("facing", "east"), ("type", "sticky"), ("short", "false")],
                ),
            ],
            (1.5, 64.0, 3.5),
            vec![json!({"tick": 1, "x": 0, "y": 64, "z": 3, "b0": 1, "b1": 5})],
        )
        .row(),
        // A spectator goes through, and a sneaking player is pushed all the same.
        scene("push-spectator", vec![piston((8, 64, 12), "east")], (9.5, 64.0, 12.5), vec![extend(1, (8, 64, 12), 5)])
            .game_mode("spectator")
            .row(),
        Scene::new("push-sneaking", cat![floor(), vec![piston((8, 64, 14), "east")]], keys(12, &["sneak"]))
            .start(9.5, 64.0, 14.5)
            .block_events(vec![extend(1, (8, 64, 14), 5)])
            .row(),
        // A sneaking player is pushed off the edge it stands at.
        Scene::new(
            "push-sneaking-off-edge",
            cat![floor(), vec![piston((8, 65, 10), "east"), put(9, 64, 10, "minecraft:stone")]],
            keys(12, &["sneak"]),
        )
        .start(9.5, 65.0, 10.5)
        .block_events(vec![extend(1, (8, 65, 10), 5)])
        .row(),
    ]
}
