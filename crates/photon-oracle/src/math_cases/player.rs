use super::*;

pub(super) fn local_player_input_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let label = string(input, "label")?;
            let original = Vec2::new(f32_field(input, "x")?, f32_field(input, "y")?);
            let modifiers = LocalPlayerInputModifiers {
                using_item: input["using_item"]
                    .as_bool()
                    .ok_or("using_item must be a boolean")?,
                passenger: input["passenger"]
                    .as_bool()
                    .ok_or("passenger must be a boolean")?,
                item_use_speed_multiplier: f32_field(input, "item_use_speed_multiplier")?,
                moving_slowly: input["moving_slowly"]
                    .as_bool()
                    .ok_or("moving_slowly must be a boolean")?,
                sneaking_movement_factor: f32_field(input, "sneaking_movement_factor")?,
            };
            let modified = modify_local_player_input(original, modifiers);
            Ok(json!({
                "label": label,
                "input": vec2_snapshot(original),
                "modified": vec2_snapshot(modified),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_apply_input_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let label = string(input, "label")?;
            let applied = apply_local_player_input(
                LocalPlayerInputState {
                    move_vector: Vec2::new(
                        f32_field(input, "move_x")?,
                        f32_field(input, "move_y")?,
                    ),
                    jump_pressed: input["jump"].as_bool().ok_or("jump must be a boolean")?,
                    current_impulses: Vec2::new(f32_field(input, "xxa")?, f32_field(input, "zza")?),
                    current_jumping: input["jumping"]
                        .as_bool()
                        .ok_or("jumping must be a boolean")?,
                    current_bob: Vec2::new(f32_field(input, "x_bob")?, f32_field(input, "y_bob")?),
                    previous_bob: Vec2::new(
                        f32_field(input, "x_bob_o")?,
                        f32_field(input, "y_bob_o")?,
                    ),
                    rotation: Vec2::new(f32_field(input, "x_rot")?, f32_field(input, "y_rot")?),
                    controlled_camera: input["controlled_camera"]
                        .as_bool()
                        .ok_or("controlled_camera must be a boolean")?,
                },
                LocalPlayerInputModifiers {
                    using_item: false,
                    passenger: false,
                    item_use_speed_multiplier: 0.2,
                    moving_slowly: false,
                    sneaking_movement_factor: 0.3,
                },
            );
            Ok(json!({
                "label": label,
                "movement": vec2_snapshot(applied.movement),
                "jumping": applied.jumping,
                "x_bob": format!("{:08x}", applied.x_bob.to_bits()),
                "y_bob": format!("{:08x}", applied.y_bob.to_bits()),
                "x_bob_o": format!("{:08x}", applied.x_bob_old.to_bits()),
                "y_bob_o": format!("{:08x}", applied.y_bob_old.to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_can_start_sprinting_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let state = LocalPlayerSprintGateInput {
                sprinting: input["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
                forward_impulse: f32_field(input, "move_y")?,
                mobility_restricted: input["mobility_restricted"]
                    .as_bool()
                    .ok_or("mobility_restricted must be a boolean")?,
                enough_food: input["enough_food"]
                    .as_bool()
                    .ok_or("enough_food must be a boolean")?,
                may_fly: input["mayfly"]
                    .as_bool()
                    .ok_or("mayfly must be a boolean")?,
                shallow_water: input["shallow_water"]
                    .as_bool()
                    .ok_or("shallow_water must be a boolean")?,
                flying: input["flying"]
                    .as_bool()
                    .ok_or("flying must be a boolean")?,
                slow_due_to_using_item: input["using_item"]
                    .as_bool()
                    .ok_or("using_item must be a boolean")?,
                fall_flying: input["fall_flying"]
                    .as_bool()
                    .ok_or("fall_flying must be a boolean")?,
                underwater: input["underwater"]
                    .as_bool()
                    .ok_or("underwater must be a boolean")?,
                moving_slowly: input["moving_slowly"]
                    .as_bool()
                    .ok_or("moving_slowly must be a boolean")?,
            };
            Ok(json!({
                "label": string(input, "label")?,
                "can_start": can_start_local_player_sprinting(state),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_fluid_falling_adjustment_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement =
                adjust_local_player_fluid_falling_movement(LocalPlayerFluidFallingInput {
                    base_gravity: f64_field(input, "base_gravity")?,
                    is_falling: input["is_falling"]
                        .as_bool()
                        .ok_or("is_falling must be a boolean")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    movement: Vec3::new(
                        f64_field(input, "x")?,
                        f64_field(input, "y")?,
                        f64_field(input, "z")?,
                    ),
                });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_climbable_movement_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states =
        inputs
            .iter()
            .map(|input| {
                let movement =
                    clamp_local_player_climbable_movement(LocalPlayerClimbableMovementInput {
                        movement: Vec3::new(
                            f64_field(input, "x")?,
                            f64_field(input, "y")?,
                            f64_field(input, "z")?,
                        ),
                        on_climbable: input["on_climbable"]
                            .as_bool()
                            .ok_or("on_climbable must be a boolean")?,
                        scaffolding: input["scaffolding"]
                            .as_bool()
                            .ok_or("scaffolding must be a boolean")?,
                        suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                            .as_bool()
                            .ok_or("suppress_sliding_down_ladder must be a boolean")?,
                    });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_friction_influenced_speed_case(
    request: &Value,
) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let speed =
                local_player_friction_influenced_speed(LocalPlayerFrictionInfluencedSpeedInput {
                    on_ground: input["on_ground"]
                        .as_bool()
                        .ok_or("on_ground must be a boolean")?,
                    block_friction: f32_field(input, "block_friction")?,
                    speed: f32_field(input, "speed")?,
                    flying: input["flying"]
                        .as_bool()
                        .ok_or("flying must be a boolean")?,
                    flying_speed: f32_field(input, "flying_speed")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    passenger: input["passenger"]
                        .as_bool()
                        .ok_or("passenger must be a boolean")?,
                });
            Ok(json!({
                "label": string(input, "label")?,
                "speed": format!("{:08x}", speed.to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_modified_friction_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let friction = local_player_modified_friction(LocalPlayerModifiedFrictionInput {
                friction: f32_field(input, "friction")?,
                modifier: f32_field(input, "modifier")?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "friction": format!("{:08x}", friction.to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_travel_flying_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement = local_player_travel_flying(LocalPlayerTravelFlyingInput {
                input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                y_rot: f32_field(input, "y_rot")?,
                in_water: input["in_water"]
                    .as_bool()
                    .ok_or("in_water must be a boolean")?,
                in_lava: input["in_lava"]
                    .as_bool()
                    .ok_or("in_lava must be a boolean")?,
                water_speed: f32_field(input, "water_speed")?,
                lava_speed: f32_field(input, "lava_speed")?,
                air_speed: f32_field(input, "air_speed")?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_travel_in_water_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement = local_player_travel_in_water(LocalPlayerTravelInWaterInput {
                input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                y_rot: f32_field(input, "y_rot")?,
                sprinting: input["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
                water_slow_down: f32_field(input, "water_slow_down")?,
                water_efficiency: f32_field(input, "water_efficiency")?,
                on_ground: input["on_ground"]
                    .as_bool()
                    .ok_or("on_ground must be a boolean")?,
                speed: f32_field(input, "speed")?,
                dolphins_grace: input["dolphins_grace"]
                    .as_bool()
                    .ok_or("dolphins_grace must be a boolean")?,
                horizontal_collision: input["horizontal_collision"]
                    .as_bool()
                    .ok_or("horizontal_collision must be a boolean")?,
                on_climbable: input["on_climbable"]
                    .as_bool()
                    .ok_or("on_climbable must be a boolean")?,
                free_after_jump: input["free_after_jump"]
                    .as_bool()
                    .ok_or("free_after_jump must be a boolean")?,
                base_gravity: f64_field(input, "base_gravity")?,
                is_falling: input["is_falling"]
                    .as_bool()
                    .ok_or("is_falling must be a boolean")?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_travel_in_lava_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement = local_player_travel_in_lava(LocalPlayerTravelInLavaInput {
                input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                y_rot: f32_field(input, "y_rot")?,
                sprinting: input["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
                shallow_lava: input["shallow_lava"]
                    .as_bool()
                    .ok_or("shallow_lava must be a boolean")?,
                horizontal_collision: input["horizontal_collision"]
                    .as_bool()
                    .ok_or("horizontal_collision must be a boolean")?,
                free_after_jump: input["free_after_jump"]
                    .as_bool()
                    .ok_or("free_after_jump must be a boolean")?,
                base_gravity: f64_field(input, "base_gravity")?,
                is_falling: input["is_falling"]
                    .as_bool()
                    .ok_or("is_falling must be a boolean")?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_handle_relative_friction_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement =
                local_player_handle_relative_friction(LocalPlayerHandleRelativeFrictionInput {
                    input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    y_rot: f32_field(input, "y_rot")?,
                    on_ground: input["on_ground"]
                        .as_bool()
                        .ok_or("on_ground must be a boolean")?,
                    block_friction: f32_field(input, "block_friction")?,
                    speed: f32_field(input, "speed")?,
                    flying_speed: f32_field(input, "flying_speed")?,
                    flying: input["flying"]
                        .as_bool()
                        .ok_or("flying must be a boolean")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    passenger: input["passenger"]
                        .as_bool()
                        .ok_or("passenger must be a boolean")?,
                    on_climbable: input["on_climbable"]
                        .as_bool()
                        .ok_or("on_climbable must be a boolean")?,
                    scaffolding: input["scaffolding"]
                        .as_bool()
                        .ok_or("scaffolding must be a boolean")?,
                    suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                        .as_bool()
                        .ok_or(
                        "suppress_sliding_down_ladder must be a boolean",
                    )?,
                    horizontal_collision: input["horizontal_collision"]
                        .as_bool()
                        .ok_or("horizontal_collision must be a boolean")?,
                    jumping: input["jumping"]
                        .as_bool()
                        .ok_or("jumping must be a boolean")?,
                });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_travel_in_air_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let levitation_amplifier = input["levitation_amplifier"]
                .as_i64()
                .ok_or("levitation_amplifier must be an integer")?;
            let levitation_amplifier = if levitation_amplifier >= 0 {
                Some(
                    i32::try_from(levitation_amplifier)
                        .map_err(|_| "levitation_amplifier is out of range".to_owned())?,
                )
            } else {
                None
            };
            let min_y = i32::try_from(input["min_y"].as_i64().ok_or("min_y must be an integer")?)
                .map_err(|_| "min_y is out of range".to_owned())?;
            let movement = local_player_travel_in_air(LocalPlayerTravelInAirInput {
                input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                y_rot: f32_field(input, "y_rot")?,
                on_ground: input["on_ground"]
                    .as_bool()
                    .ok_or("on_ground must be a boolean")?,
                block_friction: f32_field(input, "block_friction")?,
                friction_modifier: f32_field(input, "friction_modifier")?,
                speed: f32_field(input, "speed")?,
                flying_speed: f32_field(input, "flying_speed")?,
                flying: input["flying"]
                    .as_bool()
                    .ok_or("flying must be a boolean")?,
                sprinting: input["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
                passenger: input["passenger"]
                    .as_bool()
                    .ok_or("passenger must be a boolean")?,
                on_climbable: input["on_climbable"]
                    .as_bool()
                    .ok_or("on_climbable must be a boolean")?,
                scaffolding: input["scaffolding"]
                    .as_bool()
                    .ok_or("scaffolding must be a boolean")?,
                suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                    .as_bool()
                    .ok_or("suppress_sliding_down_ladder must be a boolean")?,
                horizontal_collision: input["horizontal_collision"]
                    .as_bool()
                    .ok_or("horizontal_collision must be a boolean")?,
                jumping: input["jumping"]
                    .as_bool()
                    .ok_or("jumping must be a boolean")?,
                levitation_amplifier,
                gravity: f64_field(input, "gravity")?,
                client_side: input["client_side"]
                    .as_bool()
                    .ok_or("client_side must be a boolean")?,
                has_chunk_at: input["has_chunk_at"]
                    .as_bool()
                    .ok_or("has_chunk_at must be a boolean")?,
                current_y: f64_field(input, "current_y")?,
                min_y,
                should_discard_friction: input["should_discard_friction"]
                    .as_bool()
                    .ok_or("should_discard_friction must be a boolean")?,
                air_drag_modifier: f32_field(input, "air_drag_modifier")?,
                omnidirectional_air_mover: input["omnidirectional_air_mover"]
                    .as_bool()
                    .ok_or("omnidirectional_air_mover must be a boolean")?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_update_fall_flying_movement_case(
    request: &Value,
) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let movement = local_player_update_fall_flying_movement(
                LocalPlayerUpdateFallFlyingMovementInput {
                    look_angle: f64_vec3(input.get("look").ok_or("look must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    x_rot: f32_field(input, "x_rot")?,
                    gravity: f64_field(input, "gravity")?,
                },
            );
            Ok(json!({
                "label": string(input, "label")?,
                "movement": vec3_snapshot(movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn local_player_travel_dispatch_case(request: &Value) -> Result<Value, String> {
    let inputs = request["inputs"]
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|input| {
            let result = local_player_travel_dispatch(LocalPlayerTravelDispatchInput {
                parameters: local_player_travel_dispatch_parameters(input)?,
                delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
            });
            Ok(json!({
                "label": string(input, "label")?,
                "mode": result.mode.as_str(),
                "fall_flying_climbable_fallback": result.fall_flying_climbable_fallback,
                "movement": vec3_snapshot(result.movement),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}
