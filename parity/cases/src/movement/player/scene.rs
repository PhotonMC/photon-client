//! The player scenes: a start pose, the keys held on every tick and what
//! the world around holds.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::movement::world::{column, column_with, put, put_with};

const KEYS: [&str; 7] = ["forward", "backward", "left", "right", "jump", "sneak", "sprint"];

/// `count` ticks with the keys named in `held` down.
pub fn keys(count: usize, held: &[&str]) -> Vec<Value> {
    let tick: serde_json::Map<String, Value> =
        KEYS.iter().map(|name| ((*name).to_owned(), json!(held.contains(name)))).collect();
    vec![Value::Object(tick); count]
}

/// The ticks with their look set to `y_rot` and `x_rot`.
pub fn turned(mut ticks: Vec<Value>, y_rot: Option<f64>, x_rot: Option<f64>) -> Vec<Value> {
    for tick in &mut ticks {
        if let Some(y_rot) = y_rot {
            tick["y_rot"] = json!(f32_bits(y_rot));
        }
        if let Some(x_rot) = x_rot {
            tick["x_rot"] = json!(f32_bits(x_rot));
        }
    }
    ticks
}

/// Each tick's look as `(y_rot, x_rot)`, the last held on.
pub fn looking(mut ticks: Vec<Value>, angles: &[(f64, f64)]) -> Vec<Value> {
    let count = ticks.len().max(1);
    for (index, tick) in ticks.iter_mut().enumerate() {
        let (yaw, pitch) = angles[(index * angles.len() / count).min(angles.len() - 1)];
        tick["y_rot"] = json!(f32_bits(yaw));
        tick["x_rot"] = json!(f32_bits(pitch));
    }
    ticks
}

/// A player scene.
pub struct Scene {
    label: String,
    blocks: Vec<Value>,
    ticks: Vec<Value>,
    start: (f64, f64, f64),
    y_rot: f64,
    x_rot: f64,
    game_mode: &'static str,
    auto_jump: bool,
    border: Option<(f64, f64, f64)>,
}

impl Scene {
    pub fn new(label: impl Into<String>, blocks: Vec<Value>, ticks: Vec<Value>) -> Self {
        Self {
            label: label.into(),
            blocks,
            ticks,
            start: (0.5, 64.0, 0.5),
            y_rot: 0.0,
            x_rot: 0.0,
            game_mode: "survival",
            auto_jump: false,
            border: None,
        }
    }

    /// The same scene over other blocks.
    pub fn with_blocks(self, blocks: Vec<Value>) -> Self {
        Self { blocks, ..self }
    }

    pub fn start(self, x: f64, y: f64, z: f64) -> Self {
        Self { start: (x, y, z), ..self }
    }

    pub fn y_rot(self, y_rot: f64) -> Self {
        Self { y_rot, ..self }
    }

    pub fn x_rot(self, x_rot: f64) -> Self {
        Self { x_rot, ..self }
    }

    pub fn game_mode(self, game_mode: &'static str) -> Self {
        Self { game_mode, ..self }
    }

    pub fn auto_jump(self) -> Self {
        Self { auto_jump: true, ..self }
    }

    pub fn border(self, x: f64, z: f64, size: f64) -> Self {
        Self { border: Some((x, z, size)), ..self }
    }

    pub fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "blocks": self.blocks, "chunk_radius": 1, "entity_id": 7,
            "x": f64_bits(self.start.0), "y": f64_bits(self.start.1), "z": f64_bits(self.start.2),
            "y_rot": f32_bits(self.y_rot), "x_rot": f32_bits(self.x_rot), "game_mode": self.game_mode,
            "auto_jump": self.auto_jump, "ticks": self.ticks,
        });
        if let Some((x, z, size)) = self.border {
            row["border"] = json!({"x": f64_bits(x), "z": f64_bits(z), "size": f64_bits(size)});
        }
        row
    }
}

/// A scene with no options.
pub fn scene(label: impl Into<String>, blocks: Vec<Value>, ticks: Vec<Value>) -> Value {
    Scene::new(label, blocks, ticks).row()
}

/// A wall across the way ahead.
pub fn wall() -> Vec<Value> {
    (-3..4).flat_map(|x| column(x, 4, 64, 66)).collect()
}

/// Water flowing away over a floor, each block one level lower.
pub fn stream() -> Vec<Value> {
    let mut blocks: Vec<Value> = (-1..2).map(|x| put(x, 64, -3, "minecraft:water")).collect();
    for x in -1..2 {
        for z in -2..4 {
            blocks.push(put_with(x, 64, z, "minecraft:water", &[("level", &(z + 3).to_string())]));
        }
    }
    blocks
}

/// Bubble columns drawing up and dragging down.
pub fn bubbles() -> Vec<Value> {
    let mut blocks = vec![put(0, 63, 0, "minecraft:soul_sand")];
    blocks.extend(column_with(0, 0, 64, 70, "minecraft:bubble_column", &[("drag", "false")]));
    blocks.push(put(1, 63, 0, "minecraft:magma_block"));
    blocks.extend(column_with(1, 0, 64, 70, "minecraft:bubble_column", &[("drag", "true")]));
    for x in -1..3 {
        for z in [-1, 1] {
            blocks.extend(column(x, z, 64, 70));
        }
    }
    blocks.extend(column(-1, 0, 64, 70));
    blocks.extend(column(2, 0, 64, 70));
    blocks
}

/// Lava `depth` blocks deep from `low` to `high` on both axes.
pub fn lava_pool(depth: i64, low: i64, high: i64) -> Vec<Value> {
    let mut blocks = Vec::new();
    for x in low..=high {
        for z in low..=high {
            for level in 0..depth {
                blocks.push(put(x, 64 + level, z, "minecraft:lava"));
            }
        }
    }
    blocks
}
