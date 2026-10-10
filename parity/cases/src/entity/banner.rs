//! Banners: every model the real `BannerRenderer` submits.

use serde_json::{Value, json};

use super::objects::DYES;
use super::pose::entity;
use crate::bits::f32_bits;

const PATTERNS: [&str; 36] = [
    "stripe_bottom",
    "stripe_top",
    "stripe_left",
    "stripe_right",
    "stripe_center",
    "stripe_middle",
    "stripe_downright",
    "stripe_downleft",
    "small_stripes",
    "cross",
    "straight_cross",
    "diagonal_left",
    "diagonal_right",
    "diagonal_up_left",
    "diagonal_up_right",
    "half_vertical",
    "half_vertical_right",
    "half_horizontal",
    "half_horizontal_bottom",
    "square_bottom_left",
    "triangle_bottom",
    "circle",
    "rhombus",
    "border",
    "curly_border",
    "gradient",
    "gradient_up",
    "bricks",
    "creeper",
    "skull",
    "flower",
    "mojang",
    "globe",
    "piglin",
    "flow",
    "guster",
];

fn layers(count: usize, offset: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "pattern": format!("minecraft:{}", PATTERNS[(offset + index) % PATTERNS.len()]),
                "color": DYES[(offset * 3 + index * 5) % 16],
            })
        })
        .collect()
}

fn banner(
    label: String,
    (wall, rotation): (Option<&str>, i64),
    color: &str,
    patterns: Vec<Value>,
    (phase, light): (f64, i64),
) -> Value {
    json!({
        "label": label, "wall": wall, "rotation": rotation, "color": color, "patterns": patterns,
        "phase": f32_bits(phase), "light": light,
    })
}

/// Every model the real `BannerRenderer` submits for banners standing at
/// each turn and hanging on each wall, of several colours, with no
/// patterns, a few and the most (sixteen), the flag at several phases.
pub fn banner_cases() -> Vec<Value> {
    let mut rows = Vec::new();
    for rotation in 0..16 {
        let index = rotation as usize;
        let phase = (rotation as f64 / 16.0, 0xF000F0 - 17);
        rows.push(banner(format!("floor {rotation}"), (None, rotation), DYES[index], layers(index % 4, index), phase));
    }
    for (index, wall) in ["north", "south", "east", "west"].into_iter().enumerate() {
        let phase = (0.37 * index as f64, 240);
        rows.push(banner(format!("wall {wall}"), (Some(wall), 0), DYES[index * 4], layers(3, index), phase));
    }
    for offset in (0..PATTERNS.len()).step_by(6) {
        let label = format!("every pattern from {offset}");
        rows.push(banner(label, (None, 3), "white", layers(6, offset), (0.5, 15728880)));
    }
    rows.push(banner("the most".to_owned(), (None, 9), "red", layers(16, 5), (0.99, 15728880)));
    rows.push(banner("past the most".to_owned(), (Some("south"), 0), "blue", layers(20, 7), (0.01, 15728880)));
    vec![entity("banner-pose", "banner_pose", rows)]
}
