use super::*;

pub(crate) fn parse_set_passengers_packet(
    input: &[u8],
) -> Result<EntityPassengersObservation, ConnectionError> {
    let mut input = input;
    let vehicle_id = read_varint(&mut input, "set passengers vehicle id")?;
    let count = read_varint(&mut input, "set passengers count")?;
    if count < 0 {
        return Err(ConnectionError::Codec("set passengers count"));
    }
    let count =
        usize::try_from(count).map_err(|_| ConnectionError::Codec("set passengers count"))?;
    if count > input.len() {
        return Err(ConnectionError::Codec("set passengers count"));
    }
    let mut passenger_ids = Vec::with_capacity(count);
    for _ in 0..count {
        passenger_ids.push(read_varint(&mut input, "set passenger id")?);
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set passengers trailing payload"));
    }
    Ok(EntityPassengersObservation {
        vehicle_id,
        passenger_ids,
        applied_to_world: false,
        applied_vehicle_passengers: None,
        applied_passenger_vehicles: Vec::new(),
    })
}

pub(crate) fn parse_entity_metadata_packet(
    input: &[u8],
) -> Result<EntityMetadataObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity metadata entity id")?;
    if input.last() != Some(&255) {
        return Err(ConnectionError::Codec("entity metadata end marker"));
    }
    let metadata = input.to_vec();
    Ok(EntityMetadataObservation {
        entity_id,
        metadata_length: metadata.len(),
        applied_to_world: false,
        metadata,
    })
}

/// `ClientboundSetEquipmentPacket`: the entity, then each slot's byte (the
/// top bit says another follows) and stack.
pub(crate) fn parse_entity_equipment_packet(
    input: &[u8],
) -> Result<EntityEquipmentObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "set equipment entity id")?;
    let mut slots = Vec::new();
    loop {
        let slot_byte = read_byte(&mut input, "set equipment slot")?;
        let stack = parse_optional_item_stack(&mut input, "set equipment item stack")?;
        slots.push((usize::from(slot_byte & 127), stack.client_state()));
        if slot_byte & 128 == 0 {
            break;
        }
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set equipment trailing payload"));
    }
    Ok(EntityEquipmentObservation { entity_id, slots })
}

pub(crate) fn parse_entity_position_sync_packet(
    input: &[u8],
) -> Result<EntityPositionSyncObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity position sync entity id")?;
    let x = read_f64(&mut input, "entity position sync x")?;
    let y = read_f64(&mut input, "entity position sync y")?;
    let z = read_f64(&mut input, "entity position sync z")?;
    let delta_x = read_f64(&mut input, "entity position sync delta x")?;
    let delta_y = read_f64(&mut input, "entity position sync delta y")?;
    let delta_z = read_f64(&mut input, "entity position sync delta z")?;
    let y_rot = read_f32(&mut input, "entity position sync y rotation")?;
    let x_rot = read_f32(&mut input, "entity position sync x rotation")?;
    let on_ground = read_byte(&mut input, "entity position sync on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity position sync trailing payload",
        ));
    }
    Ok(EntityPositionSyncObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub(crate) fn parse_entity_teleport_packet(
    input: &[u8],
) -> Result<EntityTeleportObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity teleport entity id")?;
    let x = read_f64(&mut input, "entity teleport x")?;
    let y = read_f64(&mut input, "entity teleport y")?;
    let z = read_f64(&mut input, "entity teleport z")?;
    let delta_x = read_f64(&mut input, "entity teleport delta x")?;
    let delta_y = read_f64(&mut input, "entity teleport delta y")?;
    let delta_z = read_f64(&mut input, "entity teleport delta z")?;
    let y_rot = read_f32(&mut input, "entity teleport y rotation")?;
    let x_rot = read_f32(&mut input, "entity teleport x rotation")?;
    let relative_flags = read_i32(&mut input, "entity teleport relative flags")?;
    let on_ground = read_byte(&mut input, "entity teleport on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity teleport trailing payload"));
    }
    Ok(EntityTeleportObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        relative_flags,
        on_ground,
        motion_before: None,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
        applied_motion: None,
    })
}

pub(crate) fn parse_move_entity_pos_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, false)
}

pub(crate) fn parse_move_entity_pos_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, true)
}

pub(crate) fn parse_move_entity_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, false, true)
}

pub(super) fn parse_move_entity_packet(
    input: &[u8],
    has_position: bool,
    has_rotation: bool,
) -> Result<EntityMoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "move entity id")?;
    let (delta_x, delta_y, delta_z) = if has_position {
        (
            read_i16(&mut input, "move entity delta x")?,
            read_i16(&mut input, "move entity delta y")?,
            read_i16(&mut input, "move entity delta z")?,
        )
    } else {
        (0, 0, 0)
    };
    let (y_rot, x_rot) = if has_rotation {
        (
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity y rotation",
            )?)),
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity x rotation",
            )?)),
        )
    } else {
        (None, None)
    };
    let on_ground = read_byte(&mut input, "move entity on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("move entity trailing payload"));
    }
    Ok(EntityMoveObservation {
        entity_id,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        has_position,
        has_rotation,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub(crate) fn parse_remove_mob_effect_packet(
    input: &[u8],
) -> Result<MobEffectRemoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "remove mob effect entity id")?;
    let effect_registry_id = read_varint(&mut input, "remove mob effect registry id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("remove mob effect trailing payload"));
    }
    Ok(MobEffectRemoveObservation {
        entity_id,
        effect_registry_id,
        applied_to_world: false,
    })
}

pub(crate) fn parse_update_mob_effect_packet(
    input: &[u8],
) -> Result<MobEffectUpdateObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "update mob effect entity id")?;
    let effect_registry_id = read_varint(&mut input, "update mob effect registry id")?;
    let amplifier = read_varint(&mut input, "update mob effect amplifier")?;
    let duration_ticks = read_varint(&mut input, "update mob effect duration")?;
    let flags = read_byte(&mut input, "update mob effect flags")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("update mob effect trailing payload"));
    }
    Ok(MobEffectUpdateObservation {
        entity_id,
        effect_registry_id,
        amplifier,
        duration_ticks,
        flags,
        applied_to_world: false,
    })
}

pub(super) fn parse_optional_item_stack(
    input: &mut &[u8],
    name: &'static str,
) -> Result<ParsedItemStackObservation, ConnectionError> {
    let item_count = read_varint(input, name)?;
    if item_count <= 0 {
        return Ok(ParsedItemStackObservation {
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            patch_bytes: Vec::new(),
        });
    }
    let item_registry_id = read_varint(input, name)?;
    let patch_start = *input;
    let patch =
        voxel_game::item_components::read_patch(input).map_err(|_| ConnectionError::Codec(name))?;
    let positive_component_type_ids = patch.added.iter().map(|(id, _)| *id).collect();
    let negative_component_type_ids = patch.removed;
    let patch_bytes = patch_start[..patch_start.len() - input.len()].to_vec();
    Ok(ParsedItemStackObservation {
        item_count: Some(item_count),
        item_registry_id: Some(item_registry_id),
        positive_component_type_ids,
        negative_component_type_ids,
        patch_bytes,
    })
}

/// An item stack as an entity's data holds it (`ItemStack.OPTIONAL_STREAM_CODEC`
/// at the start of `value`): none when the field is empty or cannot be read.
pub(crate) fn parse_entity_item_stack(value: &[u8]) -> Option<ClientItemStackState> {
    let mut input = value;
    parse_optional_item_stack(&mut input, "entity item stack")
        .ok()?
        .client_state()
}

pub(crate) fn parse_container_set_content_packet(
    input: &[u8],
) -> Result<ContainerSetContentObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container content container id")?;
    let state_id = read_varint(&mut input, "container content state id")?;
    let item_count = read_count(&mut input, "container content item count")?;
    let mut hotbar_slot_zero_item_count = None;
    let mut hotbar_slot_zero_item_registry_id = None;
    let mut hotbar_slot_zero_positive_component_type_ids = Vec::new();
    let mut hotbar_slot_zero_negative_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_item_count = None;
    let mut main_inventory_slot_zero_item_registry_id = None;
    let mut main_inventory_slot_zero_positive_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_negative_component_type_ids = Vec::new();
    let mut items = Vec::with_capacity(item_count);
    for slot in 0..item_count {
        let item = parse_optional_item_stack(&mut input, "container content item stack")?;
        let client_state = item.client_state();
        if slot == 36 {
            hotbar_slot_zero_item_count = item.item_count;
            hotbar_slot_zero_item_registry_id = item.item_registry_id;
            hotbar_slot_zero_positive_component_type_ids = item.positive_component_type_ids.clone();
            hotbar_slot_zero_negative_component_type_ids = item.negative_component_type_ids.clone();
        } else if slot == 9 {
            main_inventory_slot_zero_item_count = item.item_count;
            main_inventory_slot_zero_item_registry_id = item.item_registry_id;
            main_inventory_slot_zero_positive_component_type_ids =
                item.positive_component_type_ids.clone();
            main_inventory_slot_zero_negative_component_type_ids =
                item.negative_component_type_ids.clone();
        }
        items.push(client_state);
    }
    let carried_item = parse_optional_item_stack(&mut input, "container content carried item")?;
    let carried_client_state = carried_item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container content trailing payload"));
    }
    Ok(ContainerSetContentObservation {
        container_id,
        state_id,
        item_count,
        items,
        hotbar_slot_zero_item_count,
        hotbar_slot_zero_item_registry_id,
        hotbar_slot_zero_positive_component_type_ids,
        hotbar_slot_zero_negative_component_type_ids,
        main_inventory_slot_zero_item_count,
        main_inventory_slot_zero_item_registry_id,
        main_inventory_slot_zero_positive_component_type_ids,
        main_inventory_slot_zero_negative_component_type_ids,
        carried_item_count: carried_item.item_count,
        carried_item_registry_id: carried_item.item_registry_id,
        carried_positive_component_type_ids: carried_item.positive_component_type_ids,
        carried_negative_component_type_ids: carried_item.negative_component_type_ids,
        carried_item: carried_client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_set_slot_packet(
    input: &[u8],
) -> Result<ContainerSetSlotObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container slot container id")?;
    let state_id = read_varint(&mut input, "container slot state id")?;
    let slot = read_i16(&mut input, "container slot index")?;
    let item = parse_optional_item_stack(&mut input, "container slot item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container slot trailing payload"));
    }
    Ok(ContainerSetSlotObservation {
        container_id,
        state_id,
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_cursor_item_packet(
    input: &[u8],
) -> Result<CursorItemObservation, ConnectionError> {
    let mut input = input;
    let item = parse_optional_item_stack(&mut input, "cursor item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("cursor item trailing payload"));
    }
    Ok(CursorItemObservation {
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_set_player_inventory_packet(
    input: &[u8],
) -> Result<SetPlayerInventoryObservation, ConnectionError> {
    let mut input = input;
    let slot = read_varint(&mut input, "player inventory slot")?;
    let item = parse_optional_item_stack(&mut input, "player inventory item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player inventory trailing payload"));
    }
    Ok(SetPlayerInventoryObservation {
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_system_chat_packet(
    input: &[u8],
) -> Result<SystemChatObservation, ConnectionError> {
    let mut input = input;
    let component = voxel_core::nbt::read_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("system chat component"))?;
    let translation_key = component.and_then(|fields| {
        fields.into_iter().find_map(|field| {
            if field.key == "translate" {
                match field.value {
                    voxel_core::nbt::FieldValue::Str(value) => Some(value),
                    _ => None,
                }
            } else {
                None
            }
        })
    });
    let overlay = read_byte(&mut input, "system chat overlay")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("system chat trailing payload"));
    }
    Ok(SystemChatObservation {
        overlay,
        translation_key,
    })
}
