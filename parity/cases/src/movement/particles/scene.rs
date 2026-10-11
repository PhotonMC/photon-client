//! Particles of the reference ticked in a scene (`Particle.tick`, the smoke
//! and flame providers), with the random seed they are made with fixed.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::movement::world::put;

pub const STILL: (f64, f64, f64) = (0.0, 0.0, 0.0);
pub const NEAR: (f64, f64, f64) = (5.0, 64.0, 5.0);
pub const FAR: (f64, f64, f64) = (40.0, 64.0, 0.0);

/// The partial ticks every row is drawn at.
pub fn partials() -> Vec<&'static str> {
    vec!["00000000", "3f000000", "3f800000"]
}

/// A stone ceiling above the floor.
pub fn particle_ceiling() -> Vec<Value> {
    (-8..9).flat_map(|x| (-8..9).map(move |z| put(x, 68, z, "minecraft:stone"))).collect()
}

/// A stone wall at x = 2.
pub fn particle_wall() -> Vec<Value> {
    (63..70).flat_map(|y| (-8..9).map(move |z| put(2, y, z, "minecraft:stone"))).collect()
}

pub fn particle_slab() -> Vec<Value> {
    vec![put(0, 66, 0, "minecraft:stone_slab")]
}

/// A pool of water.
pub fn particle_pool() -> Vec<Value> {
    (-3..4)
        .flat_map(|x| (60..67).map(move |y| (x, y)))
        .flat_map(|(x, y)| (-3..4).map(move |z| put(x, y, z, "minecraft:water")))
        .collect()
}

pub fn particle_pool_floor() -> Vec<Value> {
    (-3..4).flat_map(|x| (-3..4).map(move |z| put(x, 59, z, "minecraft:stone"))).collect()
}

/// A block of stone to be inside of.
pub fn particle_solid() -> Vec<Value> {
    (-3..4)
        .flat_map(|x| (60..68).map(move |y| (x, y)))
        .flat_map(|(x, y)| (-3..4).map(move |z| put(x, y, z, "minecraft:stone")))
        .collect()
}

/// A particle ticked in a scene.
pub struct Particle {
    label: String,
    particle: String,
    blocks: Vec<Value>,
    at: (f64, f64, f64),
    delta: (f64, f64, f64),
    seed: i64,
    ticks: i64,
    sprites: i64,
    engine_seed: i64,
    fields: Vec<(String, Value)>,
}

pub fn row(
    label: &str,
    particle: &str,
    blocks: Vec<Value>,
    at: (f64, f64, f64),
    delta: (f64, f64, f64),
    when: (i64, i64),
) -> Particle {
    Particle {
        label: label.to_owned(),
        particle: particle.to_owned(),
        blocks,
        at,
        delta,
        seed: when.0,
        ticks: when.1,
        sprites: 8,
        engine_seed: 7,
        fields: vec![],
    }
}

impl Particle {
    pub fn sprites(self, sprites: i64) -> Self {
        Self { sprites, ..self }
    }

    pub fn engine_seed(self, engine_seed: i64) -> Self {
        Self { engine_seed, ..self }
    }

    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.fields.push((key.to_owned(), value));
        self
    }

    pub fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "particle": self.particle, "blocks": self.blocks, "chunk_radius": 1,
            "x": f64_bits(self.at.0), "y": f64_bits(self.at.1), "z": f64_bits(self.at.2),
            "xa": f64_bits(self.delta.0), "ya": f64_bits(self.delta.1), "za": f64_bits(self.delta.2),
            "seed": self.seed, "engine_seed": self.engine_seed, "sprites": self.sprites, "partials": partials(),
            "tick_count": self.ticks,
        });
        for (key, value) in self.fields {
            row[key] = value;
        }
        row
    }
}
