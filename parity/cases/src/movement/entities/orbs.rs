//! An experience orb ticked in a scene (`ExperienceOrb.tick` on the client).

use crate::movement::entities::prelude::*;

const ORB: &str = "minecraft:experience_orb";

fn orb(label: &str, blocks: Vec<Value>, at: (f64, f64, f64), motion: (f64, f64, f64), ticks: i64) -> Item {
    row(label, blocks, at, motion, ticks).entity_type(ORB)
}

pub fn rows() -> Vec<Value> {
    let pool: Vec<Value> = (6..9)
        .flat_map(|x| (6..9).flat_map(move |z| (64..67).map(move |y| level(x, y, z, 0, "minecraft:water"))))
        .collect();
    let ice: Vec<Value> = (12..15).flat_map(|x| (12..15).map(move |z| put(x, 63, z, "minecraft:ice"))).collect();
    // A source with water flowing away from it along the floor.
    let stream: Vec<Value> = cat![
        vec![level(0, 64, 18, 0, "minecraft:water")],
        (1..8).map(|x| level(x, 64, 18, x.min(7), "minecraft:water")).collect::<Vec<_>>()
    ];
    vec![
        // Falls on to the floor and bounces off it.
        orb("falls", floor(), (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), 40).row(),
        orb("falls-sideways", floor(), (0.5, 66.0, 0.5), (0.2, 0.0, -0.15), 40).row(),
        orb("rests", floor(), (0.5, 64.0, 0.5), (0.0, 0.0, 0.0), 12).on_ground(true).row(),
        // Slides along the floor and slows down.
        orb("slides", floor(), (0.5, 64.0, 0.5), (0.3, 0.0, 0.0), 40).on_ground(true).row(),
        orb("slides-on-ice", cat![floor(), ice], (13.5, 64.0, 13.5), (0.3, 0.0, 0.0), 40).on_ground(true).row(),
        // Falls into water and floats.
        orb("sinks-in-water", cat![floor(), pool], (7.5, 68.0, 7.5), (0.0, 0.0, 0.0), 60).row(),
        // A player near pulls the orb towards them; far away, they do not.
        orb("pulled-by-player", floor(), (0.5, 64.5, 0.5), (0.0, 0.0, 0.0), 30)
            .on_ground(true)
            .players(vec![((4.5, 64.0, 0.5), [0, 1, 0, 1])])
            .row(),
        orb("player-too-far", floor(), (0.5, 64.5, 0.5), (0.0, 0.0, 0.0), 12)
            .on_ground(true)
            .players(vec![((20.5, 64.0, 0.5), [0, 1, 0, 1])])
            .row(),
        // A stream of water flowing along the floor carries it.
        orb("carried-by-stream", cat![floor(), stream], (2.5, 64.5, 18.5), (0.0, 0.0, 0.0), 40).row(),
        // Starts inside a block: no gravity pulls it.
        orb(
            "inside-block",
            cat![floor(), vec![put(10, 64, 10, "minecraft:stone")]],
            (10.5, 64.3, 10.5),
            (0.0, 0.0, 0.0),
            12,
        )
        .row(),
    ]
}
