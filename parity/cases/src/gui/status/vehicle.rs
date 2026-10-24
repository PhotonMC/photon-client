//! The rows of a player riding a saddled horse.

use super::{Status, at, status};
use crate::line::float_text;
use serde_json::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (max, health) in [
        (15.0, 15.0),
        (15.0, 0.5),
        (15.0, 8.0),
        (30.0, 30.0),
        (30.0, 17.0),
        (53.0, 41.0),
        (90.0, 90.0),
        (100.0, 61.0),
        (2.0, 2.0),
    ] {
        let label = format!("mount-{}-{}", float_text(max), float_text(health));
        rows.push(Status { vehicle: Some((max, health, 0.0)), ..status() }.row(&label));
    }
    for jump in [0.0, 0.01, 0.3, 0.5, 0.999, 1.0] {
        let label = format!("jump-{}", float_text(jump));
        rows.push(Status { vehicle: Some((15.0, 15.0, jump)), ..status() }.row(&label));
    }
    // The air bubbles climb over the mount's rows, and the food row gives way to them.
    for max in [15.0, 30.0, 90.0] {
        let label = format!("mount-air-{}", float_text(max));
        rows.push(Status { vehicle: Some((max, max, 0.5)), air: 120, eye_in_water: true, ..status() }.row(&label));
    }
    rows.push(Status { vehicle: Some((15.0, 9.0, 0.4)), xp: (7, 0.5), air: 200, ..at(1, 12.0) }.row("mount-with-xp"));
    rows
}
