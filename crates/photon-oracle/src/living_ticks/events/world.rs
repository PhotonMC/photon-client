use super::super::*;

pub(crate) fn damage_event(world: &mut ClientWorld, entity_id: i32) -> Result<(), String> {
    world.entities_mut().apply_damage_event(entity_id);
    Ok(())
}

pub(crate) fn hurt_event(world: &mut ClientWorld, entity_id: i32) -> Result<(), String> {
    world.entities_mut().apply_hurt_animation(entity_id);
    Ok(())
}

pub(crate) fn death_event(world: &mut ClientWorld, entity_id: i32) -> Result<(), String> {
    world.entities_mut().apply_death_event(entity_id);
    Ok(())
}

pub(crate) fn move_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world.apply_entity_move(photon_world::EntityMoveInput {
        entity_id,
        delta_x: i32_field(event, "xa")? as i16,
        delta_y: i32_field(event, "ya")? as i16,
        delta_z: i32_field(event, "za")? as i16,
        has_position: true,
        y_rot: None,
        x_rot: None,
    });
    world.apply_entity_ground_state(entity_id, event["on_ground"].as_bool().unwrap_or(true));
    Ok(())
}

pub(crate) fn animate_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world
        .entities_mut()
        .apply_animate(entity_id, i32_field(event, "action")? as u8);
    Ok(())
}

pub(crate) fn sync_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world.apply_entity_position_sync(
        entity_id,
        (
            bits64(event, "x")?,
            bits64(event, "y")?,
            bits64(event, "z")?,
        ),
        (0.0, 0.0),
    );
    world.apply_entity_ground_state(entity_id, event["on_ground"].as_bool().unwrap_or(false));
    Ok(())
}

pub(crate) fn weather_event(world: &mut ClientWorld, event: &Value) -> Result<(), String> {
    let level = |key: &str| -> Result<f32, String> {
        u32::from_str_radix(string(event, key)?, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    world.set_weather(level("rain")?, level("thunder")?);
    Ok(())
}

pub(crate) fn equipment_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let stack = event["item"].as_str().map(|name| {
        let id = photon_data::item::registry::index(name).unwrap_or(0);
        photon_world::ClientItemStackState::new(1, id as i32)
    });
    let slot = match event["slot"].as_str() {
        Some("head") => photon_world::EquipmentSlot::Head,
        _ => photon_world::EquipmentSlot::MainHand,
    };
    world.entities_mut().apply_equipment(entity_id, slot, stack);
    Ok(())
}

pub(crate) fn mob_flags_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let flags = event["flags"].as_i64().ok_or("flags")? as u8;
    world.apply_entity_metadata(entity_id, &[15, 0, flags, 255]);
    Ok(())
}

pub(crate) fn seed_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
    name: &str,
) -> Result<(), String> {
    let seed = i64_field(event, "seed")?;
    if name == "minecraft:panda" {
        world.entities_mut().fix_panda_seed(entity_id, seed);
    } else {
        world.entities_mut().fix_horse_seed(entity_id, seed);
    }
    Ok(())
}

pub(crate) fn entity_motion_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world
        .entities_mut()
        .apply_motion(photon_world::EntityMotionInput {
            entity_id,
            x: bits64(event, "x")?,
            y: bits64(event, "y")?,
            z: bits64(event, "z")?,
        });
    Ok(())
}

pub(crate) fn passengers_event(world: &mut ClientWorld, event: &Value) -> Result<(), String> {
    let riders: Vec<i32> = event["riders"]
        .as_array()
        .ok_or("riders must be an array")?
        .iter()
        .filter_map(|id| id.as_i64().map(|id| id as i32))
        .collect();
    world.apply_entity_passengers(i32_field(event, "vehicle")?, &riders);
    Ok(())
}

pub(crate) fn sulfur_fuse_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let mut data = vec![photon_world::SULFUR_CUBE_FUSE_DATA_ID, 1];
    photon_core::wire::write_varint(i32_field(event, "fuse")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn head_rotation_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let angle = i32_field(event, "angle")? as i8;
    world.apply_entity_head_rotation(entity_id, photon_core::math::unpack_degrees_i8(angle));
    Ok(())
}

pub(crate) fn cube_size_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let mut data = vec![photon_world::CUBE_MOB_SIZE_DATA_ID, 1];
    photon_core::wire::write_varint(i32_field(event, "size")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn guardian_random_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let tail = f32::from_bits(
        u32::from_str_radix(event["tail"].as_str().ok_or("tail must be a string")?, 16)
            .map_err(|error| error.to_string())?,
    );
    world
        .entities_mut()
        .fix_guardian_random(entity_id, tail, i64_field(event, "seed")?);
    Ok(())
}

pub(crate) fn move_rot_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // A relative move with the turn the wire packs in a byte of 256ths.
    let degrees = |key: &str| -> Result<f32, String> {
        Ok(i32::from(i32_field(event, key)? as i8) as f32 * 360.0 / 256.0)
    };
    world.apply_entity_move(photon_world::EntityMoveInput {
        entity_id,
        delta_x: i32_field(event, "xa")? as i16,
        delta_y: i32_field(event, "ya")? as i16,
        delta_z: i32_field(event, "za")? as i16,
        has_position: true,
        y_rot: Some(degrees("y_rot")?),
        x_rot: Some(degrees("x_rot")?),
    });
    Ok(())
}

pub(crate) fn squid_speed_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let speed = u32::from_str_radix(string(event, "speed")?, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())?;
    world
        .entities_mut()
        .fix_squid_tentacle_speed(entity_id, speed);
    Ok(())
}

pub(crate) fn copper_golem_idle_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world
        .entities_mut()
        .fix_copper_golem_seed(entity_id, i64_field(event, "seed")?);
    Ok(())
}

pub(crate) fn rabbit_idle_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world.entities_mut().fix_rabbit_idle(
        entity_id,
        i32_field(event, "timeout")?,
        i64_field(event, "seed")?,
    );
    Ok(())
}

pub(crate) fn attributes_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let mut changed = Vec::new();
    for item in event["attributes"].as_array().ok_or("attributes")? {
        let (id, _) = photon_data::entity_type::attribute(item["name"].as_str().ok_or("name")?)
            .ok_or("no such attribute")?;
        let modifiers = item["modifiers"]
            .as_array()
            .ok_or("modifiers")?
            .iter()
            .map(|modifier| {
                Ok(photon_world::ClientAttributeModifierState::new(
                    modifier["id"].as_str().ok_or("id")?.to_owned(),
                    f64_field(modifier, "amount")?,
                    i32_field(modifier, "operation")? as u8,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        changed.push(photon_world::ClientAttributeState::new(
            id as i32,
            f64_field(item, "base")?,
            modifiers,
        ));
    }
    world.entities_mut().apply_attributes(entity_id, &changed);
    Ok(())
}

pub(crate) fn entity_event_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    world.apply_entity_event(entity_id, i32_field(event, "event")? as i8);
    Ok(())
}
