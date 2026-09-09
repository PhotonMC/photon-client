//! JSONL test glue. Behavior is delegated to the production library.

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use voxel_client::aabb::{Aabb, AabbBuilder, AabbBuilderError, BlockHitResult, BoundingBox};
use voxel_client::binary::{self, BlockPos, ChunkPos, SectionPos, Uuid};
use voxel_client::direction::{Axis, Direction};
use voxel_client::math::{self, Vec2, Vec3, Vec3f};
use voxel_client::nbt::{self, Field, FieldValue};
use voxel_client::random::RandomSource;
use voxel_client::shape::{BooleanOp, ShapeRotation, Shapes, VoxelShape};
use voxel_client::timer::Timer;
use voxel_client::utf8;
use voxel_client::wire::{self, ReadError};

fn string<'a>(request: &'a Value, name: &str) -> Result<&'a str, String> {
    request[name]
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err("hex must contain complete ASCII byte pairs".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .map_err(|_| "invalid hex byte".to_owned())
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read(input: &mut &[u8], wide: bool) -> Result<i64, ReadError> {
    if wide {
        wire::read_varlong(input)
    } else {
        wire::read_varint(input).map(i64::from)
    }
}

fn wire_result(request: &Value) -> Result<Value, String> {
    let wide = match string(request, "codec")? {
        "varint" => false,
        "varlong" => true,
        _ => return Err("unknown codec".into()),
    };
    let mut bytes = Vec::new();
    let mut result;
    match string(request, "operation")? {
        "write" => {
            let text = string(request, "value")?;
            let size = if wide {
                let value = text.parse::<i64>().map_err(|error| error.to_string())?;
                wire::write_varlong(value, &mut bytes);
                wire::varlong_size(value)
            } else {
                let value = text.parse::<i32>().map_err(|error| error.to_string())?;
                wire::write_varint(value, &mut bytes);
                wire::varint_size(value)
            };
            let mut remaining = bytes.as_slice();
            let value = read(&mut remaining, wide)
                .map_err(|error| format!("cannot decode library output: {error:?}"))?;
            result = json!({"outcome": "value", "size": size, "hex": hex(&bytes),
                "decoded": value.to_string(), "consumed": bytes.len() - remaining.len(),
                "remaining_hex": hex(remaining)});
        }
        "read" => {
            bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            result = match read(&mut remaining, wide) {
                Ok(value) => json!({"outcome": "value", "value": value.to_string()}),
                Err(error) => json!({"outcome": "error", "error": match error {
                    ReadError::BufferUnderflow => "buffer_underflow",
                    ReadError::TooBig => "too_big",
                }}),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
        }
        _ => return Err("unknown wire operation".into()),
    }
    Ok(result)
}

fn f32_field(value: &Value, name: &str) -> Result<f32, String> {
    let text = string(value, name)?;
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be eight hexadecimal digits"));
    }
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

fn millis(value: &Value, name: &str) -> Result<i64, String> {
    string(value, name)?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

fn f64_field(value: &Value, name: &str) -> Result<f64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())
}

fn u64_hex_field(value: &Value, name: &str) -> Result<u64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16).map_err(|error| error.to_string())
}

fn f64_value(value: &Value, name: &str) -> Result<f64, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())
}

fn vec2_snapshot(vector: Vec2) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "length": format!("{:08x}", vector.length().to_bits()),
        "length_squared": format!("{:08x}", vector.length_squared().to_bits()),
    })
}

fn vec3_snapshot(vector: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", vector.x.to_bits()),
        "y": format!("{:016x}", vector.y.to_bits()),
        "z": format!("{:016x}", vector.z.to_bits()),
        "length": format!("{:016x}", vector.length().to_bits()),
        "length_squared": format!("{:016x}", vector.length_squared().to_bits()),
    })
}

fn vec3f_snapshot(vector: Vec3f) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "z": format!("{:08x}", vector.z.to_bits()),
    })
}

fn direction_snapshot(facing: Direction) -> Value {
    let (step_x, step_y, step_z) = facing.step();
    json!({
        "name": facing.name(),
        "data3d": facing.data3d(),
        "data2d": facing.data2d(),
        "opposite": facing.opposite().name(),
        "axis": facing.axis().name(),
        "axis_direction": facing.axis_direction().name(),
        "step_x": step_x,
        "step_y": step_y,
        "step_z": step_z,
    })
}

fn blockpos(value: &Value) -> Result<BlockPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BlockPos::new(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
    ))
}

fn bounding_box(value: &Value) -> Result<BoundingBox, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BoundingBox::new(
        coordinate("min_x")?,
        coordinate("min_y")?,
        coordinate("min_z")?,
        coordinate("max_x")?,
        coordinate("max_y")?,
        coordinate("max_z")?,
    ))
}

fn blockpos_snapshot(value: BlockPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
    })
}

fn chunkpos(value: &Value) -> Result<ChunkPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(ChunkPos::new(coordinate("x")?, coordinate("z")?))
}

fn chunkpos_snapshot(value: ChunkPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "z": value.z.to_string(),
    })
}

fn chunkpos_block_coordinates_snapshot(value: ChunkPos, offset: i32) -> Value {
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

fn chunkpos_region_coordinates_snapshot(value: ChunkPos, region_x: i32, region_z: i32) -> Value {
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

fn chunkpos_block_positions_snapshot(
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

fn chunkpos_representation_snapshot(value: ChunkPos, hash_x: i32, hash_z: i32) -> Value {
    json!({
        "to_string": value.to_string_java(),
        "hash_code": value.hash_code_java().to_string(),
        "static_hash": ChunkPos::hash(hash_x, hash_z).to_string(),
    })
}

fn chunkpos_from_blockpos_snapshot(value: BlockPos) -> Value {
    let containing = ChunkPos::containing(value);
    json!({
        "containing": chunkpos_snapshot(containing),
        "packed": format!("{:x}", ChunkPos::pack_block_pos(value)),
    })
}

fn chunkpos_distances_snapshot(value: ChunkPos, other: ChunkPos) -> Value {
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

fn chunkpos_validity_snapshot(value: ChunkPos) -> Value {
    json!({
        "instance": value.is_valid(),
        "static": ChunkPos::is_valid_coordinates(value.x, value.z),
    })
}

fn chunkpos_range_closed_snapshot(values: Vec<ChunkPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(chunkpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn blockpos_result(request: &Value) -> Result<Value, String> {
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

fn section_relative_result(request: &Value) -> Result<Value, String> {
    let relative = SectionPos::section_relative_pos(blockpos(request)?);
    Ok(json!({
        "relative": relative.to_string(),
        "x": SectionPos::section_relative_x(relative).to_string(),
        "y": SectionPos::section_relative_y(relative).to_string(),
        "z": SectionPos::section_relative_z(relative).to_string(),
    }))
}

fn random_result(request: &Value) -> Result<Value, String> {
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

fn random_seed(request: &Value) -> Result<i64, String> {
    string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

fn random_state(random: &mut RandomSource, step: &Value) -> Result<Value, String> {
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

fn random_steps(random: &mut RandomSource, value: &Value) -> Result<Value, String> {
    let mut events = Vec::new();
    for step in value.as_array().ok_or("random steps must be an array")? {
        events.push(random_state(random, step)?);
    }
    Ok(Value::Array(events))
}

fn random_fork_result(request: &Value) -> Result<Value, String> {
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

fn i32_field(value: &Value, name: &str) -> Result<i32, String> {
    value[name]
        .as_i64()
        .ok_or_else(|| format!("{name} must be an integer"))?
        .try_into()
        .map_err(|_| format!("{name} must fit in i32"))
}

fn random_positional_result(request: &Value) -> Result<Value, String> {
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

fn aabb_box(value: &Value) -> Result<Aabb, String> {
    Ok(Aabb::new(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    ))
}

fn aabb_snapshot(value: Aabb) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min_x.to_bits()),
        "min_y": format!("{:016x}", value.min_y.to_bits()),
        "min_z": format!("{:016x}", value.min_z.to_bits()),
        "max_x": format!("{:016x}", value.max_x.to_bits()),
        "max_y": format!("{:016x}", value.max_y.to_bits()),
        "max_z": format!("{:016x}", value.max_z.to_bits()),
    })
}

fn aabb_builder_result(request: &Value) -> Result<Value, String> {
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

fn aabb_sizes_snapshot(value: Aabb) -> Value {
    json!({
        "x_size": format!("{:016x}", value.x_size().to_bits()),
        "y_size": format!("{:016x}", value.y_size().to_bits()),
        "z_size": format!("{:016x}", value.z_size().to_bits()),
        "size": format!("{:016x}", value.size().to_bits()),
    })
}

fn aabb_positions_snapshot(value: Aabb) -> Value {
    json!({
        "center": vec3_snapshot(value.center()),
        "bottom_center": vec3_snapshot(value.bottom_center()),
        "min_position": vec3_snapshot(value.min_position()),
        "max_position": vec3_snapshot(value.max_position()),
    })
}

fn block_hit_result_snapshot(value: Option<BlockHitResult>) -> Value {
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

fn shape_box(value: &Value) -> Result<VoxelShape, String> {
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

fn shape_snapshot(value: &VoxelShape) -> Value {
    let mut state = json!({
        "empty": value.is_empty(),
        "boxes": value.to_aabbs().into_iter().map(aabb_snapshot).collect::<Vec<_>>(),
    });
    if let Some(bounds) = value.bounds() {
        state["bounds"] = aabb_snapshot(bounds);
    }
    state
}

fn shape_snapshot_with_coords(value: &VoxelShape) -> Value {
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

fn shape_axis_limits_snapshot(value: &VoxelShape) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min(Axis::X).to_bits()),
        "max_x": format!("{:016x}", value.max(Axis::X).to_bits()),
        "min_y": format!("{:016x}", value.min(Axis::Y).to_bits()),
        "max_y": format!("{:016x}", value.max(Axis::Y).to_bits()),
        "min_z": format!("{:016x}", value.min(Axis::Z).to_bits()),
        "max_z": format!("{:016x}", value.max(Axis::Z).to_bits()),
    })
}

fn shape_result(request: &Value) -> Result<Value, String> {
    match string(request, "kind")? {
        "single" => {
            let mut value = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let mut events = vec![shape_snapshot(&value)];
            for step in request["steps"]
                .as_array()
                .ok_or("steps must be an array")?
            {
                value = match string(step, "op")? {
                    "move" => value.move_by(
                        f64_field(step, "dx")?,
                        f64_field(step, "dy")?,
                        f64_field(step, "dz")?,
                    ),
                    "move_vec3" => value.move_by_vec3(f64_vec3(
                        step.get("delta").ok_or("delta must be an object")?,
                    )?),
                    "move_block_pos" => value.move_by_block_pos(blockpos(
                        step.get("delta").ok_or("delta must be an object")?,
                    )?),
                    _ => return Err("unknown VoxelShape operation".into()),
                };
                events.push(shape_snapshot(&value));
            }
            Ok(json!({"events": events}))
        }
        "join" => {
            let operation = match string(request, "op")? {
                "or" => BooleanOp::Or,
                "and" => BooleanOp::And,
                "only_first" => BooleanOp::OnlyFirst,
                "only_second" => BooleanOp::OnlySecond,
                "false" => BooleanOp::False,
                "not_same" => BooleanOp::NotSame,
                "second" => BooleanOp::Second,
                "first" => BooleanOp::First,
                _ => return Err("unknown VoxelShape boolean operation".into()),
            };
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let joined = Shapes::join(&first, &second, operation);
            Ok(json!({
                "shape": shape_snapshot(&joined),
                "join_is_not_empty": Shapes::join_is_not_empty(&first, &second, operation),
            }))
        }
        "join_many" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("join_many needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(json!({"shape": shape_snapshot(&joined)}))
        }
        "single_encompassing" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("single_encompassing needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(json!({
                "shape": shape_snapshot(&joined.single_encompassing()),
            }))
        }
        "optimize" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("optimize needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = tail.iter().fold(first.clone(), |joined, shape| {
                Shapes::join_unoptimized(&joined, shape, BooleanOp::Or)
            });
            Ok(json!({
                "events": [
                    shape_snapshot_with_coords(&joined),
                    shape_snapshot_with_coords(&joined.optimize()),
                ],
            }))
        }
        "edges" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("edges needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let mut edges = Vec::new();
            joined.for_all_edges(|x1, y1, z1, x2, y2, z2| {
                edges.push(vec![
                    format!("{:016x}", x1.to_bits()),
                    format!("{:016x}", y1.to_bits()),
                    format!("{:016x}", z1.to_bits()),
                    format!("{:016x}", x2.to_bits()),
                    format!("{:016x}", y2.to_bits()),
                    format!("{:016x}", z2.to_bits()),
                ]);
            });
            Ok(json!({"edges": edges}))
        }
        "join_coords" => {
            let operation = match string(request, "op")? {
                "or" => BooleanOp::Or,
                "and" => BooleanOp::And,
                "only_first" => BooleanOp::OnlyFirst,
                "only_second" => BooleanOp::OnlySecond,
                "not_same" => BooleanOp::NotSame,
                "second" => BooleanOp::Second,
                "first" => BooleanOp::First,
                _ => return Err("unknown VoxelShape boolean operation".into()),
            };
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let unoptimized = Shapes::join_unoptimized(&first, &second, operation);
            let optimized = Shapes::join(&first, &second, operation);
            Ok(json!({
                "unoptimized": shape_snapshot_with_coords(&unoptimized),
                "optimized": shape_snapshot_with_coords(&optimized),
            }))
        }
        "boxes" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("boxes needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let mut boxes = Vec::new();
            joined.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
                boxes.push(vec![
                    format!("{:016x}", x1.to_bits()),
                    format!("{:016x}", y1.to_bits()),
                    format!("{:016x}", z1.to_bits()),
                    format!("{:016x}", x2.to_bits()),
                    format!("{:016x}", y2.to_bits()),
                    format!("{:016x}", z2.to_bits()),
                ]);
            });
            Ok(json!({"boxes": boxes}))
        }
        "create_aabb" => {
            let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
            let shape = Shapes::create_from_aabb(aabb)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            Ok(json!({"shape": shape_snapshot(&shape)}))
        }
        "create_coords" => {
            let value = request.get("box").ok_or("box must be an object")?;
            let shape = Shapes::create_from_coords(
                f64_field(value, "min_x")?,
                f64_field(value, "min_y")?,
                f64_field(value, "min_z")?,
                f64_field(value, "max_x")?,
                f64_field(value, "max_y")?,
                f64_field(value, "max_z")?,
            );
            Ok(json!({"shape": shape_snapshot(&shape)}))
        }
        "infinity" => {
            let infinity = Shapes::infinity();
            match string(request, "view")? {
                "snapshot" => Ok(json!({"shape": shape_snapshot(&infinity)})),
                "boxes" => {
                    let mut boxes = Vec::new();
                    infinity.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
                        boxes.push(vec![
                            format!("{:016x}", x1.to_bits()),
                            format!("{:016x}", y1.to_bits()),
                            format!("{:016x}", z1.to_bits()),
                            format!("{:016x}", x2.to_bits()),
                            format!("{:016x}", y2.to_bits()),
                            format!("{:016x}", z2.to_bits()),
                        ]);
                    });
                    Ok(json!({"boxes": boxes}))
                }
                "not_same_self" => Ok(json!({
                    "shape": shape_snapshot(&Shapes::join(
                        &infinity,
                        &infinity,
                        BooleanOp::NotSame,
                    ))
                })),
                _ => Err("unknown infinity view".into()),
            }
        }
        "constants" => match string(request, "name")? {
            "epsilon" => Ok(json!({
                "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits())
            })),
            "big_epsilon" => Ok(json!({
                "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits())
            })),
            "both" => Ok(json!({
                "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits()),
                "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits()),
            })),
            _ => Err("unknown shape constant".into()),
        },
        "axis_limits" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("axis_limits needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(shape_axis_limits_snapshot(&joined))
        }
        "axis_limits_at" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("axis_limits_at needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let values = request["queries"]
                .as_array()
                .ok_or("queries must be an array")?
                .iter()
                .map(|query| {
                    let axis =
                        Axis::from_name(string(query, "axis")?).ok_or("unknown shape axis")?;
                    let b = f64_field(query, "b")?;
                    let c = f64_field(query, "c")?;
                    let value = if string(query, "op")? == "min" {
                        joined.min_at(axis, b, c)
                    } else {
                        joined.max_at(axis, b, c)
                    };
                    Ok(format!("{:016x}", value.to_bits()))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"values": values}))
        }
        "equal" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            Ok(json!({"equal": Shapes::equal(&first, &second)}))
        }
        "object_equals" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let same_geometry = shape_box(request.get("same").ok_or("same must be an object")?)?;
            let different_geometry = shape_box(
                request
                    .get("different")
                    .ok_or("different must be an object")?,
            )?;
            Ok(json!({
                "self": shape.equals_java(&shape),
                "same_geometry": shape.equals_java(&same_geometry),
                "different_geometry": shape.equals_java(&different_geometry),
            }))
        }
        "object_equals_constants" => {
            let empty = Shapes::empty();
            let block = Shapes::block();
            let infinity = Shapes::infinity();
            let empty_thin = Shapes::box_shape(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            let block_box = Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            let infinity_box = Shapes::box_shape(
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::INFINITY,
                f64::INFINITY,
            )
            .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            Ok(json!({
                "empty_self": empty.equals_java(&Shapes::empty()),
                "empty_thin": empty.equals_java(&empty_thin),
                "empty_move": empty.equals_java(&empty.move_by(1.0, 2.0, 3.0)),
                "block_self": block.equals_java(&Shapes::block()),
                "block_box": block.equals_java(&block_box),
                "block_move": block.equals_java(&block.move_by(0.0, 0.0, 0.0)),
                "infinity_self": infinity.equals_java(&Shapes::infinity()),
                "infinity_box": infinity.equals_java(&infinity_box),
            }))
        }
        "to_string" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            Ok(json!({"value": shape.to_string_java()}))
        }
        "rotate" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate_with_center(&shape, rotation, center)),
            }))
        }
        "rotate_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_horizontal_axis" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_horizontal_axis(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal_axis_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_horizontal_axis_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_axis" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_all_axis(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_axis_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_axis_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_horizontal(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let shapes = Shapes::rotate_horizontal_with_initial(&shape, initial);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_all(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_with_initial(&shape, initial, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_attach_face" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let faces = Shapes::rotate_attach_face(&shape);
            Ok(json!({
                "faces": faces
                    .into_iter()
                    .map(|(face, shapes)| json!({
                        "face": face.name(),
                        "shapes": shapes
                            .into_iter()
                            .map(|(direction, shape)| json!({
                                "direction": direction.name(),
                                "shape": shape_snapshot(&shape),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_attach_face_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let faces = Shapes::rotate_attach_face_with_initial(&shape, initial);
            Ok(json!({
                "faces": faces
                    .into_iter()
                    .map(|(face, shapes)| json!({
                        "face": face.name(),
                        "shapes": shapes
                            .into_iter()
                            .map(|(direction, shape)| json!({
                                "direction": direction.name(),
                                "shape": shape_snapshot(&shape),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "collide" => {
            let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let distance = f64_field(request, "distance")?;
            Ok(json!({
                "distance": format!("{:016x}", shape.collide(axis, moving, distance).to_bits()),
            }))
        }
        "collide_many" => {
            let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            let shapes = entries
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let distance = f64_field(request, "distance")?;
            Ok(json!({
                "distance": format!(
                    "{:016x}",
                    Shapes::collide(axis, moving, &shapes, distance).to_bits()
                ),
            }))
        }
        "clip" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let queries = request
                .get("queries")
                .ok_or("queries must be an array")?
                .as_array()
                .ok_or("queries must be an array")?;
            let mut events = Vec::with_capacity(queries.len());
            for query in queries {
                let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
            }
            Ok(json!({"events": events}))
        }
        "clip_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let shape = Shapes::join(&first, &second, BooleanOp::Or);
            let queries = request
                .get("queries")
                .ok_or("queries must be an array")?
                .as_array()
                .ok_or("queries must be an array")?;
            let mut events = Vec::with_capacity(queries.len());
            for query in queries {
                let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
            }
            Ok(json!({"events": events}))
        }
        "closest" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "closest_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let shape = Shapes::join(&first, &second, BooleanOp::Or);
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "face" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "face_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "block_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "block_occludes_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "merged_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown merged occlusion direction")?;
            Ok(json!({
                "occludes": Shapes::merged_face_occludes(&shape, &occluder, direction),
            }))
        }
        "face_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            Ok(json!({
                "occludes": Shapes::face_shape_occludes(&shape, &occluder),
            }))
        }
        _ => Err("unknown VoxelShape kind".into()),
    }
}

fn f64_vec3(value: &Value) -> Result<Vec3, String> {
    Ok(Vec3::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
        f64_field(value, "z")?,
    ))
}

fn clip_snapshot(value: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", value.x.to_bits()),
        "y": format!("{:016x}", value.y.to_bits()),
        "z": format!("{:016x}", value.z.to_bits()),
    })
}

fn aabb_result(request: &Value) -> Result<Value, String> {
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

fn math_result(request: &Value) -> Result<Value, String> {
    let operation = string(request, "operation")?;
    if operation == "random" {
        return random_result(request);
    }
    if operation == "random_fork" {
        return random_fork_result(request);
    }
    if operation == "random_positional" {
        return random_positional_result(request);
    }
    if operation == "aabb_builder" {
        return aabb_builder_result(request);
    }
    if operation == "aabb" {
        return aabb_result(request);
    }
    if operation == "shape" {
        return shape_result(request);
    }
    if operation == "blockpos" {
        return blockpos_result(request);
    }
    if operation == "section_relative" {
        return section_relative_result(request);
    }
    if operation == "vec3_codec" {
        return match string(request, "mode")? {
            "decode" => {
                let values = request["values"]
                    .as_array()
                    .ok_or("values must be an array")?
                    .iter()
                    .enumerate()
                    .map(|(index, value)| f64_value(value, &format!("values[{index}]")))
                    .collect::<Result<Vec<_>, _>>()?;
                match Vec3::codec_decode(&values) {
                    Ok(value) => Ok(json!({
                        "outcome": "value",
                        "value": vec3_snapshot(value),
                    })),
                    Err(math::Vec3CodecError::WrongLength { partial, .. }) => {
                        let mut result = json!({
                            "outcome": "error",
                            "error": "Input is not a list of 3 elements",
                        });
                        if let Some(value) = partial {
                            result["partial"] = vec3_snapshot(value);
                        }
                        Ok(result)
                    }
                }
            }
            "encode" => {
                let value = Vec3::new(
                    f64_field(request, "x")?,
                    f64_field(request, "y")?,
                    f64_field(request, "z")?,
                );
                Ok(json!({
                    "outcome": "value",
                    "value": value
                        .codec_encode()
                        .into_iter()
                        .map(|component| format!("{:016x}", component.to_bits()))
                        .collect::<Vec<_>>(),
                }))
            }
            _ => Err("unknown Vec3 codec mode".into()),
        };
    }
    if operation == "chunkpos_codec" {
        return match string(request, "mode")? {
            "decode" => {
                let values = request["values"]
                    .as_array()
                    .ok_or("values must be an array")?
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        value
                            .as_str()
                            .ok_or_else(|| format!("values[{index}] must be a string"))?
                            .parse::<i32>()
                            .map_err(|error| error.to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                match ChunkPos::codec_decode(&values) {
                    Ok(value) => Ok(json!({
                        "outcome": "value",
                        "value": chunkpos_snapshot(value),
                    })),
                    Err(binary::ChunkPosCodecError::WrongLength { partial, .. }) => {
                        let mut result = json!({
                            "outcome": "error",
                            "error": "Input is not a list of 2 ints",
                        });
                        if let Some(value) = partial {
                            result["partial"] = chunkpos_snapshot(value);
                        }
                        Ok(result)
                    }
                }
            }
            "encode" => {
                let value = chunkpos(request)?;
                Ok(json!({
                    "outcome": "value",
                    "value": value.codec_encode(),
                }))
            }
            _ => Err("unknown ChunkPos codec mode".into()),
        };
    }
    if operation == "chunkpos_block_coordinates" {
        let value = chunkpos(request)?;
        let offset = string(request, "offset")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_block_coordinates_snapshot(value, offset),
        }));
    }
    if operation == "chunkpos_region_coordinates" {
        let value = chunkpos(request)?;
        let region_x = string(request, "region_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let region_z = string(request, "region_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_region_coordinates_snapshot(value, region_x, region_z),
        }));
    }
    if operation == "chunkpos_block_positions" {
        let value = chunkpos(request)?;
        let block_x = string(request, "block_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let y = string(request, "y")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let block_z = string(request, "block_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let contains = request["contains"]
            .as_array()
            .ok_or("contains must be an array")?;
        return Ok(json!({
            "value": chunkpos_block_positions_snapshot(value, block_x, y, block_z, contains)?,
        }));
    }
    if operation == "chunkpos_representation" {
        let value = chunkpos(request)?;
        let hash_x = string(request, "hash_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let hash_z = string(request, "hash_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_representation_snapshot(value, hash_x, hash_z),
        }));
    }
    if operation == "chunkpos_from_blockpos" {
        return Ok(json!({
            "value": chunkpos_from_blockpos_snapshot(blockpos(request)?),
        }));
    }
    if operation == "chunkpos_distances" {
        let value = chunkpos(request)?;
        let other = ChunkPos::new(
            string(request, "other_x")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
            string(request, "other_z")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({
            "value": chunkpos_distances_snapshot(value, other),
        }));
    }
    if operation == "chunkpos_from_section_node" {
        let section_node = u64::from_str_radix(string(request, "section_node")?, 16)
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": format!("{:x}", ChunkPos::from_section_node(section_node)),
        }));
    }
    if operation == "chunkpos_range_closed" {
        let positions = match string(request, "mode")? {
            "center-radius" => {
                let center = chunkpos(request)?;
                let radius = string(request, "radius")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?;
                ChunkPos::range_closed(center, radius)
            }
            "from-to" => {
                let from = ChunkPos::new(
                    string(request, "from_x")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                    string(request, "from_z")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                );
                let to = ChunkPos::new(
                    string(request, "to_x")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                    string(request, "to_z")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                );
                ChunkPos::range_closed_from_to(from, to)
            }
            _ => return Err("unknown ChunkPos range mode".into()),
        };
        return Ok(json!({
            "value": chunkpos_range_closed_snapshot(positions),
        }));
    }
    if operation == "chunkpos_constant" {
        let value = match string(request, "name")? {
            "zero" => json!({
                "chunk_pos": chunkpos_snapshot(ChunkPos::ZERO),
            }),
            "invalid_chunk_pos" => json!({
                "packed": format!("{:x}", ChunkPos::INVALID_CHUNK_POS),
            }),
            "region_size" => json!({
                "value": ChunkPos::REGION_SIZE.to_string(),
            }),
            "region_max_index" => json!({
                "value": ChunkPos::REGION_MAX_INDEX.to_string(),
            }),
            _ => return Err("unknown ChunkPos constant".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "chunkpos_packed_accessors" {
        let packed = u64_hex_field(request, "packed")?;
        return Ok(json!({
            "value": {
                "x": ChunkPos::get_x(packed).to_string(),
                "z": ChunkPos::get_z(packed).to_string(),
            },
        }));
    }
    if operation == "sectionpos_packed_accessors" {
        let packed = u64_hex_field(request, "packed")?;
        return Ok(json!({
            "value": {
                "x": SectionPos::get_x(packed).to_string(),
                "y": SectionPos::get_y(packed).to_string(),
                "z": SectionPos::get_z(packed).to_string(),
            },
        }));
    }
    if operation == "chunkpos_validity" {
        return Ok(json!({
            "value": chunkpos_validity_snapshot(chunkpos(request)?),
        }));
    }
    if operation == "vec2" {
        let mut vector = Vec2::new(f32_field(request, "x")?, f32_field(request, "y")?);
        let mut events = vec![vec2_snapshot(vector)];
        let steps = request["steps"]
            .as_array()
            .ok_or("steps must be an array")?;
        for step in steps {
            vector = match string(step, "action")? {
                "normalize" => vector.normalized(),
                "scale" => vector.scale(f32_field(step, "factor")?),
                _ => return Err("unknown Vec2 action".into()),
            };
            events.push(vec2_snapshot(vector));
        }
        return Ok(json!({"events": events}));
    }
    if operation == "direction" {
        let mut facing =
            Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let mut events = vec![direction_snapshot(facing)];
        let steps = request["steps"]
            .as_array()
            .ok_or("steps must be an array")?;
        for step in steps {
            let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
            let clockwise = step["clockwise"]
                .as_bool()
                .ok_or("clockwise must be a boolean")?;
            facing = if clockwise {
                facing.clockwise(axis)
            } else {
                facing.counter_clockwise(axis)
            };
            events.push(direction_snapshot(facing));
        }
        return Ok(json!({"events": events}));
    }
    if operation == "coord" {
        let value = string(request, "value")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let out = match string(request, "fn")? {
            "block_to_section" => math::block_to_section_coord(value),
            "section_to_block" => math::section_to_block_coord(value),
            "section_relative" => math::section_relative(value),
            _ => return Err("unknown coord function".into()),
        };
        return Ok(json!({"value": out.to_string()}));
    }
    if operation == "vec3" {
        let mut vector = Vec3::new(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        let mut events = vec![vec3_snapshot(vector)];
        let steps = request["steps"]
            .as_array()
            .ok_or("steps must be an array")?;
        for step in steps {
            vector = match string(step, "action")? {
                "normalize" => vector.normalized(),
                "scale" => vector.scale(f64_field(step, "factor")?),
                "reverse" => vector.reverse(),
                "multiply_vec3" => vector.multiply_vec3(f64_vec3(
                    step.get("scale").ok_or("scale must be an object")?,
                )?),
                "multiply_components" => vector.multiply_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "horizontal" => vector.horizontal(),
                "add_vec3" => vector.add_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?),
                "add_scalar" => vector.add_scalar(f64_field(step, "value")?),
                "add_components" => vector.add_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "subtract_vec3" => vector.subtract_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?),
                "subtract_scalar" => vector.subtract_scalar(f64_field(step, "value")?),
                "subtract_components" => vector.subtract_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "vector_to" => vector.vector_to(f64_vec3(
                    step.get("target").ok_or("target must be an object")?,
                )?),
                "cross" => vector.cross(f64_vec3(
                    step.get("other").ok_or("other must be an object")?,
                )?),
                _ => return Err("unknown Vec3 action".into()),
            };
            events.push(vec3_snapshot(vector));
        }
        return Ok(json!({"events": events}));
    }
    if operation == "vec3_dot" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.dot(second).to_bits())}));
    }
    if operation == "vec3_distance_to_sqr" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.distance_to_sqr(second).to_bits())}));
    }
    if operation == "vec3_distance_to" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.distance_to(second).to_bits())}));
    }
    if operation == "vec3_distance_to_sqr_components" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let value = first.distance_to_sqr_components(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({"value": format!("{:016x}", value.to_bits())}));
    }
    if operation == "vec3_horizontal_distances" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({
            "distance": format!("{:016x}", first.horizontal_distance().to_bits()),
            "distance_squared": format!("{:016x}", first.horizontal_distance_squared().to_bits()),
        }));
    }
    if operation == "vec3_closer_than" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({
            "position": first.closer_than_position(second, f64_field(request, "distance")?),
            "xz_y": first.closer_than_xz_y(
                second,
                f64_field(request, "distance_xz")?,
                f64_field(request, "distance_y")?,
            ),
        }));
    }
    if operation == "vec3_offset_random" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let offset = f32_field(request, "offset")?;
        let seed = random_seed(request)?;
        let mut random = RandomSource::create(seed);
        let offset_random = first.offset_random(&mut random, offset);
        let mut random_xz = RandomSource::create(seed);
        let offset_random_xz = first.offset_random_xz(&mut random_xz, offset);
        return Ok(json!({
            "offset_random": vec3_snapshot(offset_random),
            "offset_random_xz": vec3_snapshot(offset_random_xz),
        }));
    }
    if operation == "vec3_lerp" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        let alpha = f64_field(request, "alpha")?;
        return Ok(json!({"value": vec3_snapshot(first.lerp(second, alpha))}));
    }
    if operation == "vec3_rotate_clockwise_90" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec3_snapshot(first.rotate_clockwise_90())}));
    }
    if operation == "vec3_is_finite" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": first.is_finite()}));
    }
    if operation == "vec3_get" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({"value": format!("{:016x}", first.get(axis).to_bits())}));
    }
    if operation == "vec3_with" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let value = f64_field(request, "value")?;
        return Ok(json!({"value": vec3_snapshot(first.with(axis, value))}));
    }
    if operation == "vec3_relative" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let direction =
            Direction::from_name(string(request, "direction")?).ok_or("unknown direction")?;
        let distance = f64_field(request, "distance")?;
        return Ok(json!({"value": vec3_snapshot(first.relative(direction, distance))}));
    }
    if operation == "vec3_align" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis_values = request["axes"].as_array().ok_or("axes must be an array")?;
        let mut axes = Vec::with_capacity(axis_values.len());
        for axis in axis_values {
            let name = axis.as_str().ok_or("axis must be a string")?;
            axes.push(Axis::from_name(name).ok_or("unknown axis")?);
        }
        return Ok(json!({"value": vec3_snapshot(first.align(&axes))}));
    }
    if operation == "vec3_projected_on" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let onto = f64_vec3(request.get("onto").ok_or("onto must be an object")?)?;
        return Ok(json!({"value": vec3_snapshot(first.projected_on(onto))}));
    }
    if operation == "vec3_to_vector3f" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec3f_snapshot(first.to_vector3f())}));
    }
    if operation == "vec3_coordinates" {
        let pos = blockpos(request)?;
        let value = match string(request, "kind")? {
            "lower_corner" => Vec3::at_lower_corner_of(pos),
            "lower_corner_offset" => Vec3::at_lower_corner_with_offset(
                pos,
                f64_field(request, "offset_x")?,
                f64_field(request, "offset_y")?,
                f64_field(request, "offset_z")?,
            ),
            "center" => Vec3::at_center_of(pos),
            "bottom_center" => Vec3::at_bottom_center_of(pos),
            "up_from_bottom_center" => {
                Vec3::up_from_bottom_center_of(pos, f64_field(request, "y_offset")?)
            }
            _ => return Err("unknown Vec3 coordinate helper".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_equality" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({
            "equals": first.equals_java(second),
            "first_hash": first.hash_code_java().to_string(),
            "second_hash": second.hash_code_java().to_string(),
        }));
    }
    if operation == "vec3_to_string" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": first.to_string_java()}));
    }
    if operation == "vec3_rotate" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let radians = f32_field(request, "radians")?;
        let value = match string(request, "axis")? {
            "x" => first.x_rot(radians),
            "y" => first.y_rot(radians),
            "z" => first.z_rot(radians),
            _ => return Err("unknown Vec3 rotation axis".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_direction_from_rotation" {
        let value = match string(request, "kind")? {
            "components" => Vec3::direction_from_rotation_components(
                f32_field(request, "rot_x")?,
                f32_field(request, "rot_y")?,
            ),
            "vec2" => {
                let rotation = request
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                Vec3::direction_from_rotation(Vec2::new(
                    f32_field(rotation, "x")?,
                    f32_field(rotation, "y")?,
                ))
            }
            _ => return Err("unknown Vec3 direction rotation kind".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_local_coordinates" {
        let value = match string(request, "kind")? {
            "apply" => {
                let rotation = request
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                Vec3::apply_local_coordinates_to_rotation(
                    Vec2::new(f32_field(rotation, "x")?, f32_field(rotation, "y")?),
                    f64_vec3(
                        request
                            .get("direction")
                            .ok_or("direction must be an object")?,
                    )?,
                )
            }
            "add" => {
                let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
                let direction = f64_vec3(
                    request
                        .get("direction")
                        .ok_or("direction must be an object")?,
                )?;
                first.add_local_coordinates(direction)
            }
            _ => return Err("unknown Vec3 local coordinate kind".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_rotation" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec2_snapshot(first.rotation())}));
    }
    if operation == "vec2_rotate" {
        let first = request.get("first").ok_or("first must be an object")?;
        let vector = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(vector.rotate(f64_field(request, "angle")?)),
        }));
    }
    if operation == "vec2_dot" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": format!("{:08x}", first.dot(second).to_bits()),
        }));
    }
    if operation == "vec2_add_vec2" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.add_vec2(second)),
        }));
    }
    if operation == "vec2_add_scalar" {
        let first = request.get("first").ok_or("first must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.add_scalar(f32_field(request, "value")?)),
        }));
    }
    if operation == "vec2_distance_to_sqr" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": format!("{:08x}", first.distance_to_sqr(second).to_bits()),
        }));
    }
    if operation == "vec2_negated" {
        let first = request.get("first").ok_or("first must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.negated()),
        }));
    }
    if operation == "vec2_equality" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "equals": first.equals_java(second),
            "first_hash": first.hash_code_java().to_string(),
            "second_hash": second.hash_code_java().to_string(),
        }));
    }
    if operation == "vec2_constant" {
        let value = match string(request, "name")? {
            "zero" => Vec2::ZERO,
            "one" => Vec2::ONE,
            "unit_x" => Vec2::UNIT_X,
            "neg_unit_x" => Vec2::NEG_UNIT_X,
            "unit_y" => Vec2::UNIT_Y,
            "neg_unit_y" => Vec2::NEG_UNIT_Y,
            "max" => Vec2::MAX,
            "min" => Vec2::MIN,
            _ => return Err("unknown Vec2 constant".into()),
        };
        return Ok(json!({"value": vec2_snapshot(value)}));
    }
    if operation == "vec3_constant" {
        let value = match string(request, "name")? {
            "zero" => Vec3::ZERO,
            "x_axis" => Vec3::X_AXIS,
            "y_axis" => Vec3::Y_AXIS,
            "z_axis" => Vec3::Z_AXIS,
            _ => return Err("unknown Vec3 constant".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    match string(request, "type")? {
        "f32" => {
            let value = f32_field(request, "value")?;
            match operation {
                "floor" => Ok(json!({"value": math::floor_f32(value).to_string()})),
                "wrap_degrees" => {
                    Ok(json!({"bits": format!("{:08x}", math::wrap_degrees_f32(value).to_bits())}))
                }
                "clamp" => Ok(json!({"bits": format!("{:08x}", math::clamp_f32(value,
                    f32_field(request, "min")?, f32_field(request, "max")?).to_bits())})),
                _ => Err("unknown math operation".into()),
            }
        }
        "f64" => {
            let value = f64_field(request, "value")?;
            match operation {
                "floor" => Ok(json!({"value": math::floor_f64(value).to_string()})),
                "wrap_degrees" => {
                    Ok(json!({"bits": format!("{:016x}", math::wrap_degrees_f64(value).to_bits())}))
                }
                "clamp" => Ok(json!({"bits": format!("{:016x}", math::clamp_f64(value,
                    f64_field(request, "min")?, f64_field(request, "max")?).to_bits())})),
                _ => Err("unknown floating-point operation".into()),
            }
        }
        _ => Err("unknown floating-point type".into()),
    }
}

fn timer_snapshot(timer: &Timer) -> Value {
    json!({
        "game_delta": format!("{:08x}", timer.game_time_delta_ticks().to_bits()),
        "partial": format!("{:08x}", timer.game_time_delta_partial_tick(false).to_bits()),
        "partial_ignore_frozen": format!("{:08x}", timer.game_time_delta_partial_tick(true).to_bits()),
        "real_delta": format!("{:08x}", timer.realtime_delta_ticks().to_bits()),
    })
}

fn timer_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? != "trace" {
        return Err("unknown timer operation".into());
    }
    let mut timer = Timer::new(f32_field(request, "tps")?, millis(request, "start_ms")?);
    let mut target = f32_field(request, "target_mspt")?;
    let mut events = vec![timer_snapshot(&timer)];
    let steps = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?;
    for step in steps {
        let ticks = match string(step, "action")? {
            "game" => Some(timer.advance_game_time(millis(step, "ms")?, target)),
            "real" => {
                timer.advance_real_time(millis(step, "ms")?);
                None
            }
            "pause" | "freeze" => {
                let enabled = step["value"].as_bool().ok_or("value must be a boolean")?;
                if step["action"] == "pause" {
                    timer.update_pause_state(enabled);
                } else {
                    timer.update_frozen_state(enabled);
                }
                None
            }
            "target" => {
                target = f32_field(step, "value")?;
                None
            }
            _ => return Err("unknown timer action".into()),
        };
        let mut state = timer_snapshot(&timer);
        if let Some(ticks) = ticks {
            state["ticks"] = json!(ticks);
        }
        events.push(state);
    }
    Ok(json!({"events": events}))
}

fn utf8_result(request: &Value) -> Result<Value, String> {
    let max_length = request["max_length"]
        .as_u64()
        .ok_or("max_length must be an unsigned integer")? as usize;
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match utf8::read_utf8(&mut remaining, max_length) {
                Ok(units) => Ok(json!({
                    "utf16_hex": units.iter().map(|unit| format!("{unit:04x}")).collect::<String>(),
                    "utf16_length": units.len(),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": match error {
                        utf8::ReadError::DecoderError => "decoder_error",
                        utf8::ReadError::BufferUnderflow => "buffer_underflow",
                        utf8::ReadError::TooBig => "too_big",
                    },
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        "write" => {
            let text = string(request, "utf16_hex")?;
            if !text.is_ascii() || !text.len().is_multiple_of(4) {
                return Err("utf16_hex must contain complete ASCII unit quadruples".into());
            }
            let mut units = Vec::with_capacity(text.len() / 4);
            for index in (0..text.len()).step_by(4) {
                let unit = u16::from_str_radix(&text[index..index + 4], 16)
                    .map_err(|_| "invalid UTF-16 unit".to_owned())?;
                units.push(unit);
            }
            let mut buffer = prefix.clone();
            match utf8::write_utf8(&mut buffer, &units, max_length) {
                Ok(()) => Ok(json!({
                    "outcome": "value",
                    "written": buffer.len() - prefix.len(),
                    "hex": hex(&buffer),
                })),
                Err(utf8::WriteError::EncoderError) => Ok(json!({
                    "outcome": "error",
                    "error": "encoder_error",
                    "written": buffer.len() - prefix.len(),
                    "hex": hex(&buffer),
                })),
            }
        }
        _ => Err("unknown UTF-8 operation".into()),
    }
}

fn binary_result(request: &Value) -> Result<Value, String> {
    fn hex_u64(text: &str) -> Result<u64, String> {
        if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("UUID half must be sixteen hexadecimal digits".into());
        }
        u64::from_str_radix(text, 16).map_err(|error| error.to_string())
    }

    fn coordinate(value: &Value, name: &str) -> Result<i32, String> {
        value[name]
            .as_str()
            .ok_or_else(|| format!("{name} must be a string"))?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    }

    fn bit_indices(value: &Value) -> Result<Vec<u64>, String> {
        let bits = value["bits"].as_array().ok_or("bits must be an array")?;
        let mut indices = Vec::with_capacity(bits.len());
        for entry in bits {
            indices.push(
                entry
                    .as_u64()
                    .ok_or("bit index must be an unsigned integer")?,
            );
        }
        Ok(indices)
    }

    fn bit_size(value: &Value) -> Result<usize, String> {
        value["size"]
            .as_u64()
            .ok_or("size must be an unsigned integer".to_owned())
            .map(|size| size as usize)
    }

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    let codec = string(request, "codec")?;
    fn binary_error(error: binary::ReadError) -> &'static str {
        match error {
            binary::ReadError::BufferUnderflow => "buffer_underflow",
            binary::ReadError::DecoderError => "decoder_error",
            binary::ReadError::TooBig => "too_big",
        }
    }
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match codec {
                "uuid" => match binary::read_uuid(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "most_hex": format!("{:016x}", value.most),
                        "least_hex": format!("{:016x}", value.least),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "blockpos" => match binary::read_blockpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "y": value.y.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "sectionpos" => match binary::read_sectionpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "y": value.y.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "chunkpos" => match binary::read_chunkpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "vec3" => match binary::read_vec3(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": format!("{:016x}", value.x.to_bits()),
                        "y": format!("{:016x}", value.y.to_bits()),
                        "z": format!("{:016x}", value.z.to_bits()),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "lp_vec3" => match binary::read_lp_vec3(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": format!("{:016x}", value.x.to_bits()),
                        "y": format!("{:016x}", value.y.to_bits()),
                        "z": format!("{:016x}", value.z.to_bits()),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "longarray" => match binary::read_long_array(&mut remaining) {
                    Ok(values) => Ok(json!({
                        "values": values.iter().map(ToString::to_string).collect::<Vec<_>>(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "bitset" => match binary::read_bitset(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "bits": value.to_bits(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "fixedbitset" => {
                    let size = bit_size(request)?;
                    match binary::read_fixed_bitset(&mut remaining, size) {
                        Ok(value) => Ok(json!({
                            "bits": value.to_bits(),
                            "outcome": "value",
                            "consumed": framed.len() - remaining.len(),
                            "remaining_hex": hex(remaining),
                        })),
                        Err(error) => Ok(json!({
                            "outcome": "error",
                            "error": binary_error(error),
                            "consumed": framed.len() - remaining.len(),
                            "remaining_hex": hex(remaining),
                        })),
                    }
                }
                _ => Err("unknown binary codec".into()),
            }
        }
        "write" => {
            let mut buffer = prefix.clone();
            match codec {
                "uuid" => {
                    let value = Uuid::new(
                        hex_u64(string(request, "most_hex")?)?,
                        hex_u64(string(request, "least_hex")?)?,
                    );
                    binary::write_uuid(value, &mut buffer);
                }
                "blockpos" => {
                    let value = BlockPos::new(
                        coordinate(request, "x")?,
                        coordinate(request, "y")?,
                        coordinate(request, "z")?,
                    );
                    binary::write_blockpos(value, &mut buffer);
                }
                "sectionpos" => {
                    let value = SectionPos::new(
                        coordinate(request, "x")?,
                        coordinate(request, "y")?,
                        coordinate(request, "z")?,
                    );
                    binary::write_sectionpos(value, &mut buffer);
                }
                "chunkpos" => {
                    let value = ChunkPos::new(coordinate(request, "x")?, coordinate(request, "z")?);
                    binary::write_chunkpos(value, &mut buffer);
                }
                "vec3" => {
                    let value = Vec3::new(
                        f64_field(request, "x")?,
                        f64_field(request, "y")?,
                        f64_field(request, "z")?,
                    );
                    binary::write_vec3(value, &mut buffer);
                }
                "lp_vec3" => {
                    let value = Vec3::new(
                        f64_field(request, "x")?,
                        f64_field(request, "y")?,
                        f64_field(request, "z")?,
                    );
                    binary::write_lp_vec3(value, &mut buffer);
                }
                "longarray" => {
                    let values = request["values"]
                        .as_array()
                        .ok_or("values must be an array")?;
                    let mut longs = Vec::with_capacity(values.len());
                    for entry in values {
                        longs.push(
                            entry
                                .as_str()
                                .ok_or("long value must be a string")?
                                .parse::<i64>()
                                .map_err(|error| error.to_string())?,
                        );
                    }
                    binary::write_long_array(&longs, &mut buffer);
                }
                "bitset" => {
                    let indices = bit_indices(request)?;
                    binary::write_bitset(&binary::BitSet::from_bits(&indices), &mut buffer);
                }
                "fixedbitset" => {
                    let size = bit_size(request)?;
                    let indices = bit_indices(request)?;
                    match binary::write_fixed_bitset(
                        &binary::BitSet::from_bits(&indices),
                        size,
                        &mut buffer,
                    ) {
                        Ok(()) => {}
                        Err(binary::WriteError::EncoderError) => {
                            return Ok(json!({
                                "outcome": "error",
                                "error": "encoder_error",
                                "written": buffer.len() - prefix.len(),
                                "hex": hex(&buffer),
                            }));
                        }
                    }
                }
                _ => return Err("unknown binary codec".into()),
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        _ => Err("unknown binary operation".into()),
    }
}

fn nbt_type_name(value: &FieldValue) -> &'static str {
    match value {
        FieldValue::Byte(_) => "byte",
        FieldValue::Short(_) => "short",
        FieldValue::Int(_) => "int",
        FieldValue::Long(_) => "long",
        FieldValue::Float(_) => "float",
        FieldValue::Double(_) => "double",
        FieldValue::ByteArray(_) => "byte-array",
        FieldValue::Str(_) => "string",
        FieldValue::List(_) => "list",
        FieldValue::Compound(_) => "compound",
        FieldValue::IntArray(_) => "int-array",
        FieldValue::LongArray(_) => "long-array",
    }
}

fn nbt_value_json(value: &FieldValue) -> Value {
    match value {
        FieldValue::Byte(value) => json!(value.to_string()),
        FieldValue::Short(value) => json!(value.to_string()),
        FieldValue::Int(value) => json!(value.to_string()),
        FieldValue::Long(value) => json!(value.to_string()),
        FieldValue::Float(value) => json!(format!("{:08x}", value.to_bits())),
        FieldValue::Double(value) => json!(format!("{:016x}", value.to_bits())),
        FieldValue::ByteArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::Str(value) => json!(value),
        FieldValue::List(values) => {
            Value::Array(values.iter().map(nbt_description_json).collect::<Vec<_>>())
        }
        FieldValue::Compound(fields) => {
            let mut object = serde_json::Map::new();
            let mut sorted: Vec<&Field> = fields.iter().collect();
            sorted.sort_by(|a, b| a.key.cmp(&b.key));
            for field in sorted {
                object.insert(field.key.clone(), nbt_description_json(&field.value));
            }
            Value::Object(object)
        }
        FieldValue::IntArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::LongArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
    }
}

fn nbt_description_json(value: &FieldValue) -> Value {
    json!({
        "type": nbt_type_name(value),
        "value": nbt_value_json(value),
    })
}

fn nbt_field_json(field: &Field) -> Value {
    json!({
        "key": field.key,
        "type": nbt_type_name(&field.value),
        "value": nbt_value_json(&field.value),
    })
}

fn nbt_tree_values<T>(node: &Value, value_name: &str) -> Result<Vec<T>, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    node["value"]
        .as_array()
        .ok_or_else(|| format!("{value_name} value must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("{value_name} values must be strings"))?
                .parse::<T>()
                .map_err(|error| error.to_string())
        })
        .collect()
}

fn nbt_tree_value(node: &Value) -> Result<FieldValue, String> {
    match string(node, "type")? {
        "byte" => Ok(FieldValue::Byte(
            string(node, "value")?
                .parse::<i8>()
                .map_err(|error| error.to_string())?,
        )),
        "short" => Ok(FieldValue::Short(
            string(node, "value")?
                .parse::<i16>()
                .map_err(|error| error.to_string())?,
        )),
        "int" => Ok(FieldValue::Int(
            string(node, "value")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        )),
        "long" => Ok(FieldValue::Long(
            string(node, "value")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?,
        )),
        "float" => Ok(FieldValue::Float(f32::from_bits(
            u32::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "double" => Ok(FieldValue::Double(f64::from_bits(
            u64::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "byte-array" => Ok(FieldValue::ByteArray(nbt_tree_values(node, "byte-array")?)),
        "string" => Ok(FieldValue::Str(string(node, "value")?.to_owned())),
        "list" => Ok(FieldValue::List(
            node["value"]
                .as_array()
                .ok_or("list value must be an array")?
                .iter()
                .map(nbt_tree_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "compound" => {
            let object = node["value"]
                .as_object()
                .ok_or("compound value must be an object")?;
            let fields = object
                .iter()
                .map(|(key, value)| {
                    Ok(Field {
                        key: key.clone(),
                        value: nbt_tree_value(value)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(FieldValue::Compound(fields))
        }
        "int-array" => Ok(FieldValue::IntArray(nbt_tree_values(node, "int-array")?)),
        "long-array" => Ok(FieldValue::LongArray(nbt_tree_values(node, "long-array")?)),
        _ => Err("unknown NBT field type".into()),
    }
}

fn nbt_result(request: &Value) -> Result<Value, String> {
    fn nbt_error(error: nbt::ReadError) -> &'static str {
        match error {
            nbt::ReadError::BufferUnderflow => "buffer_underflow",
            nbt::ReadError::DecoderError => "decoder_error",
            nbt::ReadError::EncoderError => "encoder_error",
            nbt::ReadError::TooBig => "too_big",
        }
    }

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match nbt::read_nbt(&mut remaining) {
                Ok(None) => Ok(json!({
                    "null": true,
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Ok(Some(fields)) => {
                    let mut out: Vec<Value> = fields.iter().map(nbt_field_json).collect();
                    out.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
                    Ok(json!({
                        "fields": out,
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    }))
                }
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": nbt_error(error),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        "write" => {
            let mut buffer = prefix.clone();
            if request.get("null_root").and_then(Value::as_bool) == Some(true) {
                nbt::write_nbt(None, &mut buffer);
            } else if let Some(tree) = request.get("tree") {
                let value = nbt_tree_value(tree)?;
                let FieldValue::Compound(fields) = value else {
                    return Err("NBT tree root must be a compound".into());
                };
                nbt::write_nbt(Some(&fields), &mut buffer);
            } else {
                let fields = request["fields"]
                    .as_array()
                    .ok_or("fields must be an array")?;
                let mut root = Vec::with_capacity(fields.len());
                for entry in fields {
                    let key = string(entry, "key")?.to_owned();
                    let value = match string(entry, "type")? {
                        "byte" => FieldValue::Byte(
                            string(entry, "value")?
                                .parse::<i8>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "short" => FieldValue::Short(
                            string(entry, "value")?
                                .parse::<i16>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "int" => FieldValue::Int(
                            string(entry, "value")?
                                .parse::<i32>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "long" => FieldValue::Long(
                            string(entry, "value")?
                                .parse::<i64>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "float" => FieldValue::Float(f32::from_bits(
                            u32::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "double" => FieldValue::Double(f64::from_bits(
                            u64::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "string" => FieldValue::Str(string(entry, "value")?.to_owned()),
                        _ => return Err("unknown NBT field type".into()),
                    };
                    root.push(Field { key, value });
                }
                nbt::write_nbt(Some(&root), &mut buffer);
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        _ => Err("unknown NBT operation".into()),
    }
}

fn respond(request: Value) -> Result<Value, String> {
    if request["schema_version"].as_u64() != Some(1) {
        return Err("unsupported schema_version".into());
    }
    let case_id = string(&request, "case_id")?;
    let result = match string(&request, "family")? {
        "wire" => wire_result(&request)?,
        "timer" => timer_result(&request)?,
        "math" => math_result(&request)?,
        "utf8" => utf8_result(&request)?,
        "binary" => binary_result(&request)?,
        "nbt" => nbt_result(&request)?,
        _ => return Err("unknown behavior family".into()),
    };
    Ok(json!({"schema_version": 1, "case_id": case_id, "result": result}))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = io::stdin().lock();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (index, line) in input.lines().enumerate() {
        let request = serde_json::from_str(&line?)?;
        let response = respond(request)
            .map_err(|message| io::Error::other(format!("line {}: {message}", index + 1)))?;
        serde_json::to_writer(&mut output, &response)?;
        writeln!(output)?;
    }
    output.flush()?;
    Ok(())
}
