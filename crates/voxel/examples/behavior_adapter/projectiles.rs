use super::*;

pub(super) fn client_world_entity_snowball_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 120)
}

pub(super) fn client_world_entity_arrow_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 6)
}

pub(super) fn client_world_entity_experience_orb_tick_result(
    request: &Value,
) -> Result<Value, String> {
    client_world_entity_tick_result(request, 49)
}

pub(super) fn client_world_entity_primed_tnt_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 133)
}

pub(super) fn client_world_entity_falling_block_tick_result(
    request: &Value,
) -> Result<Value, String> {
    client_world_entity_tick_result(request, 51)
}

pub(super) fn client_world_entity_firework_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 53)
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
