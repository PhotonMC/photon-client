//! `Entity.applyEffectsFromBlocks` for an entity a piston just moved: the blocks it now stands in
//! act on its motion (a bubble column lifts it, a cobweb slows it).

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::ids::EntityId;
use photon_core::java_float::JavaF64;
use photon_core::math::{Vec3, floor_f64};
use photon_core::shape::VoxelShape;

use crate::ClientWorld;
use crate::entity::ClientEntityDimensions;
use crate::local_player::{
    InsideBody, InsideVisit, Level, LivingEffects, Movement, living_inside_effect,
    powder_snow_collision_shape, walk_inside_blocks,
};

/// The parts of an entity the inside-block effects read and change.
struct PushedBody {
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    dimensions: ClientEntityDimensions,
    fall_distance: f64,
    stuck: Vec3,
}

impl InsideBody for PushedBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        powder_snow_collision_shape(self.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let block_pos = BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        );
        living_inside_effect(
            level,
            visit,
            LivingEffects {
                delta: &mut self.delta,
                fall_distance: &mut self.fall_distance,
                stuck: &mut self.stuck,
                position: self.position,
                on_ground: self.on_ground,
                width: self.dimensions.width(),
                block_pos,
            },
        );
    }
}

impl ClientWorld {
    /// The blocks between where a pushed entity was and where it is act on it.
    pub(super) fn apply_effects_after_push(&mut self, id: EntityId, from: Vec3) {
        let Some(entity) = self.entities.at(id) else {
            return;
        };
        let (x, y, z) = entity.position();
        let (dx, dy, dz) = entity.motion();
        let mut body = PushedBody {
            position: Vec3::new(x, y, z),
            delta: Vec3::new(dx, dy, dz),
            on_ground: entity.on_ground,
            dimensions: entity.dimensions(),
            fall_distance: 0.0,
            stuck: Vec3::ZERO,
        };
        let movement = Movement {
            from,
            to: body.position,
            original: None,
        };
        let level = Level::new(self);
        if walk_inside_blocks(&mut body, &level, &[movement]).is_err() {
            return;
        }
        let delta = body.delta;
        let Some(entity) = self.entities.at_mut(id) else {
            return;
        };
        entity.vel.motion.x = JavaF64::new(delta.x);
        entity.vel.motion.y = JavaF64::new(delta.y);
        entity.vel.motion.z = JavaF64::new(delta.z);
    }
}
