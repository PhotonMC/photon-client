//! The `math` family: original synthetic inputs for the reference's math, boxes, shapes, positions, options
//! and the client level's data.

mod aabb;
mod client_world;
mod direction;
mod local_player;
mod mth;
mod options;
mod positions;
mod shape;
mod values;
mod vec2;
mod vec3;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [
        aabb::construction::cases(),
        aabb::queries::cases(),
        client_world::entity_motion::cases(),
        client_world::entity_state::cases(),
        client_world::entity_ticks::cases(),
        client_world::input::cases(),
        client_world::level::cases(),
        client_world::light::cases(),
        client_world::passengers::cases(),
        client_world::player::cases(),
        direction::cases(),
        local_player::friction::cases(),
        local_player::input::cases(),
        local_player::travel::cases(),
        mth::algorithms::cases(),
        mth::angles::cases(),
        mth::edge_values::cases(),
        mth::interpolation::cases(),
        mth::numbers::cases(),
        options::cases(),
        positions::blockpos::cases(),
        positions::chunkpos::cases(),
        positions::search::cases(),
        positions::sectionpos::cases(),
        shape::border::cases(),
        shape::boxes::cases(),
        shape::collision::cases(),
        shape::dimensions::cases(),
        shape::packets::cases(),
        shape::queries::cases(),
        shape::rotation::cases(),
        shape::world_collision::cases(),
        shape::world_index::cases(),
        vec2::cases(),
        vec3::basic::cases(),
        vec3::distance::cases(),
        vec3::rotation::cases(),
    ]
    .concat()
}
