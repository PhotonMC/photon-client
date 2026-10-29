//! A fishing bobber's client tick (`FishingHook.tick`). The server tracks the bobber only every
//! five ticks, so between its packets the client flies it, floats it and holds it to what it
//! hooked, as the reference does.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::ids::EntityId;
use photon_core::java_float::JavaF64;
use photon_core::math::{Vec3, floor_f64};
use photon_core::random::RandomSource;

use crate::ClientWorld;
use crate::collision::collide_entity_movement;
use crate::entity::ClientEntityDimensions;
use crate::local_player::Level;
use crate::{metadata_boolean_value, metadata_int_value};

/// `FishingHook.DATA_HOOKED_ENTITY` and `DATA_BITING`.
pub(crate) const HOOKED_ENTITY_DATA_ID: u8 = 8;
pub(crate) const BITING_DATA_ID: u8 = 9;
/// `Entity.getBlockPosBelowThatAffectsMyMovement`.
const BELOW_OFFSET: f32 = 0.500_001;

/// `FishingHook.FishHookState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum HookPhase {
    #[default]
    Flying,
    HookedInEntity,
    Bobbing,
}

/// What a bobber keeps between ticks.
#[derive(Debug, Clone)]
pub struct HookState {
    phase: HookPhase,
    /// The hooked entity's id plus one, none at zero (`DATA_HOOKED_ENTITY`).
    hooked: i32,
    biting: bool,
    /// the synchronized random (`FishingHook`'s own), reseeded each tick from the bobber's UUID and the game time.
    synchronized: RandomSource,
    /// The entity's own random source, which the float draws from.
    own: RandomSource,
}

/// The random sources are no part of what a bobber is equal to.
impl PartialEq for HookState {
    fn eq(&self, other: &Self) -> bool {
        (self.phase, self.hooked, self.biting) == (other.phase, other.hooked, other.biting)
    }
}

impl Eq for HookState {}

impl HookState {
    fn new(entity_id: EntityId) -> Self {
        Self {
            phase: HookPhase::Flying,
            hooked: 0,
            biting: false,
            synchronized: RandomSource::create(0),
            own: RandomSource::create(i64::from(entity_id.0)),
        }
    }
}

/// `EntityType.FISHING_BOBBER`'s size.
fn hook_box(position: Vec3) -> Aabb {
    ClientEntityDimensions::scalable(0.25, 0.25).make_bounding_box(position)
}

/// One tick's outcome: the new position, motion and ground flag.
struct HookStep {
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    /// The motion it moved with, which `updateRotation` turns it towards; none when it did not move.
    moved_with: Option<Vec3>,
}

/// The water height at the bobber's block (`FluidState.getHeight` of water, else nothing).
fn water_height(level: &Level<'_>, pos: BlockPos) -> f32 {
    let info = level.info(pos);
    if info.is_water() {
        level.fluid_height(pos, &info)
    } else {
        0.0
    }
}

/// The phase changes and the float, before gravity and the move; `Err` holds the motion the tick
/// ends with when it ends there.
fn steer(state: &mut HookState, position: Vec3, delta: Vec3, height: f32) -> Result<Vec3, Vec3> {
    let in_water = height > 0.0;
    match state.phase {
        HookPhase::Flying if state.hooked > 0 => {
            state.phase = HookPhase::HookedInEntity;
            Err(Vec3::ZERO)
        }
        HookPhase::Flying if in_water => {
            state.phase = HookPhase::Bobbing;
            Err(Vec3::new(delta.x * 0.3, delta.y * 0.2, delta.z * 0.3))
        }
        HookPhase::Flying => Ok(delta),
        HookPhase::HookedInEntity => Err(Vec3::ZERO),
        HookPhase::Bobbing => {
            let mut force = position.y + delta.y - floor_f64(position.y) as f64 - f64::from(height);
            if force.abs() < 0.01 {
                force += crate::java_math::signum(force) * 0.1;
            }
            let pull = force * f64::from(state.own.next_float()) * 0.2;
            let mut delta = Vec3::new(delta.x * 0.9, delta.y - pull, delta.z * 0.9);
            if in_water && state.biting {
                let sink = -0.1
                    * f64::from(state.synchronized.next_float())
                    * f64::from(state.synchronized.next_float());
                delta = delta.add_vec3(Vec3::new(0.0, sink, 0.0));
            }
            Ok(delta)
        }
    }
}

/// `FishingHook.tick` after the interpolation and the base tick, as the client runs it.
fn hook_step(
    level: &Level<'_>,
    state: &mut HookState,
    (position, delta, on_ground): (Vec3, Vec3, bool),
) -> HookStep {
    let pos = BlockPos::new(
        floor_f64(position.x),
        floor_f64(position.y),
        floor_f64(position.z),
    );
    let height = water_height(level, pos);
    let mut delta = match steer(state, position, delta, height) {
        Ok(delta) => delta,
        Err(delta) => {
            return HookStep {
                position,
                delta,
                on_ground,
                moved_with: None,
            };
        }
    };
    if state.phase == HookPhase::Flying {
        delta = clip_flight(level, position, delta);
    }
    if !level.info(pos).is_water() && !on_ground && state.hooked == 0 {
        delta = delta.add_vec3(Vec3::new(0.0, -0.03, 0.0));
    }
    move_hook(
        level,
        state.phase == HookPhase::Flying,
        (position, delta, on_ground),
    )
}

/// `FishingHook.checkCollision` against the blocks (`onHitBlock`): a flight that meets a block's
/// collision shape this tick is cut down to the hit's distance, which `HitResult.distanceTo`
/// measures squared.
fn clip_flight(level: &Level<'_>, position: Vec3, delta: Vec3) -> Vec3 {
    match level.world.clip_blocks(position, position.add_vec3(delta)) {
        Some(hit) => delta.normalized().scale(hit.distance_to_sqr(position)),
        None => delta,
    }
}

/// `Entity.move` through the blocks, then a flying bobber stops on what it hit, and the drag.
fn move_hook(
    level: &Level<'_>,
    flying: bool,
    (position, mut delta, on_ground): (Vec3, Vec3, bool),
) -> HookStep {
    let moved_with = Some(delta);
    let movement = collide_hook(level, position, delta);
    let moved = movement.length_squared();
    let position = if moved > 1.0e-7 || delta.length_squared() - moved < 1.0e-7 {
        position.add_vec3(movement)
    } else {
        position
    };
    // The ground is felt only by a move with some height in it.
    let on_ground = if delta.y.abs() > 0.0 {
        delta.y != movement.y && delta.y < 0.0
    } else {
        on_ground
    };
    let horizontal = delta.x != movement.x || delta.z != movement.z;
    if flying && (on_ground || horizontal) {
        delta = Vec3::ZERO;
    }
    let speed = f64::from(speed_factor(level, position));
    let delta = Vec3::new(delta.x * speed, delta.y, delta.z * speed);
    HookStep {
        position,
        delta: delta.scale(0.92),
        on_ground,
        moved_with,
    }
}

/// `Entity.collide` of the bobber's box through the blocks.
fn collide_hook(level: &Level<'_>, position: Vec3, delta: Vec3) -> Vec3 {
    let bounding_box = hook_box(position);
    match level.colliders(bounding_box.expand_towards_vec3(delta), 0.0, position) {
        Ok(colliders) if delta.length_squared() != 0.0 => {
            collide_entity_movement(delta, bounding_box, &colliders)
        }
        _ => delta,
    }
}

/// `Entity.getBlockSpeedFactor` where the bobber now is.
fn speed_factor(level: &Level<'_>, position: Vec3) -> f32 {
    let here = BlockPos::new(
        floor_f64(position.x),
        floor_f64(position.y),
        floor_f64(position.z),
    );
    let below = level.on_pos(None, position, BELOW_OFFSET);
    level.block_speed_factor(here, below)
}

impl ClientWorld {
    /// The entities the client moves between their packets: a dropped item and a bobber.
    pub(crate) fn tick_client_moved(&mut self, entity_id: EntityId) {
        self.tick_item_entity(entity_id);
        self.tick_fishing_hook(entity_id);
    }

    /// One client tick of a fishing bobber's movement, after its interpolation.
    pub(crate) fn tick_fishing_hook(&mut self, entity_id: EntityId) {
        let game_time = self.game_time() as i64;
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        if !entity.is_fishing_bobber() {
            return;
        }
        let seed = entity.uuid.least as i64 ^ game_time;
        let (x, y, z) = entity.position();
        let (dx, dy, dz) = entity.motion();
        let motion = (Vec3::new(x, y, z), Vec3::new(dx, dy, dz), entity.on_ground);
        let mut state = entity
            .kind
            .fishing_hook_or_insert_with(|| Box::new(HookState::new(entity_id)))
            .clone();
        state.synchronized.set_seed(seed);
        if state.phase == HookPhase::HookedInEntity {
            self.follow_hooked(entity_id, &mut state);
            self.store_hook(entity_id, state, None);
            return;
        }
        let step = hook_step(&Level::new(self), &mut state, motion);
        self.store_hook(entity_id, state, Some(step));
    }

    /// The hooked phase: the bobber sits at the hooked entity, or flies again once it is gone.
    fn follow_hooked(&mut self, entity_id: EntityId, state: &mut HookState) {
        let hooked = (state.hooked > 0)
            .then(|| self.hooked_at(EntityId(state.hooked - 1)))
            .flatten();
        match hooked {
            Some(at) => {
                if let Some(mut entity) = self.entities.at_mut(entity_id) {
                    entity.set_position(at);
                }
            }
            None if state.hooked > 0 => {
                state.hooked = 0;
                state.phase = HookPhase::Flying;
            }
            None => {}
        }
    }

    /// `Entity.getY(0.8)` of what a bobber hooked: an entity of the level, or the local player
    /// (whom `ClientLevel.getEntity` finds too).
    fn hooked_at(&self, hooked: EntityId) -> Option<(f64, f64, f64)> {
        if let Some(entity) = self.entities.at(hooked) {
            let (x, y, z) = entity.position();
            return Some((x, y + f64::from(entity.bb_height()) * 0.8, z));
        }
        let player = self
            .local_player
            .as_ref()
            .filter(|player| player.id == hooked)?;
        let (x, y, z) = player.position();
        let height = player.pose.player_dimensions().height();
        Some((x, y + f64::from(height) * 0.8, z))
    }

    fn store_hook(&mut self, entity_id: EntityId, state: HookState, step: Option<HookStep>) {
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        if let Some(step) = step {
            entity.set_position((step.position.x, step.position.y, step.position.z));
            if let Some(moved_with) = step.moved_with {
                entity.motion.x = JavaF64::new(moved_with.x);
                entity.motion.y = JavaF64::new(moved_with.y);
                entity.motion.z = JavaF64::new(moved_with.z);
                crate::objects::flight::update_rotation(&mut entity);
            }
            entity.motion.x = JavaF64::new(step.delta.x);
            entity.motion.y = JavaF64::new(step.delta.y);
            entity.motion.z = JavaF64::new(step.delta.z);
            entity.on_ground = step.on_ground;
        }
        *entity
            .kind
            .fishing_hook_or_insert_with(|| Box::new(HookState::new(entity_id))) = state;
    }

    /// `FishingHook.onSyncedDataUpdated`: the hooked entity, and a bite that pulls the bobber
    /// under.
    pub fn apply_fishing_hook_metadata(&mut self, entity_id: EntityId, metadata: &[u8]) {
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        if !entity.is_fishing_bobber() {
            return;
        }
        let hooked = metadata_int_value(metadata, HOOKED_ENTITY_DATA_ID);
        let biting = metadata_boolean_value(metadata, BITING_DATA_ID);
        let (dx, _, dz) = entity.motion();
        let state = entity
            .kind
            .fishing_hook_or_insert_with(|| Box::new(HookState::new(entity_id)));
        if let Some(hooked) = hooked {
            state.hooked = hooked;
        }
        let Some(biting) = biting else {
            return;
        };
        state.biting = biting;
        if biting {
            let sink = -0.4 * (state.synchronized.next_float() * (1.0_f32 - 0.6) + 0.6);
            entity.motion.x = JavaF64::new(dx);
            entity.motion.y = JavaF64::new(f64::from(sink));
            entity.motion.z = JavaF64::new(dz);
        }
    }
}
