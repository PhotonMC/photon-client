use super::*;

pub(crate) fn sectionpos_packed_accessors_case(request: &Value) -> Result<Value, String> {
    let packed = u64_hex_field(request, "packed")?;
    Ok(json!({
        "value": {
            "x": SectionPos::get_x(packed).to_string(),
            "y": SectionPos::get_y(packed).to_string(),
            "z": SectionPos::get_z(packed).to_string(),
        },
    }))
}

pub(crate) fn sectionpos_block_coordinates_case(request: &Value) -> Result<Value, String> {
    Ok(json!({
        "value": sectionpos_block_coordinates_snapshot(sectionpos(request)?),
    }))
}

pub(crate) fn sectionpos_relative_to_block_case(request: &Value) -> Result<Value, String> {
    let value = sectionpos(request)?;
    let relative = string(request, "relative")?
        .parse::<i16>()
        .map_err(|error| error.to_string())? as u16;
    Ok(json!({
        "value": {
            "relative_x": SectionPos::section_relative_x(relative).to_string(),
            "relative_y": SectionPos::section_relative_y(relative).to_string(),
            "relative_z": SectionPos::section_relative_z(relative).to_string(),
            "block_x": value.relative_to_block_x(relative).to_string(),
            "block_y": value.relative_to_block_y(relative).to_string(),
            "block_z": value.relative_to_block_z(relative).to_string(),
            "block_position": blockpos_snapshot(value.relative_to_block_pos(relative)),
        },
    }))
}

pub(crate) fn sectionpos_spatial_case(request: &Value) -> Result<Value, String> {
    Ok(json!({
        "value": sectionpos_spatial_snapshot(sectionpos(request)?),
    }))
}

pub(crate) fn sectionpos_packed_transformations_case(request: &Value) -> Result<Value, String> {
    let coordinate = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(json!({
        "value": sectionpos_packed_transformations_snapshot(
            u64_hex_field(request, "block_node")?,
            u64_hex_field(request, "section_node")?,
            coordinate("x")?,
            coordinate("z")?,
        ),
    }))
}

pub(crate) fn sectionpos_as_long_case(request: &Value) -> Result<Value, String> {
    let coordinate = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(json!({
        "value": sectionpos_as_long_snapshot(
            u64_hex_field(request, "block_node")?,
            (coordinate("x")?, coordinate("y")?, coordinate("z")?),
            SectionPos::new(
                coordinate("section_x")?,
                coordinate("section_y")?,
                coordinate("section_z")?,
            ),
        ),
    }))
}

pub(crate) fn sectionpos_offset_case(request: &Value) -> Result<Value, String> {
    let coordinate = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    let direction = Direction::from_name(string(request, "direction")?)
        .ok_or("unknown SectionPos offset direction")?;
    Ok(json!({
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
    }))
}

pub(crate) fn sectionpos_ranges_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({
        "value": sectionpos_ranges_snapshot(positions),
    }))
}

pub(crate) fn sectionpos_blocks_inside_case(request: &Value) -> Result<Value, String> {
    Ok(json!({
        "value": blockpos_positions_snapshot(sectionpos(request)?.blocks_inside()),
    }))
}

pub(crate) fn sectionpos_around_and_at_block_pos_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({
        "value": sectionpos_nodes_snapshot(section_nodes),
    }))
}

pub(crate) fn sectionpos_double_coordinates_case(request: &Value) -> Result<Value, String> {
    let value = f64_field(request, "value")?;
    let result = match string(request, "fn")? {
        "pos_to_section" => SectionPos::pos_to_section_coord(value),
        "block_to_section" => SectionPos::block_to_section_coord_f64(value),
        _ => return Err("unknown SectionPos double coordinate function".into()),
    };
    Ok(json!({"value": result.to_string()}))
}

pub(crate) fn sectionpos_of_overloads_case(request: &Value) -> Result<Value, String> {
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
    Ok(json!({"value": sectionpos_snapshot(value)}))
}
