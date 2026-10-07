use super::*;

pub(super) fn client_world_chunk_cache_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::default());
    world.configure_chunk_cache(i32_field(request, "server_view_radius")?);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        let action = string(operation, "action")?;
        let accepted = match action {
            "replace" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let section_values = operation["sections"]
                    .as_array()
                    .ok_or("sections must be an array")?;
                let mut section_data = Vec::new();
                for section in section_values {
                    section_data.extend(decode_hex(
                        section.as_str().ok_or("section payload must be a string")?,
                    )?);
                }
                let observations = decode_chunk_section_data(&section_data)?;
                if observations.len() != section_values.len() {
                    return Err("section decoder count does not match input".into());
                }
                world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    observations.iter().map(|section| SectionInput {
                        non_empty_block_count: section.non_empty_block_count,
                        fluid_count: section.fluid_count,
                        block_states: &section.block_state_values,
                        biomes: &section.biome_values,
                    }),
                )
            }
            "drop" => world.unload_chunk(
                i32_field(operation, "chunk_x")?,
                i32_field(operation, "chunk_z")?,
            ),
            "center" => {
                world.update_chunk_cache_center(
                    i32_field(operation, "chunk_x")?,
                    i32_field(operation, "chunk_z")?,
                );
                true
            }
            "radius" => {
                world.update_chunk_cache_radius(i32_field(operation, "server_view_radius")?);
                true
            }
            "section_emptiness" => {
                world.update_chunk_section_emptiness(
                    i32_field(operation, "section_x")?,
                    i32_field(operation, "section_y")?,
                    i32_field(operation, "section_z")?,
                    operation["empty"]
                        .as_bool()
                        .ok_or("empty must be a boolean")?,
                );
                true
            }
            "flip" => {
                world.flip_chunk_cache_update_sets();
                true
            }
            _ => return Err(format!("unknown client chunk cache action: {action}")),
        };

        let checks = operation["checks"]
            .as_array()
            .ok_or("checks must be an array")?
            .iter()
            .map(|check| {
                let chunk_x = i32_field(check, "x")?;
                let chunk_z = i32_field(check, "z")?;
                let (in_range, index) = world
                    .chunk_cache_location(chunk_x, chunk_z)
                    .ok_or("chunk cache is not configured")?;
                Ok(json!({
                    "x": chunk_x,
                    "z": chunk_z,
                    "in_range": in_range,
                    "index": index,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let snapshot = world
            .client_chunk_cache_snapshot()
            .ok_or("chunk cache is not configured")?;
        events.push(json!({
            "action": action,
            "accepted": accepted,
            "cache": client_chunk_cache_snapshot_json(&snapshot),
            "checks": checks,
        }));
    }

    Ok(json!({"events": events}))
}
