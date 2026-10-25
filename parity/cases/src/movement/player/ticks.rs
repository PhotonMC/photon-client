//! The reference's own client player, ticked through scenes.
//!
//! Each row is a scene, a start pose and the keys held on every tick. The
//! expected output is everything the tick leaves behind (position, motion,
//! the collision flags, the pose) and every packet the player asked to send,
//! as the bytes the packet's own codec writes. This is the whole client
//! tick, not its parts: input, `aiStep`, `travel`, `move` and the position
//! packets, in the order the reference runs them.

use serde_json::{Value, json};

use super::scene::{Scene, bubbles, keys, scene, stream, turned, wall};
use crate::case::{case, merge};
use crate::movement::world::{cat, column, column_of, column_with, floor, floor_block, floor_of, put, put_with};

fn region(x: (i64, i64), z: (i64, i64), y: i64, block: &str) -> Vec<Value> {
    (x.0..x.1).flat_map(|x| (z.0..z.1).map(move |z| (x, z))).map(|(x, z)| put(x, y, z, block)).collect()
}

fn pool() -> Vec<Value> {
    (-6..7)
        .flat_map(|x| (-6..7).map(move |z| (x, z)))
        .flat_map(|(x, z)| (64..67).map(move |y| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, "minecraft:water"))
        .collect()
}

fn ladder() -> Vec<Value> {
    column_with(0, 1, 64, 70, "minecraft:ladder", &[("facing", "north")])
}

fn buried(block: &str) -> Vec<Value> {
    (-1..2)
        .flat_map(|x| (64..67).map(move |y| (x, y)))
        .flat_map(|(x, y)| (-1..2).map(move |z| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, block))
        .collect()
}

fn plants() -> Vec<Value> {
    let names = ["short_grass", "fern", "dandelion", "sweet_berry_bush", "oak_sapling"];
    let mut blocks: Vec<Value> = (-2..3)
        .flat_map(|x| (-2..12).map(move |z| (x, z)))
        .map(|(x, z)| put(x, 64, z, &format!("minecraft:{}", names[(x + z).rem_euclid(5) as usize])))
        .collect();
    for x in -2..3 {
        for y in [64, 65] {
            let half = if y == 64 { "lower" } else { "upper" };
            blocks.push(put_with(x, y, 12, "minecraft:tall_grass", &[("half", half)]));
        }
    }
    blocks
}

fn powder_snow_floor() -> Vec<Value> {
    let outside = floor().into_iter().filter(|block| {
        let (x, z) = (block["x"].as_i64().unwrap_or(0), block["z"].as_i64().unwrap_or(0));
        !(-3..=3).contains(&x) || !(-3..=7).contains(&z)
    });
    let pit: Vec<Value> = (-3..4)
        .flat_map(|x| (-3..8).map(move |z| (x, z)))
        .flat_map(|(x, z)| (61..64).map(move |y| (x, y, z)))
        .map(|(x, y, z)| put(x, y, z, "minecraft:powder_snow"))
        .collect();
    cat![outside.collect(), pit, floor_of(60, "minecraft:stone", -16, 31)]
}

/// Scenes on foot: walking, jumping, sprinting and sneaking on floors of
/// different grip, stairs, ladders, water, lava and the world border.
/// Walking, turning, sprinting, sneaking and jumping on a flat floor.
fn walking_rows() -> Vec<Value> {
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        scene("stand-still", floor(), keys(6, &[])),
        // The view is a pig's: the keys are still sent, no position is, and nothing moves.
        Scene::new("camera-elsewhere-forward", floor(), keys(8, &["forward", "sprint"]))
            .camera_entity(41, "minecraft:pig")
            .row(),
        // A spectator flies, and only the camera entity's keys move it up and down.
        Scene::new("camera-elsewhere-spectator-rise", floor(), keys(6, &["jump"]))
            .game_mode("spectator")
            .camera_entity(41, "minecraft:pig")
            .row(),
        Scene::new("camera-elsewhere-spectator-sink", floor(), keys(6, &["sneak"]))
            .game_mode("spectator")
            .camera_entity(41, "minecraft:pig")
            .row(),
        Scene::new("camera-elsewhere-turned", floor(), turned(keys(6, &["jump"]), Some(90.0), Some(20.0)))
            .camera_entity(41, "minecraft:pig")
            .row(),
        walk("walk-forward", floor(), 20, &["forward"]),
        walk("walk-backward", floor(), 12, &["backward"]),
        walk("strafe-left", floor(), 12, &["left"]),
        walk("walk-diagonal", floor(), 12, &["forward", "right"]),
        Scene::new("walk-turned-45", floor(), keys(12, &["forward"])).y_rot(45.0).row(),
        scene(
            "walk-then-turn",
            floor(),
            [keys(6, &["forward"]), turned(keys(8, &["forward"]), Some(90.0), None)].concat(),
        ),
        scene("walk-then-stop", floor(), [keys(8, &["forward"]), keys(10, &[])].concat()),
        walk("sprint", floor(), 20, &["forward", "sprint"]),
        scene("sprint-then-release", floor(), [keys(10, &["forward", "sprint"]), keys(6, &["forward"])].concat()),
        walk("sneak-walk", floor(), 14, &["forward", "sneak"]),
        scene("jump-once", floor(), [keys(1, &["jump"]), keys(14, &[])].concat()),
        walk("jump-held", floor(), 30, &["jump"]),
        walk("walk-jump", floor(), 20, &["forward", "jump"]),
        walk("sprint-jump", floor(), 24, &["forward", "sprint", "jump"]),
    ]
}

/// Walls, steps, ledges and floors of every grip.
fn stepping_rows() -> Vec<Value> {
    let slab_step = region((-3, 4), (3, 8), 64, "minecraft:stone_slab");
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        walk("walk-into-wall", cat![floor(), wall()], 20, &["forward"]),
        walk("step-onto-slab", cat![floor(), slab_step], 16, &["forward"]),
        walk("walk-into-block-step", cat![floor(), block_step()], 16, &["forward"]),
        scene(
            "jump-onto-block",
            cat![floor(), block_step()],
            [keys(4, &["forward"]), keys(10, &["forward", "jump"])].concat(),
        ),
        Scene::new("fall-onto-floor", floor(), keys(30, &[])).start(0.5, 72.0, 0.5).row(),
        walk("walk-off-ledge", ledge(), 20, &["forward"]),
        walk("sneak-at-ledge", ledge(), 24, &["forward", "sneak"]),
        scene("walk-on-ice", floor_block("minecraft:ice"), [keys(10, &["forward"]), keys(12, &[])].concat()),
        walk("walk-on-soul-sand", floor_block("minecraft:soul_sand"), 16, &["forward"]),
        Scene::new("land-on-slime", floor_block("minecraft:slime_block"), keys(40, &[])).start(0.5, 72.0, 0.5).row(),
        walk("walk-on-honey", floor_block("minecraft:honey_block"), 16, &["forward"]),
    ]
}

/// Ladders and swimming, and the creative and spectator flights.
fn climbing_rows() -> Vec<Value> {
    let ladder_wall = column(0, 2, 64, 70);
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        walk("climb-ladder", cat![floor(), ladder(), ladder_wall.clone()], 30, &["forward"]),
        Scene::new("swim", cat![floor_block("minecraft:stone"), pool()], keys(20, &["forward"]))
            .start(0.5, 65.0, 0.5)
            .row(),
        Scene::new("swim-up", cat![floor_block("minecraft:stone"), pool()], keys(20, &["jump"]))
            .start(0.5, 64.2, 0.5)
            .row(),
        Scene::new("auto-jump", cat![floor(), block_step()], keys(20, &["forward"])).auto_jump().row(),
        scene("look-down-walk", floor(), turned(keys(10, &["forward"]), None, Some(90.0))),
        Scene::new(
            "creative-double-jump-fly",
            floor(),
            [keys(1, &["jump"]), keys(2, &[]), keys(1, &["jump"]), keys(10, &["jump"]), keys(10, &["forward"])]
                .concat(),
        )
        .game_mode("creative")
        .row(),
        Scene::new("spectator-float", floor(), keys(10, &["forward", "jump"])).game_mode("spectator").row(),
        walk("sprint-swim", cat![floor(), pool()], 24, &["forward", "sprint"]),
        Scene::new("sneak-sink", cat![floor(), pool()], keys(16, &["sneak"])).start(0.5, 65.5, 0.5).row(),
        Scene::new("fall-into-water", cat![floor(), pool()], keys(40, &[])).start(0.5, 90.0, 0.5).row(),
        Scene::new("fall-far", floor(), keys(40, &[])).start(0.5, 95.0, 0.5).row(),
    ]
}

/// Ceilings, streams, lava, slime and ladders at a ledge.
/// Ceilings, streams, lava and ladders.
fn obstacle_rows() -> Vec<Value> {
    let ladder_wall = column(0, 2, 64, 70);
    let ceiling = region((-4, 5), (-4, 9), 65, "minecraft:stone");
    let low_roof = region((-2, 3), (-2, 3), 66, "minecraft:stone");
    let lava = region((-3, 4), (-3, 8), 64, "minecraft:lava");
    let trapdoor_ladder = cat![
        column_with(0, 1, 64, 65, "minecraft:ladder", &[("facing", "north")]),
        vec![put_with(0, 66, 1, "minecraft:oak_trapdoor", &[("facing", "north"), ("open", "true")])],
    ];
    let side_wall: Vec<Value> = (-16..4).flat_map(|x| column(x, 1, 64, 65)).collect();
    let waterfall = cat![
        (0..2)
            .flat_map(|x| (64..72).map(move |y| put_with(x, y, 0, "minecraft:water", &[("level", "8")])))
            .collect::<Vec<Value>>(),
        (-1..3).flat_map(|x| column(x, 1, 64, 72)).collect::<Vec<Value>>(),
    ];
    let fenced_stream = cat![stream(), (-2..4).map(|z| put(2, 64, z, "minecraft:oak_fence")).collect::<Vec<Value>>()];
    let honey_wall = column_of(1, 0, 64, 72, "minecraft:honey_block");
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        walk("crawl-under-ceiling", cat![floor(), ceiling], 16, &["forward"]),
        walk("jump-under-ceiling", cat![floor(), low_roof], 14, &["jump"]),
        Scene::new("honey-wall-slide", cat![floor(), honey_wall], keys(30, &["forward"]))
            .start(0.5, 72.0, 0.5)
            .y_rot(-90.0)
            .row(),
        scene("flowing-water-push", cat![floor(), stream()], keys(20, &[])),
        Scene::new(
            "waterfall-swim",
            cat![floor(), waterfall],
            [keys(10, &["forward"]), keys(20, &["forward", "jump"])].concat(),
        )
        .start(0.5, 66.0, 0.5)
        .row(),
        walk("stream-past-fence", cat![floor(), fenced_stream], 20, &["left"]),
        walk("lava-wade", cat![floor(), lava], 16, &["forward"]),
        walk("trapdoor-ladder", cat![floor(), trapdoor_ladder, ladder_wall], 40, &["forward"]),
        Scene::new("sprint-along-wall", cat![floor(), side_wall], keys(16, &["forward", "sprint"])).y_rot(85.0).row(),
    ]
}

/// Slime, flight and ladders at a ledge.
fn landing_rows() -> Vec<Value> {
    let edge_ladder = cat![column_with(0, 2, 56, 62, "minecraft:ladder", &[("facing", "north")]), column(0, 3, 56, 63)];
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        Scene::new("sneak-onto-slime", floor_block("minecraft:slime_block"), keys(24, &["sneak"]))
            .start(0.5, 68.0, 0.5)
            .row(),
        Scene::new(
            "creative-fly-sprint-land",
            floor(),
            [
                keys(1, &["jump"]),
                keys(1, &[]),
                keys(1, &["jump"]),
                keys(8, &["forward", "sprint"]),
                keys(10, &["sneak"]),
                keys(4, &[]),
            ]
            .concat(),
        )
        .game_mode("creative")
        .row(),
        Scene::new("auto-jump-sprint", cat![floor(), block_step()], keys(20, &["forward", "sprint"])).auto_jump().row(),
        walk("walk-off-ledge-into-ladder", cat![ledge(), edge_ladder], 30, &["forward"]),
    ]
}

fn block_step() -> Vec<Value> {
    (-3..4).flat_map(|x| (3..8).flat_map(move |z| column(x, z, 64, 64))).collect()
}

fn ledge() -> Vec<Value> {
    region((-3, 4), (-3, 2), 63, "minecraft:stone")
}

/// Scenes in blocks that hold a player back, hurt, the bubble columns, the
/// world border, and a player buried in a block.
/// Blocks that hold a player back, and being hurt.
fn held_back() -> Vec<Value> {
    let powder_ahead = region((-3, 4), (2, 5), 64, "minecraft:powder_snow");
    let webs: Vec<Value> = (-1..2).flat_map(|x| (64..66).map(move |y| put(x, y, 2, "minecraft:cobweb"))).collect();
    let berries: Vec<Value> = (-1..2)
        .flat_map(|x| (2..4).map(move |z| put_with(x, 64, z, "minecraft:sweet_berry_bush", &[("age", "1")])))
        .collect();
    let walk =
        |label: &'static str, blocks: Vec<Value>, count: usize, held: &[&str]| scene(label, blocks, keys(count, held));
    vec![
        scene(
            "sink-in-powder-snow",
            powder_snow_floor(),
            [keys(10, &[]), keys(20, &["forward"]), keys(20, &["forward", "jump"])].concat(),
        ),
        walk("walk-into-powder-snow", cat![floor(), powder_ahead], 30, &["forward"]),
        scene(
            "walk-through-cobweb",
            cat![floor(), webs],
            [keys(30, &["forward"]), keys(6, &["forward", "jump"])].concat(),
        ),
        walk("walk-through-berry-bush", cat![floor(), berries], 30, &["forward"]),
        scene(
            "sneak-in-plants",
            cat![floor(), plants()],
            [
                keys(4, &["forward"]),
                keys(10, &["forward", "sneak"]),
                keys(10, &["forward"]),
                keys(6, &["sneak"]),
                keys(6, &[]),
            ]
            .concat(),
        ),
        scene("hurt-and-die", floor(), hurt()),
    ]
}

/// A player hurt by each kind of damage, then killed.
fn hurt() -> Vec<Value> {
    [
        json!({"damage": "minecraft:generic"}),
        json!({"damage": "minecraft:drown"}),
        json!({"damage": "minecraft:on_fire"}),
        json!({"damage": "minecraft:sweet_berry_bush"}),
        json!({"damage": "minecraft:freeze"}),
        json!({"damage": "minecraft:thorns"}),
        json!({"damage": "minecraft:lava"}),
        json!({"death": true}),
        json!({}),
    ]
    .into_iter()
    .zip(keys(9, &[]))
    .map(|(extra, mut tick)| {
        merge(&mut tick, extra);
        tick
    })
    .collect()
}

/// The bubble columns, the world border, and a player buried in a block.
fn border_rows() -> Vec<Value> {
    let mut rows = vec![
        Scene::new("ride-bubble-column", cat![floor(), bubbles()], keys(40, &[])).start(0.5, 64.0, 0.5).row(),
        Scene::new("sink-in-bubble-column", cat![floor(), bubbles()], keys(30, &[])).start(1.5, 69.0, 0.5).row(),
        Scene::new("walk-into-border", floor(), keys(30, &["forward"])).border(0.5, 0.5, 6.0).row(),
        Scene::new(
            "sprint-jump-into-border",
            floor(),
            [keys(10, &["forward", "sprint"]), keys(20, &["forward", "sprint", "jump"])].concat(),
        )
        .border(0.5, 0.5, 6.0)
        .row(),
        Scene::new("slide-along-border-corner", floor(), keys(24, &["forward", "left"]))
            .border(0.5, 0.5, 5.0)
            .y_rot(45.0)
            .row(),
        Scene::new("start-outside-border", floor(), keys(10, &["forward"])).border(40.5, 40.5, 6.0).row(),
        Scene::new("crouch-in-border", floor(), keys(20, &["forward", "sneak"])).border(0.5, 0.5, 4.0).row(),
    ];
    // The block the screen effects cover the view with, for a player buried
    // in it: one that blocks the view, one that does not, one that is not
    // drawn, and one that is only part of the way round.
    for (name, block) in [
        ("stone", "minecraft:stone"),
        ("glass", "minecraft:glass"),
        ("barrier", "minecraft:barrier"),
        ("soul-sand", "minecraft:soul_sand"),
    ] {
        rows.push(
            Scene::new(format!("buried-in-{name}"), cat![floor(), buried(block)], keys(4, &[]))
                .start(0.5, 64.0, 0.5)
                .row(),
        );
    }
    rows
}

/// A nether portal's effect builds while the player stands in it and fades once they leave; an end
/// portal builds none.
fn portal_rows() -> Vec<Value> {
    let nether = |x: i64, z: i64| put_with(x, 64, z, "minecraft:nether_portal", &[("axis", "x")]);
    let strip: Vec<Value> = (2..5).map(|z| nether(0, z)).collect();
    vec![
        Scene::new("stand-in-nether-portal", cat![floor(), vec![nether(0, 0)]], keys(40, &[]))
            .start(0.5, 64.0, 0.5)
            .row(),
        Scene::new("walk-through-nether-portal", cat![floor(), strip], keys(30, &["forward"]))
            .start(0.5, 64.0, 0.5)
            .y_rot(0.0)
            .row(),
        Scene::new(
            "leave-nether-portal",
            cat![floor(), vec![nether(0, 0)]],
            [keys(16, &[]), keys(30, &["backward"])].concat(),
        )
        .start(0.5, 64.0, 0.5)
        .row(),
        Scene::new("stand-in-end-portal", cat![floor(), vec![put(0, 64, 0, "minecraft:end_portal")]], keys(20, &[]))
            .start(0.5, 64.0, 0.5)
            .row(),
    ]
}

pub fn local_player_ticks() -> Value {
    let rows = [
        walking_rows(),
        stepping_rows(),
        climbing_rows(),
        obstacle_rows(),
        landing_rows(),
        held_back(),
        border_rows(),
        portal_rows(),
        super::flight::rows(),
    ]
    .concat();
    case("movement/local-player-ticks", "movement", "local_player_ticks", json!({"rows": rows}))
}
