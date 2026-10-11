//! Particle quads: the quad faces the camera whichever way it looks, at each
//! partial tick.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

/// The picture windows of a particle's sprites.
fn sprite_uvs(count: i64) -> Vec<Vec<String>> {
    (0..count)
        .map(|i| {
            let left = (i * 16 + 1) as f64 / 128.0;
            let right = (i * 16 + 15) as f64 / 128.0;
            vec![f32_bits(left), f32_bits(right), f32_bits(3.0 / 64.0), f32_bits(11.0 / 64.0)]
        })
        .collect()
}

/// A quad scenario.
struct Quad {
    label: &'static str,
    particle: &'static str,
    when: (i64, i64),
    camera: (f64, f64, f64),
    rotation: (f64, f64),
    partial: f64,
    roll: Option<(f64, f64)>,
    sprites: i64,
}

fn quad(
    label: &'static str,
    particle: &'static str,
    when: (i64, i64),
    camera: (f64, f64, f64),
    view: ((f64, f64), f64),
) -> Quad {
    Quad { label, particle, when, camera, rotation: view.0, partial: view.1, roll: None, sprites: 8 }
}

impl Quad {
    fn roll(self, from: f64, to: f64) -> Self {
        Self { roll: Some((from, to)), ..self }
    }

    fn sprites(self, sprites: i64) -> Self {
        Self { sprites, ..self }
    }

    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "particle": self.particle, "blocks": [], "chunk_radius": 1,
            "x": f64_bits(0.5), "y": f64_bits(70.0), "z": f64_bits(0.5),
            "xa": f64_bits(0.01), "ya": f64_bits(0.02), "za": f64_bits(-0.01),
            "seed": self.when.0, "engine_seed": 7, "sprite_uvs": sprite_uvs(self.sprites), "tick_count": self.when.1,
            "camera_x": f64_bits(self.camera.0), "camera_y": f64_bits(self.camera.1),
            "camera_z": f64_bits(self.camera.2), "y_rot": f32_bits(self.rotation.0),
            "x_rot": f32_bits(self.rotation.1), "partial": f32_bits(self.partial),
        });
        if let Some((from, to)) = self.roll {
            row["o_roll"] = json!(f32_bits(from));
            row["roll"] = json!(f32_bits(to));
        }
        row
    }
}

pub fn particle_quads() -> Value {
    let rows = vec![
        // The quad faces the camera whichever way it looks, at each partial tick.
        quad("smoke-facing-south", "smoke", (12345, 4), (0.5, 70.0, 5.5), ((0.0, 0.0), 0.0)),
        quad("smoke-turned", "smoke", (12345, 4), (3.25, 71.5, -2.5), ((90.0, 30.0), 0.5)),
        quad("smoke-behind-and-above", "smoke", (77, 6), (-4.0, 80.0, 6.0), ((-135.0, -60.0), 0.37)),
        quad("smoke-looking-up", "smoke", (77, 6), (0.5, 60.0, 0.5), ((200.0, 80.0), 1.0)),
        quad("smoke-far-from-the-origin", "smoke", (5, 3), (1000.25, 70.0, -30000.5), ((10.0, 5.0), 0.25)),
        quad("flame-facing-south", "flame", (12345, 3), (0.5, 70.0, 5.5), ((0.0, 0.0), 0.0)).sprites(1),
        quad("flame-turned", "flame", (4242, 5), (-3.0, 69.0, 2.0), ((45.0, -20.0), 0.75)).sprites(1),
        quad("small-flame", "small_flame", (4242, 5), (-3.0, 69.0, 2.0), ((45.0, -20.0), 0.75)).sprites(1),
        // A turn of the quad about its own axis: the particle's roll, between its last and its new.
        // A puff turned about the vertical only (`LOOKAT_Y`), the camera looking level, down and up.
        quad("trial-detection-level", "trial_spawner_detection", (31, 3), (0.5, 70.0, 5.5), ((0.0, 0.0), 0.5)),
        quad("trial-detection-turned", "trial_spawner_detection", (31, 3), (3.25, 71.5, -2.5), ((90.0, 30.0), 0.5)),
        quad(
            "trial-detection-ominous-above",
            "trial_spawner_detection_ominous",
            (8, 5),
            (-4.0, 80.0, 6.0),
            ((-135.0, -60.0), 0.37),
        ),
        quad("smoke-rolled", "smoke", (12345, 4), (3.25, 71.5, -2.5), ((90.0, 30.0), 0.5)).roll(0.25, 0.75),
        quad("smoke-rolled-at-the-end", "smoke", (12345, 4), (0.5, 70.0, 5.5), ((0.0, 0.0), 1.0)).roll(-1.5, 2.0),
    ];
    let rows: Vec<Value> = rows.into_iter().map(Quad::row).collect();
    case("movement/particle-quads", "movement", "particle_quads", json!({ "rows": rows }))
}
