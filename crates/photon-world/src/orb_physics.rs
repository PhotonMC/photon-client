//! An experience orb's client tick (`ExperienceOrb.tick`). Between the server's packets the client
//! moves the orb itself: it falls, bounces off the ground, slides and floats up in water.

use photon_core::binary::BlockPos;
use photon_core::ids::EntityId;
use photon_core::java_float::JavaF64;
use photon_core::math::{Vec3, floor_f64};

use super::ClientWorld;
use super::collision::collide_entity_movement;
use super::constants::{
    VANILLA_EXPERIENCE_ORB_AIR_FRICTION, VANILLA_EXPERIENCE_ORB_GRAVITY,
    VANILLA_PLAYER_ENTITY_TYPE_ID,
};
use super::entity::Support;
use super::item_physics::{fluid_pass, square_dimensions, step_on};
use super::local_player::Level;

/// `EntityType.EXPERIENCE_ORB`'s width and height.
const SIZE: f32 = 0.5;
/// `Entity.getBlockPosBelowThatAffectsMyMovement` of an orb.
const BELOW_OFFSET: f32 = 0.999_999;

/// The orb as it goes into the tick.
struct OrbState {
    /// Where the player the orb follows is aiming at (`followNearbyPlayer`), if one is near.
    pull_target: Option<Vec3>,
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    support: Support,
    no_gravity: bool,
}

/// What the tick changes.
struct OrbStep {
    position: Vec3,
    delta: Vec3,
    on_ground: bool,
    support: Support,
}

/// What `followNearbyPlayer` adds to the motion: towards the player, the harder the nearer.
fn pull(towards: Vec3) -> Vec3 {
    let power = 1.0 - towards.length_squared().sqrt() / 8.0;
    let unit = towards.normalized();
    let strength = power * power * 0.1;
    Vec3::new(unit.x * strength, unit.y * strength, unit.z * strength)
}

/// `ExperienceOrb.tick` after the first tick: the water's current and buoyancy, or gravity unless
/// the orb is inside a block, `Entity.move` against the blocks, drag, ground friction and the
/// bounce off what it falls on. (Lava's random kicks and the pull towards a near player are not
/// carried yet.)
fn orb_step(level: &Level<'_>, state: &OrbState) -> Option<OrbStep> {
    let dimensions = square_dimensions(SIZE);
    let position = state.position;
    let (fluids, mut delta) = fluid_pass(level, SIZE, (position, state.delta))?;
    let colliding = !level
        .no_collision(dimensions.make_bounding_box(position), 0.0, position)
        .ok()?;
    let gravity = if state.no_gravity {
        0.0
    } else {
        VANILLA_EXPERIENCE_ORB_GRAVITY
    };
    if fluids.water.eyes_inside {
        delta = Vec3::new(
            delta.x * f64::from(0.99_f32),
            (delta.y + f64::from(5.0e-4_f32)).min(f64::from(0.06_f32)),
            delta.z * f64::from(0.99_f32),
        );
    } else if !colliding && gravity != 0.0 {
        delta = delta.add_vec3(Vec3::new(0.0, -gravity, 0.0));
    }

    if let Some(target) = state.pull_target {
        delta = delta.add_vec3(pull(target.subtract_vec3(position)));
    }
    let fall_speed = delta.y;
    let (position, on_ground, support, landed, movement) = move_orb(level, state, delta)?;
    let delta = restitute_orb(
        level,
        (position, on_ground, support),
        (delta, movement),
        gravity,
    );
    let delta = drag(
        level,
        (position, on_ground, support),
        (delta, landed.then_some(fall_speed)),
        gravity,
    );
    Some(OrbStep {
        position,
        delta,
        on_ground,
        support,
    })
}

/// 26.3: a bump keeps none of the orb's speed (`restituteMovementAfterCollisions`).
fn restitute_orb(
    level: &Level<'_>,
    (position, on_ground, support): (Vec3, bool, Support),
    (delta, movement): (Vec3, Vec3),
    gravity: f64,
) -> Vec3 {
    let vertical = delta.y.abs() > 0.0 && delta.y != movement.y;
    let x = !photon_core::math::equal_f64(delta.x, movement.x);
    let z = !photon_core::math::equal_f64(delta.z, movement.z);
    if !(vertical || x || z) {
        return delta;
    }
    let effect = level.on_pos(support.main_block, position, 0.2);
    crate::item_physics::restitute(
        &level.info(effect),
        (x, z, vertical && on_ground),
        (delta, movement, vertical),
        (gravity, 0.98),
    )
}

/// `Entity.move` of the orb against the blocks: where it ends up, the ground flags and whether it
/// landed.
fn move_orb(
    level: &Level<'_>,
    state: &OrbState,
    delta: Vec3,
) -> Option<(Vec3, bool, Support, bool, Vec3)> {
    let dimensions = square_dimensions(SIZE);
    let mut position = state.position;
    let mut on_ground = state.on_ground;
    let mut support = state.support;
    let bounding_box = dimensions.make_bounding_box(position);
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
    let landed = delta.y != movement.y && delta.y < 0.0;
    if delta.y.abs() > 0.0 {
        on_ground = landed;
        level
            .check_supporting_block(
                (&mut support.main_block, &mut support.no_blocks),
                (dimensions.make_bounding_box(position), position, 0.0),
                on_ground,
                Some(movement),
            )
            .ok()?;
    }
    Some((position, on_ground, support, landed, movement))
}

/// The block's speed factor, a slime block's slowing, drag, ground friction and the bounce off what
/// the orb fell on (`fall_speed` is given when it landed).
fn drag(
    level: &Level<'_>,
    (position, on_ground, support): (Vec3, bool, Support),
    (delta, fall_speed): (Vec3, Option<f64>),
    gravity: f64,
) -> Vec3 {
    let here = BlockPos::new(
        floor_f64(position.x),
        floor_f64(position.y),
        floor_f64(position.z),
    );
    let below = level.on_pos(support.main_block, position, BELOW_OFFSET);
    let speed_factor = f64::from(level.block_speed_factor(here, below));
    let mut delta = Vec3::new(delta.x * speed_factor, delta.y, delta.z * speed_factor);
    delta = step_on(level, (position, support.main_block), on_ground, delta);
    let mut friction = VANILLA_EXPERIENCE_ORB_AIR_FRICTION;
    if on_ground {
        friction *= level.info(below).friction;
    }
    let friction = f64::from(friction);
    delta = Vec3::new(delta.x * friction, delta.y * friction, delta.z * friction);
    match fall_speed {
        Some(fall) if fall < -gravity => Vec3::new(delta.x, -fall * 0.4, delta.z),
        _ => delta,
    }
}

impl ClientWorld {
    /// `ExperienceOrb.followNearbyPlayer`'s pick: the nearest player within eight blocks who is
    /// not a spectator nor dying, and the point halfway up their eyes the orb is drawn to.
    fn orb_pull_target(&self, orb: Vec3) -> Option<Vec3> {
        let remote = self
            .entities
            .all()
            .filter(|entity| entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID)
            .filter(|entity| !entity.is_spectator() && !entity.is_dead_or_dying())
            .map(|entity| {
                let (x, y, z) = entity.position();
                (Vec3::new(x, y, z), entity.scaled_dimensions().eye_height())
            });
        let local = self
            .local_player
            .as_ref()
            .filter(|player| !player.game_mode.is_spectator() && player.health() > 0.0)
            .map(|player| {
                let (x, y, z) = player.position();
                (Vec3::new(x, y, z), player.player_dimensions().eye_height())
            });
        remote
            .chain(local)
            .map(|(position, eye)| (position, eye, position.subtract_vec3(orb).length_squared()))
            .filter(|&(_, _, distance)| distance < 64.0)
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(position, eye, _)| {
                Vec3::new(position.x, position.y + f64::from(eye) / 2.0, position.z)
            })
    }
}

impl ClientWorld {
    /// One client tick of an experience orb's movement, after its position has eased and its
    /// first tick has passed.
    pub(crate) fn tick_experience_orb_movement(&mut self, entity_id: EntityId, moves: bool) {
        let Some(entity) = self.entities.at(entity_id).filter(|_| moves) else {
            return;
        };
        let (x, y, z) = entity.position();
        let (dx, dy, dz) = entity.motion();
        let state = OrbState {
            pull_target: self.orb_pull_target(Vec3::new(x, y, z)),
            position: Vec3::new(x, y, z),
            delta: Vec3::new(dx, dy, dz),
            on_ground: entity.on_ground,
            support: entity.tracking.support,
            no_gravity: entity.status.no_gravity,
        };
        let Some(step) = orb_step(&Level::new(self), &state) else {
            return;
        };
        let mut entity = self
            .entities
            .at_mut(entity_id)
            .expect("the orb was read above");
        entity.set_position((step.position.x, step.position.y, step.position.z));
        entity.vel.motion.x = JavaF64::new(step.delta.x);
        entity.vel.motion.y = JavaF64::new(step.delta.y);
        entity.vel.motion.z = JavaF64::new(step.delta.z);
        entity.on_ground = step.on_ground;
        entity.tracking.support = step.support;
    }
}
