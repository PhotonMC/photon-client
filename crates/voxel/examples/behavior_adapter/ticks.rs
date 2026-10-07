use super::*;

pub(super) fn vanilla_tags_of(
    folder: &str,
    registry_name: &str,
    index_of: &dyn Fn(&str) -> Option<i32>,
) -> Result<ClientboundUpdateTagsPacket, String> {
    fn resolve(
        root: &std::path::Path,
        name: &str,
        seen: &mut BTreeMap<String, Vec<i32>>,
        index_of: &dyn Fn(&str) -> Option<i32>,
    ) -> Result<Vec<i32>, String> {
        if let Some(done) = seen.get(name) {
            return Ok(done.clone());
        }
        let path = root.join(format!("{name}.json"));
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let parsed: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for entry in parsed["values"].as_array().ok_or("tag values")? {
            let (id, required) = match entry {
                Value::String(id) => (id.as_str(), true),
                other => (
                    other["id"].as_str().ok_or("tag entry id")?,
                    other["required"].as_bool().unwrap_or(true),
                ),
            };
            if let Some(tag) = id.strip_prefix('#') {
                let path = tag.strip_prefix("minecraft:").ok_or("foreign tag")?;
                out.extend(resolve(root, path, seen, index_of)?);
            } else {
                match index_of(id) {
                    Some(index) => out.push(index),
                    None if !required => {}
                    None => return Err(format!("unknown entry {id} in tag {name}")),
                }
            }
        }
        seen.insert(name.to_owned(), out.clone());
        Ok(out)
    }
    fn walk(
        dir: &std::path::Path,
        root: &std::path::Path,
        names: &mut Vec<String>,
    ) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                walk(&path, root, names)?;
            } else if let Some(stem) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                names.push(stem.trim_end_matches(".json").replace('\\', "/"));
            }
        }
        Ok(())
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data/minecraft/tags")
        .join(folder);
    let mut names = Vec::new();
    walk(&root, &root, &mut names)?;
    names.sort();
    let mut seen = BTreeMap::new();
    let mut tags = Vec::new();
    for name in names {
        let mut values = resolve(&root, &name, &mut seen, index_of)?;
        values.sort_unstable();
        values.dedup();
        let id = Identifier::parse(&format!("minecraft:{name}")).map_err(|_| "tag id")?;
        tags.push(TagEntry::new(id, values));
    }
    let registry = Identifier::parse(registry_name).map_err(|_| "registry id")?;
    Ok(ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        registry, tags,
    )]))
}

/// The scene's blocks in a client world of loaded chunks, as the movement
/// oracle builds its level.
pub(super) fn movement_world(row: &Value) -> Result<ClientWorld, String> {
    use voxel_data::block::registry;
    use voxel_world::{DimensionBounds, SectionInput};
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
            state as i32,
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
                                states[(y * 256 + z * 16 + x) as usize] = state;
                                let (block, _) = registry::block_for_state(state as u32)
                                    .ok_or("state out of range")?;
                                if !matches!(
                                    block.name,
                                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                                ) {
                                    non_empty += 1;
                                }
                                if voxel_world::local_player::state_has_fluid(state) {
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

/// `item_entity_ticks`: a dropped item put in the scene by the server's
/// packets (place, motion, ground flag), then ticked with the level.
pub(super) fn item_entity_ticks_result(request: &Value) -> Result<Value, String> {
    let bits64 = |row: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(row, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let hex = |value: f64| format!("{:016x}", value.to_bits());
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        let entity_id = i32_field(row, "entity_id")?;
        let item_type = voxel_data::entity_type::id("minecraft:item").ok_or("no item type")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id: item_type,
            x: bits64(row, "x")?,
            y: bits64(row, "y")?,
            z: bits64(row, "z")?,
            movement: (
                bits64(row, "motion_x")?,
                bits64(row, "motion_y")?,
                bits64(row, "motion_z")?,
            ),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("item entity was rejected".into());
        }
        if row["on_ground"]
            .as_bool()
            .ok_or("on_ground must be a bool")?
        {
            world.apply_entity_ground_state(entity_id, true);
        }
        let mut ticks = Vec::new();
        for tick in 0..i32_field(row, "tick_count")? {
            // The packets that arrived since the last tick, as the net layer
            // hands them to the world.
            for event in row["events"].as_array().ok_or("events must be an array")? {
                if i32_field(event, "tick")? != tick {
                    continue;
                }
                let on_ground = event["on_ground"].as_bool().unwrap_or(false);
                match string(event, "type")? {
                    "move" => {
                        world.apply_entity_move(voxel_world::EntityMoveInput {
                            entity_id,
                            delta_x: i32_field(event, "xa")? as i16,
                            delta_y: i32_field(event, "ya")? as i16,
                            delta_z: i32_field(event, "za")? as i16,
                            has_position: true,
                            y_rot: None,
                            x_rot: None,
                        });
                        world.apply_entity_ground_state(entity_id, on_ground);
                    }
                    "sync" => {
                        world.apply_entity_position_sync(
                            entity_id,
                            (
                                bits64(event, "x")?,
                                bits64(event, "y")?,
                                bits64(event, "z")?,
                            ),
                            (0.0, 0.0),
                        );
                        world.apply_entity_ground_state(entity_id, on_ground);
                    }
                    "motion" => {
                        world.apply_entity_motion(voxel_world::EntityMotionInput {
                            entity_id,
                            x: bits64(event, "motion_x")?,
                            y: bits64(event, "motion_y")?,
                            z: bits64(event, "motion_z")?,
                        });
                    }
                    other => return Err(format!("unknown event {other}")),
                }
            }
            world.tick_entities();
            let entity = world.entity_at(entity_id).ok_or("item entity vanished")?;
            let (x, y, z) = entity.position();
            let (motion_x, motion_y, motion_z) = entity.motion();
            ticks.push(json!({
                "x": hex(x), "y": hex(y), "z": hex(z),
                "motion_x": hex(motion_x), "motion_y": hex(motion_y), "motion_z": hex(motion_z),
                "on_ground": entity.on_ground(),
            }));
        }
        out.push(json!({"label": row["label"], "outcome": "ticked", "ticks": ticks}));
    }
    Ok(json!({"rows": out}))
}

/// `projectile_ticks`: a projectile the server added, ticked by the client between the server's
/// packets against the blocks and the entities the scene holds.
pub(super) fn projectile_ticks_result(request: &Value) -> Result<Value, String> {
    let bits64 = |row: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(row, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let bits32 = |row: &Value, key: &str| -> Result<f32, String> {
        u32::from_str_radix(string(row, key)?, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let hex64 = |value: f64| format!("{:016x}", value.to_bits());
    let hex32 = |value: f32| format!("{:08x}", value.to_bits());
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        for other in row["others"].as_array().ok_or("others must be an array")? {
            let id = i32_field(other, "id")?;
            let name = string(other, "type")?;
            let type_id = voxel_data::entity_type::id(name).ok_or_else(|| format!("no {name}"))?;
            if !world.apply_entity(EntityInput {
                entity_id: id,
                uuid: Uuid::new(0, u64::from(id as u32)),
                type_id,
                x: bits64(other, "x")?,
                y: bits64(other, "y")?,
                z: bits64(other, "z")?,
                movement: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                data: 0,
            }) {
                return Err(format!("{name} was rejected"));
            }
        }
        let entity_id = i32_field(row, "entity_id")?;
        let name = string(row, "type")?;
        let type_id = voxel_data::entity_type::id(name).ok_or_else(|| format!("no {name}"))?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id,
            x: bits64(row, "x")?,
            y: bits64(row, "y")?,
            z: bits64(row, "z")?,
            movement: (
                bits64(row, "motion_x")?,
                bits64(row, "motion_y")?,
                bits64(row, "motion_z")?,
            ),
            rotation: (bits32(row, "y_rot")?, bits32(row, "x_rot")?, 0.0),
            data: row.get("owner").and_then(Value::as_i64).unwrap_or(0) as i32,
        }) {
            return Err(format!("{name} was rejected"));
        }
        if let Some(dangerous) = row.get("dangerous").and_then(Value::as_bool) {
            // The skull's data, as the server's entity data packet sets it.
            world.apply_entity_metadata(entity_id, &[8, 8, u8::from(dangerous), 255]);
        }
        let mut ticks = Vec::new();
        for _ in 0..i32_field(row, "tick_count")? {
            world.tick_entities();
            let described = match world.entity_at(entity_id) {
                Some(entity) => {
                    let (x, y, z) = entity.position();
                    let (mx, my, mz) = entity.motion();
                    let (y_rot, x_rot, _) = entity.rotation();
                    json!({
                        "present": true,
                        "x": hex64(x), "y": hex64(y), "z": hex64(z),
                        "motion_x": hex64(mx), "motion_y": hex64(my), "motion_z": hex64(mz),
                        "x_rot": hex32(x_rot), "y_rot": hex32(y_rot),
                        "in_water": entity.is_in_water(),
                    })
                }
                None => json!({"present": false}),
            };
            ticks.push(described);
        }
        out.push(json!({"label": row["label"], "outcome": "ticked", "ticks": ticks}));
    }
    Ok(json!({"rows": out}))
}
