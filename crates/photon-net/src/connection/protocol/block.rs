use super::*;

/// `ClientboundBlockEventPacket`: the block, the event and its data (the block's registry id
/// is left unread).
pub(crate) fn parse_block_event_packet(input: &[u8]) -> Result<BlockEvent, ConnectionError> {
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
    pub state_id: i32,
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
    pub(crate) tag: Option<Vec<u8>>,
}

/// Wire-level fields decoded from one clientbound block-destruction progress packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDestructionObservation {
    pub entity_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub progress: u8,
}

pub(crate) fn parse_block_update_packet(
    input: &[u8],
) -> Result<BlockUpdateObservation, ConnectionError> {
    let mut input = input;
    let position = photon_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block update position",
    )? as u64);
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
    let position = photon_core::binary::BlockPos::unpack(read_i64(
        &mut input,
        "block entity position",
    )? as u64);
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
    let position = photon_core::binary::BlockPos::unpack(read_i64(
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

/// `ClientboundLevelEventPacket`: an event of the level at a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEvent {
    pub event: i32,
    pub pos: (i32, i32, i32),
    pub data: i32,
    pub global: bool,
}

/// Position, event id and event data of a block event.
pub(crate) type BlockEvent = ((i32, i32, i32), u8, u8);
