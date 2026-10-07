use super::*;

/// `RecipeBookSettings.read`: open and filtering flags for crafting,
/// furnace, blast furnace and smoker, in that order.
pub(crate) fn parse_recipe_book_settings_packet(
    input: &[u8],
) -> Result<[voxel_world::BookTypeSettings; 4], ConnectionError> {
    let mut input = input;
    let mut settings = [voxel_world::BookTypeSettings::default(); 4];
    for entry in &mut settings {
        entry.open = read_byte(&mut input, "recipe book open")? != 0;
        entry.filtering = read_byte(&mut input, "recipe book filtering")? != 0;
    }
    Ok(settings)
}

/// `ClientboundPlaceGhostRecipePacket`.
pub(crate) fn parse_place_ghost_recipe_packet(
    input: &[u8],
) -> Result<(i32, voxel_world::RecipeDisplay), ConnectionError> {
    let mut input = input;
    let container = read_varint(&mut input, "ghost container")?;
    Ok((container, read_recipe_display(&mut input)?))
}

pub(crate) fn parse_set_health_packet(input: &[u8]) -> Result<HealthObservation, ConnectionError> {
    let mut input = input;
    let packet = HealthObservation {
        health: read_f32(&mut input, "set health health")?,
        food: read_varint(&mut input, "set health food")?,
        saturation: read_f32(&mut input, "set health saturation")?,
        applied_to_world: false,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set health trailing payload"));
    }
    Ok(packet)
}

pub fn parse_player_abilities_packet(
    input: &[u8],
) -> Result<PlayerAbilitiesPacket, ConnectionError> {
    let mut input = input;
    let flags = read_byte(&mut input, "player abilities flags")?;
    let packet = PlayerAbilitiesPacket {
        invulnerable: flags & 0x01 != 0,
        flying: flags & 0x02 != 0,
        can_fly: flags & 0x04 != 0,
        can_instabuild: flags & 0x08 != 0,
        flying_speed: read_f32(&mut input, "player abilities flying speed")?,
        walking_speed: read_f32(&mut input, "player abilities walking speed")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player abilities trailing payload"));
    }
    Ok(packet)
}

pub fn parse_game_event_packet(input: &[u8]) -> Result<GameEventPacket, ConnectionError> {
    let mut input = input;
    let packet = GameEventPacket {
        event_id: read_byte(&mut input, "game event id")?,
        param: read_f32(&mut input, "game event parameter")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("game event trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn parse_update_attributes_packet(
    input: &[u8],
) -> Result<AttributeUpdateObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "update attributes entity id")?;
    let attribute_count = read_count(&mut input, "update attributes count")?;
    if attribute_count > 128 {
        return Err(ConnectionError::Codec("update attributes count"));
    }
    let mut attributes = Vec::with_capacity(attribute_count);
    for _ in 0..attribute_count {
        let attribute_id = read_varint(&mut input, "update attribute id")?;
        let base_value = read_f64(&mut input, "update attribute base value")?;
        let modifier_count = read_count(&mut input, "update attribute modifier count")?;
        let mut modifiers = Vec::with_capacity(modifier_count);
        for _ in 0..modifier_count {
            let id = read_identifier_string(&mut input, "update attribute modifier id")?;
            let amount = read_f64(&mut input, "update attribute modifier amount")?;
            let operation = read_byte(&mut input, "update attribute modifier operation")?;
            modifiers.push(ClientAttributeModifierState::from_wire(
                id, amount, operation,
            ));
        }
        attributes.push(ClientAttributeState::from_wire(
            attribute_id,
            base_value,
            modifiers,
        ));
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("update attributes trailing payload"));
    }
    Ok(AttributeUpdateObservation {
        entity_id,
        attributes,
        applied_to_world: false,
    })
}

pub(super) fn parse_spawn_info(input: &mut &[u8]) -> Result<SpawnInfoObservation, ConnectionError> {
    let dimension_type_id = read_varint(input, "spawn dimension type")?;
    let dimension = read_identifier_string(input, "spawn dimension")?;
    let seed = read_i64(input, "spawn seed")?;
    let game_type = read_byte(input, "spawn game type")?;
    let previous_game_type = match read_byte(input, "spawn previous game type")? {
        u8::MAX => None,
        value => Some(value),
    };
    let is_debug = read_bool(input, "spawn debug flag")?;
    let is_flat = read_bool(input, "spawn flat flag")?;
    let last_death_location = if read_bool(input, "spawn death-location presence")? {
        Some(GlobalPositionObservation {
            dimension: read_identifier_string(input, "spawn death-location dimension")?,
            position: read_i64(input, "spawn death-location position")?,
        })
    } else {
        None
    };
    let portal_cooldown = read_varint(input, "spawn portal cooldown")?;
    let sea_level = read_varint(input, "spawn sea level")?;
    Ok(SpawnInfoObservation {
        dimension_type_id,
        dimension,
        seed,
        game_type,
        previous_game_type,
        is_debug,
        is_flat,
        last_death_location,
        portal_cooldown,
        sea_level,
    })
}

pub(super) fn read_identifier_string(
    input: &mut &[u8],
    name: &'static str,
) -> Result<String, ConnectionError> {
    voxel_core::identifier::read_identifier(input)
        .map(|identifier| identifier.as_string())
        .map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn parse_block_update_packet(
    input: &[u8],
) -> Result<BlockUpdateObservation, ConnectionError> {
    let mut input = input;
    let position =
        voxel_core::binary::BlockPos::unpack(read_i64(&mut input, "block update position")? as u64);
    let state_id = read_varint(&mut input, "block update state")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block update trailing payload"));
    }
    Ok(BlockUpdateObservation {
        x: position.x,
        y: position.y,
        z: position.z,
        state_id,
    })
}

pub(crate) fn parse_block_entity_data_packet(
    input: &[u8],
) -> Result<BlockEntityObservation, ConnectionError> {
    let mut input = input;
    let position =
        voxel_core::binary::BlockPos::unpack(read_i64(&mut input, "block entity position")? as u64);
    let type_id = read_varint(&mut input, "block entity type")?;
    let tag = read_nbt_bytes(&mut input, "block entity nbt")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block entity trailing payload"));
    }
    let tag_length = tag.as_ref().map_or(0, Vec::len);
    Ok(BlockEntityObservation {
        x: position.x,
        y: position.y,
        z: position.z,
        type_id,
        tag_length,
        applied_to_world: false,
        tag,
    })
}

pub(crate) fn parse_block_destruction_packet(
    input: &[u8],
) -> Result<BlockDestructionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "block destruction entity id")?;
    let position = voxel_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block destruction position",
    )? as u64);
    let progress = read_byte(&mut input, "block destruction progress")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block destruction trailing payload"));
    }
    Ok(BlockDestructionObservation {
        entity_id,
        x: position.x,
        y: position.y,
        z: position.z,
        progress,
    })
}

pub(crate) fn parse_add_entity_packet(
    input: &[u8],
) -> Result<AddEntityObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "add entity id")?;
    let uuid = voxel_core::binary::read_uuid(&mut input)
        .map_err(|_| ConnectionError::Codec("add entity uuid"))?;
    let entity_type_id = read_varint(&mut input, "add entity type")?;
    let x = read_f64(&mut input, "add entity x")?;
    let y = read_f64(&mut input, "add entity y")?;
    let z = read_f64(&mut input, "add entity z")?;
    let movement = read_lp_vec3(&mut input, "add entity movement")?;
    let x_rot = decode_packed_degrees(read_byte(&mut input, "add entity x rotation")?);
    let y_rot = decode_packed_degrees(read_byte(&mut input, "add entity y rotation")?);
    let y_head_rot = decode_packed_degrees(read_byte(&mut input, "add entity head rotation")?);
    let data = read_varint(&mut input, "add entity data")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("add entity trailing payload"));
    }
    Ok(AddEntityObservation {
        entity_id,
        uuid,
        entity_type_id,
        x,
        y,
        z,
        movement,
        x_rot,
        y_rot,
        y_head_rot,
        data,
        applied_to_world: false,
        applied_rotation: None,
    })
}

/// `ClientboundSetEntityLinkPacket`: the entity on a lead and the entity that holds it (zero
/// when it is let go), each as an `int`.
pub(crate) fn parse_set_entity_link_packet(input: &[u8]) -> Result<(i32, i32), ConnectionError> {
    let mut input = input;
    let source = read_i32(&mut input, "entity link source id")?;
    let holder = read_i32(&mut input, "entity link holder id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity link trailing payload"));
    }
    Ok((source, holder))
}

/// `ClientboundMoveMinecartPacket`: the minecart and the steps it took along the track, each a
/// position and a movement (doubles), two turns (a byte each), and a weight (a float).
pub(crate) fn parse_move_minecart_packet(
    input: &[u8],
) -> Result<(i32, Vec<MinecartStep>), ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "minecart entity id")?;
    let count = read_varint(&mut input, "minecart step count")?;
    let mut steps = Vec::new();
    for _ in 0..count {
        let mut vec3 = |name| -> Result<(f64, f64, f64), ConnectionError> {
            Ok((
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
            ))
        };
        let position = vec3("minecart step position")?;
        let movement = vec3("minecart step movement")?;
        let y_rot = decode_packed_degrees(read_byte(&mut input, "minecart step y rot")?);
        let x_rot = decode_packed_degrees(read_byte(&mut input, "minecart step x rot")?);
        let weight = read_f32(&mut input, "minecart step weight")?;
        steps.push(MinecartStep {
            position,
            movement,
            y_rot,
            x_rot,
            weight,
        });
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("minecart steps trailing payload"));
    }
    Ok((entity_id, steps))
}

/// `ClientboundMapItemDataPacket`: the map's id, scale and lock, the decorations (if sent) and the
/// patch of colours (a width of zero is none).
pub(crate) fn parse_map_item_data_packet(
    input: &[u8],
) -> Result<voxel_world::map::MapItemDataInput, ConnectionError> {
    use voxel_world::map::{MapDecoration, MapItemDataInput, MapPatch};
    let mut input = input;
    let map_id = read_varint(&mut input, "map id")?;
    let scale = read_byte(&mut input, "map scale")? as i8;
    let locked = read_bool(&mut input, "map locked")?;
    let decorations = if read_bool(&mut input, "map decorations present")? {
        let count = read_varint(&mut input, "map decoration count")?;
        let mut decorations = Vec::new();
        for _ in 0..count {
            let type_id = read_varint(&mut input, "map decoration type")?;
            let x = read_byte(&mut input, "map decoration x")? as i8;
            let y = read_byte(&mut input, "map decoration y")? as i8;
            let rot = read_byte(&mut input, "map decoration rot")? as i8;
            let name = if read_bool(&mut input, "map decoration name present")? {
                let before = input;
                voxel_core::nbt::read_any_nbt(&mut input)
                    .map_err(|_| ConnectionError::Codec("map decoration name"))?;
                Some(before[..before.len() - input.len()].to_vec())
            } else {
                None
            };
            decorations.push(MapDecoration {
                type_id,
                x,
                y,
                rot: rot & 15,
                name,
            });
        }
        Some(decorations)
    } else {
        None
    };
    let width = read_byte(&mut input, "map patch width")?;
    let patch = if width > 0 {
        let height = read_byte(&mut input, "map patch height")?;
        let start_x = read_byte(&mut input, "map patch x")?;
        let start_y = read_byte(&mut input, "map patch y")?;
        let length = read_count(&mut input, "map patch colour count")?;
        let colors = input
            .get(..length)
            .ok_or(ConnectionError::Codec("map patch colours"))?
            .to_vec();
        input = &input[length..];
        Some(MapPatch {
            start_x: usize::from(start_x),
            start_y: usize::from(start_y),
            width: usize::from(width),
            height: usize::from(height),
            colors,
        })
    } else {
        None
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("map item data trailing payload"));
    }
    Ok(MapItemDataInput {
        map_id,
        scale,
        locked,
        decorations,
        patch,
    })
}

pub(crate) fn parse_set_entity_motion_packet(
    input: &[u8],
) -> Result<EntityMotionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity motion entity id")?;
    let movement = read_lp_vec3(&mut input, "entity motion movement")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity motion trailing payload"));
    }
    Ok(EntityMotionObservation {
        entity_id,
        x: movement.0,
        y: movement.1,
        z: movement.2,
        applied_to_world: false,
        applied_motion: None,
    })
}

pub(crate) fn parse_rotate_head_packet(
    input: &[u8],
) -> Result<EntityHeadRotationObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity head rotation entity id")?;
    let y_head_rot = decode_packed_degrees(read_byte(&mut input, "entity head rotation angle")?);
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity head rotation trailing payload",
        ));
    }
    Ok(EntityHeadRotationObservation {
        entity_id,
        y_head_rot,
        applied_to_world: false,
        applied_head_rotation: None,
    })
}

pub(crate) fn parse_remove_entities_packet(
    input: &[u8],
) -> Result<RemoveEntitiesObservation, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "remove entities count")?;
    let mut entity_ids = Vec::new();
    if count > 0 {
        let count =
            usize::try_from(count).map_err(|_| ConnectionError::Codec("remove entities count"))?;
        entity_ids.reserve(count.min(input.len()));
        for _ in 0..count {
            entity_ids.push(read_varint(&mut input, "remove entity id")?);
        }
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("remove entities trailing payload"));
    }
    Ok(RemoveEntitiesObservation {
        entity_ids,
        applied_entity_count: 0,
    })
}
