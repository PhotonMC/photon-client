use super::*;
use photon_core::ids::EntityId;

/// `ClientboundEntityEventPacket.STREAM_CODEC`: the entity's id and the
/// event (a byte).
/// `ClientboundAnimatePacket`: the entity and the action.
pub fn parse_animate_packet(input: &[u8]) -> Result<(EntityId, u8), ConnectionError> {
    let mut input = input;
    let id = EntityId(read_varint(&mut input, "animate entity id")?);
    let action = read_byte(&mut input, "animate action")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("animate trailing payload"));
    }
    Ok((id, action))
}

pub fn parse_entity_event_packet(input: &[u8]) -> Result<(EntityId, i8), ConnectionError> {
    let mut input = input;
    let id = EntityId(read_i32(&mut input, "entity event id")?);
    let event = *input
        .first()
        .ok_or(ConnectionError::Codec("entity event"))? as i8;
    Ok((id, event))
}

/// The entity a `ClientboundDamageEventPacket` or a
/// `ClientboundHurtAnimationPacket` names: both begin with its id as a
/// var int. The rest (the source, the yaw) is not read.
pub fn parse_entity_id_prefix(input: &[u8]) -> Result<i32, ConnectionError> {
    let mut input = input;
    photon_core::wire::read_varint(&mut input).map_err(|_| ConnectionError::Codec("entity id"))
}

/// `ClientboundDamageEventPacket`: the entity hurt, and the registry id of the damage type that
/// hurt it (`None` when the packet carries the type itself instead of naming one, which a server
/// that syncs its registries does not do). The rest (the causes, the place) is not read.
pub fn parse_damage_event_packet(
    input: &[u8],
) -> Result<(EntityId, Option<usize>), ConnectionError> {
    let mut input = input;
    let id = EntityId(read_varint(&mut input, "damage event entity id")?);
    // `ByteBufCodecs.holder`: the registry id plus one, or zero for a type written out.
    let holder = read_varint(&mut input, "damage event type")?;
    Ok((
        id,
        usize::try_from(holder)
            .ok()
            .and_then(|value| value.checked_sub(1)),
    ))
}

/// `ClientboundHurtAnimationPacket`: the entity and the yaw the blow came from.
pub fn parse_hurt_animation_packet(input: &[u8]) -> Result<(EntityId, f32), ConnectionError> {
    let mut input = input;
    let id = EntityId(read_varint(&mut input, "hurt animation entity")?);
    let yaw = f32::from_bits(read_i32(&mut input, "hurt animation yaw")? as u32);
    Ok((id, yaw))
}

/// Wire-level fields decoded from one clientbound attribute update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeUpdateObservation {
    pub entity_id: EntityId,
    pub attributes: Vec<ClientAttributeState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound entity-metadata packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMetadataObservation {
    pub entity_id: EntityId,
    pub metadata_length: usize,
    pub applied_to_world: bool,
    pub metadata: Vec<u8>,
}

/// What one clientbound set-equipment packet puts in the entity's slots, by
/// `EquipmentSlot` ordinal; an empty stack clears its slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityEquipmentObservation {
    pub entity_id: EntityId,
    pub slots: Vec<(usize, Option<ClientItemStackState>)>,
}

pub fn parse_update_attributes_packet(
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
        entity_id: EntityId(entity_id),
        attributes,
        applied_to_world: false,
    })
}

/// Wire-level fields decoded from one clientbound mob-effect removal packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectRemoveObservation {
    pub entity_id: EntityId,
    pub effect_registry_id: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound mob-effect update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectUpdateObservation {
    pub entity_id: EntityId,
    pub effect_registry_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: u8,
    pub applied_to_world: bool,
}

pub fn parse_entity_metadata_packet(
    input: &[u8],
) -> Result<EntityMetadataObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity metadata entity id")?;
    if input.last() != Some(&255) {
        return Err(ConnectionError::Codec("entity metadata end marker"));
    }
    let metadata = input.to_vec();
    Ok(EntityMetadataObservation {
        entity_id: EntityId(entity_id),
        metadata_length: metadata.len(),
        applied_to_world: false,
        metadata,
    })
}

/// `ClientboundSetEquipmentPacket`: the entity, then each slot's byte (the
/// top bit says another follows) and stack.
pub fn parse_entity_equipment_packet(
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
    Ok(EntityEquipmentObservation {
        entity_id: EntityId(entity_id),
        slots,
    })
}

pub fn parse_remove_mob_effect_packet(
    input: &[u8],
) -> Result<MobEffectRemoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "remove mob effect entity id")?;
    let effect_registry_id = read_varint(&mut input, "remove mob effect registry id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("remove mob effect trailing payload"));
    }
    Ok(MobEffectRemoveObservation {
        entity_id: EntityId(entity_id),
        effect_registry_id,
        applied_to_world: false,
    })
}

pub fn parse_update_mob_effect_packet(
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
        entity_id: EntityId(entity_id),
        effect_registry_id,
        amplifier,
        duration_ticks,
        flags,
        applied_to_world: false,
    })
}

wrapped_packet!(
    /// `ClientboundSetSimulationDistancePacket`: the distance, in chunks.
    SetSimulationDistance(i32) = parse_entity_id_prefix,
    OnError::Ignore("entity_id_prefix")
);

impl Decode for crate::common::ClientboundKeepAlivePacket {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        crate::common::ClientboundKeepAlivePacket::decode(&mut input)
            .map_err(|_| ConnectionError::Codec("play keep alive"))
    }
}

impl Decode for crate::common::ClientboundPingPacket {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        crate::common::ClientboundPingPacket::decode(&mut input)
            .map_err(|_| ConnectionError::Codec("play ping"))
    }
}

/// `ClientboundUpdateTagsPacket`.
pub struct UpdateTags(pub photon_core::tags::ClientboundUpdateTagsPacket);

impl Decode for UpdateTags {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        photon_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
            .map(Self)
            .map_err(|_| ConnectionError::Codec("update tags"))
    }
}

/// `ClientboundSetTimePacket`: the game time, and the world clocks as (clock, total ticks,
/// partial tick, rate).
pub struct SetTime {
    pub game_time: i64,
    pub clocks: Vec<(i32, i64, f32, f32)>,
}

impl Decode for SetTime {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let game_time = read_section_position(&mut input)?;
        let count =
            wire::read_varint(&mut input).map_err(|_| ConnectionError::Codec("set time"))?;
        let mut clocks = Vec::new();
        for _ in 0..count {
            let clock = wire::read_varint(&mut input)
                .map_err(|_| ConnectionError::Codec("set time clock"))?;
            let total_ticks = wire::read_varlong(&mut input)
                .map_err(|_| ConnectionError::Codec("set time ticks"))?;
            let partial_tick = read_f32(&mut input, "float")?;
            let rate = read_f32(&mut input, "float")?;
            clocks.push((clock, total_ticks, partial_tick, rate));
        }
        Ok(Self { game_time, clocks })
    }
}

wrapped_packet!(
    /// `ClientboundSetEntityLinkPacket`.
    SetEntityLink((EntityId, EntityId)) = parse_set_entity_link_packet,
    OnError::Ignore("set_entity_link")
);

wrapped_packet!(
    /// `ClientboundAnimatePacket`.
    Animate((EntityId, u8)) = parse_animate_packet,
    OnError::Ignore("animate")
);

wrapped_packet!(
    /// `ClientboundEntityEventPacket`.
    EntityEvent((EntityId, i8)) = parse_entity_event_packet
);

wrapped_packet!(
    /// `ClientboundDamageEventPacket`.
    DamageEvent((EntityId, Option<usize>)) = parse_damage_event_packet,
    OnError::Ignore("damage_event")
);

wrapped_packet!(
    /// `ClientboundHurtAnimationPacket`.
    HurtAnimation((EntityId, f32)) = parse_hurt_animation_packet,
    OnError::Ignore("hurt_animation")
);

wrapped_packet!(
    /// `ClientboundSetEntityDataPacket`.
    SetEntityData(EntityMetadataObservation) = parse_entity_metadata_packet,
    OnError::Ignore("entity_metadata")
);

wrapped_packet!(
    /// `ClientboundSetEquipmentPacket`.
    SetEquipment(EntityEquipmentObservation) = parse_entity_equipment_packet,
    OnError::Ignore("entity_equipment")
);

wrapped_packet!(
    /// `ClientboundUpdateAttributesPacket`.
    UpdateAttributes(AttributeUpdateObservation) = parse_update_attributes_packet
);

wrapped_packet!(
    /// `ClientboundUpdateMobEffectPacket`.
    UpdateMobEffect(MobEffectUpdateObservation) = parse_update_mob_effect_packet
);

wrapped_packet!(
    /// `ClientboundRemoveMobEffectPacket`.
    RemoveMobEffect(MobEffectRemoveObservation) = parse_remove_mob_effect_packet
);
