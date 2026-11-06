//! A dropped item's client tick (`ItemEntity.tick`). The server sends an
//! item's position only now and then; between its packets the client moves
//! the item itself, so it falls, lands, bounces and slides as it would there.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::ids::EntityId;
use photon_core::java_float::JavaF64;
use photon_core::math::{Vec3, floor_f64};

use super::ClientWorld;
use super::collision::collide_entity_movement;
use super::constants::VANILLA_ITEM_ENTITY_TYPE_ID;
use super::entity::{ClientEntityDimensions, Support};
use super::fluid::{self, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use super::local_player::Level;

/// `ItemEntity.getDefaultGravity`.
const GRAVITY: f64 = 0.04;
/// `Entity.getAirDrag`.
const AIR_DRAG: f32 = 0.98;
/// `Entity.getBlockPosBelowThatAffectsMyMovement` of an item.
const BELOW_OFFSET: f32 = 0.999_999;
/// `Entity.getOnPosLegacy`, which the block effects read.
const STEP_ON_OFFSET: f32 = 0.2;

/// What one tick of an item changes.
struct ItemStep {
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    main_supporting_block: Option<BlockPos>,
    on_ground_no_blocks: bool,
}

/// The item's state going into the tick.
struct ItemState {
    id: EntityId,
    tick_count: i32,
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    main_supporting_block: Option<BlockPos>,
    on_ground_no_blocks: bool,
    no_gravity: bool,
}

/// `EntityType.ITEM`'s size.
pub(crate) fn item_box(position: Vec3) -> Aabb {
    square_dimensions(ITEM_SIZE).make_bounding_box(position)
}

/// `EntityType.ITEM`'s width and height.
const ITEM_SIZE: f32 = 0.25;

/// The size of an entity as wide as it is tall.
pub(crate) fn square_dimensions(size: f32) -> ClientEntityDimensions {
    ClientEntityDimensions::scalable(size, size)
}

/// `Entity.restituteMovementAfterCollisions` for an item (26.3): a bump keeps none of its speed
/// unless the block it lands on bounces.
pub(crate) fn restitute(
    effect: &super::local_player::StateInfo,
    (x_collision, z_collision, below): (bool, bool, bool),
    (current, movement, vertical): (Vec3, Vec3, bool),
    (gravity, air_drag): (f64, f32),
) -> Vec3 {
    use super::local_player::flag;
    // Before 26.3 the client does not simulate these entities, so a collision leaves their
    // speed as it was (`canSimulateMovement` guards the restitution).
    if photon_core::versions::behaviour::item_collision_keeps_speed() {
        return current;
    }
    let mut restitution = 0.0_f64;
    let mut after = current;
    if x_collision {
        after = Vec3::new(-current.x * restitution, after.y, after.z);
    }
    if z_collision {
        after = Vec3::new(after.x, after.y, -current.z * restitution);
    }
    if vertical {
        if below {
            let slow = -current.y <= gravity;
            restitution = if !slow && effect.flags & flag::SUPPRESSES_BOUNCE == 0 {
                // `getBlockBounciness`: what is not a living thing bounces a fifth less.
                f64::from(effect.bounce * 0.8_f32).max(restitution)
            } else {
                0.0
            };
        }
        let (compensation, drag) = if restitution > 0.0 {
            let portion = movement.y / current.y;
            (
                portion * gravity,
                1.0 + portion * (f64::from(air_drag) - 1.0),
            )
        } else {
            (0.0, 1.0)
        };
        after = Vec3::new(
            after.x,
            (compensation - current.y) * drag * restitution,
            after.z,
        );
    }
    after
}

/// `Entity.applyEffectsFromBlocks`'s `stepOn` for an item on the ground: a
/// slime block slows what slides over it. (The block effects of being inside
/// a block, as a cobweb's, are not carried yet.)
pub(crate) fn step_on(
    level: &Level<'_>,
    (position, main_supporting_block): (Vec3, Option<BlockPos>),
    on_ground: bool,
    delta: Vec3,
) -> Vec3 {
    if !on_ground {
        return delta;
    }
    let on_pos = level.on_pos(main_supporting_block, position, STEP_ON_OFFSET);
    if level.info(on_pos).name != "minecraft:slime_block" || delta.y.abs() >= 0.1 {
        return delta;
    }
    let scale = 0.4 + delta.y.abs() * 0.2;
    Vec3::new(delta.x * scale, delta.y, delta.z * scale)
}

/// What the fluids around an item do to it: `Entity.updateFluidInteraction`
/// finds the water and lava the box is in, and the currents push the item.
pub(crate) fn fluid_pass(
    level: &Level<'_>,
    size: f32,
    (position, delta): (Vec3, Vec3),
) -> Option<(fluid::Fluids, Vec3)> {
    let dimensions = square_dimensions(size);
    let fluids = fluid::scan(
        level,
        (
            dimensions.make_bounding_box(position),
            position,
            dimensions.eye_height(),
        ),
        false,
    )
    .ok()?;
    let mut delta = delta;
    if fluids.water.height > 0.0 {
        delta = fluid::apply_current(fluids.water, delta, WATER_FLOW_SCALE, Pushed::Other);
    }
    if fluids.lava.height > 0.0 {
        delta = fluid::apply_current(fluids.lava, delta, LAVA_FLOW_SCALE, Pushed::Other);
    }
    Some((fluids, delta))
}

/// `ItemEntity.setFluidMovement`: the item slows in the fluid and floats up
/// while it sinks slower than a sixtieth of a block a tick.
fn fluid_movement(delta: Vec3, multiplier: f32) -> Vec3 {
    let multiplier = f64::from(multiplier);
    let lift = if delta.y < f64::from(0.06_f32) {
        f64::from(5.0e-4_f32)
    } else {
        0.0
    };
    Vec3::new(delta.x * multiplier, delta.y + lift, delta.z * multiplier)
}

/// The block's speed factor, the step-on effect, then drag and ground friction on a moved item.
fn item_drag(
    level: &Level<'_>,
    (position, main_supporting_block, on_ground): (Vec3, Option<BlockPos>, bool),
    mut delta: Vec3,
) -> Vec3 {
    let here = BlockPos::new(
        floor_f64(position.x),
        floor_f64(position.y),
        floor_f64(position.z),
    );
    let below_pos = level.on_pos(main_supporting_block, position, BELOW_OFFSET);
    let speed_factor = f64::from(level.block_speed_factor(here, below_pos));
    delta = Vec3::new(delta.x * speed_factor, delta.y, delta.z * speed_factor);
    delta = step_on(level, (position, main_supporting_block), on_ground, delta);

    let mut friction = AIR_DRAG;
    if on_ground {
        friction *= level.info(below_pos).friction;
    }
    delta = Vec3::new(
        delta.x * f64::from(friction),
        delta.y * f64::from(AIR_DRAG),
        delta.z * f64::from(friction),
    );
    if on_ground && delta.y < 0.0 {
        delta = Vec3::new(delta.x, delta.y * -0.5, delta.z);
    }
    delta
}

/// 26.3: what a dropped item runs into keeps none of its speed
/// (`restituteMovementAfterCollisions`).
fn restitute_item(
    level: &Level<'_>,
    (no_gravity, main_supporting_block, position, on_ground): (bool, Option<BlockPos>, Vec3, bool),
    (delta, movement): (Vec3, Vec3),
) -> Vec3 {
    let vertical_collision = delta.y.abs() > 0.0 && delta.y != movement.y;
    let x_collision = !photon_core::math::equal_f64(delta.x, movement.x);
    let z_collision = !photon_core::math::equal_f64(delta.z, movement.z);
    if !(vertical_collision || x_collision || z_collision) {
        return delta;
    }
    let effect_pos = level.on_pos(main_supporting_block, position, STEP_ON_OFFSET);
    restitute(
        &level.info(effect_pos),
        (x_collision, z_collision, vertical_collision && on_ground),
        (delta, movement, vertical_collision),
        (if no_gravity { 0.0 } else { GRAVITY }, AIR_DRAG),
    )
}

/// The movement of a dropped item that is not at rest: `Entity.move` against the blocks, the
/// bounce, the block's speed factor, then drag and ground friction.
fn item_move(level: &Level<'_>, state: &ItemState, mut delta: Vec3) -> Option<ItemStep> {
    let mut position = state.position;
    let mut on_ground = state.on_ground;
    let mut main_supporting_block = state.main_supporting_block;
    let mut on_ground_no_blocks = state.on_ground_no_blocks;
    let bounding_box = item_box(position);
    let movement = if delta.length_squared() == 0.0 {
        delta
    } else {
        let colliders = level
            .colliders(bounding_box.expand_towards_vec3(delta), 0.0, position)
            .ok()?;
        collide_entity_movement(delta, bounding_box, &colliders)
    };
    let moved = movement.length_squared();
    if moved > 1.0e-7 || delta.length_squared() - moved < 1.0e-7 {
        position = position.add_vec3(movement);
    }
    let moved_vertically = delta.y.abs() > 0.0;
    let vertical_collision = moved_vertically && delta.y != movement.y;
    if moved_vertically {
        let below = vertical_collision && delta.y < 0.0;
        on_ground = below;
        level
            .check_supporting_block(
                (&mut main_supporting_block, &mut on_ground_no_blocks),
                (item_box(position), position, 0.0),
                on_ground,
                Some(movement),
            )
            .ok()?;
    }
    delta = restitute_item(
        level,
        (state.no_gravity, main_supporting_block, position, on_ground),
        (delta, movement),
    );
    delta = item_drag(level, (position, main_supporting_block, on_ground), delta);
    Some(ItemStep {
        position,
        delta,
        on_ground,
        main_supporting_block,
        on_ground_no_blocks,
    })
}

/// `ItemEntity.tick` after `baseTick`, with the branches the client runs:
/// the fluids' current and buoyancy, or gravity, `Entity.move` (collision
/// with the blocks, the flags it sets, the block's speed factor), then drag,
/// ground friction and the bounce, and the fluids' current once more.
fn item_step(level: &Level<'_>, state: &ItemState) -> Option<ItemStep> {
    let mut position = state.position;
    // `baseTick`'s fluid pass, before the movement is chosen.
    let (fluids, mut delta) = fluid_pass(level, ITEM_SIZE, (position, state.delta))?;
    if fluids.water.height > f64::from(0.1_f32) {
        delta = fluid_movement(delta, 0.99);
    } else if fluids.lava.height > f64::from(0.1_f32) {
        delta = fluid_movement(delta, 0.95);
    } else if !state.no_gravity {
        delta = delta.add_vec3(Vec3::new(0.0, -GRAVITY, 0.0));
    }
    let mut on_ground = state.on_ground;
    let mut main_supporting_block = state.main_supporting_block;
    let mut on_ground_no_blocks = state.on_ground_no_blocks;

    let resting = on_ground
        && delta.x * delta.x + delta.z * delta.z <= f64::from(1.0e-5_f32)
        && (state.tick_count.wrapping_add(state.id.0)) % 4 != 0;
    if resting {
        delta = step_on(level, (position, main_supporting_block), on_ground, delta);
    } else {
        let moved = item_move(level, state, delta)?;
        (position, delta, on_ground) = (moved.position, moved.delta, moved.on_ground);
        main_supporting_block = moved.main_supporting_block;
        on_ground_no_blocks = moved.on_ground_no_blocks;
    }
    // The pass at the end of the tick, which the next tick's movement feels.
    let (_, delta) = fluid_pass(level, ITEM_SIZE, (position, delta))?;
    Some(ItemStep {
        position,
        delta,
        on_ground,
        main_supporting_block,
        on_ground_no_blocks,
    })
}

impl ClientWorld {
    /// One client tick of a dropped item's movement.
    pub(crate) fn tick_item_entity(&mut self, entity_id: EntityId) {
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        if entity.type_id != VANILLA_ITEM_ENTITY_TYPE_ID {
            return;
        }
        let (x, y, z) = entity.position();
        let (dx, dy, dz) = entity.motion();
        let state = ItemState {
            id: entity_id,
            tick_count: entity.tick_count,
            position: Vec3::new(x, y, z),
            delta: Vec3::new(dx, dy, dz),
            on_ground: entity.on_ground,
            main_supporting_block: entity.tracking.support.main_block,
            on_ground_no_blocks: entity.tracking.support.no_blocks,
            no_gravity: entity.status.no_gravity,
        };
        let Some(step) = item_step(&Level::new(self), &state) else {
            return;
        };
        let mut entity = self
            .entities
            .at_mut(entity_id)
            .expect("the item was read above");
        entity.set_position((step.position.x, step.position.y, step.position.z));
        entity.vel.motion.x = JavaF64::new(step.delta.x);
        entity.vel.motion.y = JavaF64::new(step.delta.y);
        entity.vel.motion.z = JavaF64::new(step.delta.z);
        entity.on_ground = step.on_ground;
        entity.tracking.support = Support {
            main_block: step.main_supporting_block,
            no_blocks: step.on_ground_no_blocks,
        };
    }

    /// `Entity.setOnGround`'s `checkSupportingBlock`, for an item whose
    /// ground state a packet just set.
    pub(crate) fn check_item_supporting_block(&mut self, entity_id: EntityId) {
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        if entity.type_id != VANILLA_ITEM_ENTITY_TYPE_ID {
            return;
        }
        let (x, y, z) = entity.position();
        let position = Vec3::new(x, y, z);
        let mut main_supporting_block = entity.tracking.support.main_block;
        let mut on_ground_no_blocks = entity.tracking.support.no_blocks;
        let checked = Level::new(self).check_supporting_block(
            (&mut main_supporting_block, &mut on_ground_no_blocks),
            (item_box(position), position, 0.0),
            entity.on_ground,
            None,
        );
        if checked.is_err() {
            return;
        }
        let mut entity = self
            .entities
            .at_mut(entity_id)
            .expect("the item was read above");
        entity.tracking.support = Support {
            main_block: main_supporting_block,
            no_blocks: on_ground_no_blocks,
        };
    }
}
