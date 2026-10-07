//! What the crosshair lands on: `LocalPlayer.raycastHitResult` for a player
//! whose item sets no attack range of its own. The nearest block outline
//! within reach, or a pickable entity that is nearer, each held to its own
//! interaction range.

use voxel_core::math::Vec3;
use voxel_world::ClientWorld;
use voxel_world::local_player::LocalPlayer;

use crate::game_mode::Aim;

/// `Attributes.BLOCK_INTERACTION_RANGE` and `ENTITY_INTERACTION_RANGE` of the
/// local player as the server last told, with the defaults when unsent.
pub fn interaction_ranges(world: &ClientWorld) -> (f64, f64) {
    let miner = crate::game_mode::Miner::of(world, 0);
    (
        miner.block_interaction_range,
        miner.entity_interaction_range,
    )
}

/// `LocalPlayer.pick`, from the eyes along the view.
pub fn pick(world: &ClientWorld, player: &LocalPlayer, eye: Vec3) -> Aim {
    let (block_range, entity_range) = interaction_ranges(world);
    let mut max_distance = block_range.max(entity_range);
    let mut max_distance_sq = max_distance * max_distance;
    let (y_rot, x_rot) = player.rotation();
    let look = Vec3::direction_from_rotation_components(x_rot, y_rot);
    let scaled = |distance: f64| {
        Vec3::new(
            eye.x + look.x * distance,
            eye.y + look.y * distance,
            eye.z + look.z * distance,
        )
    };
    // `Entity.pick`: a miss lies where the segment ends.
    let block_hit = world.clip_outlines(eye, scaled(max_distance));
    let block_location = block_hit
        .as_ref()
        .map_or(scaled(max_distance), |hit| hit.location);
    let block_distance_sq = block_location.distance_to_sqr(eye);
    if block_hit.is_some() {
        max_distance_sq = block_distance_sq;
        max_distance = max_distance_sq.sqrt();
    }
    let to = scaled(max_distance);
    let search = player
        .bounding_box()
        .expand_towards(
            look.x * max_distance,
            look.y * max_distance,
            look.z * max_distance,
        )
        .inflate_all(1.0);
    let entity_hit = world
        .entity_hit_result(player.entity_id(), eye, to, search, max_distance_sq)
        .filter(|hit| hit.location.distance_to_sqr(eye) < block_distance_sq);
    if let Some(hit) = entity_hit {
        // `filterHitResult`: a hit past the range is a miss.
        if hit.location.distance_to_sqr(eye) >= entity_range * entity_range {
            return Aim::Miss;
        }
        let (x, y, z) = hit.position;
        let bounds = hit.bounds;
        return Aim::Entity {
            id: hit.entity_id,
            location: [hit.location.x, hit.location.y, hit.location.z],
            position: [x, y, z],
            bounds: [
                bounds.min_x,
                bounds.min_y,
                bounds.min_z,
                bounds.max_x,
                bounds.max_y,
                bounds.max_z,
            ],
        };
    }
    match block_hit {
        Some(hit) if hit.location.distance_to_sqr(eye) < block_range * block_range => Aim::Block {
            x: hit.block_pos.x,
            y: hit.block_pos.y,
            z: hit.block_pos.z,
            face: hit.direction.data3d() as u8,
            location: [hit.location.x, hit.location.y, hit.location.z],
            inside: hit.inside,
        },
        _ => Aim::Miss,
    }
}
