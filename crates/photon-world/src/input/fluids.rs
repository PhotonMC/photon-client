use super::*;

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

/// The state consumed by LivingEntity's fluid falling adjustment.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerFluidFallingInput {
    pub base_gravity: f64,
    pub is_falling: bool,
    pub sprinting: bool,
    pub movement: Vec3,
}

/// The environment state consumed by LivingEntity.travelInWater.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInWaterParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub on_climbable: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInWater.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInWaterInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub on_climbable: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment state consumed by LivingEntity.travelInLava.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInLavaParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub shallow_lava: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInLava.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInLavaInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub shallow_lava: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}
