use super::*;

/// Wire-level fields decoded from one clientbound add-entity packet.
#[derive(Debug, Clone, PartialEq)]
pub struct AddEntityObservation {
    pub entity_id: i32,
    pub uuid: Uuid,
    pub entity_type_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub movement: (f64, f64, f64),
    pub x_rot: f32,
    pub y_rot: f32,
    pub y_head_rot: f32,
    pub data: i32,
    pub applied_to_world: bool,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// One client-world tick applied to a supported world entity after its add packet.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldEntityTickObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub after_position: (f64, f64, f64),
    pub before_motion: (f64, f64, f64),
    pub after_motion: (f64, f64, f64),
    pub before_rotation: (f32, f32, f32),
    pub after_rotation: (f32, f32, f32),
    pub tick_count: i32,
}

/// Two client-world ticks applied to a newly added Experience Orb.
#[derive(Debug, Clone, PartialEq)]
pub struct ExperienceOrbTickObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub after_first_tick_position: (f64, f64, f64),
    pub after_second_tick_position: (f64, f64, f64),
    pub before_motion: (f64, f64, f64),
    pub after_first_tick_motion: (f64, f64, f64),
    pub after_second_tick_motion: (f64, f64, f64),
    pub tick_count: i32,
}

/// Wire-level fields decoded from one clientbound remove-entities packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveEntitiesObservation {
    pub entity_ids: Vec<i32>,
    pub applied_entity_count: usize,
}

/// Wire-level fields decoded from one clientbound passenger-list packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPassengersObservation {
    pub vehicle_id: i32,
    pub passenger_ids: Vec<i32>,
    pub applied_to_world: bool,
    pub applied_vehicle_passengers: Option<Vec<i32>>,
    pub applied_passenger_vehicles: Vec<(i32, Option<i32>)>,
}

/// One client-world tick observed across a vehicle and a known passenger.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPassengerTickObservation {
    pub vehicle_id: i32,
    pub passenger_id: i32,
    pub passenger_type_id: i32,
    pub before_vehicle_tick_count: i32,
    pub after_vehicle_tick_count: i32,
    pub before_passenger_tick_count: i32,
    pub after_passenger_tick_count: i32,
    pub before_passenger_position: (f64, f64, f64),
    pub after_passenger_position: (f64, f64, f64),
    pub before_passenger_motion: (f64, f64, f64),
    pub after_passenger_motion: (f64, f64, f64),
    pub before_passenger_rotation: (f32, f32, f32),
    pub after_passenger_rotation: (f32, f32, f32),
    pub before_passenger_body_y_rotation: f32,
    pub after_passenger_body_y_rotation: f32,
}

pub fn parse_add_entity_packet(input: &[u8]) -> Result<AddEntityObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "add entity id")?;
    let uuid = photon_core::binary::read_uuid(&mut input)
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
pub fn parse_set_entity_link_packet(input: &[u8]) -> Result<(i32, i32), ConnectionError> {
    let mut input = input;
    let source = read_i32(&mut input, "entity link source id")?;
    let holder = read_i32(&mut input, "entity link holder id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity link trailing payload"));
    }
    Ok((source, holder))
}

pub fn parse_remove_entities_packet(
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

/// `ClientboundTakeItemEntityPacket`: the entity taken, who took it, and how many.
pub fn parse_take_item_entity_packet(input: &[u8]) -> Result<(i32, i32, i32), ConnectionError> {
    let mut input = input;
    let item_id = read_varint(&mut input, "take item entity id")?;
    let player_id = read_varint(&mut input, "take item player id")?;
    let amount = read_varint(&mut input, "take item amount")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("take item entity trailing payload"));
    }
    Ok((item_id, player_id, amount))
}

pub fn parse_set_passengers_packet(
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

wrapped_packet!(
    /// `ClientboundAddEntityPacket`.
    AddEntity(AddEntityObservation) = parse_add_entity_packet,
    OnError::Ignore("add_entity")
);

wrapped_packet!(
    /// `ClientboundRemoveEntitiesPacket`.
    RemoveEntities(RemoveEntitiesObservation) = parse_remove_entities_packet,
    OnError::Ignore("remove_entities")
);

wrapped_packet!(
    /// `ClientboundTakeItemEntityPacket`.
    TakeItemEntity((i32, i32, i32)) = parse_take_item_entity_packet,
    OnError::Ignore("take_item_entity")
);

wrapped_packet!(
    /// `ClientboundSetPassengersPacket`.
    SetPassengers(EntityPassengersObservation) = parse_set_passengers_packet,
    OnError::Ignore("set_passengers")
);
