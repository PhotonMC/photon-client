use super::*;

pub(super) fn world_border_collision_shape_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("world border input must be an object")?;
            let center_x = f64_value(
                input
                    .get("center_x")
                    .ok_or("center_x must be a bit string")?,
                "center_x",
            )?;
            let center_z = f64_value(
                input
                    .get("center_z")
                    .ok_or("center_z must be a bit string")?,
                "center_z",
            )?;
            let size = f64_value(
                input.get("size").ok_or("size must be a bit string")?,
                "size",
            )?;
            let absolute_max_size = input
                .get("absolute_max_size")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .ok_or("absolute_max_size must be an i32")?;
            Ok(shape_snapshot(&world_border_collision_shape(
                center_x,
                center_z,
                size,
                absolute_max_size,
            )))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn world_border_collision_shape_moving_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|entry| {
            let input = entry
                .as_object()
                .ok_or("world border input must be an object")?;
            let center_x = f64_value(
                input
                    .get("center_x")
                    .ok_or("center_x must be a bit string")?,
                "center_x",
            )?;
            let center_z = f64_value(
                input
                    .get("center_z")
                    .ok_or("center_z must be a bit string")?,
                "center_z",
            )?;
            let from = f64_value(
                input.get("from").ok_or("from must be a bit string")?,
                "from",
            )?;
            let to = f64_value(input.get("to").ok_or("to must be a bit string")?, "to")?;
            let absolute_max_size = input
                .get("absolute_max_size")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .ok_or("absolute_max_size must be an i32")?;
            let duration = input
                .get("duration")
                .and_then(Value::as_i64)
                .ok_or("duration must be an i64")?;
            let tick_count = input
                .get("tick_count")
                .and_then(Value::as_i64)
                .filter(|value| *value >= 0)
                .ok_or("tick_count must be a non-negative i64")?;
            let partial_ticks = input
                .get("partial_ticks")
                .ok_or("partial_ticks must be an array")?
                .as_array()
                .ok_or("partial_ticks must be an array")?
                .iter()
                .map(|value| f32_value(value, "partial_tick"))
                .collect::<Result<Vec<_>, String>>()?;
            let mut border = ClientWorldBorder::new(center_x, center_z, absolute_max_size, from);
            border.lerp_size_between(from, to, duration);
            let mut states = Vec::new();
            for tick in 0..=tick_count {
                let samples = partial_ticks
                    .iter()
                    .map(|partial_tick| {
                        json!({
                            "partial_tick": format!("{:08x}", partial_tick.to_bits()),
                            "min_x": format!("{:016x}", border.min_x(*partial_tick).to_bits()),
                            "max_x": format!("{:016x}", border.max_x(*partial_tick).to_bits()),
                            "min_z": format!("{:016x}", border.min_z(*partial_tick).to_bits()),
                            "max_z": format!("{:016x}", border.max_z(*partial_tick).to_bits()),
                        })
                    })
                    .collect::<Vec<_>>();
                let status = match border.status() {
                    ClientWorldBorderStatus::Stationary => "STATIONARY",
                    ClientWorldBorderStatus::Shrinking => "SHRINKING",
                    ClientWorldBorderStatus::Growing => "GROWING",
                };
                states.push(json!({
                    "size": format!("{:016x}", border.size().to_bits()),
                    "lerp_time": border.lerp_time(),
                    "lerp_target": format!("{:016x}", border.lerp_target().to_bits()),
                    "status": status,
                    "samples": samples,
                    "collision_shape": shape_snapshot(&border.collision_shape()),
                }));
                if tick < tick_count {
                    border.tick();
                }
            }
            Ok(json!(states))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn client_world_border_state_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let tracks = inputs
        .iter()
        .map(|entry| {
            let events = entry
                .get("events")
                .ok_or("events must be an array")?
                .as_array()
                .ok_or("events must be an array")?;
            let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
            let mut states = Vec::new();
            for event in events {
                let operation = string(event, "op")?;
                match operation {
                    "initialize" => {
                        world.apply_world_border_initialize(ClientWorldBorderInitialization {
                            center_x: f64_field(event, "center_x")?,
                            center_z: f64_field(event, "center_z")?,
                            old_size: f64_field(event, "old_size")?,
                            new_size: f64_field(event, "new_size")?,
                            lerp_time: i64_field(event, "lerp_time")?,
                            absolute_max_size: i32_field(event, "absolute_max_size")?,
                            warning_blocks: i32_field(event, "warning_blocks")?,
                            warning_time: i32_field(event, "warning_time")?,
                        })
                    }
                    "center" => world.apply_world_border_center(
                        f64_field(event, "center_x")?,
                        f64_field(event, "center_z")?,
                    ),
                    "lerp" => world.apply_world_border_lerp_size(
                        f64_field(event, "old_size")?,
                        f64_field(event, "new_size")?,
                        i64_field(event, "lerp_time")?,
                    ),
                    "size" => world.apply_world_border_size(f64_field(event, "size")?),
                    "warning_blocks" => {
                        world.apply_world_border_warning_blocks(i32_field(event, "value")?)
                    }
                    "warning_time" => {
                        world.apply_world_border_warning_time(i32_field(event, "value")?)
                    }
                    "tick" => world.tick(),
                    _ => return Err(format!("unknown world border event: {operation}")),
                }
                states.push(client_world_border_snapshot(
                    operation,
                    world.game_time(),
                    world.world_border(),
                ));
            }
            Ok(json!(states))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"tracks": tracks}))
}

fn border_side_name(side: u8) -> &'static str {
    match side {
        photon_render::border::SOUTH => "SOUTH",
        photon_render::border::WEST => "WEST",
        photon_render::border::NORTH => "NORTH",
        _ => "EAST",
    }
}

pub(super) fn world_border_render_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(world_border_render_state)
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

fn world_border_render_state(entry: &Value) -> Result<Value, String> {
    let input = entry
        .as_object()
        .ok_or("world border input must be an object")?;
    let center_x = f64_value(
        input
            .get("center_x")
            .ok_or("center_x must be a bit string")?,
        "center_x",
    )?;
    let center_z = f64_value(
        input
            .get("center_z")
            .ok_or("center_z must be a bit string")?,
        "center_z",
    )?;
    let absolute_max_size = input
        .get("absolute_max_size")
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or("absolute_max_size must be an i32")?;

    let border = if let Some(from_value) = input.get("from") {
        let from = f64_value(from_value, "from")?;
        let to = f64_value(input.get("to").ok_or("to must be a bit string")?, "to")?;
        let duration = input
            .get("duration")
            .and_then(Value::as_i64)
            .ok_or("duration must be an i64")?;
        let ticks = input
            .get("ticks")
            .and_then(Value::as_i64)
            .filter(|value| *value >= 0)
            .ok_or("ticks must be a non-negative i64")?;
        let mut border = ClientWorldBorder::new(center_x, center_z, absolute_max_size, from);
        border.lerp_size_between(from, to, duration);
        for _ in 0..ticks {
            border.tick();
        }
        border
    } else {
        let size = f64_value(
            input.get("size").ok_or("size must be a bit string")?,
            "size",
        )?;
        ClientWorldBorder::new(center_x, center_z, absolute_max_size, size)
    };

    let partial_tick = f32_value(
        input
            .get("partial_tick")
            .ok_or("partial_tick must be a bit string")?,
        "partial_tick",
    )?;
    let camera_x = f64_value(
        input
            .get("camera_x")
            .ok_or("camera_x must be a bit string")?,
        "camera_x",
    )?;
    let camera_y = f64_value(
        input
            .get("camera_y")
            .ok_or("camera_y must be a bit string")?,
        "camera_y",
    )?;
    let camera_z = f64_value(
        input
            .get("camera_z")
            .ok_or("camera_z must be a bit string")?,
        "camera_z",
    )?;
    let render_distance = f64_value(
        input
            .get("render_distance")
            .ok_or("render_distance must be a bit string")?,
        "render_distance",
    )?;
    let half_height_y = f32_value(
        input
            .get("half_height_y")
            .ok_or("half_height_y must be a bit string")?,
        "half_height_y",
    )?;
    let time_millis = input
        .get("time_millis")
        .and_then(Value::as_i64)
        .ok_or("time_millis must be an i64")?;

    let min_x = border.min_x(partial_tick);
    let max_x = border.max_x(partial_tick);
    let min_z = border.min_z(partial_tick);
    let max_z = border.max_z(partial_tick);

    let extract = photon_render::border::extract(
        &border,
        partial_tick,
        (camera_x, camera_z),
        render_distance,
    );

    let closest =
        photon_render::border::closest_sides(min_x, max_x, min_z, max_z, camera_x, camera_z)
            .into_iter()
            .map(|(side, distance)| {
                json!({
                    "direction": border_side_name(side),
                    "distance": format!("{:016x}", distance.to_bits()),
                })
            })
            .collect::<Vec<_>>();

    let quads = if extract.alpha <= 0.0 {
        Vec::new()
    } else {
        photon_render::border::quads(
            min_x,
            max_x,
            min_z,
            max_z,
            [camera_x, camera_y, camera_z],
            [0, 0, 0],
            render_distance,
            half_height_y,
            time_millis,
            extract.tint,
            extract.alpha as f32,
        )
        .into_iter()
        .map(|(side, corners)| {
            json!({
                "side": border_side_name(side),
                "corners": corners
                    .iter()
                    .map(|vertex| json!({
                        "x": format!("{:08x}", vertex.position[0].to_bits()),
                        "y": format!("{:08x}", vertex.position[1].to_bits()),
                        "z": format!("{:08x}", vertex.position[2].to_bits()),
                        "u": format!("{:08x}", vertex.uv[0].to_bits()),
                        "v": format!("{:08x}", vertex.uv[1].to_bits()),
                    }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>()
    };

    Ok(json!({
        "alpha": format!("{:016x}", extract.alpha.to_bits()),
        "tint_red": format!("{:08x}", extract.tint[0].to_bits()),
        "tint_green": format!("{:08x}", extract.tint[1].to_bits()),
        "tint_blue": format!("{:08x}", extract.tint[2].to_bits()),
        "closest": closest,
        "quads": quads,
    }))
}
