//! JSONL test glue. Behavior is delegated to the production library.

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use voxel_client::aabb::{Aabb, AabbBuilder, AabbBuilderError, BlockHitResult, BoundingBox};
use voxel_client::binary::{self, BlockPos, ChunkPos, SectionPos, Uuid};
use voxel_client::direction::{
    Axis, AxisDirection, Direction, DirectionYRotError, DirectionYRotationError, Plane,
};
use voxel_client::math::{self, Quaternionf, Vec2, Vec3, Vec3f};
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

fn quaternion_snapshot(quaternion: Quaternionf) -> Value {
    json!({
        "x": format!("{:08x}", quaternion.x.to_bits()),
        "y": format!("{:08x}", quaternion.y.to_bits()),
        "z": format!("{:08x}", quaternion.z.to_bits()),
        "w": format!("{:08x}", quaternion.w.to_bits()),
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

fn sectionpos(value: &Value) -> Result<SectionPos, String> {
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

fn chunkpos_snapshot(value: ChunkPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "z": value.z.to_string(),
    })
}

fn sectionpos_snapshot(value: SectionPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
        "packed": format!("{:x}", value.as_long()),
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

fn sectionpos_block_coordinates_snapshot(value: SectionPos) -> Value {
    json!({
        "min_block_x": value.min_block_x().to_string(),
        "min_block_y": value.min_block_y().to_string(),
        "min_block_z": value.min_block_z().to_string(),
        "max_block_x": value.max_block_x().to_string(),
        "max_block_y": value.max_block_y().to_string(),
        "max_block_z": value.max_block_z().to_string(),
    })
}

fn sectionpos_spatial_snapshot(value: SectionPos) -> Value {
    json!({
        "origin": blockpos_snapshot(value.origin()),
        "center": blockpos_snapshot(value.center()),
        "chunk": chunkpos_snapshot(value.chunk()),
    })
}

fn sectionpos_packed_transformations_snapshot(
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

fn sectionpos_as_long_snapshot(
    block_node: u64,
    x: i32,
    y: i32,
    z: i32,
    section_x: i32,
    section_y: i32,
    section_z: i32,
) -> Value {
    json!({
        "from_block_pos": format!("{:x}", SectionPos::as_long_from_block_pos(BlockPos::unpack(block_node))),
        "coordinates": format!("{:x}", SectionPos::as_long_coordinates(x, y, z)),
        "instance": format!("{:x}", SectionPos::new(section_x, section_y, section_z).as_long()),
    })
}

fn sectionpos_offset_snapshot(
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

fn sectionpos_ranges_snapshot(values: Vec<SectionPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(sectionpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn blockpos_positions_snapshot(values: Vec<BlockPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(blockpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn sectionpos_nodes_snapshot(values: Vec<u64>) -> Value {
    json!({
        "section_nodes": values
            .into_iter()
            .map(|value| format!("{value:x}"))
            .collect::<Vec<_>>(),
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

fn i32_string_field(value: &Value, name: &str) -> Result<i32, String> {
    string(value, name)?
        .parse::<i32>()
        .map_err(|error| format!("{name} must fit in i32: {error}"))
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
    if operation == "sectionpos_block_coordinates" {
        return Ok(json!({
            "value": sectionpos_block_coordinates_snapshot(sectionpos(request)?),
        }));
    }
    if operation == "sectionpos_relative_to_block" {
        let value = sectionpos(request)?;
        let relative = string(request, "relative")?
            .parse::<i16>()
            .map_err(|error| error.to_string())? as u16;
        return Ok(json!({
            "value": {
                "relative_x": SectionPos::section_relative_x(relative).to_string(),
                "relative_y": SectionPos::section_relative_y(relative).to_string(),
                "relative_z": SectionPos::section_relative_z(relative).to_string(),
                "block_x": value.relative_to_block_x(relative).to_string(),
                "block_y": value.relative_to_block_y(relative).to_string(),
                "block_z": value.relative_to_block_z(relative).to_string(),
                "block_position": blockpos_snapshot(value.relative_to_block_pos(relative)),
            },
        }));
    }
    if operation == "sectionpos_spatial" {
        return Ok(json!({
            "value": sectionpos_spatial_snapshot(sectionpos(request)?),
        }));
    }
    if operation == "sectionpos_packed_transformations" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        return Ok(json!({
            "value": sectionpos_packed_transformations_snapshot(
                u64_hex_field(request, "block_node")?,
                u64_hex_field(request, "section_node")?,
                coordinate("x")?,
                coordinate("z")?,
            ),
        }));
    }
    if operation == "sectionpos_as_long" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        return Ok(json!({
            "value": sectionpos_as_long_snapshot(
                u64_hex_field(request, "block_node")?,
                coordinate("x")?,
                coordinate("y")?,
                coordinate("z")?,
                coordinate("section_x")?,
                coordinate("section_y")?,
                coordinate("section_z")?,
            ),
        }));
    }
    if operation == "sectionpos_offset" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let direction = Direction::from_name(string(request, "direction")?)
            .ok_or("unknown SectionPos offset direction")?;
        return Ok(json!({
            "value": sectionpos_offset_snapshot(
                u64_hex_field(request, "section_node")?,
                direction,
                (
                    coordinate("step_x")?,
                    coordinate("step_y")?,
                    coordinate("step_z")?,
                ),
                SectionPos::new(coordinate("x")?, coordinate("y")?, coordinate("z")?),
            ),
        }));
    }
    if operation == "sectionpos_ranges" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let positions = match string(request, "mode")? {
            "between" => SectionPos::between_closed(
                coordinate("min_x")?,
                coordinate("min_y")?,
                coordinate("min_z")?,
                coordinate("max_x")?,
                coordinate("max_y")?,
                coordinate("max_z")?,
            ),
            "cube" => SectionPos::cube(
                SectionPos::new(coordinate("x")?, coordinate("y")?, coordinate("z")?),
                coordinate("radius")?,
            ),
            "around-chunk" => SectionPos::around_chunk(
                ChunkPos::new(coordinate("chunk_x")?, coordinate("chunk_z")?),
                coordinate("radius")?,
                coordinate("min_section")?,
                coordinate("max_section")?,
            ),
            _ => return Err("unknown SectionPos range mode".into()),
        };
        return Ok(json!({
            "value": sectionpos_ranges_snapshot(positions),
        }));
    }
    if operation == "sectionpos_blocks_inside" {
        return Ok(json!({
            "value": blockpos_positions_snapshot(sectionpos(request)?.blocks_inside()),
        }));
    }
    if operation == "sectionpos_around_and_at_block_pos" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let section_nodes = match string(request, "mode")? {
            "coordinates" => SectionPos::around_and_at_coordinates(
                coordinate("x")?,
                coordinate("y")?,
                coordinate("z")?,
            ),
            "blockpos" => SectionPos::around_and_at_block_pos(blockpos(request)?),
            "packed" => SectionPos::around_and_at_packed(u64_hex_field(request, "block_node")?),
            _ => return Err("unknown SectionPos around-and-at mode".into()),
        };
        return Ok(json!({
            "value": sectionpos_nodes_snapshot(section_nodes),
        }));
    }
    if operation == "sectionpos_double_coordinates" {
        let value = f64_field(request, "value")?;
        let result = match string(request, "fn")? {
            "pos_to_section" => SectionPos::pos_to_section_coord(value),
            "block_to_section" => SectionPos::block_to_section_coord_f64(value),
            _ => return Err("unknown SectionPos double coordinate function".into()),
        };
        return Ok(json!({"value": result.to_string()}));
    }
    if operation == "sectionpos_of_overloads" {
        let value = match string(request, "mode")? {
            "blockpos" => SectionPos::from_block_pos(blockpos(request)?),
            "chunkpos" => SectionPos::from_chunk_pos(
                chunkpos(request)?,
                string(request, "section_y")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
            ),
            "position" => SectionPos::from_position(Vec3::new(
                f64_field(request, "x")?,
                f64_field(request, "y")?,
                f64_field(request, "z")?,
            )),
            "packed" => SectionPos::unpack(u64_hex_field(request, "section_node")?),
            _ => return Err("unknown SectionPos of overload mode".into()),
        };
        return Ok(json!({"value": sectionpos_snapshot(value)}));
    }
    if operation == "chunkpos_validity" {
        return Ok(json!({
            "value": chunkpos_validity_snapshot(chunkpos(request)?),
        }));
    }
    if operation == "mth_floating_primitives" {
        let function = string(request, "fn")?;
        let bits = match function {
            "sin" => format!(
                "{:08x}",
                math::mth_sin_f64(f64_field(request, "value")?).to_bits()
            ),
            "cos" => format!(
                "{:08x}",
                math::mth_cos_f64(f64_field(request, "value")?).to_bits()
            ),
            "sqrt" => format!(
                "{:08x}",
                math::mth_sqrt_f32(f32_field(request, "value")?).to_bits()
            ),
            "abs" => format!(
                "{:08x}",
                math::mth_abs_f32(f32_field(request, "value")?).to_bits()
            ),
            "clamp" => match string(request, "type")? {
                "f32" => format!(
                    "{:08x}",
                    math::clamp_f32(
                        f32_field(request, "value")?,
                        f32_field(request, "min")?,
                        f32_field(request, "max")?,
                    )
                    .to_bits()
                ),
                "f64" => format!(
                    "{:016x}",
                    math::clamp_f64(
                        f64_field(request, "value")?,
                        f64_field(request, "min")?,
                        f64_field(request, "max")?,
                    )
                    .to_bits()
                ),
                _ => return Err("unknown Mth clamp type".into()),
            },
            _ => return Err("unknown Mth floating primitive".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_constants" {
        return match string(request, "name")? {
            "pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_PI.to_bits())})),
            "half_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_HALF_PI.to_bits())})),
            "two_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_TWO_PI.to_bits())})),
            "deg_to_rad" => Ok(json!({"bits": format!("{:08x}", math::MTH_DEG_TO_RAD.to_bits())})),
            "rad_to_deg" => Ok(json!({"bits": format!("{:08x}", math::MTH_RAD_TO_DEG.to_bits())})),
            "epsilon" => Ok(json!({"bits": format!("{:08x}", math::MTH_EPSILON.to_bits())})),
            "sqrt_of_two" => {
                Ok(json!({"bits": format!("{:08x}", math::MTH_SQRT_OF_TWO.to_bits())}))
            }
            "y_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Y_AXIS)})),
            "x_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_X_AXIS)})),
            "z_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Z_AXIS)})),
            _ => Err("unknown Mth constant".into()),
        };
    }
    if operation == "mth_power_helpers" {
        let function = string(request, "fn")?;
        let input = string(request, "input")?;
        let value = match string(request, "type")? {
            "i32" => {
                let input = input.parse::<i32>().map_err(|error| error.to_string())?;
                match function {
                    "smallest_encompassing_power_of_two" => {
                        json!(math::smallest_encompassing_power_of_two(input).to_string())
                    }
                    "smallest_square_side" => {
                        json!(math::smallest_square_side(input)?.to_string())
                    }
                    "is_power_of_two" => json!(math::is_power_of_two_i32(input)),
                    "ceillog2" => json!(math::ceillog2(input).to_string()),
                    "log2" => json!(math::log2(input).to_string()),
                    _ => return Err("unknown Mth i32 power helper".into()),
                }
            }
            "i64" => {
                let input = input.parse::<i64>().map_err(|error| error.to_string())?;
                if function != "is_power_of_two" {
                    return Err("unknown Mth i64 power helper".into());
                }
                json!(math::is_power_of_two_i64(input))
            }
            _ => return Err("unknown Mth power helper type".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "mth_frac" {
        if string(request, "fn")? != "frac" {
            return Err("unknown Mth frac function".into());
        }
        let bits = match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::frac_f32(f32_field(request, "input")?).to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::frac_f64(f64_field(request, "input")?).to_bits()
            ),
            _ => return Err("unknown Mth frac type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_create_insecure_uuid" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let mut random = RandomSource::create(seed);
        let uuid = math::mth_create_insecure_uuid(&mut random);
        return Ok(json!({
            "most_hex": format!("{:016x}", uuid.most),
            "least_hex": format!("{:016x}", uuid.least),
        }));
    }
    if operation == "mth_murmur_hash3_mixer" {
        let input = string(request, "input")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": math::mth_murmur_hash3_mixer(input).to_string(),
        }));
    }
    if operation == "mth_binary_search" {
        let from = string(request, "from")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let to = string(request, "to")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let threshold = string(request, "threshold")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let value = match string(request, "condition")? {
            "ge" => math::mth_binary_search(from, to, |value| value >= threshold),
            "gt" => math::mth_binary_search(from, to, |value| value > threshold),
            _ => return Err("unknown Mth binary search condition".into()),
        };
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_ray_intersects_aabb" {
        let ray_start = f64_vec3(
            request
                .get("ray_start")
                .ok_or("ray_start must be an object")?,
        )?;
        let ray_dir = f64_vec3(request.get("ray_dir").ok_or("ray_dir must be an object")?)?;
        let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
        return Ok(json!({
            "value": math::mth_ray_intersects_aabb(ray_start, ray_dir, aabb),
        }));
    }
    if operation == "mth_fast_inv_sqrt" {
        let bits = math::mth_fast_inv_sqrt_f64(f64_field(request, "input")?).to_bits();
        return Ok(json!({"bits": format!("{bits:016x}")}));
    }
    if operation == "mth_fast_inv_cube_root" {
        let bits = math::mth_fast_inv_cube_root_f32(f32_field(request, "input")?).to_bits();
        return Ok(json!({"bits": format!("{bits:08x}")}));
    }
    if operation == "mth_hsv_to_color" {
        let hue = f32_field(request, "hue")?;
        let saturation = f32_field(request, "saturation")?;
        let value = f32_field(request, "value")?;
        let alpha = string(request, "alpha")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let color = match string(request, "mode")? {
            "rgb" => math::mth_hsv_to_rgb_f32(hue, saturation, value),
            "argb" => math::mth_hsv_to_argb_f32(hue, saturation, value, alpha),
            _ => return Err("unknown Mth HSV color mode".into()),
        };
        return Ok(match color {
            Ok(value) => json!({"value": value.to_string()}),
            Err(error) => json!({"outcome": "error", "error": error}),
        });
    }
    if operation == "mth_angle_wrap_difference" {
        if string(request, "type")? != "f32" {
            return Err("unknown Mth angle wrap/difference type".into());
        }
        let value = match string(request, "fn")? {
            "wrap_degrees_90" => math::mth_wrap_degrees_90_f32(f32_field(request, "value")?),
            "degrees_difference" => math::mth_degrees_difference_f32(
                f32_field(request, "from")?,
                f32_field(request, "to")?,
            ),
            "degrees_difference_abs" => math::mth_degrees_difference_abs_f32(
                f32_field(request, "from")?,
                f32_field(request, "to")?,
            ),
            _ => return Err("unknown Mth angle wrap/difference function".into()),
        };
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_angle_approach" {
        if string(request, "type")? != "f32" {
            return Err("unknown Mth angle approach type".into());
        }
        let value = match string(request, "fn")? {
            "rotate_if_necessary" => math::mth_rotate_if_necessary_f32(
                f32_field(request, "base")?,
                f32_field(request, "target")?,
                f32_field(request, "max")?,
            ),
            "approach" => math::mth_approach_f32(
                f32_field(request, "current")?,
                f32_field(request, "target")?,
                f32_field(request, "increment")?,
            ),
            "approach_degrees" => math::mth_approach_degrees_f32(
                f32_field(request, "current")?,
                f32_field(request, "target")?,
                f32_field(request, "increment")?,
            ),
            _ => return Err("unknown Mth angle approach function".into()),
        };
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_get_int" {
        let input = if request["input"].is_null() {
            None
        } else {
            Some(string(request, "input")?)
        };
        let default = string(request, "default")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({"value": math::mth_get_int(input, default).to_string()}));
    }
    if operation == "mth_get_seed" {
        let value = math::mth_get_seed(
            i32_field(request, "x")?,
            i32_field(request, "y")?,
            i32_field(request, "z")?,
        );
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_atan2" {
        let value = math::mth_atan2(f64_field(request, "y")?, f64_field(request, "x")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_inv_sqrt" {
        let bits = match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::mth_inv_sqrt_f32(f32_field(request, "input")?).to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::mth_inv_sqrt_f64(f64_field(request, "input")?).to_bits()
            ),
            _ => return Err("unknown Mth invSqrt type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_integer_helpers" {
        let function = string(request, "fn")?;
        let input = string(request, "input")?;
        let divisor = string(request, "divisor")?;
        let value = match string(request, "type")? {
            "i32" => {
                let input = input.parse::<i32>().map_err(|error| error.to_string())?;
                let divisor = divisor.parse::<i32>().map_err(|error| error.to_string())?;
                match function {
                    "floor_div" => math::floor_div_i32(input, divisor),
                    "positive_modulo" => math::positive_modulo_i32(input, divisor),
                    "positive_ceil_div" => math::positive_ceil_div_i32(input, divisor),
                    "round_toward" => math::round_toward_i32(input, divisor),
                    _ => return Err("unknown Mth i32 function".into()),
                }
                .to_string()
            }
            "i64" => {
                let input = input.parse::<i64>().map_err(|error| error.to_string())?;
                let divisor = divisor.parse::<i64>().map_err(|error| error.to_string())?;
                match function {
                    "positive_ceil_div" => math::positive_ceil_div_i64(input, divisor),
                    "round_toward" => math::round_toward_i64(input, divisor),
                    _ => return Err("unknown Mth i64 function".into()),
                }
                .to_string()
            }
            _ => return Err("unknown Mth integer type".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "mth_integer_basics" {
        let function = string(request, "fn")?;
        let input = string(request, "input")?;
        let value = match string(request, "type")? {
            "i32" => {
                let input = input.parse::<i32>().map_err(|error| error.to_string())?;
                match function {
                    "abs" => json!(math::abs_i32(input).to_string()),
                    "clamp" => json!(
                        math::clamp_i32(
                            input,
                            string(request, "min")?
                                .parse::<i32>()
                                .map_err(|error| error.to_string())?,
                            string(request, "max")?
                                .parse::<i32>()
                                .map_err(|error| error.to_string())?,
                        )
                        .to_string()
                    ),
                    "is_multiple" => json!(math::is_multiple_of_i32(
                        input,
                        string(request, "divisor")?
                            .parse::<i32>()
                            .map_err(|error| error.to_string())?,
                    )),
                    "wrap_degrees" => json!(math::wrap_degrees_i32(input).to_string()),
                    _ => return Err("unknown Mth i32 basic function".into()),
                }
            }
            "i64" => {
                let input = input.parse::<i64>().map_err(|error| error.to_string())?;
                if function != "clamp" {
                    return Err("unknown Mth i64 basic function".into());
                }
                json!(
                    math::clamp_i64(
                        input,
                        string(request, "min")?
                            .parse::<i64>()
                            .map_err(|error| error.to_string())?,
                        string(request, "max")?
                            .parse::<i64>()
                            .map_err(|error| error.to_string())?,
                    )
                    .to_string()
                )
            }
            _ => return Err("unknown Mth integer basic type".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "mth_angle_helpers" {
        let function = string(request, "fn")?;
        let input = string(request, "input")?;
        let result = match function {
            "pack_degrees" => {
                if string(request, "type")? != "f32" {
                    return Err("unknown Mth packDegrees type".into());
                }
                let value = f32_field(request, "input")?;
                json!({"value": math::pack_degrees_f32(value).to_string()})
            }
            "unpack_degrees" => {
                if string(request, "type")? != "i8" {
                    return Err("unknown Mth unpackDegrees type".into());
                }
                let value = input.parse::<i8>().map_err(|error| error.to_string())?;
                json!({"bits": format!("{:08x}", math::unpack_degrees_i8(value).to_bits())})
            }
            "wrap_degrees" => {
                if string(request, "type")? != "i64" {
                    return Err("unknown Mth long wrapDegrees type".into());
                }
                let value = input.parse::<i64>().map_err(|error| error.to_string())?;
                json!({"bits": format!("{:08x}", math::wrap_degrees_i64(value).to_bits())})
            }
            _ => return Err("unknown Mth angle function".into()),
        };
        return Ok(result);
    }
    if operation == "mth_rounding_helpers" {
        let width = string(request, "type")?;
        let function = string(request, "fn")?;
        let value = match (function, width) {
            ("ceil", "f32") => math::ceil_f32(f32_field(request, "input")?).to_string(),
            ("ceil", "f64") => math::ceil_f64(f64_field(request, "input")?).to_string(),
            ("lfloor", "f64") => math::lfloor_f64(f64_field(request, "input")?).to_string(),
            ("ceil_long", "f64") => math::ceil_long_f64(f64_field(request, "input")?).to_string(),
            ("ceil", _) => return Err("unknown Mth ceil type".into()),
            ("lfloor", _) => return Err("unknown Mth lfloor type".into()),
            ("ceil_long", _) => return Err("unknown Mth ceilLong type".into()),
            _ => return Err("unknown Mth rounding function".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "mth_positive_modulo" {
        let width = string(request, "type")?;
        let bits = match width {
            "f32" => format!(
                "{:08x}",
                math::positive_modulo_f32(f32_field(request, "input")?, f32_field(request, "mod")?)
                    .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::positive_modulo_f64(f64_field(request, "input")?, f64_field(request, "mod")?)
                    .to_bits()
            ),
            _ => return Err("unknown Mth positiveModulo type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_abs_max" {
        let width = string(request, "type")?;
        let bits = match width {
            "f32" => format!(
                "{:08x}",
                math::abs_max_f32(f32_field(request, "first")?, f32_field(request, "second")?)
                    .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::abs_max_f64(f64_field(request, "first")?, f64_field(request, "second")?)
                    .to_bits()
            ),
            _ => return Err("unknown Mth absMax type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_equal" {
        let width = string(request, "type")?;
        let equal = match width {
            "f32" => math::equal_f32(f32_field(request, "first")?, f32_field(request, "second")?),
            "f64" => math::equal_f64(f64_field(request, "first")?, f64_field(request, "second")?),
            _ => return Err("unknown Mth equal type".into()),
        };
        return Ok(json!({"equal": equal}));
    }
    if operation == "mth_clamped_lerp" {
        let width = string(request, "type")?;
        let bits = match width {
            "f32" => format!(
                "{:08x}",
                math::clamped_lerp_f32(
                    f32_field(request, "factor")?,
                    f32_field(request, "min")?,
                    f32_field(request, "max")?,
                )
                .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::clamped_lerp_f64(
                    f64_field(request, "factor")?,
                    f64_field(request, "min")?,
                    f64_field(request, "max")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth clampedLerp type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_clamped_map_f64" {
        let value = math::mth_clamped_map_f64(
            f64_field(request, "value")?,
            f64_field(request, "from_min")?,
            f64_field(request, "from_max")?,
            f64_field(request, "to_min")?,
            f64_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_clamped_map_f32" {
        let value = math::mth_clamped_map_f32(
            f32_field(request, "value")?,
            f32_field(request, "from_min")?,
            f32_field(request, "from_max")?,
            f32_field(request, "to_min")?,
            f32_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_map_f64" {
        let value = math::mth_map_f64(
            f64_field(request, "value")?,
            f64_field(request, "from_min")?,
            f64_field(request, "from_max")?,
            f64_field(request, "to_min")?,
            f64_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_map_f32" {
        let value = math::mth_map_f32(
            f32_field(request, "value")?,
            f32_field(request, "from_min")?,
            f32_field(request, "from_max")?,
            f32_field(request, "to_min")?,
            f32_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_wobble_f64" {
        let value = math::mth_wobble_f64(f64_field(request, "coord")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_interpolation" {
        let function = string(request, "fn")?;
        let bits = match function {
            "lerp2" => format!(
                "{:016x}",
                math::mth_lerp2_f64(
                    f64_field(request, "alpha1")?,
                    f64_field(request, "alpha2")?,
                    f64_field(request, "x00")?,
                    f64_field(request, "x10")?,
                    f64_field(request, "x01")?,
                    f64_field(request, "x11")?,
                )
                .to_bits()
            ),
            "lerp3" => format!(
                "{:016x}",
                math::mth_lerp3_f64(
                    f64_field(request, "alpha1")?,
                    f64_field(request, "alpha2")?,
                    f64_field(request, "alpha3")?,
                    f64_field(request, "x000")?,
                    f64_field(request, "x100")?,
                    f64_field(request, "x010")?,
                    f64_field(request, "x110")?,
                    f64_field(request, "x001")?,
                    f64_field(request, "x101")?,
                    f64_field(request, "x011")?,
                    f64_field(request, "x111")?,
                )
                .to_bits()
            ),
            "catmullrom" => format!(
                "{:08x}",
                math::mth_catmullrom_f32(
                    f32_field(request, "alpha")?,
                    f32_field(request, "p0")?,
                    f32_field(request, "p1")?,
                    f32_field(request, "p2")?,
                    f32_field(request, "p3")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth interpolation function".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_next_random" {
        let mut random = RandomSource::create(random_seed(request)?);
        let width = string(request, "type")?;
        return match width {
            "i32" => {
                let value = math::mth_next_int(
                    &mut random,
                    i32_string_field(request, "min")?,
                    i32_string_field(request, "max")?,
                )
                .map_err(|_| "bound must be positive".to_owned())?;
                Ok(json!({"value": value.to_string()}))
            }
            "f32" => Ok(json!({
                "bits": format!(
                    "{:08x}",
                    math::mth_next_float(
                        &mut random,
                        f32_field(request, "min")?,
                        f32_field(request, "max")?,
                    )
                    .to_bits()
                )
            })),
            "f64" => Ok(json!({
                "bits": format!(
                    "{:016x}",
                    math::mth_next_double(
                        &mut random,
                        f64_field(request, "min")?,
                        f64_field(request, "max")?,
                    )
                    .to_bits()
                )
            })),
            _ => Err("unknown Mth next random type".into()),
        };
    }
    if operation == "mth_random_helpers" {
        let mut random = RandomSource::create(random_seed(request)?);
        return match string(request, "fn")? {
            "inclusive" => {
                let value = math::mth_random_between_inclusive(
                    &mut random,
                    i32_string_field(request, "min")?,
                    i32_string_field(request, "max")?,
                )
                .map_err(|_| "bound must be positive".to_owned())?;
                Ok(json!({
                    "value": value.to_string(),
                    "follow_up": random.next_int().to_string(),
                }))
            }
            "between" => {
                let value = math::mth_random_between(
                    &mut random,
                    f32_field(request, "min")?,
                    f32_field(request, "max")?,
                );
                Ok(json!({
                    "bits": format!("{:08x}", value.to_bits()),
                    "follow_up_bits": format!("{:08x}", random.next_float().to_bits()),
                }))
            }
            "normal" => {
                let value = math::mth_normal(
                    &mut random,
                    f32_field(request, "mean")?,
                    f32_field(request, "deviation")?,
                );
                Ok(json!({
                    "bits": format!("{:08x}", value.to_bits()),
                    "follow_up_bits": format!("{:016x}", random.next_gaussian().to_bits()),
                }))
            }
            _ => Err("unknown Mth random helper".into()),
        };
    }
    if operation == "mth_length" {
        let width = string(request, "type")?;
        let function = string(request, "fn")?;
        let bits = match (width, function) {
            ("f64_2", "length_squared") => format!(
                "{:016x}",
                math::mth_length_squared_f64_2(f64_field(request, "x")?, f64_field(request, "y")?,)
                    .to_bits()
            ),
            ("f64_2", "length") => format!(
                "{:016x}",
                math::mth_length_f64_2(f64_field(request, "x")?, f64_field(request, "y")?,)
                    .to_bits()
            ),
            ("f32_2", "length") => format!(
                "{:08x}",
                math::mth_length_f32_2(f32_field(request, "x")?, f32_field(request, "y")?,)
                    .to_bits()
            ),
            ("f64_3", "length_squared") => format!(
                "{:016x}",
                math::mth_length_squared_f64_3(
                    f64_field(request, "x")?,
                    f64_field(request, "y")?,
                    f64_field(request, "z")?,
                )
                .to_bits()
            ),
            ("f64_3", "length") => format!(
                "{:016x}",
                math::mth_length_f64_3(
                    f64_field(request, "x")?,
                    f64_field(request, "y")?,
                    f64_field(request, "z")?,
                )
                .to_bits()
            ),
            ("f32_3", "length_squared") => format!(
                "{:08x}",
                math::mth_length_squared_f32_3(
                    f32_field(request, "x")?,
                    f32_field(request, "y")?,
                    f32_field(request, "z")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth length overload".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_quantize" {
        let value = f64_field(request, "value")?;
        let resolution = string(request, "resolution")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": math::mth_quantize(value, resolution).to_string()
        }));
    }
    if operation == "mth_out_from_origin" {
        let origin = i32_string_field(request, "origin")?;
        let lower_bound = i32_string_field(request, "lower")?;
        let upper_bound = i32_string_field(request, "upper")?;
        let step_size = match string(request, "mode")? {
            "default" => 1,
            "step" => i32_string_field(request, "step")?,
            _ => return Err("unknown Mth outFromOrigin mode".into()),
        };
        let result = match math::mth_out_from_origin(origin, lower_bound, upper_bound, step_size) {
            Ok(values) => json!({"outcome": "value", "value": values}),
            Err(error) => json!({"outcome": "error", "error": error}),
        };
        return Ok(result);
    }
    if operation == "mth_mul_and_truncate" {
        let numerator = i32_string_field(request, "numerator")?;
        let denominator = i32_string_field(request, "denominator")?;
        let factor = i32_string_field(request, "factor")?;
        return Ok(json!({
            "value": math::mth_mul_and_truncate(numerator, denominator, factor).to_string()
        }));
    }
    if operation == "mth_rotation_around_axis" {
        let axis = Vec3f::new(
            f32_field(request, "axis_x")?,
            f32_field(request, "axis_y")?,
            f32_field(request, "axis_z")?,
        );
        let rotation = Quaternionf::new(
            f32_field(request, "rotation_x")?,
            f32_field(request, "rotation_y")?,
            f32_field(request, "rotation_z")?,
            f32_field(request, "rotation_w")?,
        );
        return Ok(json!({
            "value": quaternion_snapshot(math::mth_rotation_around_axis(axis, rotation))
        }));
    }
    if operation == "mth_inverse_lerp" {
        let bits = match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::mth_inverse_lerp_f32(
                    f32_field(request, "value")?,
                    f32_field(request, "min")?,
                    f32_field(request, "max")?,
                )
                .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::mth_inverse_lerp_f64(
                    f64_field(request, "value")?,
                    f64_field(request, "min")?,
                    f64_field(request, "max")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth inverseLerp type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_lerp" {
        let bits = match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::mth_lerp_f32(
                    f32_field(request, "alpha")?,
                    f32_field(request, "p0")?,
                    f32_field(request, "p1")?,
                )
                .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::mth_lerp_f64(
                    f64_field(request, "alpha")?,
                    f64_field(request, "p0")?,
                    f64_field(request, "p1")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth lerp type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_lerp_int" || operation == "mth_lerp_discrete" {
        let alpha = f32_field(request, "alpha")?;
        let start = i32_string_field(request, "p0")?;
        let end = i32_string_field(request, "p1")?;
        let value = if operation == "mth_lerp_int" {
            math::mth_lerp_int(alpha, start, end)
        } else {
            math::mth_lerp_discrete(alpha, start, end)
        };
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_smoothstep" || operation == "mth_smoothstep_derivative" {
        let x = f64_field(request, "x")?;
        let value = if operation == "mth_smoothstep" {
            math::mth_smoothstep(x)
        } else {
            math::mth_smoothstep_derivative(x)
        };
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_sign" {
        let value = math::mth_sign(f64_field(request, "x")?);
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_rot_lerp" {
        let bits = match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::mth_rot_lerp_f32(
                    f32_field(request, "alpha")?,
                    f32_field(request, "from")?,
                    f32_field(request, "to")?,
                )
                .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::mth_rot_lerp_f64(
                    f64_field(request, "alpha")?,
                    f64_field(request, "from")?,
                    f64_field(request, "to")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth rotLerp type".into()),
        };
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_rot_lerp_rad" {
        let value = math::mth_rot_lerp_rad(
            f32_field(request, "alpha")?,
            f32_field(request, "from")?,
            f32_field(request, "to")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_triangle_wave" {
        let value =
            math::mth_triangle_wave(f32_field(request, "index")?, f32_field(request, "period")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_square_f32" {
        let value = math::mth_square_f32(f32_field(request, "x")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_cube_f32" {
        let value = math::mth_cube_f32(f32_field(request, "x")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_square_f64" {
        let value = math::mth_square_f64(f64_field(request, "x")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_square_i32" {
        let value = math::mth_square_i32(
            string(request, "x")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_square_i64" {
        let value = math::mth_square_i64(
            string(request, "x")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({"value": value.to_string()}));
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
    if operation == "direction_axial_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({
            "clockwise": direction_snapshot(facing.clockwise(axis)),
            "counter_clockwise": direction_snapshot(facing.counter_clockwise(axis)),
        }));
    }
    if operation == "direction_y_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let clockwise = match facing.clockwise_y() {
            Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
            Err(DirectionYRotationError::ClockwiseVertical(direction)) => json!({
                "outcome": "error",
                "error": format!("Unable to get Y-rotated facing of {}", direction.name()),
            }),
            Err(DirectionYRotationError::CounterClockwiseVertical(_)) => {
                unreachable!("clockwise_y cannot return a counter-clockwise error")
            }
        };
        let counter_clockwise = match facing.counter_clockwise_y() {
            Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
            Err(DirectionYRotationError::CounterClockwiseVertical(direction)) => json!({
                "outcome": "error",
                "error": format!("Unable to get CCW facing of {}", direction.name()),
            }),
            Err(DirectionYRotationError::ClockwiseVertical(_)) => {
                unreachable!("counter_clockwise_y cannot return a clockwise error")
            }
        };
        return Ok(json!({"clockwise": clockwise, "counter_clockwise": counter_clockwise}));
    }
    if operation == "direction_data" {
        let value = string(request, "value")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let facing = match string(request, "type")? {
            "3d" => Direction::from_3d_data_value(value),
            "2d" => Direction::from_2d_data_value(value),
            _ => return Err("unknown Direction data type".into()),
        };
        return Ok(json!({"value": direction_snapshot(facing)}));
    }
    if operation == "direction_y_rot" {
        let y_rot = f64_field(request, "value")?;
        return Ok(json!({
            "value": direction_snapshot(Direction::from_y_rot(y_rot)),
        }));
    }
    if operation == "direction_axis" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "value": direction_snapshot(Direction::from_axis_and_direction(axis, direction)),
        }));
    }
    if operation == "direction_get" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "value": direction_snapshot(Direction::get(direction, axis)),
        }));
    }
    if operation == "direction_unit_vectors" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let (x, y, z) = facing.step();
        return Ok(json!({
            "unit_vec3i": {"x": x, "y": y, "z": z},
            "unit_vec3": vec3_snapshot(facing.unit_vec3()),
            "unit_vec3f": vec3f_snapshot(facing.unit_vec3f()),
        }));
    }
    if operation == "direction_step" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "step_x": facing.get_step_x(),
            "step_y": facing.get_step_y(),
            "step_z": facing.get_step_z(),
            "step": vec3f_snapshot(facing.step_vec3f()),
        }));
    }
    if operation == "direction_axis_orientation" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({
            "vertical": axis.is_vertical(),
            "horizontal": axis.is_horizontal(),
        }));
    }
    if operation == "direction_axis_directions" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let directions = axis.get_directions();
        return Ok(json!({
            "positive": direction_snapshot(axis.get_positive()),
            "negative": direction_snapshot(axis.get_negative()),
            "directions": directions.into_iter().map(direction_snapshot).collect::<Vec<_>>(),
        }));
    }
    if operation == "direction_axis_direction_values" {
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "step": direction.get_step(),
            "name": direction.name(),
            "opposite": direction.opposite().name(),
        }));
    }
    if operation == "direction_axis_choose" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let int_value = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let double_value = |name: &str| f64_field(request, name);
        let bool_value = |name: &str| {
            request[name]
                .as_bool()
                .ok_or_else(|| format!("{name} must be a boolean"))
        };
        return Ok(json!({
            "int": axis.choose_i32(int_value("int_x")?, int_value("int_y")?, int_value("int_z")?),
            "double": format!("{:016x}", axis.choose_f64(
                double_value("double_x")?, double_value("double_y")?, double_value("double_z")?,
            ).to_bits()),
            "boolean": axis.choose_bool(
                bool_value("bool_x")?, bool_value("bool_y")?, bool_value("bool_z")?,
            ),
        }));
    }
    if operation == "direction_axis_test" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let input = match string(request, "direction")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown direction")?),
        };
        return Ok(json!({"value": axis.test(input)}));
    }
    if operation == "direction_axis_plane" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let plane = match axis.get_plane() {
            Plane::Horizontal => "horizontal",
            Plane::Vertical => "vertical",
        };
        return Ok(json!({"plane": plane}));
    }
    if operation == "direction_plane_test" {
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let input = match string(request, "direction")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown direction")?),
        };
        return Ok(json!({"value": plane.test(input)}));
    }
    if operation == "direction_plane_order" {
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        return Ok(json!({
            "faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
            "stream_faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
            "axes": plane.axes().iter().map(|axis| axis.name()).collect::<Vec<_>>(),
            "length": plane.length(),
        }));
    }
    if operation == "direction_random" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let mut random = RandomSource::create(seed);
        let direction = Direction::get_random(&mut random);
        let axis = Axis::get_random(&mut random);
        let plane_direction = plane.get_random_direction(&mut random);
        let plane_axis = plane.get_random_axis(&mut random);
        return Ok(json!({
            "direction": direction_snapshot(direction),
            "axis": axis.name(),
            "plane_direction": direction_snapshot(plane_direction),
            "plane_axis": plane_axis.name(),
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_plane_shuffle" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let mut random = RandomSource::create(seed);
        let faces = plane
            .shuffled_copy(&mut random)
            .into_iter()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({
            "faces": faces,
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_all_shuffle" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let mut random = RandomSource::create(seed);
        let faces = Direction::all_shuffled(&mut random)
            .into_iter()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({
            "faces": faces,
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_stream" {
        let faces = Direction::stream()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({"faces": faces}));
    }
    if operation == "direction_to_y_rot" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "bits": format!("{:08x}", facing.to_y_rot().to_bits()),
        }));
    }
    if operation == "direction_get_y_rot" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(match Direction::get_y_rot(facing) {
            Ok(value) => json!({"outcome": "value", "bits": format!("{:08x}", value.to_bits())}),
            Err(DirectionYRotError::Vertical(direction)) => json!({
                "outcome": "error",
                "error": format!("No y-Rot for vertical axis: {}", direction.name()),
            }),
        });
    }
    if operation == "direction_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({"value": quaternion_snapshot(facing.get_rotation())}));
    }
    if operation == "direction_name_lookup" {
        let name = string(request, "name")?;
        return match string(request, "kind")? {
            "direction" => Ok(match Direction::from_name(name) {
                Some(direction) => json!({
                    "found": true,
                    "name": direction.name(),
                    "serialized_name": direction.name(),
                    "text": direction.name(),
                }),
                None => json!({"found": false}),
            }),
            "axis" => Ok(match Axis::from_name(name) {
                Some(axis) => json!({
                    "found": true,
                    "name": axis.name(),
                    "serialized_name": axis.name(),
                    "text": axis.name(),
                }),
                None => json!({"found": false}),
            }),
            _ => Err("unknown name lookup kind".into()),
        };
    }
    if operation == "direction_enum_text" {
        let name = string(request, "name")?;
        return match string(request, "kind")? {
            "axis_direction" => {
                let direction = match name {
                    "positive" => AxisDirection::Positive,
                    "negative" => AxisDirection::Negative,
                    _ => return Err("unknown axis direction".into()),
                };
                Ok(json!({
                    "enum_name": direction.enum_name(),
                    "name": direction.name(),
                    "text": direction.to_string_java(),
                }))
            }
            "plane" => {
                let plane = match name {
                    "horizontal" => Plane::Horizontal,
                    "vertical" => Plane::Vertical,
                    _ => return Err("unknown plane".into()),
                };
                Ok(json!({
                    "enum_name": plane.enum_name(),
                    "text": plane.to_string_java(),
                }))
            }
            _ => Err("unknown enum text kind".into()),
        };
    }
    if operation == "direction_approximate_nearest" {
        let facing = Direction::approximate_nearest(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({"value": direction_snapshot(facing)}));
    }
    if operation == "direction_approximate_nearest_overloads" {
        let f32_facing = Direction::approximate_nearest_f32(
            f32_field(request, "f32_x")?,
            f32_field(request, "f32_y")?,
            f32_field(request, "f32_z")?,
        );
        let f64_vector = Vec3::new(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({
            "f32": direction_snapshot(f32_facing),
            "f64": direction_snapshot(Direction::approximate_nearest(
                f64_vector.x, f64_vector.y, f64_vector.z,
            )),
            "vec3": direction_snapshot(Direction::approximate_nearest_vec3(f64_vector)),
        }));
    }
    if operation == "direction_nearest" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let or_else = match string(request, "or_else")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown fallback direction")?),
        };
        let facing = Direction::get_nearest(
            coordinate("x")?,
            coordinate("y")?,
            coordinate("z")?,
            or_else,
        );
        return Ok(match facing {
            Some(facing) => json!({"value": direction_snapshot(facing)}),
            None => json!({}),
        });
    }
    if operation == "direction_axis_step_order" {
        let axes = Direction::axis_step_order(f64_field(request, "x")?, f64_field(request, "z")?);
        return Ok(json!({
            "axes": axes.into_iter().map(Axis::name).collect::<Vec<_>>(),
        }));
    }
    if operation == "direction_is_facing_angle" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "value": facing.is_facing_angle(f32_field(request, "angle")?),
        }));
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
