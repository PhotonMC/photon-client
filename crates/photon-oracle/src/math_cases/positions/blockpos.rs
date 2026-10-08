use super::*;

pub(crate) fn blockpos_packed_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown direction name")?;
    let coordinate = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(blockpos_packed_helpers_snapshot(
        value,
        u64_hex_field(request, "neighbor_node")?,
        direction,
        coordinate("step_x")?,
        coordinate("step_y")?,
        coordinate("step_z")?,
    ))
}

pub(crate) fn blockpos_relative_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown direction name")?;
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis name")?;
    let steps = string(request, "steps")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(blockpos_relative_helpers_snapshot(
        value, direction, axis, steps,
    ))
}

pub(crate) fn blockpos_vector_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let delta = BlockPos::new(
        i32_string_field(request, "dx")?,
        i32_string_field(request, "dy")?,
        i32_string_field(request, "dz")?,
    );
    let scale = i32_string_field(request, "scale")?;
    Ok(blockpos_vector_helpers_snapshot(value, delta, scale))
}

pub(crate) fn blockpos_geometry_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let rotation = BlockRotation::from_name(string(request, "rotation")?)
        .ok_or("unknown BlockPos rotation")?;
    let up_vector = BlockPos::new(
        i32_string_field(request, "up_x")?,
        i32_string_field(request, "up_y")?,
        i32_string_field(request, "up_z")?,
    );
    let y = i32_string_field(request, "at_y")?;
    Ok(blockpos_geometry_helpers_snapshot(
        value, rotation, up_vector, y,
    ))
}

pub(crate) fn blockpos_distance_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let other = blockpos(request.get("other").ok_or("other must be an object")?)?;
    let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
    let distance = f64_field(request, "distance")?;
    Ok(blockpos_distance_helpers_snapshot(
        value, other, point, distance,
    ))
}

pub(crate) fn blockpos_integer_helpers_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let other = blockpos(request.get("other").ok_or("other must be an object")?)?;
    Ok(blockpos_integer_helpers_snapshot(value, other))
}

pub(crate) fn blockpos_clamp_location_case(request: &Value) -> Result<Value, String> {
    let value = blockpos(request)?;
    let location = f64_vec3(
        request
            .get("location")
            .ok_or("location must be an object")?,
    )?;
    Ok(blockpos_clamp_location_snapshot(value, location))
}

pub(crate) fn blockpos_between_closed_case(request: &Value) -> Result<Value, String> {
    let from = blockpos(request.get("from").ok_or("from must be an object")?)?;
    let to = blockpos(request.get("to").ok_or("to must be an object")?)?;
    Ok(json!({
        "from_to": blockpos_position_list(BlockPos::between_closed(from, to)),
        "coordinates": blockpos_position_list(BlockPos::between_closed_coordinates(
            i32_string_field(request, "min_x")?,
            i32_string_field(request, "min_y")?,
            i32_string_field(request, "min_z")?,
            i32_string_field(request, "max_x")?,
            i32_string_field(request, "max_y")?,
            i32_string_field(request, "max_z")?,
        )),
    }))
}

pub(crate) fn blockpos_between_closed_aabb_case(request: &Value) -> Result<Value, String> {
    let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
    Ok(blockpos_positions_snapshot(BlockPos::between_closed_aabb(
        aabb,
    )))
}

pub(crate) fn blockpos_between_closed_streams_case(request: &Value) -> Result<Value, String> {
    let from = blockpos(request.get("from").ok_or("from must be an object")?)?;
    let to = blockpos(request.get("to").ok_or("to must be an object")?)?;
    let bounds = bounding_box(
        request
            .get("bounding_box")
            .ok_or("bounding_box must be an object")?,
    )?;
    let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
    let min_x = i32_string_field(request, "min_x")?;
    let min_y = i32_string_field(request, "min_y")?;
    let min_z = i32_string_field(request, "min_z")?;
    let max_x = i32_string_field(request, "max_x")?;
    let max_y = i32_string_field(request, "max_y")?;
    let max_z = i32_string_field(request, "max_z")?;
    Ok(json!({
        "block": blockpos_position_list(BlockPos::between_closed_stream(from, to)),
        "bounding_box": blockpos_position_list(
            BlockPos::between_closed_stream_bounding_box(bounds),
        ),
        "aabb": blockpos_position_list(BlockPos::between_closed_stream_aabb(aabb)),
        "coordinates": blockpos_position_list(
            BlockPos::between_closed_stream_coordinates(
                min_x, min_y, min_z, max_x, max_y, max_z,
            ),
        ),
    }))
}

pub(crate) fn blockpos_coordinate_helpers_case(request: &Value) -> Result<Value, String> {
    let containing = BlockPos::containing(
        f64_field(request, "x")?,
        f64_field(request, "y")?,
        f64_field(request, "z")?,
    );
    let point = f64_vec3(request.get("point").ok_or("point must be an object")?);
    let point = BlockPos::containing_vec3(point?);
    let first = blockpos(request.get("first").ok_or("first must be an object")?)?;
    let second = blockpos(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({
        "containing": blockpos_snapshot(containing),
        "containing_position": blockpos_snapshot(point),
        "min": blockpos_snapshot(BlockPos::min(first, second)),
        "max": blockpos_snapshot(BlockPos::max(first, second)),
    }))
}

pub(crate) fn blockpos_between_corners_direction_case(request: &Value) -> Result<Value, String> {
    let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
    let first = blockpos(request.get("first").ok_or("first must be an object")?)?;
    let second = blockpos(request.get("second").ok_or("second must be an object")?)?;
    let direction = f64_vec3(
        request
            .get("direction")
            .ok_or("direction must be an object")?,
    )?;
    Ok(json!({
        "aabb": blockpos_position_list(
            BlockPos::between_corners_in_direction_aabb(aabb, direction),
        ),
        "block_pos": blockpos_position_list(
            BlockPos::between_corners_in_direction(first, second, direction),
        ),
        "coordinates": blockpos_position_list(
            BlockPos::between_corners_in_direction_coordinates(
                i32_string_field(request, "first_x")?,
                i32_string_field(request, "first_y")?,
                i32_string_field(request, "first_z")?,
                i32_string_field(request, "second_x")?,
                i32_string_field(request, "second_y")?,
                i32_string_field(request, "second_z")?,
                direction,
            ),
        ),
    }))
}

pub(crate) fn blockpos_breadth_first_traversal_case(request: &Value) -> Result<Value, String> {
    let start = blockpos(request.get("start").ok_or("start must be an object")?)?;
    let max_depth = i32_string_field(request, "max_depth")?;
    let max_count = i32_string_field(request, "max_count")?;
    let rules = request
        .get("nodes")
        .ok_or("nodes must be an array")?
        .as_array()
        .ok_or("nodes must be an array")?
        .iter()
        .map(|entry| {
            let position = blockpos(entry.get("position").ok_or("position must be an object")?)?;
            let status = match string(entry, "status")? {
                "accept" => binary::TraversalNodeStatus::Accept,
                "skip" => binary::TraversalNodeStatus::Skip,
                "stop" => binary::TraversalNodeStatus::Stop,
                _ => return Err("unknown traversal status".into()),
            };
            let neighbours = entry
                .get("neighbors")
                .ok_or("neighbors must be an array")?
                .as_array()
                .ok_or("neighbors must be an array")?
                .iter()
                .map(blockpos)
                .collect::<Result<Vec<_>, _>>()?;
            Ok((position, status, neighbours))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut processed = Vec::new();
    let mut provider_calls = Vec::new();
    let count = BlockPos::breadth_first_traversal(
        start,
        max_depth,
        max_count,
        |position, enqueue| {
            provider_calls.push(position);
            if let Some((_, _, neighbours)) = rules.iter().find(|(node, _, _)| *node == position) {
                for neighbour in neighbours {
                    enqueue(*neighbour);
                }
            }
        },
        |position| {
            processed.push(position);
            rules
                .iter()
                .find(|(node, _, _)| *node == position)
                .map_or(binary::TraversalNodeStatus::Accept, |(_, status, _)| {
                    *status
                })
        },
    );
    Ok(json!({
        "count": count,
        "processed": blockpos_position_list(processed),
        "provider_calls": blockpos_position_list(provider_calls),
    }))
}

pub(crate) fn blockpos_mutable_helpers_case(request: &Value) -> Result<Value, String> {
    let initial = blockpos(request.get("initial").ok_or("initial must be an object")?)?;
    let mut mutable = initial.mutable();
    let mut constructors = serde_json::Map::new();
    constructors.insert("immutable".into(), blockpos_snapshot(initial.immutable()));
    constructors.insert(
        "mutable".into(),
        blockpos_snapshot(initial.mutable().immutable()),
    );
    constructors.insert(
        "double".into(),
        blockpos_snapshot(
            MutableBlockPos::from_f64(
                f64_field(request, "double_x")?,
                f64_field(request, "double_y")?,
                f64_field(request, "double_z")?,
            )
            .immutable(),
        ),
    );
    let mut events = Vec::new();
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        let mut event = serde_json::Map::new();
        match string(step, "op")? {
            "set" => {
                mutable.set(
                    i32_string_field(step, "x")?,
                    i32_string_field(step, "y")?,
                    i32_string_field(step, "z")?,
                );
            }
            "set_double" => {
                mutable.set_f64(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
            }
            "set_block_pos" => {
                mutable.set_block_pos(blockpos(
                    step.get("value").ok_or("value must be an object")?,
                )?);
            }
            "set_packed" => {
                mutable.set_packed(u64_hex_field(step, "value")?);
            }
            "set_with_offset_direction" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                let direction =
                    Direction::from_name(string(step, "direction")?).ok_or("unknown direction")?;
                mutable.set_with_offset_direction(position, direction);
            }
            "set_with_offset_components" => {
                mutable.set_with_offset_components(
                    blockpos(step.get("position").ok_or("position must be an object")?)?,
                    i32_string_field(step, "x")?,
                    i32_string_field(step, "y")?,
                    i32_string_field(step, "z")?,
                );
            }
            "set_with_offset_block_pos" => {
                mutable.set_with_offset_block_pos(
                    blockpos(step.get("position").ok_or("position must be an object")?)?,
                    blockpos(step.get("offset").ok_or("offset must be an object")?)?,
                );
            }
            "move_direction" => {
                let direction =
                    Direction::from_name(string(step, "direction")?).ok_or("unknown direction")?;
                mutable.move_direction(direction, 1);
            }
            "move_direction_steps" => {
                let direction =
                    Direction::from_name(string(step, "direction")?).ok_or("unknown direction")?;
                mutable.move_direction(direction, i32_string_field(step, "steps")?);
            }
            "move_components" => {
                mutable.move_components(
                    i32_string_field(step, "x")?,
                    i32_string_field(step, "y")?,
                    i32_string_field(step, "z")?,
                );
            }
            "move_block_pos" => {
                mutable.move_block_pos(blockpos(
                    step.get("value").ok_or("value must be an object")?,
                )?);
            }
            "clamp" => {
                let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                mutable.clamp(
                    axis,
                    i32_string_field(step, "minimum")?,
                    i32_string_field(step, "maximum")?,
                );
            }
            "set_x" => {
                mutable.set_x(i32_string_field(step, "value")?);
            }
            "set_y" => {
                mutable.set_y(i32_string_field(step, "value")?);
            }
            "set_z" => {
                mutable.set_z(i32_string_field(step, "value")?);
            }
            "offset_return" => {
                event.insert(
                    "returned".into(),
                    blockpos_snapshot(mutable.offset(
                        i32_string_field(step, "x")?,
                        i32_string_field(step, "y")?,
                        i32_string_field(step, "z")?,
                    )),
                );
            }
            "multiply_return" => {
                event.insert(
                    "returned".into(),
                    blockpos_snapshot(mutable.multiply(i32_string_field(step, "scale")?)),
                );
            }
            "relative_return" => {
                let direction =
                    Direction::from_name(string(step, "direction")?).ok_or("unknown direction")?;
                event.insert(
                    "returned".into(),
                    blockpos_snapshot(
                        mutable.relative_direction(direction, i32_string_field(step, "steps")?),
                    ),
                );
            }
            "relative_axis_return" => {
                let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                event.insert(
                    "returned".into(),
                    blockpos_snapshot(
                        mutable.relative_axis(axis, i32_string_field(step, "steps")?),
                    ),
                );
            }
            "rotate_return" => {
                let rotation = BlockRotation::from_name(string(step, "rotation")?)
                    .ok_or("unknown rotation")?;
                event.insert(
                    "returned".into(),
                    blockpos_snapshot(mutable.rotate(rotation)),
                );
            }
            _ => return Err("unknown mutable BlockPos operation".into()),
        }
        event.insert("state".into(), blockpos_snapshot(mutable.immutable()));
        events.push(Value::Object(event));
    }
    Ok(json!({
        "constructors": Value::Object(constructors),
        "events": events,
    }))
}

pub(crate) fn blockpos_within_manhattan_case(request: &Value) -> Result<Value, String> {
    let origin = blockpos(request.get("origin").ok_or("origin must be an object")?)?;
    let reach_x = i32_string_field(request, "reach_x")?;
    let reach_y = i32_string_field(request, "reach_y")?;
    let reach_z = i32_string_field(request, "reach_z")?;
    Ok(json!({
        "positions": blockpos_position_list(BlockPos::within_manhattan(
            origin, reach_x, reach_y, reach_z,
        )),
        "stream_positions": blockpos_position_list(BlockPos::within_manhattan(
            origin, reach_x, reach_y, reach_z,
        )),
    }))
}

pub(crate) fn blockpos_spiral_around_case(request: &Value) -> Result<Value, String> {
    let center = blockpos(request.get("center").ok_or("center must be an object")?)?;
    let radius = i32_string_field(request, "radius")?;
    let first_direction = Direction::from_name(string(request, "first_direction")?)
        .ok_or("unknown first spiral direction")?;
    let second_direction = Direction::from_name(string(request, "second_direction")?)
        .ok_or("unknown second spiral direction")?;
    let positions = BlockPos::spiral_around(center, radius, first_direction, second_direction)
        .map_err(|_| "spiral directions must use different axes".to_owned())?;
    Ok(blockpos_positions_snapshot(positions))
}

pub(crate) fn blockpos_find_closest_match_case(request: &Value) -> Result<Value, String> {
    let start = blockpos(request.get("start").ok_or("start must be an object")?)?;
    let horizontal_radius = i32_string_field(request, "horizontal_radius")?;
    let vertical_radius = i32_string_field(request, "vertical_radius")?;
    let target = blockpos(request.get("target").ok_or("target must be an object")?)?;
    let value =
        BlockPos::find_closest_match(start, horizontal_radius, vertical_radius, |candidate| {
            candidate == target
        });
    let mut result = serde_json::Map::new();
    if let Some(value) = value {
        result.insert("value".into(), blockpos_snapshot(value));
    }
    Ok(Value::Object(result))
}

pub(crate) fn blockpos_neighbor_column_case(request: &Value) -> Result<Value, String> {
    Ok(blockpos_positions_snapshot(BlockPos::neighbor_column(
        i32_string_field(request, "start_x")?,
        i32_string_field(request, "start_y")?,
        i32_string_field(request, "start_z")?,
        i32_string_field(request, "end_y")?,
    )))
}

pub(crate) fn blockpos_square_out_south_east_case(request: &Value) -> Result<Value, String> {
    Ok(blockpos_positions_snapshot(
        BlockPos::new(
            i32_string_field(request, "x")?,
            i32_string_field(request, "y")?,
            i32_string_field(request, "z")?,
        )
        .square_out_south_east(),
    ))
}

pub(crate) fn blockpos_random_ranges_case(request: &Value) -> Result<Value, String> {
    let seed = random_seed(request)?;
    let limit = i32_string_field(request, "limit")?;
    let min_x = i32_string_field(request, "min_x")?;
    let min_y = i32_string_field(request, "min_y")?;
    let min_z = i32_string_field(request, "min_z")?;
    let max_x = i32_string_field(request, "max_x")?;
    let max_y = i32_string_field(request, "max_y")?;
    let max_z = i32_string_field(request, "max_z")?;
    let mut between_random = RandomSource::create(seed);
    let between = BlockPos::random_between_closed(
        &mut between_random,
        limit,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
    )
    .map_err(|_| "bound must be positive".to_owned())?;
    let after_between = between_random.next_int();
    let center = blockpos(request.get("center").ok_or("center must be an object")?)?;
    let size = i32_string_field(request, "size")?;
    let mut cube_random = RandomSource::create(seed);
    let cube = BlockPos::random_in_cube(&mut cube_random, limit, center, size)
        .map_err(|_| "bound must be positive".to_owned())?;
    let after_cube = cube_random.next_int();
    Ok(json!({
        "between": blockpos_position_list(between),
        "after_between": after_between.to_string(),
        "cube": blockpos_position_list(cube),
        "after_cube": after_cube.to_string(),
    }))
}
