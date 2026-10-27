//! Entities with an interpolation handler that the server moves in coarse steps (boats, the
//! bobber, experience orbs): the client eases them from packet to packet.

use crate::movement::entities::prelude::*;

const BOATS: [&str; 3] = ["minecraft:oak_boat", "minecraft:bamboo_raft", "minecraft:oak_chest_boat"];

/// A boat with no rider and no water: the packets move it, the client eases it.
fn boats() -> Vec<Value> {
    BOATS
        .iter()
        .map(|kind| {
            let name = kind.split(':').nth(1).unwrap_or(kind);
            proj(&format!("{name}-eased"), kind, floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 14)
                .events(vec![move_by(1, 4096, 0, 8192), move_by(5, 2048, 0, 4096), move_by(9, -2048, 0, 2048)])
                .row()
        })
        .collect()
}

fn boat_turns() -> Vec<Value> {
    vec![
        proj("boat-turns", "minecraft:oak_boat", floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 12)
            .events(vec![move_turn(1, (2048, 0, 0), (32, 4)), move_turn(5, (0, 0, 2048), (96, 0))])
            .row(),
        // A turn across the seam, and a move that leaves the entity where it is.
        proj("boat-turns-across-the-seam", "minecraft:oak_boat", floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 10)
            .y_rot(170.0)
            .events(vec![move_turn(1, (0, 0, 0), (-120, 0))])
            .row(),
        proj("boat-sync", "minecraft:oak_boat", floor(), (0.5, 65.0, 0.5), (0.0, 0.0, 0.0), 10)
            .events(vec![sync_to(2, 1.5, 65.0, 2.5)])
            .row(),
    ]
}

fn orbs() -> Vec<Value> {
    vec![
        proj("orb-eased", "minecraft:experience_orb", floor(), (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 12)
            .events(vec![move_by(1, 2048, 0, 1024), move_by(5, 1024, 0, 1024)])
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [boats(), boat_turns(), orbs()].concat()
}
