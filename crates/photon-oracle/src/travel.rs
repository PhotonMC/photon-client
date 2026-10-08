use super::*;

pub(super) fn f64_vec3(value: &Value) -> Result<Vec3, String> {
    Ok(Vec3::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
        f64_field(value, "z")?,
    ))
}

pub(super) fn local_player_travel_dispatch_parameters(
    input: &Value,
) -> Result<LocalPlayerTravelDispatchParameters, String> {
    let levitation_amplifier = i32_field(input, "levitation_amplifier")?;
    let levitation_amplifier = (levitation_amplifier >= 0).then_some(levitation_amplifier);
    Ok(LocalPlayerTravelDispatchParameters {
        input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
        y_rot: f32_field(input, "y_rot")?,
        x_rot: f32_field(input, "x_rot")?,
        look_angle: f64_vec3(input.get("look").ok_or("look must be a vector")?)?,
        in_water: input["in_water"]
            .as_bool()
            .ok_or("in_water must be a boolean")?,
        in_lava: input["in_lava"]
            .as_bool()
            .ok_or("in_lava must be a boolean")?,
        affected_by_fluids: input["affected_by_fluids"]
            .as_bool()
            .ok_or("affected_by_fluids must be a boolean")?,
        can_stand_on_fluid: input["can_stand_on_fluid"]
            .as_bool()
            .ok_or("can_stand_on_fluid must be a boolean")?,
        fall_flying: input["fall_flying"]
            .as_bool()
            .ok_or("fall_flying must be a boolean")?,
        on_climbable: input["on_climbable"]
            .as_bool()
            .ok_or("on_climbable must be a boolean")?,
        sprinting: input["sprinting"]
            .as_bool()
            .ok_or("sprinting must be a boolean")?,
        water_slow_down: f32_field(input, "water_slow_down")?,
        water_efficiency: f32_field(input, "water_efficiency")?,
        on_ground: input["on_ground"]
            .as_bool()
            .ok_or("on_ground must be a boolean")?,
        block_friction: f32_field(input, "block_friction")?,
        speed: f32_field(input, "speed")?,
        dolphins_grace: input["dolphins_grace"]
            .as_bool()
            .ok_or("dolphins_grace must be a boolean")?,
        horizontal_collision: input["horizontal_collision"]
            .as_bool()
            .ok_or("horizontal_collision must be a boolean")?,
        free_after_jump: input["free_after_jump"]
            .as_bool()
            .ok_or("free_after_jump must be a boolean")?,
        shallow_lava: input["shallow_lava"]
            .as_bool()
            .ok_or("shallow_lava must be a boolean")?,
        flying_speed: f32_field(input, "flying_speed")?,
        flying: input["flying"]
            .as_bool()
            .ok_or("flying must be a boolean")?,
        passenger: input["passenger"]
            .as_bool()
            .ok_or("passenger must be a boolean")?,
        scaffolding: input["scaffolding"]
            .as_bool()
            .ok_or("scaffolding must be a boolean")?,
        suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
            .as_bool()
            .ok_or("suppress_sliding_down_ladder must be a boolean")?,
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
        min_y: i32_field(input, "min_y")?,
        should_discard_friction: input["should_discard_friction"]
            .as_bool()
            .ok_or("should_discard_friction must be a boolean")?,
        friction_modifier: f32_field(input, "friction_modifier")?,
        air_drag_modifier: f32_field(input, "air_drag_modifier")?,
        omnidirectional_air_mover: input["omnidirectional_air_mover"]
            .as_bool()
            .ok_or("omnidirectional_air_mover must be a boolean")?,
    })
}

pub(super) fn clip_snapshot(value: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", value.x.to_bits()),
        "y": format!("{:016x}", value.y.to_bits()),
        "z": format!("{:016x}", value.z.to_bits()),
    })
}

pub(super) fn aabb_result(request: &Value) -> Result<Value, String> {
    let mut value = aabb_box(request)?;
    let mut events = vec![aabb_snapshot(value)];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        match string(step, "op")? {
            "contains" => events.push(json!({
                "hit": value.contains(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                )
            })),
            "contains_vec3" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                events.push(json!({"hit": value.contains_vec3(position)}));
            }
            "intersects_coords" => {
                events.push(json!({
                    "hit": value.intersects_coords(
                        f64_field(step, "min_x")?,
                        f64_field(step, "min_y")?,
                        f64_field(step, "min_z")?,
                        f64_field(step, "max_x")?,
                        f64_field(step, "max_y")?,
                        f64_field(step, "max_z")?,
                    )
                }));
            }
            "intersects_vec3" => {
                let min = f64_vec3(step.get("min").ok_or("min must be an object")?)?;
                let max = f64_vec3(step.get("max").ok_or("max must be an object")?)?;
                events.push(json!({"hit": value.intersects_vec3(min, max)}));
            }
            "intersects_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                events.push(json!({"hit": value.intersects_block_pos(position)}));
            }
            "collided_along_vector" => {
                let vector = f64_vec3(step.get("vector").ok_or("vector must be an object")?)?;
                let entries = step
                    .get("boxes")
                    .ok_or("boxes must be an array")?
                    .as_array()
                    .ok_or("boxes must be an array")?;
                let boxes = entries
                    .iter()
                    .map(aabb_box)
                    .collect::<Result<Vec<_>, _>>()?;
                events.push(json!({
                    "hit": value.collided_along_vector(vector, &boxes),
                }));
            }
            "from_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                value = Aabb::from_block_pos(position);
                events.push(aabb_snapshot(value));
            }
            "from_vec3" => {
                let begin = f64_vec3(step.get("begin").ok_or("begin must be an object")?)?;
                let end = f64_vec3(step.get("end").ok_or("end must be an object")?)?;
                value = Aabb::from_vec3(begin, end);
                events.push(aabb_snapshot(value));
            }
            "unit_cube_from_lower_corner" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                value = Aabb::unit_cube_from_lower_corner(position);
                events.push(aabb_snapshot(value));
            }
            "encapsulating_full_blocks" => {
                let first = blockpos(step.get("first").ok_or("first must be an object")?)?;
                let second = blockpos(step.get("second").ok_or("second must be an object")?)?;
                value = Aabb::encapsulating_full_blocks(first, second);
                events.push(aabb_snapshot(value));
            }
            "of_bounding_box" => {
                let bounds = bounding_box(step.get("box").ok_or("box must be an object")?)?;
                value = Aabb::of_bounding_box(bounds);
                events.push(aabb_snapshot(value));
            }
            "expand_vec3" => {
                value = value.expand_towards_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?);
                events.push(aabb_snapshot(value));
            }
            "equals" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({"equal": value.equals_java(other)}));
            }
            "hash_code" => events.push(json!({"hash_code": value.hash_code_java()})),
            "to_string" => events.push(json!({"value": value.to_string_java()})),
            "intersects" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({"hit": value.intersects(other)}));
            }
            "intersect" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                value = value.intersect(other);
                events.push(aabb_snapshot(value));
            }
            "minmax" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                value = value.minmax(other);
                events.push(aabb_snapshot(value));
            }
            "move" => {
                value = value.move_by(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "move_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                value = value.move_by_block_pos(position);
                events.push(aabb_snapshot(value));
            }
            "move_vec3" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                value = value.move_by_vec3(position);
                events.push(aabb_snapshot(value));
            }
            "move_vector3f" => {
                let position = step.get("position").ok_or("position must be an object")?;
                value = value.move_by_vector3f(
                    f32_field(position, "x")?,
                    f32_field(position, "y")?,
                    f32_field(position, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "sizes" => events.push(aabb_sizes_snapshot(value)),
            "has_nan" => events.push(json!({"has_nan": value.has_nan()})),
            "positions" => events.push(aabb_positions_snapshot(value)),
            "min" | "max" => {
                let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                let endpoint = if string(step, "op")? == "min" {
                    value.min(axis)
                } else {
                    value.max(axis)
                };
                events.push(json!({
                    "value": format!("{:016x}", endpoint.to_bits()),
                }));
            }
            "distance_point" => {
                let point = f64_vec3(step.get("point").ok_or("point must be an object")?)?;
                events.push(json!({
                    "value": format!("{:016x}", value.distance_to_sqr_point(point).to_bits()),
                }));
            }
            "distance_box" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({
                    "value": format!("{:016x}", value.distance_to_sqr_box(other).to_bits()),
                }));
            }
            "of_size" => {
                let center = f64_vec3(step.get("center").ok_or("center must be an object")?)?;
                value = Aabb::of_size(
                    center,
                    f64_field(step, "size_x")?,
                    f64_field(step, "size_y")?,
                    f64_field(step, "size_z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "set_min_x" => {
                value = value.set_min_x(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_min_y" => {
                value = value.set_min_y(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_min_z" => {
                value = value.set_min_z(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_x" => {
                value = value.set_max_x(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_y" => {
                value = value.set_max_y(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_z" => {
                value = value.set_max_z(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "clip" => {
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                events.push(match value.clip(from, to) {
                    Some(point) => json!({"hit": true, "point": clip_snapshot(point)}),
                    None => json!({"hit": false}),
                });
            }
            "clip_coords" => {
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                events.push(
                    match Aabb::clip_coords(
                        Vec3::new(
                            f64_field(step, "min_x")?,
                            f64_field(step, "min_y")?,
                            f64_field(step, "min_z")?,
                        ),
                        Vec3::new(
                            f64_field(step, "max_x")?,
                            f64_field(step, "max_y")?,
                            f64_field(step, "max_z")?,
                        ),
                        from,
                        to,
                    ) {
                        Some(point) => json!({"hit": true, "point": clip_snapshot(point)}),
                        None => json!({"hit": false}),
                    },
                );
            }
            "clip_many" => {
                let entries = step
                    .get("boxes")
                    .ok_or("boxes must be an array")?
                    .as_array()
                    .ok_or("boxes must be an array")?;
                let boxes = entries
                    .iter()
                    .map(aabb_box)
                    .collect::<Result<Vec<_>, _>>()?;
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(step.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(Aabb::clip_many(
                    &boxes, from, to, pos,
                )));
            }
            "expand" => {
                value = value.expand_towards(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "contract" => {
                value = value.contract(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "inflate" => {
                value = value.inflate_all(f64_field(step, "amount")?);
                events.push(aabb_snapshot(value));
            }
            "inflate_xyz" => {
                value = value.inflate(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "deflate" => {
                value = value.deflate(f64_field(step, "amount")?);
                events.push(aabb_snapshot(value));
            }
            "deflate_xyz" => {
                value = value.deflate_xyz(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            _ => return Err("unknown AABB operation".into()),
        }
    }
    Ok(json!({"events": events}))
}
