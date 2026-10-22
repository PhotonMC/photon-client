//! A moving piston pushing the player (`PistonMovingBlockEntity.moveCollidedEntities`,
//! `moveStuckEntities` and `Entity.limitPistonMovement`).

use photon_core::aabb::Aabb;
use photon_core::direction::Direction;

use super::*;
use crate::block;
use crate::block::entity::MovingPiston;
use crate::block::piston::Pos;
use crate::world::piston_shapes::{carried_shape, carries_honey, pushing_shape};

/// `MoverType`: what moves the entity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MoverType {
    SelfMove,
    Piston,
}

/// `PistonMath.getMovementArea`: the space a box sweeps when it moves `amount` along `direction`.
fn movement_area(aabb: Aabb, direction: Direction, amount: f64) -> Aabb {
    let (sx, sy, sz) = direction.step();
    let delta = amount * f64::from(sx + sy + sz);
    let (min, max) = (delta.min(0.0), delta.max(0.0));
    match direction {
        Direction::West => Aabb::new(
            aabb.min_x + min,
            aabb.min_y,
            aabb.min_z,
            aabb.min_x + max,
            aabb.max_y,
            aabb.max_z,
        ),
        Direction::East => Aabb::new(
            aabb.max_x + min,
            aabb.min_y,
            aabb.min_z,
            aabb.max_x + max,
            aabb.max_y,
            aabb.max_z,
        ),
        Direction::Down => Aabb::new(
            aabb.min_x,
            aabb.min_y + min,
            aabb.min_z,
            aabb.max_x,
            aabb.min_y + max,
            aabb.max_z,
        ),
        Direction::Up => Aabb::new(
            aabb.min_x,
            aabb.max_y + min,
            aabb.min_z,
            aabb.max_x,
            aabb.max_y + max,
            aabb.max_z,
        ),
        Direction::North => Aabb::new(
            aabb.min_x,
            aabb.min_y,
            aabb.min_z + min,
            aabb.max_x,
            aabb.max_y,
            aabb.min_z + max,
        ),
        Direction::South => Aabb::new(
            aabb.min_x,
            aabb.min_y,
            aabb.max_z + min,
            aabb.max_x,
            aabb.max_y,
            aabb.max_z + max,
        ),
    }
}

/// `PistonMovingBlockEntity.getMovement`: how far `aabb` must go along `movement` to leave
/// `outside_of`.
fn movement_out_of(outside_of: Aabb, movement: Direction, aabb: Aabb) -> f64 {
    match movement {
        Direction::East => outside_of.max_x - aabb.min_x,
        Direction::West => aabb.max_x - outside_of.min_x,
        Direction::Up => outside_of.max_y - aabb.min_y,
        Direction::Down => aabb.max_y - outside_of.min_y,
        Direction::South => outside_of.max_z - aabb.min_z,
        Direction::North => aabb.max_z - outside_of.min_z,
    }
}

/// `moveByPositionAndProgress`: a box of the block as the moving block has it by now.
fn at_progress(pos: Pos, aabb: Aabb, moving: &MovingPiston) -> Aabb {
    let current = f64::from(moving.extended_progress());
    let (sx, sy, sz) = moving.direction.step();
    aabb.move_by(
        f64::from(pos.0) + current * f64::from(sx),
        f64::from(pos.1) + current * f64::from(sy),
        f64::from(pos.2) + current * f64::from(sz),
    )
}

impl LocalPlayer {
    /// `Entity.limitPistonMovement`: a piston moves an entity at most 0.51 along one axis in a
    /// game tick, and one axis at a time.
    pub(super) fn limit_piston_movement(&mut self, level: &Level<'_>, delta: Vec3) -> Vec3 {
        if delta.length_squared() <= 1.0e-7 {
            return delta;
        }
        let now = level.world.game_time() as i64;
        if now != self.trace.piston_time {
            self.trace.piston_deltas = [0.0; 3];
            self.trace.piston_time = now;
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
        let so_far = self.trace.piston_deltas[axis];
        let total = (amount + so_far).clamp(-0.51, 0.51);
        self.trace.piston_deltas[axis] = total;
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

    /// `PistonMovingBlockEntity.tick`'s pushing, before the block goes on to `new_progress`.
    pub(crate) fn pushed_by_piston(
        &mut self,
        world: &ClientWorld,
        (pos, moving): (Pos, &MovingPiston),
        new_progress: f32,
    ) -> Result<(), Unsupported> {
        let level = Level::new(world);
        let delta_progress = f64::from(new_progress - moving.progress);
        self.moved_by_piston_block(&level, (pos, moving), delta_progress)?;
        self.carried_by_honey(&level, (pos, moving), delta_progress)
    }

    /// `moveCollidedEntities` for the player.
    fn moved_by_piston_block(
        &mut self,
        level: &Level<'_>,
        (pos, moving): (Pos, &MovingPiston),
        delta_progress: f64,
    ) -> Result<(), Unsupported> {
        // `EntityGetter.getEntities(entity, box)` leaves spectators out.
        if self.is_spectator() {
            return Ok(());
        }
        let movement = moving.movement_direction();
        let shape = pushing_shape(moving, pos);
        let Some(bounds) = shape.bounds() else {
            return Ok(());
        };
        let aabb = at_progress(pos, bounds, moving);
        let area = movement_area(aabb, movement, delta_progress).minmax(aabb);
        if !area.intersects(self.bounding_box) {
            return Ok(());
        }
        if block::state_or_air(moving.moved_state).block
            == photon_data::block::blocks::id::SLIME_BLOCK as u16
        {
            self.bounce_off_slime(movement);
        }
        let delta = self.distance_to_leave(&shape, (pos, moving), delta_progress);
        if delta <= 0.0 {
            return Ok(());
        }
        let delta = delta.min(delta_progress) + 0.01;
        self.move_by_piston(level, movement, (delta, movement))?;
        if !moving.extending && moving.source {
            self.fix_within_piston_base(level, pos, (movement, delta_progress))?;
        }
        Ok(())
    }

    /// How far the player must go to be out of the parts of the shape as they sweep, up to the
    /// distance the block moves.
    fn distance_to_leave(
        &self,
        shape: &photon_core::shape::VoxelShape,
        (pos, moving): (Pos, &MovingPiston),
        delta_progress: f64,
    ) -> f64 {
        let movement = moving.movement_direction();
        let mut delta = 0.0_f64;
        for part in shape.to_aabbs() {
            let swept = movement_area(at_progress(pos, part, moving), movement, delta_progress);
            if swept.intersects(self.bounding_box) {
                delta = delta.max(movement_out_of(swept, movement, self.bounding_box));
                if delta >= delta_progress {
                    break;
                }
            }
        }
        delta
    }

    /// A slime block sends what it pushes along its way at full speed.
    fn bounce_off_slime(&mut self, movement: Direction) {
        let (sx, sy, sz) = movement.step();
        let delta = self.delta;
        self.delta = match movement {
            Direction::East | Direction::West => Vec3::new(f64::from(sx), delta.y, delta.z),
            Direction::Up | Direction::Down => Vec3::new(delta.x, f64::from(sy), delta.z),
            Direction::North | Direction::South => Vec3::new(delta.x, delta.y, f64::from(sz)),
        };
    }

    /// `moveStuckEntities` for the player: honey takes along what stands on it.
    fn carried_by_honey(
        &mut self,
        level: &Level<'_>,
        (pos, moving): (Pos, &MovingPiston),
        delta_progress: f64,
    ) -> Result<(), Unsupported> {
        let movement = moving.movement_direction();
        if !carries_honey(moving) || matches!(movement, Direction::Up | Direction::Down) {
            return Ok(());
        }
        let top = carried_shape(moving, pos).max(photon_core::direction::Axis::Y);
        let area = at_progress(pos, Aabb::new(0.0, top, 0.0, 1.0, 1.500_001, 1.0), moving);
        let inside = self.position.x >= area.min_x
            && self.position.x <= area.max_x
            && self.position.z >= area.min_z
            && self.position.z <= area.max_z;
        let supported = self.main_supporting_block == Some(BlockPos::new(pos.0, pos.1, pos.2));
        if self.bounding_box.intersects(area) && self.on_ground && (supported || inside) {
            self.move_by_piston(level, movement, (delta_progress, movement))?;
        }
        Ok(())
    }

    /// `fixEntityWithinPistonBase`: the base of a piston that draws back does not keep what it
    /// pushed inside it.
    fn fix_within_piston_base(
        &mut self,
        level: &Level<'_>,
        pos: Pos,
        (direction, delta_progress): (Direction, f64),
    ) -> Result<(), Unsupported> {
        let entity = self.bounding_box;
        let block = Aabb::new(
            f64::from(pos.0),
            f64::from(pos.1),
            f64::from(pos.2),
            f64::from(pos.0) + 1.0,
            f64::from(pos.1) + 1.0,
            f64::from(pos.2) + 1.0,
        );
        if !entity.intersects(block) {
            return Ok(());
        }
        let opposite = direction.opposite();
        let delta = movement_out_of(block, opposite, entity) + 0.01;
        let intersected = movement_out_of(block, opposite, entity.intersect(block)) + 0.01;
        if (delta - intersected).abs() < 0.01 {
            let delta = delta.min(delta_progress) + 0.01;
            self.move_by_piston(level, direction, (delta, opposite))?;
        }
        Ok(())
    }

    /// `moveEntityByPiston`: the move, the effects of the blocks it ends in and no record of it.
    fn move_by_piston(
        &mut self,
        level: &Level<'_>,
        piston: Direction,
        (delta, movement): (f64, Direction),
    ) -> Result<(), Unsupported> {
        let level = level.moved_by_piston(piston);
        let previous = self.position;
        let (sx, sy, sz) = movement.step();
        let by = Vec3::new(
            delta * f64::from(sx),
            delta * f64::from(sy),
            delta * f64::from(sz),
        );
        self.move_as(&level, MoverType::Piston, by)?;
        let moved = [Movement {
            from: previous,
            to: self.position,
            original: None,
        }];
        self.apply_effects_from_movements(&level, &moved)?;
        self.trace.movements.pop();
        Ok(())
    }
}
