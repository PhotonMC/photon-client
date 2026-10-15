use super::super::*;

pub(crate) fn creeper_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // `SynchedEntityData` as the packet carries it: id,
    // serializer, value.
    let mut data = Vec::new();
    if let Some(direction) = event["swell_dir"].as_i64() {
        data.extend([photon_world::CREEPER_SWELL_DIR_DATA_ID, 1]);
        photon_core::wire::write_varint(direction as i32, &mut data);
    }
    if let Some(ignited) = event["ignited"].as_bool() {
        data.extend([photon_world::CREEPER_IGNITED_DATA_ID, 8, u8::from(ignited)]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn bee_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(flags) = event["flags"].as_i64() {
        data.extend([photon_world::BEE_FLAGS_DATA_ID, 0, flags as u8]);
    }
    if let Some(end) = event["anger_end"].as_i64() {
        data.extend([photon_world::BEE_ANGER_END_DATA_ID, 2]);
        photon_core::wire::write_varlong(end, &mut data);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn wolf_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(flags) = event["flags"].as_i64() {
        data.extend([photon_world::TAMABLE_FLAGS_DATA_ID, 0, flags as u8]);
    }
    if let Some(interested) = event["interested"].as_bool() {
        data.extend([
            photon_world::WOLF_INTERESTED_DATA_ID,
            8,
            u8::from(interested),
        ]);
    }
    if let Some(dye) = event["collar"].as_i64() {
        data.extend([photon_world::WOLF_COLLAR_DATA_ID, 1]);
        photon_core::wire::write_varint(dye as i32, &mut data);
    }
    if let Some(end) = event["anger_end"].as_i64() {
        data.extend([photon_world::WOLF_ANGER_END_DATA_ID, 2]);
        photon_core::wire::write_varlong(end, &mut data);
    }
    if let Some(health) = event["health"].as_str() {
        let health = u32::from_str_radix(health, 16).map_err(|e| e.to_string())?;
        // The health data is float 3 of the living entity's.
        data.extend([9, 3]);
        data.extend(health.to_be_bytes());
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn strider_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let suffocating = event["suffocating"].as_bool().ok_or("suffocating")?;
    let data = [
        photon_world::STRIDER_SUFFOCATING_DATA_ID,
        8,
        u8::from(suffocating),
        255,
    ];
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn panda_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    for (key, id) in [
        ("unhappy", photon_world::PANDA_UNHAPPY_DATA_ID),
        ("sneeze", photon_world::PANDA_SNEEZE_DATA_ID),
        ("eat", photon_world::PANDA_EAT_DATA_ID),
    ] {
        if let Some(value) = event[key].as_i64() {
            data.extend([id, 1]);
            photon_core::wire::write_varint(value as i32, &mut data);
        }
    }
    for (key, id) in [
        ("main_gene", photon_world::PANDA_MAIN_GENE_DATA_ID),
        ("hidden_gene", photon_world::PANDA_HIDDEN_GENE_DATA_ID),
        ("flags", photon_world::PANDA_FLAGS_DATA_ID),
    ] {
        if let Some(value) = event[key].as_i64() {
            data.extend([id, 0, value as u8]);
        }
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn fox_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(variant) = event["variant"].as_i64() {
        data.extend([photon_world::FOX_VARIANT_DATA_ID, 1]);
        photon_core::wire::write_varint(variant as i32, &mut data);
    }
    if let Some(flags) = event["flags"].as_i64() {
        data.extend([photon_world::FOX_FLAGS_DATA_ID, 0, flags as u8]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn polar_bear_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let standing = event["standing"].as_bool().ok_or("standing")?;
    let data = [
        photon_world::POLAR_BEAR_STANDING_DATA_ID,
        8,
        u8::from(standing),
        255,
    ];
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn cat_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(flags) = event["flags"].as_i64() {
        data.extend([photon_world::TAMABLE_FLAGS_DATA_ID, 0, flags as u8]);
    }
    for (key, id) in [
        ("lying", photon_world::CAT_LYING_DATA_ID),
        ("relaxed", photon_world::CAT_RELAX_DATA_ID),
    ] {
        if let Some(value) = event[key].as_bool() {
            data.extend([id, 8, u8::from(value)]);
        }
    }
    if let Some(dye) = event["collar"].as_i64() {
        data.extend([photon_world::CAT_COLLAR_DATA_ID, 1]);
        photon_core::wire::write_varint(dye as i32, &mut data);
    }
    if let Some(pose) = event["pose"].as_str() {
        // `Pose` ids: standing is zero, crouching five.
        let id = if pose == "CROUCHING" { 5 } else { 0 };
        data.extend([6, 20, id]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn horse_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let data = [
        photon_world::HORSE_FLAGS_DATA_ID,
        0,
        i32_field(event, "flags")? as u8,
        255,
    ];
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn camel_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(dash) = event["dash"].as_bool() {
        data.extend([photon_world::CAMEL_DASH_DATA_ID, 8, u8::from(dash)]);
    }
    if let Some(tick) = event["last_pose"].as_i64() {
        data.extend([photon_world::CAMEL_LAST_POSE_CHANGE_DATA_ID, 2]);
        photon_core::wire::write_varlong(tick, &mut data);
    }
    if let Some(pose) = event["pose"].as_str() {
        // `Pose` ids: standing is zero, sitting ten.
        let id = if pose == "SITTING" { 10 } else { 0 };
        data.extend([6, 20, id]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn copper_golem_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(weather) = event["weather"].as_i64() {
        data.extend([
            photon_world::COPPER_GOLEM_WEATHER_DATA_ID,
            38,
            weather as u8,
        ]);
    }
    if let Some(state) = event["state"].as_i64() {
        data.extend([photon_world::COPPER_GOLEM_STATE_DATA_ID, 37, state as u8]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn shulker_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(face) = event["face"].as_i64() {
        data.extend([photon_world::SHULKER_ATTACH_FACE_DATA_ID, 12, face as u8]);
    }
    for (key, id) in [
        ("peek", photon_world::SHULKER_PEEK_DATA_ID),
        ("color", photon_world::SHULKER_COLOR_DATA_ID),
    ] {
        if let Some(value) = event[key].as_i64() {
            data.extend([id, 0, value as u8]);
        }
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn glow_squid_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = vec![photon_world::GLOW_SQUID_DARK_TICKS_DATA_ID, 1];
    photon_core::wire::write_varint(i32_field(event, "dark_ticks")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn baby_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = vec![16, 8, u8::from(event["baby"].as_bool().ok_or("baby")?)];
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn axolotl_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if event.get("variant").is_some() {
        data.extend([photon_world::AXOLOTL_VARIANT_DATA_ID, 1]);
        photon_core::wire::write_varint(i32_field(event, "variant")?, &mut data);
    }
    if let Some(playing_dead) = event["playing_dead"].as_bool() {
        data.extend([
            photon_world::AXOLOTL_PLAYING_DEAD_DATA_ID,
            8,
            u8::from(playing_dead),
        ]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn flier_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if event.get("parrot_variant").is_some() {
        data.extend([photon_world::PARROT_VARIANT_DATA_ID, 1]);
        photon_core::wire::write_varint(i32_field(event, "parrot_variant")?, &mut data);
    }
    if let Some(flags) = event["bat_flags"].as_i64() {
        data.extend([photon_world::BAT_FLAGS_DATA_ID, 0, flags as u8]);
    }
    if let Some(flags) = event["vex_flags"].as_i64() {
        data.extend([photon_world::VEX_FLAGS_DATA_ID, 0, flags as u8]);
    }
    if let Some(dancing) = event["allay_dancing"].as_bool() {
        data.extend([photon_world::ALLAY_DANCING_DATA_ID, 8, u8::from(dancing)]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn guardian_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(moving) = event["moving"].as_bool() {
        data.extend([photon_world::GUARDIAN_MOVING_DATA_ID, 8, u8::from(moving)]);
    }
    if event.get("target").is_some() {
        data.extend([photon_world::GUARDIAN_ATTACK_TARGET_DATA_ID, 1]);
        photon_core::wire::write_varint(i32_field(event, "target")?, &mut data);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn frog_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // `Pose` ids: standing is zero, long jumping six, croaking eight, using
    // the tongue nine, sliding fifteen, shooting sixteen, inhaling seventeen.
    let id = match event["pose"].as_str().ok_or("pose")? {
        "LONG_JUMPING" => 6,
        "CROAKING" => 8,
        "USING_TONGUE" => 9,
        "SLIDING" => 15,
        "SHOOTING" => 16,
        "INHALING" => 17,
        "ROARING" => 11,
        "SNIFFING" => 12,
        "EMERGING" => 13,
        "DIGGING" => 14,
        _ => 0,
    };
    world.apply_entity_metadata(entity_id, &[6, 20, id, 255]);
    Ok(())
}

pub(crate) fn sniffer_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // `Sniffer.DATA_STATE`: serializer 35, then the state's id.
    let mut data = vec![photon_world::mobs::sniffer::STATE_DATA_ID, 35];
    photon_core::wire::write_varint(i32_field(event, "state")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn armadillo_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // `Armadillo.ARMADILLO_STATE`: serializer 36, then the state's id.
    let mut data = vec![photon_world::mobs::armadillo::STATE_DATA_ID, 36];
    photon_core::wire::write_varint(i32_field(event, "state")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn creaking_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // The booleans `Creaking` defines, each when the event names it.
    let mut data = Vec::new();
    for (key, id) in [
        ("can_move", photon_world::mobs::creaking::CAN_MOVE_DATA_ID),
        ("active", photon_world::mobs::creaking::IS_ACTIVE_DATA_ID),
        (
            "tearing_down",
            photon_world::mobs::creaking::IS_TEARING_DOWN_DATA_ID,
        ),
    ] {
        if let Some(value) = event[key].as_bool() {
            data.extend([id, 8, u8::from(value)]);
        }
    }
    if event["home"].as_bool() == Some(true) {
        // An optional block position: present, then the packed position.
        data.extend([photon_world::mobs::creaking::HOME_POS_DATA_ID, 11, 1]);
        data.extend(0_i64.to_be_bytes());
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn warden_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    // `Warden.CLIENT_ANGER_LEVEL`, an int.
    let mut data = vec![photon_world::mobs::warden::CLIENT_ANGER_DATA_ID, 1];
    photon_core::wire::write_varint(i32_field(event, "anger")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn turtle_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let mut data = Vec::new();
    if let Some(has_egg) = event["has_egg"].as_bool() {
        data.extend([photon_world::TURTLE_HAS_EGG_DATA_ID, 8, u8::from(has_egg)]);
    }
    if let Some(laying) = event["laying_egg"].as_bool() {
        data.extend([photon_world::TURTLE_LAYING_EGG_DATA_ID, 8, u8::from(laying)]);
    }
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}

pub(crate) fn fish_data_event(
    world: &mut ClientWorld,
    entity_id: EntityId,
    event: &Value,
) -> Result<(), String> {
    let data_id = match string(event, "fish")? {
        "salmon" => photon_world::SALMON_TYPE_DATA_ID,
        _ => photon_world::PUFFERFISH_PUFF_STATE_DATA_ID,
    };
    let mut data = vec![data_id, 1];
    photon_core::wire::write_varint(i32_field(event, "value")?, &mut data);
    data.push(255);
    world.apply_entity_metadata(entity_id, &data);
    Ok(())
}
