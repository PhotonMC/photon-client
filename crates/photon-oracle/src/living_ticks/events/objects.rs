use super::super::*;

pub(crate) fn boat_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    for (key, id) in [
        ("hurt", photon_world::BOAT_HURT_DATA_ID),
        ("hurt_dir", photon_world::BOAT_HURT_DIR_DATA_ID),
        ("bubble_time", photon_world::BOAT_BUBBLE_TIME_DATA_ID),
    ] {
        if event.get(key).is_some() {
            data.extend([id, 1]);
            photon_core::wire::write_varint(i32_field(event, key)?, &mut data);
        }
    }
    if let Some(damage) = event["damage"].as_f64() {
        data.extend([photon_world::BOAT_DAMAGE_DATA_ID, 3]);
        data.extend((damage as f32).to_be_bytes());
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn arrow_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // `AbstractArrow.IN_GROUND` (a boolean), `Arrow.ID_EFFECT_COLOR` (an int).
    let mut data = Vec::new();
    if let Some(value) = event["in_ground"].as_bool() {
        data.extend([
            photon_world::objects::arrow::IN_GROUND_DATA_ID,
            8,
            u8::from(value),
        ]);
    }
    if let Some(value) = event["color"].as_i64() {
        data.extend([photon_world::objects::arrow::EFFECT_COLOR_DATA_ID, 1]);
        photon_core::wire::write_varint(value as i32, &mut data);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn minecart_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // `VehicleEntity`'s hit and `AbstractMinecart`'s block, its offset, a furnace's fuel.
    let mut data = Vec::new();
    if let Some(value) = event["hurt"].as_i64() {
        data.extend([photon_world::objects::minecart::HURT_DATA_ID, 1]);
        photon_core::wire::write_varint(value as i32, &mut data);
    }
    if let Some(value) = event["hurt_dir"].as_i64() {
        data.extend([photon_world::objects::minecart::HURT_DIR_DATA_ID, 1]);
        photon_core::wire::write_varint(value as i32, &mut data);
    }
    if let Some(value) = event["damage"].as_f64() {
        data.extend([photon_world::objects::minecart::DAMAGE_DATA_ID, 3]);
        data.extend((value as f32).to_be_bytes());
    }
    if let Some(value) = event["display"].as_i64() {
        data.extend([
            photon_world::objects::minecart::CUSTOM_DISPLAY_BLOCK_DATA_ID,
            15,
        ]);
        photon_core::wire::write_varint(value as i32, &mut data);
    }
    if event["no_display"].as_bool() == Some(true) {
        data.extend([
            photon_world::objects::minecart::CUSTOM_DISPLAY_BLOCK_DATA_ID,
            15,
            0,
        ]);
    }
    if let Some(value) = event["offset"].as_i64() {
        data.extend([photon_world::objects::minecart::DISPLAY_OFFSET_DATA_ID, 1]);
        photon_core::wire::write_varint(value as i32, &mut data);
    }
    if let Some(value) = event["fuel"].as_bool() {
        data.extend([
            photon_world::objects::minecart::FUEL_DATA_ID,
            8,
            u8::from(value),
        ]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn crystal_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // `EndCrystal`'s data: the beam's block (optional) and whether the base shows.
    let mut data = Vec::new();
    if let Some(target) = event["target"].as_array() {
        let coordinate = |index: usize| target[index].as_i64().unwrap_or(0) as i32;
        data.extend([
            photon_world::objects::end_crystal::BEAM_TARGET_DATA_ID,
            11,
            1,
        ]);
        photon_core::binary::write_blockpos(
            photon_core::binary::BlockPos {
                x: coordinate(0),
                y: coordinate(1),
                z: coordinate(2),
            },
            &mut data,
        );
    }
    if event["no_target"].as_bool() == Some(true) {
        data.extend([
            photon_world::objects::end_crystal::BEAM_TARGET_DATA_ID,
            11,
            0,
        ]);
    }
    if let Some(bottom) = event["bottom"].as_bool() {
        data.extend([
            photon_world::objects::end_crystal::SHOW_BOTTOM_DATA_ID,
            8,
            u8::from(bottom),
        ]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn dragon_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // `EnderDragon.DATA_PHASE`, an int.
    let mut data = vec![photon_world::mobs::ender_dragon::PHASE_DATA_ID, 1];
    photon_core::wire::write_varint(i32_field(event, "phase")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn display_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // A display's data in the wire's own form (data id, serializer, value).
    let float = |value: &Value| -> Result<f32, String> {
        u32::from_str_radix(value.as_str().ok_or("a float is a string")?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let mut data = Vec::new();
    for (key, id) in [
        (
            "start_delta",
            photon_world::objects::display::START_DELTA_DATA_ID,
        ),
        (
            "duration",
            photon_world::objects::display::INTERPOLATION_DURATION_DATA_ID,
        ),
        (
            "pos_rot",
            photon_world::objects::display::POS_ROT_DURATION_DATA_ID,
        ),
        (
            "brightness",
            photon_world::objects::display::BRIGHTNESS_DATA_ID,
        ),
        ("glow", photon_world::objects::display::GLOW_COLOR_DATA_ID),
    ] {
        if event[key].is_i64() {
            data.extend([id, 1]);
            photon_core::wire::write_varint(i32_field(event, key)?, &mut data);
        }
    }
    for (key, id) in [
        (
            "view_range",
            photon_world::objects::display::VIEW_RANGE_DATA_ID,
        ),
        (
            "shadow_radius",
            photon_world::objects::display::SHADOW_RADIUS_DATA_ID,
        ),
        (
            "shadow_strength",
            photon_world::objects::display::SHADOW_STRENGTH_DATA_ID,
        ),
        ("width", photon_world::objects::display::WIDTH_DATA_ID),
        ("height", photon_world::objects::display::HEIGHT_DATA_ID),
    ] {
        if event[key].is_string() {
            data.extend([id, 3]);
            data.extend(float(&event[key])?.to_be_bytes());
        }
    }
    if event["billboard"].is_i64() {
        data.extend([photon_world::objects::display::BILLBOARD_DATA_ID, 0]);
        data.push(i32_field(event, "billboard")? as u8);
    }
    for (key, id, serializer, count) in [
        (
            "translation",
            photon_world::objects::display::TRANSLATION_DATA_ID,
            39,
            3,
        ),
        (
            "scale",
            photon_world::objects::display::SCALE_DATA_ID,
            39,
            3,
        ),
        (
            "left_rotation",
            photon_world::objects::display::LEFT_ROTATION_DATA_ID,
            40,
            4,
        ),
        (
            "right_rotation",
            photon_world::objects::display::RIGHT_ROTATION_DATA_ID,
            40,
            4,
        ),
    ] {
        if let Some(parts) = event[key].as_array() {
            data.extend([id, serializer]);
            for part in parts.iter().take(count) {
                data.extend(float(part)?.to_be_bytes());
            }
        }
    }
    if event["block_state"].is_i64() {
        data.extend([photon_world::objects::display::BLOCK_STATE_DATA_ID, 14]);
        photon_core::wire::write_varint(i32_field(event, "block_state")?, &mut data);
    }
    if let Some(text) = event["text"].as_str() {
        // A component that is only a string: a string tag.
        data.extend([photon_world::objects::display::TEXT_DATA_ID, 5, 8]);
        data.extend((text.len() as u16).to_be_bytes());
        data.extend(text.as_bytes());
    }
    for (key, id) in [
        (
            "line_width",
            photon_world::objects::display::LINE_WIDTH_DATA_ID,
        ),
        (
            "background",
            photon_world::objects::display::BACKGROUND_DATA_ID,
        ),
    ] {
        if event[key].is_i64() {
            data.extend([id, 1]);
            photon_core::wire::write_varint(i32_field(event, key)?, &mut data);
        }
    }
    for (key, id) in [
        (
            "text_opacity",
            photon_world::objects::display::TEXT_OPACITY_DATA_ID,
        ),
        (
            "style_flags",
            photon_world::objects::display::STYLE_FLAGS_DATA_ID,
        ),
    ] {
        if event[key].is_i64() {
            data.extend([id, 0]);
            data.push(i32_field(event, key)? as u8);
        }
    }
    if event["item_display"].is_i64() {
        data.extend([photon_world::objects::display::ITEM_DISPLAY_DATA_ID, 0]);
        data.push(i32_field(event, "item_display")? as u8);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    if let Some(name) = event["item"].as_str() {
        let id = photon_data::item::registry::index(name).unwrap_or(0);
        world.entities_mut().apply_item_stack(
            entity_id,
            Some(photon_world::ClientItemStackState::new(1, id as i32)),
        );
    }
    Ok(())
}

pub(crate) fn minecart_steps_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // The steps along the track, with the turns the wire packs in a byte of 256ths.
    let degrees = |step: &Value, key: &str| -> Result<f32, String> {
        Ok(i32::from(i32_field(step, key)? as i8) as f32 * 360.0 / 256.0)
    };
    let mut steps = Vec::new();
    for step in event["steps"].as_array().ok_or("steps")? {
        steps.push(photon_world::objects::minecart_track::MinecartStep {
            position: (bits64(step, "x")?, bits64(step, "y")?, bits64(step, "z")?),
            movement: (
                bits64(step, "mx")?,
                bits64(step, "my")?,
                bits64(step, "mz")?,
            ),
            y_rot: degrees(step, "y_rot")?,
            x_rot: degrees(step, "x_rot")?,
            weight: f32::from_bits(
                u32::from_str_radix(string(step, "weight")?, 16)
                    .map_err(|error| error.to_string())?,
            ),
        });
    }
    world.apply_minecart_steps(entity_id, &steps);
    Ok(())
}

pub(crate) fn wither_data_event(
    world: &mut ClientWorld,
    entity_id: i32,
    event: &Value,
) -> Result<(), String> {
    // `WitherBoss`'s int data: the side heads' targets and the invulnerable ticks.
    let mut data = Vec::new();
    for (key, id) in [
        ("target_b", photon_world::mobs::wither::TARGET_B_DATA_ID),
        ("target_c", photon_world::mobs::wither::TARGET_C_DATA_ID),
        (
            "invulnerable",
            photon_world::mobs::wither::INVULNERABLE_DATA_ID,
        ),
    ] {
        if event[key].is_i64() {
            data.extend([id, 1]);
            photon_core::wire::write_varint(i32_field(event, key)?, &mut data);
        }
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}
