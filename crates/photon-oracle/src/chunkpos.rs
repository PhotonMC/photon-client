use super::*;

pub(super) fn blockpos_integer_helpers_snapshot(value: BlockPos, other: BlockPos) -> Value {
    json!({
        "dist_manhattan": value.dist_manhattan(other),
        "dist_chessboard": value.dist_chessboard(other),
        "get": {
            "x": value.get(Axis::X),
            "y": value.get(Axis::Y),
            "z": value.get(Axis::Z),
        },
    })
}

pub(super) fn blockpos_clamp_location_snapshot(value: BlockPos, location: Vec3) -> Value {
    json!({
        "value": vec3_snapshot(value.clamp_location_within(location)),
    })
}

pub(super) fn chunkpos(value: &Value) -> Result<ChunkPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(ChunkPos::new(coordinate("x")?, coordinate("z")?))
}

pub(super) fn sectionpos(value: &Value) -> Result<SectionPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(SectionPos::new(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
    ))
}

pub(super) fn chunkpos_snapshot(value: ChunkPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "z": value.z.to_string(),
    })
}

pub(super) fn sectionpos_snapshot(value: SectionPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
        "packed": format!("{:x}", value.as_long()),
    })
}

pub(super) fn chunkpos_block_coordinates_snapshot(value: ChunkPos, offset: i32) -> Value {
    json!({
        "min_block_x": value.get_min_block_x().to_string(),
        "min_block_z": value.get_min_block_z().to_string(),
        "middle_block_x": value.get_middle_block_x().to_string(),
        "middle_block_z": value.get_middle_block_z().to_string(),
        "max_block_x": value.get_max_block_x().to_string(),
        "max_block_z": value.get_max_block_z().to_string(),
        "block_x": value.get_block_x(offset).to_string(),
        "block_z": value.get_block_z(offset).to_string(),
    })
}

pub(super) fn sectionpos_block_coordinates_snapshot(value: SectionPos) -> Value {
    json!({
        "min_block_x": value.min_block_x().to_string(),
        "min_block_y": value.min_block_y().to_string(),
        "min_block_z": value.min_block_z().to_string(),
        "max_block_x": value.max_block_x().to_string(),
        "max_block_y": value.max_block_y().to_string(),
        "max_block_z": value.max_block_z().to_string(),
    })
}

pub(super) fn sectionpos_spatial_snapshot(value: SectionPos) -> Value {
    json!({
        "origin": blockpos_snapshot(value.origin()),
        "center": blockpos_snapshot(value.center()),
        "chunk": chunkpos_snapshot(value.chunk()),
    })
}

pub(super) fn sectionpos_packed_transformations_snapshot(
    block_node: u64,
    section_node: u64,
    x: i32,
    z: i32,
) -> Value {
    json!({
        "block_to_section": format!("{:x}", SectionPos::block_to_section(block_node)),
        "zero_node": format!("{:x}", SectionPos::get_zero_node(section_node)),
        "zero_node_coordinates": format!("{:x}", SectionPos::get_zero_node_from_coordinates(x, z)),
        "section_to_chunk": format!("{:x}", SectionPos::section_to_chunk(section_node)),
    })
}

pub(super) fn sectionpos_as_long_snapshot(
    block_node: u64,
    (x, y, z): (i32, i32, i32),
    section: SectionPos,
) -> Value {
    json!({
        "from_block_pos": format!("{:x}", SectionPos::as_long_from_block_pos(BlockPos::unpack(block_node))),
        "coordinates": format!("{:x}", SectionPos::as_long_coordinates(x, y, z)),
        "instance": format!("{:x}", section.as_long()),
    })
}

pub(super) fn sectionpos_offset_snapshot(
    section_node: u64,
    direction: Direction,
    steps: (i32, i32, i32),
    initial: SectionPos,
) -> Value {
    let instance = initial.offset(steps.0, steps.1, steps.2);
    json!({
        "packed_direction": format!("{:x}", SectionPos::offset_packed(section_node, direction)),
        "packed_steps": format!("{:x}", SectionPos::offset_packed_by(section_node, steps.0, steps.1, steps.2)),
        "instance_x": instance.x.to_string(),
        "instance_y": instance.y.to_string(),
        "instance_z": instance.z.to_string(),
        "instance_packed": format!("{:x}", instance.as_long()),
    })
}

pub(super) fn sectionpos_ranges_snapshot(values: Vec<SectionPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(sectionpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

pub(super) fn blockpos_positions_snapshot(values: Vec<BlockPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(blockpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

pub(super) fn blockpos_position_list(values: Vec<BlockPos>) -> Value {
    Value::Array(values.into_iter().map(blockpos_snapshot).collect())
}

pub(super) fn sectionpos_nodes_snapshot(values: Vec<u64>) -> Value {
    json!({
        "section_nodes": values
            .into_iter()
            .map(|value| format!("{value:x}"))
            .collect::<Vec<_>>(),
    })
}

pub(super) fn chunkpos_region_coordinates_snapshot(
    value: ChunkPos,
    region_x: i32,
    region_z: i32,
) -> Value {
    json!({
        "region_x": value.get_region_x().to_string(),
        "region_z": value.get_region_z().to_string(),
        "packed_region_x": ChunkPos::get_region_x_from_packed(value.pack()).to_string(),
        "packed_region_z": ChunkPos::get_region_z_from_packed(value.pack()).to_string(),
        "region_local_x": value.get_region_local_x().to_string(),
        "region_local_z": value.get_region_local_z().to_string(),
        "min_from_region": chunkpos_snapshot(ChunkPos::min_from_region(region_x, region_z)),
        "max_from_region": chunkpos_snapshot(ChunkPos::max_from_region(region_x, region_z)),
    })
}

pub(super) fn chunkpos_block_positions_snapshot(
    value: ChunkPos,
    block_x: i32,
    y: i32,
    block_z: i32,
    contains: &[Value],
) -> Result<Value, String> {
    Ok(json!({
        "block_at": blockpos_snapshot(value.get_block_at(block_x, y, block_z)),
        "middle_block_position": blockpos_snapshot(value.get_middle_block_position(y)),
        "world_position": blockpos_snapshot(value.get_world_position()),
        "contains": contains
            .iter()
            .map(|query| blockpos(query).map(|pos| value.contains(pos)))
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

pub(super) fn chunkpos_representation_snapshot(value: ChunkPos, hash_x: i32, hash_z: i32) -> Value {
    json!({
        "to_string": value.to_string_java(),
        "hash_code": value.hash_code_java().to_string(),
        "static_hash": ChunkPos::hash(hash_x, hash_z).to_string(),
    })
}

pub(super) fn chunkpos_from_blockpos_snapshot(value: BlockPos) -> Value {
    let containing = ChunkPos::containing(value);
    json!({
        "containing": chunkpos_snapshot(containing),
        "packed": format!("{:x}", ChunkPos::pack_block_pos(value)),
    })
}

pub(super) fn chunkpos_distances_snapshot(value: ChunkPos, other: ChunkPos) -> Value {
    json!({
        "chessboard_chunk": value.get_chessboard_distance(other).to_string(),
        "chessboard_coordinates": value
            .get_chessboard_distance_coordinates(other.x, other.z)
            .to_string(),
        "distance_squared_chunk": value.distance_squared(other).to_string(),
        "distance_squared_packed": value
            .distance_squared_from_packed(other.pack())
            .to_string(),
    })
}

pub(super) fn chunkpos_validity_snapshot(value: ChunkPos) -> Value {
    json!({
        "instance": value.is_valid(),
        "static": ChunkPos::is_valid_coordinates(value.x, value.z),
    })
}

pub(super) fn chunkpos_range_closed_snapshot(values: Vec<ChunkPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(chunkpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

pub(super) fn blockpos_result(request: &Value) -> Result<Value, String> {
    let mut value = blockpos(request)?;
    let mut events = vec![blockpos_snapshot(value)];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        value = match string(step, "action")? {
            "offset" => value.offset(
                string(step, "x")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
                string(step, "y")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
                string(step, "z")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
            ),
            "min" => BlockPos::min(value, blockpos(step)?),
            "max" => BlockPos::max(value, blockpos(step)?),
            _ => return Err("unknown BlockPos action".into()),
        };
        events.push(blockpos_snapshot(value));
    }
    Ok(json!({"events": events}))
}

pub(super) fn section_relative_result(request: &Value) -> Result<Value, String> {
    let relative = SectionPos::section_relative_pos(blockpos(request)?);
    Ok(json!({
        "relative": relative.to_string(),
        "x": SectionPos::section_relative_x(relative).to_string(),
        "y": SectionPos::section_relative_y(relative).to_string(),
        "z": SectionPos::section_relative_z(relative).to_string(),
    }))
}

pub(super) fn random_result(request: &Value) -> Result<Value, String> {
    let seed = random_seed(request)?;
    let mut random = RandomSource::create(seed);
    let mut events = vec![json!({})];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        events.push(random_state(&mut random, step)?);
    }
    Ok(json!({"events": events}))
}

pub(super) fn random_seed(request: &Value) -> Result<i64, String> {
    string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

pub(super) fn random_state(random: &mut RandomSource, step: &Value) -> Result<Value, String> {
    let mut state = serde_json::Map::new();
    match string(step, "op")? {
        "nextInt" => {
            state.insert("value".into(), json!(random.next_int().to_string()));
        }
        "nextIntBound" => {
            let bound = step["bound"]
                .as_i64()
                .ok_or("bound must be an integer")?
                .try_into()
                .map_err(|_| "bound must fit in i32".to_owned())?;
            let value = random
                .next_int_bound(bound)
                .map_err(|_| "bound must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "nextLong" => {
            state.insert("value".into(), json!(random.next_long().to_string()));
        }
        "nextBoolean" => {
            state.insert("value".into(), json!(random.next_boolean().to_string()));
        }
        "nextFloat" => {
            state.insert(
                "bits".into(),
                json!(format!("{:08x}", random.next_float().to_bits())),
            );
        }
        "nextDouble" => {
            state.insert(
                "bits".into(),
                json!(format!("{:016x}", random.next_double().to_bits())),
            );
        }
        "nextGaussian" => {
            state.insert(
                "bits".into(),
                json!(format!("{:016x}", random.next_gaussian().to_bits())),
            );
        }
        "setSeed" => {
            let seed = string(step, "seed")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?;
            random.set_seed(seed);
            state.insert("op".into(), json!("setSeed"));
        }
        "nextIntBetweenInclusive" => {
            let value = random
                .next_int_between_inclusive(i32_field(step, "min")?, i32_field(step, "max")?)
                .map_err(|_| "bound must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "nextIntRange" => {
            let value = random
                .next_int_range(i32_field(step, "origin")?, i32_field(step, "bound")?)
                .map_err(|_| "bound - origin must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "triangleDouble" => {
            state.insert(
                "bits".into(),
                json!(format!(
                    "{:016x}",
                    random
                        .triangle_f64(f64_field(step, "mean")?, f64_field(step, "spread")?)
                        .to_bits()
                )),
            );
        }
        "triangleFloat" => {
            state.insert(
                "bits".into(),
                json!(format!(
                    "{:08x}",
                    random
                        .triangle_f32(f32_field(step, "mean")?, f32_field(step, "spread")?)
                        .to_bits()
                )),
            );
        }
        "consumeCount" => {
            random.consume_count(i32_field(step, "rounds")?);
            state.insert("op".into(), json!("consumeCount"));
        }
        _ => return Err("unknown random operation".into()),
    }
    Ok(Value::Object(state))
}

pub(super) fn random_steps(random: &mut RandomSource, value: &Value) -> Result<Value, String> {
    let mut events = Vec::new();
    for step in value.as_array().ok_or("random steps must be an array")? {
        events.push(random_state(random, step)?);
    }
    Ok(Value::Array(events))
}

pub(super) fn random_fork_result(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    let prefix_events = random_steps(
        &mut random,
        request
            .get("prefix_steps")
            .ok_or("prefix_steps must be an array")?,
    )?;
    let mut child = random.fork();
    let child_events = random_steps(
        &mut child,
        request
            .get("child_steps")
            .ok_or("child_steps must be an array")?,
    )?;
    let parent_events = random_steps(
        &mut random,
        request
            .get("parent_steps")
            .ok_or("parent_steps must be an array")?,
    )?;
    Ok(json!({
        "prefix_events": prefix_events,
        "child_events": child_events,
        "parent_events": parent_events,
    }))
}

pub(super) fn i32_field(value: &Value, name: &str) -> Result<i32, String> {
    value[name]
        .as_i64()
        .ok_or_else(|| format!("{name} must be an integer"))?
        .try_into()
        .map_err(|_| format!("{name} must fit in i32"))
}

pub(super) fn i64_field(value: &Value, name: &str) -> Result<i64, String> {
    value[name]
        .as_i64()
        .ok_or_else(|| format!("{name} must be an i64"))
}

pub(super) fn i32_string_field(value: &Value, name: &str) -> Result<i32, String> {
    string(value, name)?
        .parse::<i32>()
        .map_err(|error| format!("{name} must fit in i32: {error}"))
}
