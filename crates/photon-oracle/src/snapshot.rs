use super::*;

pub(super) fn random_positional_result(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    let factory = random.fork_positional();
    let mut factory_events = Vec::new();
    for step in request
        .get("factory_steps")
        .ok_or("factory_steps must be an array")?
        .as_array()
        .ok_or("factory_steps must be an array")?
    {
        let mut child = match string(step, "op")? {
            "at" => factory.at(
                i32_field(step, "x")?,
                i32_field(step, "y")?,
                i32_field(step, "z")?,
            ),
            "fromHashOf" => factory.from_hash_of(string(step, "name")?),
            "fromSeed" => factory.from_seed(
                string(step, "seed")?
                    .parse::<i64>()
                    .map_err(|error| error.to_string())?,
            ),
            _ => return Err("unknown positional random operation".into()),
        };
        let events = random_steps(
            &mut child,
            step.get("steps").ok_or("steps must be an array")?,
        )?;
        factory_events.push(json!({"op": string(step, "op")?, "events": events}));
    }
    let parent_events = random_steps(
        &mut random,
        request
            .get("parent_steps")
            .ok_or("parent_steps must be an array")?,
    )?;
    Ok(json!({
        "factory_events": factory_events,
        "parent_events": parent_events,
    }))
}

pub(super) fn aabb_box(value: &Value) -> Result<Aabb, String> {
    Ok(Aabb::new(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    ))
}

pub(super) fn aabb_snapshot(value: Aabb) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min_x.to_bits()),
        "min_y": format!("{:016x}", value.min_y.to_bits()),
        "min_z": format!("{:016x}", value.min_z.to_bits()),
        "max_x": format!("{:016x}", value.max_x.to_bits()),
        "max_y": format!("{:016x}", value.max_y.to_bits()),
        "max_z": format!("{:016x}", value.max_z.to_bits()),
    })
}

pub(super) fn aabb_builder_result(request: &Value) -> Result<Value, String> {
    let points = request
        .get("points")
        .ok_or("points must be an array")?
        .as_array()
        .ok_or("points must be an array")?;
    let mut builder = AabbBuilder::new();
    let mut events = vec![json!({"defined": builder.is_defined()})];
    for point in points {
        builder.include(
            f32_field(point, "x")?,
            f32_field(point, "y")?,
            f32_field(point, "z")?,
        );
        events.push(json!({"defined": builder.is_defined()}));
    }
    let build = match builder.build() {
        Ok(value) => json!({"outcome": "value", "box": aabb_snapshot(value)}),
        Err(AabbBuilderError::Undefined) => json!({
            "outcome": "error",
            "error": "undefined",
        }),
    };
    Ok(json!({"events": events, "build": build}))
}

pub(super) fn aabb_sizes_snapshot(value: Aabb) -> Value {
    json!({
        "x_size": format!("{:016x}", value.x_size().to_bits()),
        "y_size": format!("{:016x}", value.y_size().to_bits()),
        "z_size": format!("{:016x}", value.z_size().to_bits()),
        "size": format!("{:016x}", value.size().to_bits()),
    })
}

pub(super) fn aabb_positions_snapshot(value: Aabb) -> Value {
    json!({
        "center": vec3_snapshot(value.center()),
        "bottom_center": vec3_snapshot(value.bottom_center()),
        "min_position": vec3_snapshot(value.min_position()),
        "max_position": vec3_snapshot(value.max_position()),
    })
}

pub(super) fn block_hit_result_snapshot(value: Option<BlockHitResult>) -> Value {
    match value {
        Some(hit) => json!({
            "hit": true,
            "type": "BLOCK",
            "location": clip_snapshot(hit.location),
            "direction": hit.direction.name(),
            "block_pos": blockpos_snapshot(hit.block_pos),
            "inside": hit.inside,
            "world_border_hit": hit.world_border_hit,
        }),
        None => json!({"hit": false}),
    }
}

pub(super) fn shape_box(value: &Value) -> Result<VoxelShape, String> {
    Shapes::box_shape(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    )
    .map_err(|_| "shape minimum must not exceed maximum".to_owned())
}

pub(super) fn block_collision_fixture(
    request: &Value,
) -> Result<(ClientWorld, BTreeMap<BlockStateId, VoxelShape>), String> {
    let mut shapes_by_state = BTreeMap::new();
    for entry in request
        .get("shape_states")
        .ok_or("shape_states must be an array")?
        .as_array()
        .ok_or("shape_states must be an array")?
    {
        let state_id = BlockStateId(i32_field(entry, "state_id")?);
        let shape = shape_box(entry.get("shape").ok_or("shape must be an object")?)?;
        shapes_by_state.insert(state_id, shape);
    }

    Ok((block_collision_world_fixture(request)?, shapes_by_state))
}

pub(super) fn block_collision_world_fixture(request: &Value) -> Result<ClientWorld, String> {
    let mut chunk_blocks = BTreeMap::new();
    for entry in request
        .get("placements")
        .ok_or("placements must be an array")?
        .as_array()
        .ok_or("placements must be an array")?
    {
        let x = i32_field(entry, "x")?;
        let y = i32_field(entry, "y")?;
        let z = i32_field(entry, "z")?;
        if !(0..16).contains(&y) {
            return Err("block collision fixture Y must be in 0..16".into());
        }
        let state_id = BlockStateId(i32_field(entry, "state_id")?);
        let blocks = chunk_blocks
            .entry((x.div_euclid(16), z.div_euclid(16)))
            .or_insert_with(|| vec![0; 4_096]);
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y as usize;
        let local_z = z.rem_euclid(16) as usize;
        blocks[local_y * 256 + local_z * 16 + local_x] = state_id.0;
    }

    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    for ((chunk_x, chunk_z), blocks) in chunk_blocks {
        world.replace_chunk(
            chunk_x,
            chunk_z,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
    }

    Ok(world)
}

pub(super) fn entity_collision_candidates(
    request: &Value,
) -> Result<Vec<ClientEntityCollisionCandidate>, String> {
    request
        .get("candidates")
        .ok_or("candidates must be an array")?
        .as_array()
        .ok_or("candidates must be an array")?
        .iter()
        .map(|entry| {
            let candidate = entry
                .as_object()
                .ok_or("entity collision candidate must be an object")?;
            Ok(ClientEntityCollisionCandidate {
                bounding_box: aabb_box(
                    candidate
                        .get("box")
                        .ok_or("candidate box must be an object")?,
                )?,
                is_source: candidate
                    .get("is_source")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                spectator: candidate
                    .get("spectator")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                can_be_collided_with: candidate
                    .get("can_be_collided_with")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_pushable: candidate
                    .get("is_pushable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                same_vehicle: candidate
                    .get("same_vehicle")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

pub(super) fn entity_dimensions_snapshot(value: ClientEntityDimensions) -> Value {
    json!({
        "width": format!("{:08x}", value.width().to_bits()),
        "height": format!("{:08x}", value.height().to_bits()),
        "eye_height": format!("{:08x}", value.eye_height().to_bits()),
        "fixed": value.is_fixed(),
    })
}

pub(super) fn shape_snapshot(value: &VoxelShape) -> Value {
    let mut state = json!({
        "empty": value.is_empty(),
        "boxes": value.to_aabbs().into_iter().map(aabb_snapshot).collect::<Vec<_>>(),
    });
    if let Some(bounds) = value.bounds() {
        state["bounds"] = aabb_snapshot(bounds);
    }
    state
}

pub(super) fn client_world_border_status_snapshot(status: ClientWorldBorderStatus) -> &'static str {
    match status {
        ClientWorldBorderStatus::Stationary => "STATIONARY",
        ClientWorldBorderStatus::Shrinking => "SHRINKING",
        ClientWorldBorderStatus::Growing => "GROWING",
    }
}

pub(super) fn client_world_border_snapshot(
    event: &str,
    game_time: u64,
    border: &ClientWorldBorder,
) -> Value {
    let samples = [0.0_f32, 0.5_f32, 1.0_f32]
        .into_iter()
        .map(|partial_tick| {
            json!({
                "partial_tick": format!("{:08x}", partial_tick.to_bits()),
                "min_x": format!("{:016x}", border.min_x(partial_tick).to_bits()),
                "max_x": format!("{:016x}", border.max_x(partial_tick).to_bits()),
                "min_z": format!("{:016x}", border.min_z(partial_tick).to_bits()),
                "max_z": format!("{:016x}", border.max_z(partial_tick).to_bits()),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "event": event,
        "game_time": game_time,
        "center_x": format!("{:016x}", border.center_x().to_bits()),
        "center_z": format!("{:016x}", border.center_z().to_bits()),
        "absolute_max_size": border.absolute_max_size(),
        "warning_blocks": border.warning_blocks(),
        "warning_time": border.warning_time(),
        "size": format!("{:016x}", border.size().to_bits()),
        "lerp_time": border.lerp_time(),
        "lerp_target": format!("{:016x}", border.lerp_target().to_bits()),
        "status": client_world_border_status_snapshot(border.status()),
        "samples": samples,
        "collision_shape": shape_snapshot(&border.collision_shape()),
    })
}

pub(super) fn shape_snapshot_with_coords(value: &VoxelShape) -> Value {
    let mut state = shape_snapshot(value);
    state["coords"] = json!({
        "x": value
            .coords(Axis::X)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
        "y": value
            .coords(Axis::Y)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
        "z": value
            .coords(Axis::Z)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
    });
    state
}

pub(super) fn shape_axis_limits_snapshot(value: &VoxelShape) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min(Axis::X).to_bits()),
        "max_x": format!("{:016x}", value.max(Axis::X).to_bits()),
        "min_y": format!("{:016x}", value.min(Axis::Y).to_bits()),
        "max_y": format!("{:016x}", value.max(Axis::Y).to_bits()),
        "min_z": format!("{:016x}", value.min(Axis::Z).to_bits()),
        "max_z": format!("{:016x}", value.max(Axis::Z).to_bits()),
    })
}
