//! The particle engine of a level: what the level lets in, the options, the
//! packets, the level events and the blocks about the player that animate.

use serde_json::{Value, json};

use super::scene::partials;
use crate::bits::f64_bits;

/// A particle engine scenario.
pub struct Engine {
    label: String,
    events: Vec<Value>,
    status: String,
    ticks: i64,
    seeds: i64,
    camera: (f64, f64, f64),
    level_seed: i64,
    listener_seed: i64,
    engine_seed: i64,
    describe: i64,
    sprites: i64,
    blocks: Vec<Value>,
    fields: Vec<(String, Value)>,
}

pub fn engine_row(label: &str, events: Vec<Value>) -> Engine {
    Engine {
        label: label.to_owned(),
        events,
        status: "ALL".to_owned(),
        ticks: 3,
        seeds: 8,
        camera: (0.0, 64.0, 0.0),
        level_seed: 101,
        listener_seed: 202,
        engine_seed: 303,
        describe: 4,
        sprites: 8,
        blocks: vec![],
        fields: vec![],
    }
}

impl Engine {
    pub fn status(self, status: &str) -> Self {
        Self { status: status.to_owned(), ..self }
    }

    pub fn ticks(self, ticks: i64) -> Self {
        Self { ticks, ..self }
    }

    pub fn seeds(self, seeds: i64) -> Self {
        Self { seeds, ..self }
    }

    pub fn describe(self, describe: i64) -> Self {
        Self { describe, ..self }
    }

    /// Where the camera is (the limiter reads the distance to it).
    pub fn camera(self, camera: (f64, f64, f64)) -> Self {
        Self { camera, ..self }
    }

    pub fn blocks(self, blocks: Vec<Value>) -> Self {
        Self { blocks, ..self }
    }

    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.fields.push((key.to_owned(), value));
        self
    }

    pub fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "blocks": self.blocks, "chunk_radius": 1, "events": self.events,
            "status": self.status, "tick_count": self.ticks, "particle_seeds": self.seeds,
            "particle_seed_base": 1000, "particle_seed_step": 7919, "camera_x": f64_bits(self.camera.0),
            "camera_y": f64_bits(self.camera.1), "camera_z": f64_bits(self.camera.2),
            "level_seed": self.level_seed, "listener_seed": self.listener_seed, "engine_seed": self.engine_seed,
            "sprites": self.sprites, "describe": self.describe, "partials": partials(),
        });
        for (key, value) in self.fields {
            row[key] = value;
        }
        row
    }
}
