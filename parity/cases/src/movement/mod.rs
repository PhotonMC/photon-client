//! The `movement` family: the player, the entities, the sky and the elytra.

mod elytra;
mod entities;
mod fog;
mod particles;
mod player;
mod sky;
mod world;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    let mut cases = player::cases();
    cases.extend(entities::cases());
    cases.push(sky::sky_flash());
    cases.push(fog::fog_data());
    cases.push(fog::lightmap_data());
    cases.push(elytra::elytra_state());
    cases.extend(particles::cases());
    cases
}
