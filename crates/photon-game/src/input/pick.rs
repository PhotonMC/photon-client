//! What the crosshair lands on: `LocalPlayer.raycastHitResult` for a player
//! whose item sets no attack range of its own. The nearest block outline
//! within reach, or a pickable entity that is nearer, each held to its own
//! interaction range.

use photon_core::aabb::Aabb;
use photon_core::ids::EntityId;
use photon_core::math::Vec3;
use photon_world::ClientWorld;
use photon_world::local_player::LocalPlayer;

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

/// The entity a pick is made from (`GameRenderer`'s camera entity): who it is, which way it
/// looks and the box the entity search starts from.
#[derive(Debug, Clone, Copy)]
pub struct Viewer {
    pub entity_id: EntityId,
    /// Yaw and pitch, in degrees.
    pub rotation: (f32, f32),
    pub bounds: Aabb,
}

impl Viewer {
    pub fn of_player(player: &LocalPlayer) -> Self {
        Self {
            entity_id: player.entity_id(),
            rotation: player.rotation(),
            bounds: player.bounding_box(),
        }
    }
}

/// `filterHitResult` for an entity: a hit past the range is a miss.
fn entity_aim(hit: &photon_world::EntityHit, entity_range: f64, eye: Vec3) -> Aim {
    if hit.location.distance_to_sqr(eye) >= entity_range * entity_range {
        return Aim::Miss;
    }
    let (x, y, z) = hit.position;
    let bounds = hit.bounds;
    Aim::Entity {
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
    }
}

/// `LocalPlayer.pick`, from the eyes along the view.
pub fn pick(world: &ClientWorld, player: &LocalPlayer, eye: Vec3) -> Aim {
    pick_from(world, &Viewer::of_player(player), eye)
}

/// `LocalPlayer.pick` of the camera entity: its eyes, its look, and its own box left out.
pub fn pick_from(world: &ClientWorld, viewer: &Viewer, eye: Vec3) -> Aim {
    let (block_range, entity_range) = interaction_ranges(world);
    let mut max_distance = block_range.max(entity_range);
    let mut max_distance_sq = max_distance * max_distance;
    let (y_rot, x_rot) = viewer.rotation;
    let look = Vec3::view_vector(x_rot, y_rot);
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
    let search = viewer
        .bounds
        .expand_towards(
            look.x * max_distance,
            look.y * max_distance,
            look.z * max_distance,
        )
        .inflate_all(1.0);
    let entity_hit = world
        .entity_hit_result(viewer.entity_id, eye, to, search, max_distance_sq)
        .filter(|hit| hit.location.distance_to_sqr(eye) < block_distance_sq);
    if let Some(hit) = entity_hit {
        return entity_aim(&hit, entity_range, eye);
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

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::binary::Uuid;
    use photon_world::{DimensionBounds, EntityInput};

    fn spawn(world: &mut ClientWorld, id: i32, x: f64) {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(1, u64::try_from(id).expect("positive")),
            type_id: photon_data::entity_type::id("minecraft:pig").expect("the type"),
            x,
            y: 9.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }

    /// Looking along +x (yaw 270) from a pig's eyes: the pig next to it is hit, never itself.
    #[test]
    fn a_camera_entity_picks_from_its_eyes_and_never_itself() {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        spawn(&mut world, 1, 0.0);
        spawn(&mut world, 2, 2.0);
        let viewer = Viewer {
            entity_id: EntityId(1),
            rotation: (270.0, 0.0),
            bounds: world
                .entities()
                .at(EntityId(1))
                .expect("the pig")
                .bounding_box(),
        };
        let eye = Vec3::new(0.0, 9.5, 0.0);
        assert!(matches!(
            pick_from(&world, &viewer, eye),
            Aim::Entity {
                id: EntityId(2),
                ..
            }
        ));
        let behind = Viewer {
            rotation: (90.0, 0.0),
            ..viewer
        };
        assert!(matches!(pick_from(&world, &behind, eye), Aim::Miss));
    }
}
