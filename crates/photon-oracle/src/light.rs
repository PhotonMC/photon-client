use super::*;

pub(super) fn client_world_section_lookup_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let min_section_y = min_y.div_euclid(16);
    let tracks = request["tracks"]
        .as_array()
        .ok_or("tracks must be an array")?
        .iter()
        .map(|track| {
            let chunk_x = i32_field(track, "chunk_x")?;
            let chunk_z = i32_field(track, "chunk_z")?;
            let section_values = track["sections"]
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

            let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
            world.replace_chunk(
                chunk_x,
                chunk_z,
                observations.iter().map(|section| SectionInput {
                    non_empty_block_count: section.non_empty_block_count,
                    fluid_count: section.fluid_count,
                    block_states: &section.block_state_values,
                    biomes: &section.biome_values,
                }),
            );

            let sections = observations
                .iter()
                .enumerate()
                .map(|(index, section)| {
                    json!({
                        "section_y": min_section_y + index as i32,
                        "non_empty_block_count": section.non_empty_block_count,
                        "fluid_count": section.fluid_count,
                        "has_only_air": section.non_empty_block_count == 0,
                        "has_fluid": section.fluid_count > 0,
                    })
                })
                .collect::<Vec<_>>();

            let queries = track["queries"]
                .as_array()
                .ok_or("queries must be an array")?
                .iter()
                .map(|query| {
                    let x = i32_field(query, "x")?;
                    let y = i32_field(query, "y")?;
                    let z = i32_field(query, "z")?;
                    let query_section_y = y.div_euclid(16);
                    let mut state = json!({
                        "label": string(query, "label")?,
                        "section_y": query_section_y,
                    });
                    if let Some(block_state_id) = world.block_state_at(x, y, z) {
                        state["block_state_id"] = json!(block_state_id);
                    }
                    if let Some(biome_id) = world.biome_at(x, y, z) {
                        state["biome_id"] = json!(biome_id);
                    }
                    if let Some((non_empty, fluid)) =
                        world.section_counts_at(x.div_euclid(16), query_section_y, z.div_euclid(16))
                    {
                        state["section_counts"] = json!([non_empty, fluid]);
                    }
                    Ok(state)
                })
                .collect::<Result<Vec<_>, String>>()?;

            Ok(json!({
                "label": string(track, "label")?,
                "sections": sections,
                "queries": queries,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"tracks": tracks}))
}

pub(super) fn client_world_block_update_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let chunk_x = i32_field(request, "chunk_x")?;
    let chunk_z = i32_field(request, "chunk_z")?;
    let section_values = request["sections"]
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

    let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
    if !world.replace_chunk(
        chunk_x,
        chunk_z,
        observations.iter().map(|section| SectionInput {
            non_empty_block_count: section.non_empty_block_count,
            fluid_count: section.fluid_count,
            block_states: &section.block_state_values,
            biomes: &section.biome_values,
        }),
    ) {
        return Err("initial chunk was rejected".into());
    }

    let updates = request["updates"]
        .as_array()
        .ok_or("updates must be an array")?;
    let mut events = Vec::with_capacity(updates.len());
    for update in updates {
        let x = i32_field(update, "x")?;
        let y = i32_field(update, "y")?;
        let z = i32_field(update, "z")?;
        let previous_state_id = world.block_state_at(x, y, z);
        let accepted = world.update_block_state(x, y, z, i32_field(update, "state_id")?);
        let mut state = json!({
            "label": string(update, "label")?,
            "accepted": accepted,
        });
        if accepted {
            let updated_state_id = world
                .block_state_at(x, y, z)
                .ok_or("accepted block update has no state")?;
            let section_counts = world
                .section_counts_at(x.div_euclid(16), y.div_euclid(16), z.div_euclid(16))
                .ok_or("accepted block update has no section counts")?;
            state["previous_state_id"] =
                json!(previous_state_id.ok_or("accepted block update has no previous state")?);
            state["updated_state_id"] = json!(updated_state_id);
            state["section_counts"] = json!([section_counts.0, section_counts.1]);
            state["has_only_air"] = json!(section_counts.0 == 0);
            state["has_fluid"] = json!(section_counts.1 > 0);
        }

        let queries = update["queries"]
            .as_array()
            .ok_or("queries must be an array")?
            .iter()
            .map(|query| {
                let query_x = i32_field(query, "x")?;
                let query_y = i32_field(query, "y")?;
                let query_z = i32_field(query, "z")?;
                let mut query_state = json!({
                    "label": string(query, "label")?,
                    "section_y": query_y.div_euclid(16),
                });
                if let Some(block_state_id) = world.block_state_at(query_x, query_y, query_z) {
                    query_state["block_state_id"] = json!(block_state_id);
                }
                if let Some((non_empty, fluid)) = world.section_counts_at(
                    query_x.div_euclid(16),
                    query_y.div_euclid(16),
                    query_z.div_euclid(16),
                ) {
                    query_state["section_counts"] = json!([non_empty, fluid]);
                }
                Ok(query_state)
            })
            .collect::<Result<Vec<_>, String>>()?;
        state["queries"] = json!(queries);
        events.push(state);
    }
    Ok(json!({"events": events}))
}

pub(super) fn light_bitset_field(value: &Value, name: &str) -> Result<BitSet, String> {
    let bits = value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|bit| {
            bit.as_u64()
                .ok_or_else(|| format!("{name} entries must be unsigned integers"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BitSet::from_bits(&bits))
}

pub(super) fn light_updates_field(value: &Value, name: &str) -> Result<Vec<Vec<u8>>, String> {
    value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|update| {
            decode_hex(
                update
                    .as_str()
                    .ok_or_else(|| format!("{name} entries must be hexadecimal strings"))?,
            )
        })
        .collect()
}

pub(super) fn client_world_light_queries(
    world: &ClientWorld,
    value: &Value,
) -> Result<Value, String> {
    value
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query| {
            let layer = string(query, "layer")?;
            let x = i32_field(query, "x")?;
            let y = i32_field(query, "y")?;
            let z = i32_field(query, "z")?;
            let light = match layer {
                "sky" => world.sky_light_at(x, y, z),
                "block" => world.block_light_at(x, y, z),
                _ => return Err(format!("unknown light layer: {layer}")),
            };
            let mut state = json!({"label": string(query, "label")?});
            if let Some(light) = light {
                state["value"] = json!(light);
            }
            Ok(state)
        })
        .collect::<Result<Vec<_>, String>>()
        .map(Value::Array)
}

pub(super) fn client_world_light_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        let action = string(operation, "action")?;
        let label = string(operation, "label")?;
        let mut event = json!({
            "action": action,
            "label": label,
            "queued_for_unloaded_chunk": false,
            "stored": false,
            "applied_sky_sections": 0,
            "applied_block_sections": 0,
            "queries": [],
        });

        match action {
            "apply" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let sky_mask = light_bitset_field(operation, "sky_mask")?;
                let sky_empty_mask = light_bitset_field(operation, "sky_empty_mask")?;
                let sky_updates = light_updates_field(operation, "sky_updates")?;
                let block_mask = light_bitset_field(operation, "block_mask")?;
                let block_empty_mask = light_bitset_field(operation, "block_empty_mask")?;
                let block_updates = light_updates_field(operation, "block_updates")?;
                let application = world.apply_light_updates(
                    chunk_x,
                    chunk_z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &sky_mask,
                            empty_mask: &sky_empty_mask,
                            updates: &sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &block_mask,
                            empty_mask: &block_empty_mask,
                            updates: &block_updates,
                        },
                    },
                );
                if application.invalid_sky_updates != 0 || application.invalid_block_updates != 0 {
                    return Err("light update data layer is not 2048 bytes".into());
                }
                // The oracle runs the engine after each packet, as a frame would.
                world.flush_light();
                event["queued_for_unloaded_chunk"] = json!(application.queued);
                // `hasLightData`: any addressed section with a layer.
                let engine = world.light_engine();
                let min = engine.min_light_section();
                let stored = |layer, mask: &BitSet, empty: &BitSet| {
                    (0..engine.light_section_count()).any(|index| {
                        (mask.contains(index as u64) || empty.contains(index as u64))
                            && engine
                                .data_layer(layer, (chunk_x, min + index, chunk_z))
                                .is_some()
                    })
                };
                event["stored"] = json!(
                    stored(
                        photon_world::light::LightLayer::Sky,
                        &sky_mask,
                        &sky_empty_mask
                    ) || stored(
                        photon_world::light::LightLayer::Block,
                        &block_mask,
                        &block_empty_mask
                    )
                );
                event["applied_sky_sections"] = json!(application.applied_sky_sections);
                event["applied_block_sections"] = json!(application.applied_block_sections);
            }
            "load" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let section_count = operation["section_count"]
                    .as_u64()
                    .ok_or("section_count must be an unsigned integer")?
                    .try_into()
                    .map_err(|_| "section_count does not fit in usize")?;
                let non_empty_sections = operation["non_empty_sections"]
                    .as_array()
                    .ok_or("non_empty_sections must be an array")?
                    .iter()
                    .map(|index| {
                        index
                            .as_u64()
                            .ok_or_else(|| {
                                "non_empty_sections entries must be unsigned integers".to_owned()
                            })?
                            .try_into()
                            .map_err(|_| "non_empty section index does not fit in usize".to_owned())
                    })
                    .collect::<Result<Vec<usize>, String>>()?;
                let mut block_sections = Vec::with_capacity(section_count);
                let mut biome_sections = Vec::with_capacity(section_count);
                for index in 0..section_count {
                    let non_empty = non_empty_sections.contains(&index);
                    let mut blocks = vec![0; 4_096];
                    if non_empty {
                        blocks[0] = 1;
                    }
                    block_sections.push(blocks);
                    biome_sections.push(vec![0; 64]);
                }
                if !world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    block_sections
                        .iter()
                        .zip(&biome_sections)
                        .map(|(blocks, biomes)| SectionInput {
                            non_empty_block_count: if blocks[0] == 0 { 0 } else { 1 },
                            fluid_count: 0,
                            block_states: blocks,
                            biomes,
                        }),
                ) {
                    return Err("light test chunk was rejected".into());
                }
                world.enable_chunk_light(chunk_x, chunk_z);
                world.run_light_updates();
                let engine = world.light_engine();
                let section = (chunk_x, min_y.div_euclid(16), chunk_z);
                event["stored"] = json!(
                    engine
                        .data_layer(photon_world::light::LightLayer::Sky, section)
                        .is_some()
                        || engine
                            .data_layer(photon_world::light::LightLayer::Block, section)
                            .is_some()
                );
            }
            "unload" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                world.unload_chunk(chunk_x, chunk_z);
                world.flush_light();
            }
            _ => return Err(format!("unknown client world light action: {action}")),
        }

        event["queries"] = client_world_light_queries(
            &world,
            operation
                .get("queries")
                .unwrap_or(&Value::Array(Vec::new())),
        )?;
        events.push(event);
    }

    Ok(json!({"events": events}))
}

pub(super) fn client_attribute_modifier(
    value: &Value,
) -> Result<ClientAttributeModifierState, String> {
    let operation = value["operation"]
        .as_u64()
        .ok_or("attribute modifier operation must be an unsigned integer")?
        .try_into()
        .map_err(|_| "attribute modifier operation does not fit in u8".to_owned())?;
    Ok(ClientAttributeModifierState::new(
        string(value, "id")?.to_owned(),
        f64_field(value, "amount")?,
        operation,
    ))
}

pub(super) fn client_attribute_state(value: &Value) -> Result<ClientAttributeState, String> {
    let modifiers = value["modifiers"]
        .as_array()
        .ok_or("attribute modifiers must be an array")?
        .iter()
        .map(client_attribute_modifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ClientAttributeState::new(
        i32_field(value, "attribute_id")?,
        f64_field(value, "base")?,
        modifiers,
    ))
}
