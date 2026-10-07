use super::*;

/// Apply Java LocalPlayer's item, slow-movement, and square-speed modifiers.
pub fn modify_local_player_input(input: Vec2, modifiers: LocalPlayerInputModifiers) -> Vec2 {
    if input.length_squared() == 0.0 {
        return input;
    }

    let mut modified = input.scale(0.98_f32);
    if modifiers.using_item && !modifiers.passenger {
        modified = modified.scale(modifiers.item_use_speed_multiplier);
    }
    if modifiers.moving_slowly {
        modified = modified.scale(modifiers.sneaking_movement_factor);
    }

    modify_input_speed_for_square_movement(modified)
}

/// Apply Java LocalPlayer.applyInput to the local player's movement fields.
pub fn apply_local_player_input(
    state: LocalPlayerInputState,
    modifiers: LocalPlayerInputModifiers,
) -> LocalPlayerAppliedInput {
    if !state.controlled_camera {
        return LocalPlayerAppliedInput {
            movement: state.current_impulses.scale(0.98_f32),
            jumping: state.current_jumping,
            x_bob: state.current_bob.x,
            y_bob: state.current_bob.y,
            x_bob_old: state.previous_bob.x,
            y_bob_old: state.previous_bob.y,
        };
    }

    let modified_input = modify_local_player_input(state.move_vector, modifiers);
    let bob_delta = Vec2::new(
        (state.rotation.x - state.current_bob.x) * 0.5_f32,
        (state.rotation.y - state.current_bob.y) * 0.5_f32,
    );
    let next_bob = state.current_bob.add_vec2(bob_delta);
    LocalPlayerAppliedInput {
        movement: modified_input,
        jumping: state.jump_pressed,
        x_bob: next_bob.x,
        y_bob: next_bob.y,
        x_bob_old: state.current_bob.x,
        y_bob_old: state.current_bob.y,
    }
}

/// Apply the non-passenger LocalPlayer.canStartSprinting predicate.
pub fn can_start_local_player_sprinting(state: LocalPlayerSprintGateInput) -> bool {
    !state.sprinting
        && state.forward_impulse > 1.0e-5_f32
        && !state.mobility_restricted
        && (state.enough_food || state.may_fly)
        && (state.flying || !state.shallow_water)
        && !state.slow_due_to_using_item
        && (!state.fall_flying || state.underwater)
        && (!state.moving_slowly || state.underwater)
}

/// Advance Java LocalPlayer's non-passenger sprint trigger window.
pub fn advance_local_player_sprint_trigger(
    state: LocalPlayerSprintTriggerInput,
) -> LocalPlayerSprintTriggerResult {
    let mut sprint_trigger_time = state.sprint_trigger_time;
    if sprint_trigger_time > 0 {
        sprint_trigger_time -= 1;
    }

    if state.previous_input.shift
        || state.conditions.slow_due_to_using_item
        || state.current_input.backward
    {
        sprint_trigger_time = 0;
    }

    let can_start = can_start_local_player_sprinting(LocalPlayerSprintGateInput {
        sprinting: state.sprinting,
        forward_impulse: state.current_move_vector.y,
        mobility_restricted: state.conditions.mobility_restricted,
        enough_food: state.conditions.enough_food,
        may_fly: state.conditions.may_fly,
        shallow_water: state.conditions.shallow_water,
        flying: state.conditions.flying,
        slow_due_to_using_item: state.conditions.slow_due_to_using_item,
        fall_flying: state.conditions.fall_flying,
        underwater: state.conditions.underwater,
        moving_slowly: state.conditions.moving_slowly,
    });
    let mut sprinting = state.sprinting;
    if can_start {
        if state.previous_move_vector.y.partial_cmp(&1.0e-5_f32) != Some(Ordering::Greater) {
            if sprint_trigger_time > 0 {
                sprinting = true;
            } else {
                sprint_trigger_time = state.sprint_window;
            }
        }

        if state.current_input.sprint {
            sprinting = true;
        }
    }

    LocalPlayerSprintTriggerResult {
        sprinting,
        sprint_trigger_time,
    }
}

/// Apply Java LivingEntity.getFluidFallingAdjustedMovement.
pub fn adjust_local_player_fluid_falling_movement(state: LocalPlayerFluidFallingInput) -> Vec3 {
    if state.base_gravity != 0.0 && !state.sprinting {
        let gravity_step = state.base_gravity / 16.0;
        let movement_y = if state.is_falling
            && (state.movement.y - 0.005).abs() >= 0.003
            && (state.movement.y - gravity_step).abs() < 0.003
        {
            -0.003
        } else {
            state.movement.y - gravity_step
        };
        Vec3::new(state.movement.x, movement_y, state.movement.z)
    } else {
        state.movement
    }
}

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

/// Apply Java LivingEntity.travelInWater with collision and fluid lookup
/// results supplied by the caller.
pub fn local_player_travel_in_water(state: LocalPlayerTravelInWaterInput) -> Vec3 {
    let mut slow_down = if state.sprinting {
        0.9_f32
    } else {
        state.water_slow_down
    };
    let mut speed = 0.02_f32;
    let mut water_walker = state.water_efficiency;
    if !state.on_ground {
        water_walker *= 0.5_f32;
    }
    if water_walker > 0.0_f32 {
        slow_down += (0.54600006_f32 - slow_down) * water_walker;
        speed += (state.speed - speed) * water_walker;
    }
    if state.dolphins_grace {
        slow_down = 0.96_f32;
    }

    let relative_movement = local_player_relative_movement(state.input, speed, state.y_rot);
    let moved = state.delta_movement.add_vec3(relative_movement);
    let moved = if moved.is_finite() {
        moved
    } else {
        state.delta_movement
    };
    let mut movement = moved;
    if state.horizontal_collision && state.on_climbable {
        movement = Vec3::new(movement.x, 0.2, movement.z);
    }
    let scaled = movement.multiply_components(
        f64::from(slow_down),
        f64::from(0.8_f32),
        f64::from(slow_down),
    );
    let adjusted = adjust_local_player_fluid_falling_movement(LocalPlayerFluidFallingInput {
        base_gravity: state.base_gravity,
        is_falling: state.is_falling,
        sprinting: state.sprinting,
        movement: scaled,
    });
    if adjusted.is_finite() {
        movement = adjusted;
    }

    if state.horizontal_collision && state.free_after_jump {
        let jump_movement = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
        if jump_movement.is_finite() {
            movement = jump_movement;
        }
    }
    movement
}

/// Apply Java LivingEntity.travelInLava with fluid lookup and collision
/// results supplied by the caller.
pub fn local_player_travel_in_lava(state: LocalPlayerTravelInLavaInput) -> Vec3 {
    let relative_movement = local_player_relative_movement(state.input, 0.02_f32, state.y_rot);
    let moved = state.delta_movement.add_vec3(relative_movement);
    let mut movement = if moved.is_finite() {
        moved
    } else {
        state.delta_movement
    };

    if state.shallow_lava {
        let scaled = movement.multiply_components(0.5, f64::from(0.8_f32), 0.5);
        if scaled.is_finite() {
            movement = scaled;
        }
        let adjusted = adjust_local_player_fluid_falling_movement(LocalPlayerFluidFallingInput {
            base_gravity: state.base_gravity,
            is_falling: state.is_falling,
            sprinting: state.sprinting,
            movement,
        });
        if adjusted.is_finite() {
            movement = adjusted;
        }
    } else {
        let scaled = movement.scale(0.5);
        if scaled.is_finite() {
            movement = scaled;
        }
    }

    if state.base_gravity != 0.0 {
        let with_gravity = movement.add_components(0.0, -state.base_gravity / 4.0, 0.0);
        if with_gravity.is_finite() {
            movement = with_gravity;
        }
    }

    if state.horizontal_collision && state.free_after_jump {
        let jump_movement = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
        if jump_movement.is_finite() {
            movement = jump_movement;
        }
    }
    movement
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

/// Apply Java LivingEntity.travelInAir with block, effect, and level state
/// resolved by the caller.
pub fn local_player_travel_in_air(state: LocalPlayerTravelInAirInput) -> Vec3 {
    let original_delta_movement = state.delta_movement;
    let block_friction = if state.on_ground {
        local_player_modified_friction(LocalPlayerModifiedFrictionInput {
            friction: state.block_friction,
            modifier: state.friction_modifier,
        })
    } else {
        1.0_f32
    };
    let movement = local_player_handle_relative_friction(LocalPlayerHandleRelativeFrictionInput {
        input: state.input,
        delta_movement: state.delta_movement,
        y_rot: state.y_rot,
        on_ground: state.on_ground,
        block_friction,
        speed: state.speed,
        flying_speed: state.flying_speed,
        flying: state.flying,
        sprinting: state.sprinting,
        passenger: state.passenger,
        on_climbable: state.on_climbable,
        scaffolding: state.scaffolding,
        suppress_sliding_down_ladder: state.suppress_sliding_down_ladder,
        horizontal_collision: state.horizontal_collision,
        jumping: state.jumping,
    });
    let movement_y = if let Some(amplifier) = state.levitation_amplifier {
        movement.y + (0.05 * (f64::from(amplifier) + 1.0) - movement.y) * 0.2
    } else if !state.client_side || state.has_chunk_at {
        movement.y - state.gravity
    } else if state.current_y > f64::from(state.min_y) {
        -0.1
    } else {
        0.0
    };

    let candidate = if state.should_discard_friction {
        Vec3::new(movement.x, movement_y, movement.z)
    } else {
        let air_drag = local_player_modified_friction(LocalPlayerModifiedFrictionInput {
            friction: 0.91_f32,
            modifier: state.air_drag_modifier,
        });
        let friction = block_friction * air_drag;
        let vertical_friction = if state.omnidirectional_air_mover {
            air_drag
        } else {
            local_player_modified_friction(LocalPlayerModifiedFrictionInput {
                friction: 0.98_f32,
                modifier: state.air_drag_modifier,
            })
        };
        Vec3::new(
            movement.x * f64::from(friction),
            movement_y * f64::from(vertical_friction),
            movement.z * f64::from(friction),
        )
    };
    if candidate.is_finite() {
        candidate
    } else {
        original_delta_movement
    }
}

/// Apply Java LivingEntity.updateFallFlyingMovement with look and gravity
/// state resolved by the caller.
pub fn local_player_update_fall_flying_movement(
    state: LocalPlayerUpdateFallFlyingMovementInput,
) -> Vec3 {
    let lean_angle = state.x_rot * MTH_DEG_TO_RAD;
    let look_horizontal_length =
        (state.look_angle.x * state.look_angle.x + state.look_angle.z * state.look_angle.z).sqrt();
    let move_horizontal_length = state.delta_movement.horizontal_distance();
    let lift_force = f64::from(lean_angle).cos().powi(2);
    let mut movement =
        state
            .delta_movement
            .add_components(0.0, state.gravity * (-1.0 + lift_force * 0.75), 0.0);

    if movement.y < 0.0 && look_horizontal_length > 0.0 {
        let convert = movement.y * -0.1 * lift_force;
        movement = movement.add_components(
            state.look_angle.x * convert / look_horizontal_length,
            convert,
            state.look_angle.z * convert / look_horizontal_length,
        );
    }

    if lean_angle < 0.0 && look_horizontal_length > 0.0 {
        let convert =
            move_horizontal_length * -f64::from(mth_sin_f64(f64::from(lean_angle))) * 0.04;
        movement = movement.add_components(
            -state.look_angle.x * convert / look_horizontal_length,
            convert * 3.2,
            -state.look_angle.z * convert / look_horizontal_length,
        );
    }

    if look_horizontal_length > 0.0 {
        movement = movement.add_components(
            (state.look_angle.x / look_horizontal_length * move_horizontal_length - movement.x)
                * 0.1,
            0.0,
            (state.look_angle.z / look_horizontal_length * move_horizontal_length - movement.z)
                * 0.1,
        );
    }

    movement.multiply_components(
        f64::from(0.99_f32),
        f64::from(0.98_f32),
        f64::from(0.99_f32),
    )
}
