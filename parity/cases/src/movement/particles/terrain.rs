//! Particles made of terrain: what a provider makes, and a block breaking.

use serde_json::{Value, json};

use super::scene::partials;
use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

/// A terrain particle scenario.
struct Terrain {
    label: String,
    mode: &'static str,
    block: &'static str,
    properties: Vec<(&'static str, &'static str)>,
    extra: Vec<(String, Value)>,
}

fn terrain(label: impl Into<String>, mode: &'static str, block: &'static str) -> Terrain {
    Terrain { label: label.into(), mode, block, properties: vec![], extra: vec![] }
}

impl Terrain {
    fn properties(self, properties: &[(&'static str, &'static str)]) -> Self {
        Self { properties: properties.to_vec(), ..self }
    }

    fn with(mut self, key: &str, value: Value) -> Self {
        self.extra.push((key.to_owned(), value));
        self
    }

    fn row(self) -> Value {
        let mut state = json!({"block": self.block});
        if !self.properties.is_empty() {
            let properties: serde_json::Map<String, Value> =
                self.properties.iter().map(|(name, value)| ((*name).to_owned(), json!(value))).collect();
            state["properties"] = Value::Object(properties);
        }
        let mut placed = state.clone();
        crate::case::merge(&mut placed, json!({"x": 2, "y": 64, "z": 3}));
        let mut row = json!({
            "label": self.label, "mode": self.mode, "state": state, "pos_x": 2, "pos_y": 64, "pos_z": 3,
            "blocks": [placed], "chunk_radius": 1, "partials": partials(),
            "sprite_uv": [f32_bits(0.25), f32_bits(0.3125), f32_bits(0.5), f32_bits(0.5625)],
        });
        for (key, value) in self.extra {
            row[key] = value;
        }
        row
    }
}

/// A particle of one block, made by a provider.
struct Single {
    terrain: Terrain,
    provider: &'static str,
    seed: i64,
    push: (f64, f64, f64),
    ticks: i64,
    engine_seed: i64,
}

fn single(label: &'static str, block: &'static str) -> Single {
    Single {
        terrain: terrain(label, "single", block),
        provider: "block",
        seed: 12345,
        push: (0.1, 0.2, -0.1),
        ticks: 24,
        engine_seed: 9,
    }
}

impl Single {
    fn provider(self, provider: &'static str, engine_seed: i64, ticks: i64) -> Self {
        Self { provider, engine_seed, ticks, ..self }
    }

    fn seed(self, seed: i64) -> Self {
        Self { seed, ..self }
    }

    fn push(self, x: f64, y: f64, z: f64) -> Self {
        Self { push: (x, y, z), ..self }
    }

    fn row(self) -> Value {
        self.terrain
            .with("provider", json!(self.provider))
            .with("seed", json!(self.seed))
            .with("engine_seed", json!(self.engine_seed))
            .with("tick_count", json!(self.ticks))
            .with("x", json!(f64_bits(2.5)))
            .with("y", json!(f64_bits(64.5)))
            .with("z", json!(f64_bits(3.5)))
            .with("xa", json!(f64_bits(self.push.0)))
            .with("ya", json!(f64_bits(self.push.1)))
            .with("za", json!(f64_bits(self.push.2)))
            .row()
    }
}

fn singles() -> Vec<Value> {
    vec![
        // What a provider makes: the draws of the constructor, the tint, the window of the texture.
        single("stone", "minecraft:stone").row(),
        single("stone-another-seed", "minecraft:stone").seed(-77).push(-0.4, 0.0, 0.3).row(),
        single("spruce-leaves-tinted", "minecraft:spruce_leaves").row(),
        single("birch-leaves-tinted", "minecraft:birch_leaves").seed(5).row(),
        single("grass-block-not-tinted", "minecraft:grass_block").row(),
        // Types that make nothing.
        single("air", "minecraft:air").row(),
        single("moving-piston", "minecraft:moving_piston").row(),
        single("barrier", "minecraft:barrier").row(),
        single("structure-void", "minecraft:structure_void").row(),
        // The two other providers of the class: still, short-lived; and rising, long-lived.
        single("crumbling", "minecraft:stone").provider("crumbling", 3, 12).row(),
        single("crumbling-another", "minecraft:stone").provider("crumbling", 4, 12).row(),
        single("dust-pillar", "minecraft:stone").provider("dust_pillar", 3, 30).push(0.0, 0.3, 0.0).row(),
    ]
}

fn destroyed() -> Vec<Value> {
    let destroy = |label: &'static str, block: &'static str| terrain(label, "destroy", block);
    vec![
        // A block breaking: one particle in each box of its shape, as many as its size asks.
        destroy("destroy-stone", "minecraft:stone").row(),
        destroy("destroy-slab", "minecraft:stone_slab")
            .properties(&[("type", "bottom"), ("waterlogged", "false")])
            .row(),
        destroy("destroy-stairs", "minecraft:oak_stairs")
            .properties(&[("facing", "north"), ("half", "bottom"), ("shape", "straight"), ("waterlogged", "false")])
            .row(),
        destroy("destroy-inner-stairs", "minecraft:oak_stairs")
            .properties(&[("facing", "east"), ("half", "top"), ("shape", "inner_left"), ("waterlogged", "false")])
            .row(),
        destroy("destroy-fence", "minecraft:oak_fence")
            .properties(&[
                ("north", "true"),
                ("east", "false"),
                ("south", "true"),
                ("west", "false"),
                ("waterlogged", "false"),
            ])
            .row(),
        destroy("destroy-chest", "minecraft:chest")
            .properties(&[("facing", "north"), ("type", "single"), ("waterlogged", "false")])
            .row(),
        destroy("destroy-torch", "minecraft:torch").row(),
        destroy("destroy-barrier", "minecraft:barrier").row(),
        destroy("destroy-air", "minecraft:air").row(),
    ]
}

fn breaking() -> Vec<Value> {
    let direction = |label: String, block: &'static str, direction: &str, level_seed: i64| {
        terrain(label, "breaking", block).with("direction", json!(direction)).with("level_seed", json!(level_seed))
    };
    let mut rows: Vec<Value> = ["DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"]
        .iter()
        .map(|name| direction(format!("breaking-stone-{}", name.to_lowercase()), "minecraft:stone", name, 31).row())
        .collect();
    rows.extend([
        direction("breaking-slab-up".to_owned(), "minecraft:stone_slab", "UP", 8)
            .properties(&[("type", "bottom"), ("waterlogged", "false")])
            .row(),
        direction("breaking-slab-side".to_owned(), "minecraft:stone_slab", "EAST", 9)
            .properties(&[("type", "top"), ("waterlogged", "false")])
            .row(),
        direction("breaking-barrier".to_owned(), "minecraft:barrier", "UP", 1).row(),
    ]);
    rows
}

pub fn particle_terrain() -> Value {
    let rows = [singles(), destroyed(), breaking()].concat();
    case("movement/particle-terrain", "movement", "particle_terrain", json!({ "rows": rows }))
}
