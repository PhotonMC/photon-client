//! The movement family: particles of the reference (`Particle.tick`, the
//! smoke and flame providers) ticked in a scene, with the random seed they
//! are made with fixed; and the registry of particle types they are named by.

mod ambience_rows;
mod ambient_rows;
mod animating;
mod beacon_rows;
mod book_rows;
mod conduit_rows;
mod engine;
mod engine_rows;
mod engine_rows_late;
mod events;
mod fireworks;
mod layouts;
mod pickup;
mod piston_rows;
mod portal_rows;
mod quads;
mod scene;
mod sulfur_rows;
mod terrain;
mod ticks;
mod vault_rows;

use serde_json::{Value, json};

use crate::case::case;

fn table(id: &str, operation: &str, label: &str) -> Value {
    case(id, "movement", operation, json!({"rows": [{"label": label}]}))
}

pub fn cases() -> Vec<Value> {
    let rows = |id: &str, operation: &str, rows: Vec<Value>| case(id, "movement", operation, json!({ "rows": rows }));
    vec![
        table("movement/particle-types", "particle_types", "registry"),
        rows("movement/particle-ticks", "particle_ticks", ticks::rows()),
        rows(
            "movement/particle-engine",
            "particle_engine",
            [engine_rows::rows(), animating::rows(), fireworks::rows()].concat(),
        ),
        quads::particle_quads(),
        terrain::particle_terrain(),
        pickup::item_pickup(),
        table("movement/fire-flammability", "fire_flammability", "all"),
        table("movement/block-map-colors", "block_map_colors", "all"),
        table("movement/leaf-particles", "leaf_particles", "all"),
    ]
}
