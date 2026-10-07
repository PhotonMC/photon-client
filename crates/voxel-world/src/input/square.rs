use super::*;

/// Apply Java LivingEntity.travel's top-level branch selection and its
/// resolved no-collision movement path.
pub fn local_player_travel_dispatch(
    state: LocalPlayerTravelDispatchInput,
) -> LocalPlayerTravelDispatchResult {
    let parameters = state.parameters;
    let should_travel_in_fluid = (parameters.in_water || parameters.in_lava)
        && parameters.affected_by_fluids
        && !parameters.can_stand_on_fluid;
    let mode = if should_travel_in_fluid {
        if parameters.in_water {
            LocalPlayerTravelMode::Water
        } else {
            LocalPlayerTravelMode::Lava
        }
    } else if parameters.fall_flying {
        LocalPlayerTravelMode::FallFlying
    } else {
        LocalPlayerTravelMode::Air
    };

    let is_falling = state.delta_movement.y <= 0.0;
    let air_movement = || {
        local_player_travel_in_air(LocalPlayerTravelInAirInput {
            input: parameters.input,
            delta_movement: state.delta_movement,
            y_rot: parameters.y_rot,
            on_ground: parameters.on_ground,
            block_friction: parameters.block_friction,
            friction_modifier: parameters.friction_modifier,
            speed: parameters.speed,
            flying_speed: parameters.flying_speed,
            flying: parameters.flying,
            sprinting: parameters.sprinting,
            passenger: parameters.passenger,
            on_climbable: parameters.on_climbable,
            scaffolding: parameters.scaffolding,
            suppress_sliding_down_ladder: parameters.suppress_sliding_down_ladder,
            horizontal_collision: parameters.horizontal_collision,
            jumping: parameters.jumping,
            levitation_amplifier: parameters.levitation_amplifier,
            gravity: parameters.gravity,
            client_side: parameters.client_side,
            has_chunk_at: parameters.has_chunk_at,
            current_y: parameters.current_y,
            min_y: parameters.min_y,
            should_discard_friction: parameters.should_discard_friction,
            air_drag_modifier: parameters.air_drag_modifier,
            omnidirectional_air_mover: parameters.omnidirectional_air_mover,
        })
    };
    let (fall_flying_climbable_fallback, movement) = match mode {
        LocalPlayerTravelMode::Water => (
            false,
            local_player_travel_in_water(LocalPlayerTravelInWaterInput {
                input: parameters.input,
                delta_movement: state.delta_movement,
                y_rot: parameters.y_rot,
                sprinting: parameters.sprinting,
                water_slow_down: parameters.water_slow_down,
                water_efficiency: parameters.water_efficiency,
                on_ground: parameters.on_ground,
                speed: parameters.speed,
                dolphins_grace: parameters.dolphins_grace,
                horizontal_collision: parameters.horizontal_collision,
                on_climbable: parameters.on_climbable,
                free_after_jump: parameters.free_after_jump,
                base_gravity: parameters.gravity,
                is_falling,
            }),
        ),
        LocalPlayerTravelMode::Lava => (
            false,
            local_player_travel_in_lava(LocalPlayerTravelInLavaInput {
                input: parameters.input,
                delta_movement: state.delta_movement,
                y_rot: parameters.y_rot,
                sprinting: parameters.sprinting,
                shallow_lava: parameters.shallow_lava,
                horizontal_collision: parameters.horizontal_collision,
                free_after_jump: parameters.free_after_jump,
                base_gravity: parameters.gravity,
                is_falling,
            }),
        ),
        LocalPlayerTravelMode::FallFlying if parameters.on_climbable => (true, air_movement()),
        LocalPlayerTravelMode::FallFlying => (
            false,
            local_player_update_fall_flying_movement(LocalPlayerUpdateFallFlyingMovementInput {
                look_angle: parameters.look_angle,
                delta_movement: state.delta_movement,
                x_rot: parameters.x_rot,
                gravity: parameters.gravity,
            }),
        ),
        LocalPlayerTravelMode::Air => (false, air_movement()),
    };

    LocalPlayerTravelDispatchResult {
        mode,
        fall_flying_climbable_fallback,
        movement,
    }
}

/// Apply the no-collision movement and drag path of Java LivingEntity.travelFlying.
pub fn local_player_travel_flying(state: LocalPlayerTravelFlyingInput) -> Vec3 {
    let (speed, drag) = if state.in_water {
        (state.water_speed, f64::from(0.8_f32))
    } else if state.in_lava {
        (state.lava_speed, 0.5)
    } else {
        (state.air_speed, f64::from(0.91_f32))
    };

    let relative_movement = local_player_relative_movement(state.input, speed, state.y_rot);

    let moved = state.delta_movement.add_vec3(relative_movement);
    let moved = if moved.is_finite() {
        moved
    } else {
        state.delta_movement
    };
    moved.scale(drag)
}

pub(crate) fn local_player_relative_movement(input: Vec3, speed: f32, y_rot: f32) -> Vec3 {
    let input_length_squared = input.length_squared();
    if input_length_squared < 1.0e-7 {
        Vec3::ZERO
    } else {
        let movement = if input_length_squared > 1.0 {
            input.normalized()
        } else {
            input
        }
        .scale(f64::from(speed));
        let angle = y_rot * MTH_DEG_TO_RAD;
        let sine = f64::from(mth_sin_f64(f64::from(angle)));
        let cosine = f64::from(mth_cos_f64(f64::from(angle)));
        Vec3::new(
            movement.x * cosine - movement.z * sine,
            movement.y,
            movement.z * cosine + movement.x * sine,
        )
    }
}

pub(super) fn modify_input_speed_for_square_movement(input: Vec2) -> Vec2 {
    let length = input.length();
    if length <= 0.0 {
        return input;
    }

    let direction = input.scale(1.0_f32 / length);
    let direction_x = direction.x.abs();
    let direction_y = direction.y.abs();
    let tangent = if direction_y > direction_x {
        direction_x / direction_y
    } else {
        direction_y / direction_x
    };
    // Vec2.length() follows Mth.sqrt(1.0F + square(tangent)) with the
    // reference's f32 addition and f64 sqrt narrowing.
    let distance_to_unit_square = Vec2::new(1.0_f32, tangent).length();
    let modified_length = java_math::min_f32(length * distance_to_unit_square, 1.0_f32);
    direction.scale(modified_length)
}

pub(super) fn java_max_f64(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else if left >= right {
        left
    } else {
        right
    }
}

pub(super) fn calculate_impulse(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}
