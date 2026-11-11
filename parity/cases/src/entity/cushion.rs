//! A cushion (`CushionRenderer`): the side its turn snaps to decides where the model is drawn,
//! turned to that side, flipped and lowered. Only the newest version has cushions.

use serde_json::Value;

use super::objects::placed;
use super::pose::{entity, fields, row};

pub fn cushion() -> Vec<Value> {
    if crate::components::is_26_2() {
        return Vec::new();
    }
    let defaults = fields!(y_rot = 0.0).to_vec();
    let turns = [
        ("south", 0.0),
        ("west", 90.0),
        ("north", 180.0),
        ("east", -90.0),
        ("turned-a-little", 37.0),
        ("turned-the-other-way", -23.5),
        ("on-the-edge-to-west", 45.0),
        ("just-before-the-edge", 44.9),
        ("on-the-edge-to-east", -45.0),
        ("just-over-the-edge", -45.1),
        ("a-full-turn", 360.0),
        ("far-round", 1234.5),
    ];
    let poses = turns
        .iter()
        .map(|(label, y_rot)| placed(row(label, &defaults, fields!(y_rot = *y_rot)), [10.5, 64.25, -3.75]))
        .collect();
    vec![entity("cushion-pose", "cushion_pose", poses)]
}
