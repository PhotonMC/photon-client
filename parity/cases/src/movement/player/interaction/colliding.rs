//! The player against the boxes of entities it cannot pass (`Level.getEntityCollisions`): it
//! stands on a boat and walks into one.

use serde_json::{Value, json};

use super::facing;
use crate::bits::f64_bits;
use crate::movement::player::scene::keys;

fn boat(id: i64, kind: &str, (x, y, z): (f64, f64, f64)) -> Value {
    json!({"id": id, "type": format!("minecraft:{kind}"), "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

pub fn rows() -> Vec<Value> {
    let boat_at = |kind: &str, z: f64| vec![boat(40, kind, (0.5, 64.0, z))];
    vec![
        facing("stand-on-boat", boat_at("oak_boat", 0.5), keys(30, &[])).start(0.5, 65.0, 0.5).row(),
        facing("stand-on-bamboo-raft", boat_at("bamboo_raft", 0.5), keys(30, &[])).start(0.5, 65.0, 0.5).row(),
        facing("walk-into-boat", boat_at("oak_boat", 2.5), keys(25, &["forward"])).row(),
        facing("walk-into-zombie", vec![boat(41, "zombie", (0.5, 64.0, 2.0))], keys(25, &["forward"])).row(),
        facing("walk-into-minecart", vec![boat(42, "minecart", (0.5, 64.0, 2.0))], keys(25, &["forward"])).row(),
        facing("sneak-off-boat-edge", boat_at("oak_boat", 0.5), keys(20, &["forward", "sneak"]))
            .start(0.5, 64.5625, 0.5)
            .row(),
        facing("jump-onto-boat", boat_at("oak_boat", 1.6), keys(30, &["forward", "jump"])).row(),
    ]
}
