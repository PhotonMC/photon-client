//! The local player's travel in flying, water and lava.

use super::*;

pub(crate) fn local_player_travel_flying_case(request: &Value) -> Result<Value, String> {
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

pub(crate) fn local_player_travel_in_water_case(request: &Value) -> Result<Value, String> {
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

pub(crate) fn local_player_travel_in_lava_case(request: &Value) -> Result<Value, String> {
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
