use super::*;

pub(crate) fn chunkpos_codec_case(request: &Value) -> Result<Value, String> {
    match string(request, "mode")? {
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
    }
}

pub(crate) fn chunkpos_block_coordinates_case(request: &Value) -> Result<Value, String> {
    let value = chunkpos(request)?;
    let offset = string(request, "offset")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": chunkpos_block_coordinates_snapshot(value, offset),
    }))
}

pub(crate) fn chunkpos_region_coordinates_case(request: &Value) -> Result<Value, String> {
    let value = chunkpos(request)?;
    let region_x = string(request, "region_x")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let region_z = string(request, "region_z")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": chunkpos_region_coordinates_snapshot(value, region_x, region_z),
    }))
}

pub(crate) fn chunkpos_block_positions_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({
        "value": chunkpos_block_positions_snapshot(value, block_x, y, block_z, contains)?,
    }))
}

pub(crate) fn chunkpos_representation_case(request: &Value) -> Result<Value, String> {
    let value = chunkpos(request)?;
    let hash_x = string(request, "hash_x")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let hash_z = string(request, "hash_z")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": chunkpos_representation_snapshot(value, hash_x, hash_z),
    }))
}

pub(crate) fn chunkpos_from_blockpos_case(request: &Value) -> Result<Value, String> {
    Ok(json!({
        "value": chunkpos_from_blockpos_snapshot(blockpos(request)?),
    }))
}

pub(crate) fn chunkpos_distances_case(request: &Value) -> Result<Value, String> {
    let value = chunkpos(request)?;
    let other = ChunkPos::new(
        string(request, "other_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?,
        string(request, "other_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?,
    );
    Ok(json!({
        "value": chunkpos_distances_snapshot(value, other),
    }))
}

pub(crate) fn chunkpos_from_section_node_case(request: &Value) -> Result<Value, String> {
    let section_node = u64::from_str_radix(string(request, "section_node")?, 16)
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": format!("{:x}", ChunkPos::from_section_node(section_node)),
    }))
}

pub(crate) fn chunkpos_range_closed_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({
        "value": chunkpos_range_closed_snapshot(positions),
    }))
}

pub(crate) fn chunkpos_constant_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({"value": value}))
}

pub(crate) fn chunkpos_packed_accessors_case(request: &Value) -> Result<Value, String> {
    let packed = u64_hex_field(request, "packed")?;
    Ok(json!({
        "value": {
            "x": ChunkPos::get_x(packed).to_string(),
            "z": ChunkPos::get_z(packed).to_string(),
        },
    }))
}

pub(crate) fn chunkpos_validity_case(request: &Value) -> Result<Value, String> {
    Ok(json!({
        "value": chunkpos_validity_snapshot(chunkpos(request)?),
    }))
}
