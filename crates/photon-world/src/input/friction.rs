use super::*;

/// Apply Java LivingEntity.handleOnClimbable to local player motion.
pub fn clamp_local_player_climbable_movement(state: LocalPlayerClimbableMovementInput) -> Vec3 {
    if !state.on_climbable {
        return state.movement;
    }

    // The Java source passes -0.15F and 0.15F to the double overload, so the
    // bounds are the f32 values widened to f64.
    let max = f64::from(0.15_f32);
    let mut movement_y = java_max_f64(state.movement.y, -max);
    if movement_y < 0.0 && !state.scaffolding && state.suppress_sliding_down_ladder {
        movement_y = 0.0;
    }
    Vec3::new(
        java_math::clamp(state.movement.x, -max, max),
        movement_y,
        java_math::clamp(state.movement.z, -max, max),
    )
}

/// Apply Java LivingEntity.getFrictionInfluencedSpeed and getFlyingSpeed.
pub fn local_player_friction_influenced_speed(
    state: LocalPlayerFrictionInfluencedSpeedInput,
) -> f32 {
    if state.on_ground {
        if state.block_friction > 0.6_f32 {
            state.speed
                * (0.21600002_f32
                    / (state.block_friction * state.block_friction * state.block_friction))
        } else {
            state.speed
        }
    } else if state.flying && !state.passenger {
        if state.sprinting {
            state.flying_speed * 2.0_f32
        } else {
            state.flying_speed
        }
    } else if state.sprinting {
        0.025999999_f32
    } else {
        0.02_f32
    }
}

/// Apply Java LivingEntity.computeModifiedFriction.
pub fn local_player_modified_friction(state: LocalPlayerModifiedFrictionInput) -> f32 {
    clamp_f32(
        1.0_f32 - (1.0_f32 - state.friction) * state.modifier,
        0.0_f32,
        1.0_f32,
    )
}

/// Apply Java LivingEntity.handleRelativeFrictionAndCalculateMovement with
/// collision movement supplied by the caller as a no-op path.
pub fn local_player_handle_relative_friction(
    state: LocalPlayerHandleRelativeFrictionInput,
) -> Vec3 {
    let speed = local_player_friction_influenced_speed(LocalPlayerFrictionInfluencedSpeedInput {
        on_ground: state.on_ground,
        block_friction: state.block_friction,
        speed: state.speed,
        flying: state.flying,
        flying_speed: state.flying_speed,
        sprinting: state.sprinting,
        passenger: state.passenger,
    });
    let relative_movement = local_player_relative_movement(state.input, speed, state.y_rot);
    let moved = state.delta_movement.add_vec3(relative_movement);
    let mut movement = if moved.is_finite() {
        moved
    } else {
        state.delta_movement
    };

    let clamped = clamp_local_player_climbable_movement(LocalPlayerClimbableMovementInput {
        movement,
        on_climbable: state.on_climbable,
        scaffolding: state.scaffolding,
        suppress_sliding_down_ladder: state.suppress_sliding_down_ladder,
    });
    if clamped.is_finite() {
        movement = clamped;
    }

    if (state.horizontal_collision || state.jumping) && state.on_climbable {
        let step_movement = Vec3::new(movement.x, 0.2, movement.z);
        if step_movement.is_finite() {
            movement = step_movement;
        }
    }
    movement
}

/// The environment state consumed by LivingEntity's climbable movement clamp.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerClimbableMovementInput {
    pub movement: Vec3,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
}

/// The state consumed by LivingEntity's friction-influenced speed helper.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerFrictionInfluencedSpeedInput {
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying: bool,
    pub flying_speed: f32,
    pub sprinting: bool,
    pub passenger: bool,
}

/// The block friction and attribute modifier consumed by LivingEntity travel.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerModifiedFrictionInput {
    pub friction: f32,
    pub modifier: f32,
}

/// The environment state consumed by LivingEntity's relative friction path.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerHandleRelativeFrictionParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
}

/// The environment and motion state consumed by LivingEntity's relative
/// friction path.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerHandleRelativeFrictionInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
}
