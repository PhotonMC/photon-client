use super::*;

/// The scene's blocks in a client world of loaded chunks, as the movement
/// oracle builds its level.
pub(super) fn movement_world(row: &Value) -> Result<ClientWorld, String> {
    use photon_data::block::registry;
    use photon_world::{DimensionBounds, SectionInput};
    // The overworld's bounds, as the reference level is built with.
    let dimension = DimensionBounds::new(-64, 384);
    let mut placed = std::collections::HashMap::new();
    for block in row["blocks"].as_array().ok_or("blocks must be an array")? {
        let name = string(block, "block")?;
        let definition = registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
        let mut chosen = std::collections::BTreeMap::new();
        if let Some(properties) = block["properties"].as_object() {
            for (key, value) in properties {
                chosen.insert(
                    key.clone(),
                    value
                        .as_str()
                        .ok_or("property values are strings")?
                        .to_owned(),
                );
            }
        }
        let state = definition
            .state_id(&chosen)
            .ok_or_else(|| format!("bad properties for {name}"))?;
        placed.insert(
            (
                i32_field(block, "x")?,
                i32_field(block, "y")?,
                i32_field(block, "z")?,
            ),
            state,
        );
    }
    let mut world = ClientWorld::new(dimension);
    let radius = i32_field(row, "chunk_radius")?;
    let biomes = vec![0_i32; 64];
    for chunk_x in -radius..=radius {
        for chunk_z in -radius..=radius {
            let mut sections = Vec::new();
            for index in 0..dimension.section_count() {
                let min_y = dimension.min_y + index as i32 * 16;
                let mut states = vec![0_i32; 4096];
                let mut non_empty = 0_i16;
                let mut fluids = 0_i16;
                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            if let Some(&state) =
                                placed.get(&(chunk_x * 16 + x, min_y + y, chunk_z * 16 + z))
                            {
                                states[(y * 256 + z * 16 + x) as usize] = state.0;
                                let (block, _) =
                                    registry::block_for_state(state).ok_or("state out of range")?;
                                if !matches!(
                                    block.name,
                                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                                ) {
                                    non_empty += 1;
                                }
                                if photon_world::local_player::state_has_fluid(state) {
                                    fluids += 1;
                                }
                            }
                        }
                    }
                }
                sections.push((non_empty, fluids, states));
            }
            world.replace_chunk(
                chunk_x,
                chunk_z,
                sections
                    .iter()
                    .map(|(non_empty, fluids, states)| SectionInput {
                        non_empty_block_count: *non_empty,
                        fluid_count: *fluids,
                        block_states: states,
                        biomes: &biomes,
                    }),
            );
        }
    }

    // The reference level's registry holds the vanilla tags.
    world.apply_update_tags_packet(&vanilla_block_tags()?);
    Ok(world)
}

/// `light_changes`: the client world lit from nothing, then block updates,
/// each followed by a run of the light engine; a box of light after each.
pub(super) fn light_changes_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("light rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        world.light_from_sources();
        world.run_light_updates();
        let box_ = &row["box"];
        let origin = (
            i32_field(box_, "x")?,
            i32_field(box_, "y")?,
            i32_field(box_, "z")?,
        );
        let size = (
            i32_field(box_, "size_x")?,
            i32_field(box_, "size_y")?,
            i32_field(box_, "size_z")?,
        );
        let snapshot = |world: &ClientWorld| {
            let mut block = String::new();
            let mut sky = String::new();
            for y in 0..size.1 {
                for z in 0..size.2 {
                    for x in 0..size.0 {
                        let (x, y, z) = (origin.0 + x, origin.1 + y, origin.2 + z);
                        block.push_str(&format!("{:x}", world.block_brightness(x, y, z)));
                        sky.push_str(&format!("{:x}", world.sky_brightness(x, y, z)));
                    }
                }
            }
            json!({"block": block, "sky": sky})
        };
        let mut snapshots = vec![snapshot(&world)];
        for event in row["events"].as_array().ok_or("events must be an array")? {
            world.apply_server_block_state(
                i32_field(event, "x")?,
                i32_field(event, "y")?,
                i32_field(event, "z")?,
                named_state(event)?,
            );
            world.run_light_updates();
            snapshots.push(snapshot(&world));
        }
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "lit",
            "snapshots": snapshots,
        }));
    }
    Ok(json!({"rows": out}))
}

/// The count of the stack in an inventory menu slot, zero when empty.
pub(super) fn hand_count(world: &ClientWorld, slot: i16) -> i32 {
    world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .map_or(0, |stack| stack.item_count())
}
