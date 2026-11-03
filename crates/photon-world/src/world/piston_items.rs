//! A moving piston pushing the dropped items in its way (`PistonMovingBlockEntity.moveCollidedEntities`
//! and `Entity.move(MoverType.PISTON, ..)` for an item).

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::direction::Direction;
use photon_core::ids::{EntityId, EntityTypeId};
use photon_core::java_float::JavaF64;
use photon_core::math::Vec3;

use super::piston_shapes::{carried_shape, carries_honey, pushing_shape};
use crate::ClientWorld;
use crate::block;
use crate::block::entity::MovingPiston;
use crate::block::piston::Pos;
use crate::collision::{collide_entity_movement, select_step_up_movement};
use crate::entity::armor_stand_type_id;
use crate::local_player::Level;
use crate::local_player::piston::{at_progress, movement_area, movement_out_of};

/// How far an item must go to be out of the parts of the shape as they sweep, up to the distance
/// the block moves.
fn distance_to_leave(
    shape: &photon_core::shape::VoxelShape,
    (pos, moving): (Pos, &MovingPiston),
    item: Aabb,
    delta_progress: f64,
) -> f64 {
    let movement = moving.movement_direction();
    let mut delta = 0.0_f64;
    for part in shape.to_aabbs() {
        let swept = movement_area(at_progress(pos, part, moving), movement, delta_progress);
        if swept.intersects(item) {
            delta = delta.max(movement_out_of(swept, movement, item));
            if delta >= delta_progress {
                break;
            }
        }
    }
    delta
}

/// The types a piston leaves alone: those whose `getPistonPushReaction` is IGNORE (displays,
/// interactions, markers, effect clouds and ominous item spawners) and the ones attached to a
/// block, whose `move` does nothing on the client.
fn piston_ignores(type_id: EntityTypeId) -> bool {
    use photon_data::entity_type::ids;
    [
        ids::BLOCK_DISPLAY,
        ids::ITEM_DISPLAY,
        ids::TEXT_DISPLAY,
        ids::INTERACTION,
        ids::MARKER,
        ids::ITEM_FRAME,
        ids::GLOW_ITEM_FRAME,
        ids::PAINTING,
        ids::LEASH_KNOT,
        ids::AREA_EFFECT_CLOUD,
        ids::OMINOUS_ITEM_SPAWNER,
    ]
    .contains(&type_id)
}

impl ClientWorld {
    /// The ids of the entities a piston pushes, in id order: all but those whose
    /// `getPistonPushReaction` is IGNORE (displays, interactions, effect clouds, ominous item
    /// spawners and marker armor stands).
    fn pushable_ids(&self) -> Vec<EntityId> {
        let armor_stand = armor_stand_type_id();
        let mut ids: Vec<EntityId> = self
            .entities
            .all()
            .filter(|entity| {
                !piston_ignores(entity.type_id)
                    && !entity.is_spectator()
                    && !(entity.type_id == armor_stand && entity.armor_stand().is_marker())
            })
            .map(|entity| entity.id())
            .collect();
        ids.sort_by_key(|id| id.0);
        ids
    }

    /// The box of a pushable entity where it is.
    fn pushable_box(&self, id: EntityId) -> Option<Aabb> {
        Some(self.entities.at(id)?.bounding_box())
    }

    fn item_position(&self, id: EntityId) -> Option<Vec3> {
        let (x, y, z) = self.entities.at(id)?.position();
        Some(Vec3::new(x, y, z))
    }

    /// `moveCollidedEntities` for the dropped items, before the block goes on to `new_progress`.
    pub(crate) fn push_items_by_piston(
        &mut self,
        (pos, moving): (Pos, &MovingPiston),
        new_progress: f32,
    ) {
        let delta_progress = f64::from(new_progress - moving.progress);
        let movement = moving.movement_direction();
        let shape = pushing_shape(moving, pos);
        let Some(bounds) = shape.bounds() else {
            return;
        };
        let aabb = at_progress(pos, bounds, moving);
        let area = movement_area(aabb, movement, delta_progress).minmax(aabb);
        let slime = block::state_or_air(moving.moved_state).block
            == photon_data::block::blocks::id::SLIME_BLOCK as u16;
        for id in self.pushable_ids() {
            let Some(entity_box) = self.pushable_box(id) else {
                continue;
            };
            if !area.intersects(entity_box) {
                continue;
            }
            if slime {
                self.bounce_item(id, movement);
            }
            let delta = distance_to_leave(&shape, (pos, moving), entity_box, delta_progress);
            if delta <= 0.0 {
                continue;
            }
            let delta = delta.min(delta_progress) + 0.01;
            self.move_item_by_piston(id, movement, (delta, movement));
            if !moving.extending && moving.source {
                self.fix_item_within_piston_base(id, pos, (movement, delta_progress));
            }
        }
    }

    /// `moveStuckEntities` for the other entities: honey takes along what stands on it.
    pub(crate) fn carry_items_by_honey(
        &mut self,
        (pos, moving): (Pos, &MovingPiston),
        new_progress: f32,
    ) {
        let movement = moving.movement_direction();
        if !carries_honey(moving) || matches!(movement, Direction::Up | Direction::Down) {
            return;
        }
        let top = carried_shape(moving, pos).max(photon_core::direction::Axis::Y);
        let area = at_progress(pos, Aabb::new(0.0, top, 0.0, 1.0, 1.500_001, 1.0), moving);
        let delta_progress = f64::from(new_progress - moving.progress);
        let below = BlockPos::new(pos.0, pos.1, pos.2);
        for id in self.pushable_ids() {
            if self.stands_on_honey(id, area, below) {
                self.move_item_by_piston(id, movement, (delta_progress, movement));
            }
        }
    }

    /// `matchesStickyCritera`: on the ground, in the area, and over the block or supported by it.
    fn stands_on_honey(&self, id: EntityId, area: Aabb, below: BlockPos) -> bool {
        let Some(entity) = self.entities.at(id) else {
            return false;
        };
        let (x, _, z) = entity.position();
        let inside = x >= area.min_x && x <= area.max_x && z >= area.min_z && z <= area.max_z;
        let supported = entity.tracking.support.main_block == Some(below);
        entity.on_ground && entity.bounding_box().intersects(area) && (supported || inside)
    }

    /// A slime block sends what it pushes along its way at full speed.
    fn bounce_item(&mut self, id: EntityId, movement: Direction) {
        let Some(entity) = self.entities.at_mut(id) else {
            return;
        };
        let (sx, sy, sz) = movement.step();
        match movement {
            Direction::East | Direction::West => {
                entity.vel.motion.x = JavaF64::new(f64::from(sx));
            }
            Direction::Up | Direction::Down => {
                entity.vel.motion.y = JavaF64::new(f64::from(sy));
            }
            Direction::North | Direction::South => {
                entity.vel.motion.z = JavaF64::new(f64::from(sz));
            }
        }
    }

    /// `fixEntityWithinPistonBase` for an item.
    fn fix_item_within_piston_base(
        &mut self,
        id: EntityId,
        pos: Pos,
        (direction, delta_progress): (Direction, f64),
    ) {
        let Some(entity) = self.pushable_box(id) else {
            return;
        };
        let block = Aabb::new(
            f64::from(pos.0),
            f64::from(pos.1),
            f64::from(pos.2),
            f64::from(pos.0) + 1.0,
            f64::from(pos.1) + 1.0,
            f64::from(pos.2) + 1.0,
        );
        if !entity.intersects(block) {
            return;
        }
        let opposite = direction.opposite();
        let delta = movement_out_of(block, opposite, entity) + 0.01;
        let intersected = movement_out_of(block, opposite, entity.intersect(block)) + 0.01;
        if (delta - intersected).abs() < 0.01 {
            let delta = delta.min(delta_progress) + 0.01;
            self.move_item_by_piston(id, direction, (delta, opposite));
        }
    }

    /// The types that set `noPhysics` for good: `Entity.move` just sets their position.
    fn passes_through_blocks(&self, id: EntityId) -> bool {
        use photon_data::entity_type::ids;
        self.entities.at(id).is_some_and(|entity| {
            [ids::ENDER_DRAGON, ids::SHULKER_BULLET].contains(&entity.type_id)
        })
    }

    /// `Entity.move` of an entity without physics: no limit, no collision, no ground flags.
    fn move_without_physics(&mut self, id: EntityId, by: Vec3) {
        let Some(at) = self.item_position(id) else {
            return;
        };
        if let Some(mut entity) = self.entities.at_mut(id) {
            entity.set_position((at.x + by.x, at.y + by.y, at.z + by.z));
        }
    }

    /// `Entity.limitPistonMovement`: a piston moves an entity at most 0.51 along one axis in a
    /// game tick, and one axis at a time.
    fn limit_item_piston_movement(&mut self, id: EntityId, delta: Vec3) -> Vec3 {
        if delta.length_squared() <= 1.0e-7 {
            return delta;
        }
        let now = self.game_time() as i64;
        let Some(mut entity) = self.entities.at_mut(id) else {
            return Vec3::ZERO;
        };
        if entity.tracking.piston.0 != now {
            entity.tracking.piston = (now, [0; 3]);
        }
        let (axis, amount) = if delta.x != 0.0 {
            (0, delta.x)
        } else if delta.y != 0.0 {
            (1, delta.y)
        } else if delta.z != 0.0 {
            (2, delta.z)
        } else {
            return Vec3::ZERO;
        };
        let so_far = f64::from_bits(entity.tracking.piston.1[axis]);
        let total = (amount + so_far).clamp(-0.51, 0.51);
        entity.tracking.piston.1[axis] = total.to_bits();
        let applied = total - so_far;
        if applied.abs() <= f64::from(1.0e-5_f32) {
            return Vec3::ZERO;
        }
        match axis {
            0 => Vec3::new(applied, 0.0, 0.0),
            1 => Vec3::new(0.0, applied, 0.0),
            _ => Vec3::new(0.0, 0.0, applied),
        }
    }

    /// `moveEntityByPiston` for an item: `Entity.move(MoverType.PISTON, ..)`, its collision with
    /// the blocks and the ground flags it sets.
    fn move_item_by_piston(
        &mut self,
        id: EntityId,
        piston: Direction,
        (delta, movement): (f64, Direction),
    ) {
        let (sx, sy, sz) = movement.step();
        let by = Vec3::new(
            delta * f64::from(sx),
            delta * f64::from(sy),
            delta * f64::from(sz),
        );
        if self.passes_through_blocks(id) {
            self.move_without_physics(id, by);
            return;
        }
        let by = self.limit_item_piston_movement(id, by);
        if by.x == 0.0 && by.y == 0.0 && by.z == 0.0 {
            return;
        }
        let before = self.item_position(id);
        let Some(step) = self.piston_item_step(id, piston, by) else {
            return;
        };
        let Some(mut entity) = self.entities.at_mut(id) else {
            return;
        };
        entity.set_position((step.position.x, step.position.y, step.position.z));
        entity.on_ground = step.on_ground;
        entity.tracking.support = step.support;
        if let Some(from) = before {
            self.apply_effects_after_push(id, from);
        }
    }

    /// `Entity.move`'s collision and ground flags for an item a piston moves by `by`.
    fn piston_item_step(&self, id: EntityId, piston: Direction, by: Vec3) -> Option<PistonStep> {
        let entity = self.entities.at(id)?;
        let mut support = entity.tracking.support;
        let mut on_ground = entity.on_ground;
        let start = self.item_position(id)?;
        let mut position = start;
        let level = Level::new(self).moved_by_piston(piston);
        let bounding_box = self.pushable_box(id)?;
        let max_up_step = entity
            .attribute_value(photon_data::entity_type::ids::STEP_HEIGHT_ATTRIBUTE)
            .map_or(0.0, |height| height as f32);
        let moved = collide_with_step(
            &level,
            (bounding_box, position),
            (by, on_ground),
            max_up_step,
        )?;
        let length = moved.length_squared();
        if length > 1.0e-7 || by.length_squared() - length < 1.0e-7 {
            position = position.add_vec3(moved);
        }
        if by.y.abs() > 0.0 {
            on_ground = by.y != moved.y && by.y < 0.0;
            level
                .check_supporting_block(
                    (&mut support.main_block, &mut support.no_blocks),
                    (
                        bounding_box.move_by_vec3(position.subtract_vec3(start)),
                        position,
                        0.0,
                    ),
                    on_ground,
                    Some(moved),
                )
                .ok()?;
        }
        Some(PistonStep {
            position,
            on_ground,
            support,
        })
    }
}

/// `Entity.collide`: the movement the blocks leave, and a step up on to what a grounded entity
/// walks into when it can climb (`maxUpStep`, the step height of a living entity).
fn collide_with_step(
    level: &Level<'_>,
    (aabb, position): (Aabb, Vec3),
    (movement, on_ground): (Vec3, bool),
    max_up_step: f32,
) -> Option<Vec3> {
    let colliders = level
        .colliders(aabb.expand_towards_vec3(movement), 0.0, position)
        .ok()?;
    let step = collide_entity_movement(movement, aabb, &colliders);
    let landed = movement.y != step.y && movement.y < 0.0;
    let blocked = movement.x != step.x || movement.z != step.z;
    if max_up_step <= 0.0 || !(landed || on_ground) || !blocked {
        return Some(step);
    }
    let grounded = if landed {
        aabb.move_by(0.0, step.y, 0.0)
    } else {
        aabb
    };
    let mut area = grounded.expand_towards(movement.x, f64::from(max_up_step), movement.z);
    if !landed {
        area = area.expand_towards(0.0, f64::from(-1.0e-5_f32), 0.0);
    }
    let colliders = level.colliders(area, 0.0, position).ok()?;
    Some(select_step_up_movement(
        movement,
        step,
        (aabb, grounded),
        &colliders,
        (max_up_step, step.y as f32),
    ))
}

/// What a piston's move leaves of an item.
struct PistonStep {
    position: Vec3,
    on_ground: bool,
    support: crate::entity::Support,
}
