use super::*;

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

/// The environment state consumed by LivingEntity.travelFlying.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelFlyingParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub in_water: bool,
    pub in_lava: bool,
    pub water_speed: f32,
    pub lava_speed: f32,
    pub air_speed: f32,
}

/// The environment and motion state consumed by LivingEntity.travelFlying.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelFlyingInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub in_water: bool,
    pub in_lava: bool,
    pub water_speed: f32,
    pub lava_speed: f32,
    pub air_speed: f32,
}

/// The environment state consumed by LivingEntity.travelInAir.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInAirParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub friction_modifier: f32,
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
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInAir.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInAirInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub friction_modifier: f32,
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
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment state consumed by LivingEntity.updateFallFlyingMovement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerUpdateFallFlyingMovementParameters {
    pub look_angle: Vec3,
    pub x_rot: f32,
    pub gravity: f64,
}

/// The environment and motion state consumed by
/// LivingEntity.updateFallFlyingMovement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerUpdateFallFlyingMovementInput {
    pub look_angle: Vec3,
    pub delta_movement: Vec3,
    pub x_rot: f32,
    pub gravity: f64,
}

/// The four top-level branches selected by LivingEntity.travel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPlayerTravelMode {
    Water,
    Lava,
    FallFlying,
    Air,
}

/// The environment state consumed by LivingEntity.travel dispatch.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub look_angle: Vec3,
    pub in_water: bool,
    pub in_lava: bool,
    pub affected_by_fluids: bool,
    pub can_stand_on_fluid: bool,
    pub fall_flying: bool,
    pub on_climbable: bool,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub shallow_lava: bool,
    pub flying_speed: f32,
    pub flying: bool,
    pub passenger: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub jumping: bool,
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub friction_modifier: f32,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment and current motion consumed by LivingEntity.travel dispatch.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchInput {
    pub parameters: LocalPlayerTravelDispatchParameters,
    pub delta_movement: Vec3,
}

/// The selected travel branch and resulting no-collision movement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchResult {
    pub mode: LocalPlayerTravelMode,
    pub fall_flying_climbable_fallback: bool,
    pub movement: Vec3,
}

impl LocalPlayerTravelMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Water => "water",
            Self::Lava => "lava",
            Self::FallFlying => "fall_flying",
            Self::Air => "air",
        }
    }
}
