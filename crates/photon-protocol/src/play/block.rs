use super::*;
use photon_core::ids::{BlockStateId, EntityId};

/// `ClientboundBlockEventPacket`: the block, the event and its data (the block's registry id
/// is left unread).
pub fn parse_block_event_packet(input: &[u8]) -> Result<BlockEvent, ConnectionError> {
    const NAME: &str = "block event";
    let mut input = input;
    let input = &mut input;
    let pos = photon_core::binary::BlockPos::unpack(read_i64(input, NAME)? as u64);
    let event = read_byte(input, NAME)?;
    let data = read_byte(input, NAME)?;
    Ok(((pos.x, pos.y, pos.z), event, data))
}

/// `ClientboundLevelEventPacket`, read from the front of `input`.
pub fn read_level_event_packet(input: &mut &[u8]) -> Result<LevelEvent, ConnectionError> {
    let event = read_i32(input, "level event")?;
    let pos =
        photon_core::binary::BlockPos::unpack(read_i64(input, "level event position")? as u64);
    let data = read_i32(input, "level event data")?;
    let global = read_bool(input, "level event global")?;
    Ok(LevelEvent {
        event,
        pos: (pos.x, pos.y, pos.z),
        data,
        global,
    })
}

/// Wire-level fields decoded from one clientbound block update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockUpdateObservation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state_id: BlockStateId,
}

/// Wire-level fields decoded from one clientbound block-entity data record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityObservation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub type_id: i32,
    pub tag_length: usize,
    pub applied_to_world: bool,
    pub tag: Option<Vec<u8>>,
}

/// Wire-level fields decoded from one clientbound block-destruction progress packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDestructionObservation {
    pub entity_id: EntityId,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub progress: u8,
}

pub fn parse_block_update_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<BlockUpdateObservation, ConnectionError> {
    let mut input = input;
    let position = photon_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block update position",
    )? as u64);
    let state_id = WireBlockStateId::read(&mut input, "block update state")?.canonical(registries);
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

pub fn parse_block_entity_data_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<BlockEntityObservation, ConnectionError> {
    let mut input = input;
    let position = photon_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block entity position",
    )? as u64);
    let type_id = registries
        .block_entity_types
        .canonical_or_unknown(read_varint(&mut input, "block entity type")?);
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

pub fn parse_block_destruction_packet(
    input: &[u8],
) -> Result<BlockDestructionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "block destruction entity id")?;
    let position = photon_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block destruction position",
    )? as u64);
    let progress = read_byte(&mut input, "block destruction progress")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("block destruction trailing payload"));
    }
    Ok(BlockDestructionObservation {
        entity_id: EntityId(entity_id),
        x: position.x,
        y: position.y,
        z: position.z,
        progress,
    })
}

/// `ClientboundLevelEventPacket`: an event of the level at a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEvent {
    pub event: i32,
    pub pos: (i32, i32, i32),
    pub data: i32,
    pub global: bool,
}

/// Position, event id and event data of a block event.
pub type BlockEvent = ((i32, i32, i32), u8, u8);

wrapped_wire_packet!(
    /// `ClientboundBlockUpdatePacket`.
    BlockUpdate(BlockUpdateObservation) = parse_block_update_packet
);
wrapped_packet!(
    /// `ClientboundBlockDestructionPacket`.
    BlockDestruction(BlockDestructionObservation) = parse_block_destruction_packet
);
wrapped_wire_packet!(
    /// `ClientboundBlockEntityDataPacket`.
    BlockEntityData(BlockEntityObservation) = parse_block_entity_data_packet
);
wrapped_packet!(
    /// `ClientboundBlockEventPacket`.
    BlockEventPacket(BlockEvent) = parse_block_event_packet
);

/// `ClientboundLevelEventPacket`.
pub struct LevelEventPacket(pub LevelEvent);

impl Decode for LevelEventPacket {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let event = read_level_event_packet(&mut input)?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec("level event trailing payload"));
        }
        Ok(Self(event))
    }
}

/// `ClientboundBlockChangedAckPacket`: the sequence number it answers.
pub struct BlockChangedAck(pub i32);

impl Decode for BlockChangedAck {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        read_varint(&mut &payload[..], "block changed ack sequence").map(Self)
    }
}

/// `ClientboundSectionBlocksUpdatePacket`: the blocks of one section that changed, each with its
/// position in the level and its new state.
pub struct SectionBlocksUpdate {
    pub blocks: Vec<([i32; 3], BlockStateId)>,
}

impl Decode for SectionBlocksUpdate {
    fn decode(
        payload: &[u8],
        registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let section = read_section_position(&mut input)?;
        let section_x = (section >> 42) as i32;
        let section_y = ((section << 44) >> 44) as i32;
        let section_z = ((section << 22) >> 42) as i32;
        let count = wire::read_varint(&mut input)
            .map_err(|_| ConnectionError::Codec("section blocks update"))?;
        let mut blocks = Vec::new();
        for _ in 0..count {
            let packed = wire::read_varlong(&mut input)
                .map_err(|_| ConnectionError::Codec("section blocks update"))?;
            let state = WireBlockStateId((packed as u64 >> 12) as i32).canonical(registries);
            let local = (packed & 0xfff) as i32;
            blocks.push((
                [
                    section_x * 16 + ((local >> 8) & 15),
                    section_y * 16 + (local & 15),
                    section_z * 16 + ((local >> 4) & 15),
                ],
                state,
            ));
        }
        Ok(Self { blocks })
    }
}
