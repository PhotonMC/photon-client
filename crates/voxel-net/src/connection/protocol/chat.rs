use super::*;

pub(super) fn read_chat_component(
    input: &mut &[u8],
    name: &'static str,
) -> Result<voxel_game::text::Component, ConnectionError> {
    voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .ok_or(ConnectionError::Codec(name))
}

pub(super) fn read_chat_string(
    input: &mut &[u8],
    max: usize,
    name: &'static str,
) -> Result<String, ConnectionError> {
    voxel_core::utf8::read_utf8(input, max)
        .map(|units| String::from_utf16_lossy(&units))
        .map_err(|_| ConnectionError::Codec(name))
}

/// `ChatTypeDecoration.STREAM_CODEC`.
pub(super) fn read_chat_decoration(
    input: &mut &[u8],
) -> Result<voxel_game::chat::ChatDecoration, ConnectionError> {
    let translation_key = read_chat_string(input, 32767, "chat decoration key")?;
    let count = read_varint(input, "chat decoration parameters")?;
    let mut parameters = Vec::new();
    for _ in 0..count {
        let id = read_varint(input, "chat decoration parameter")?;
        parameters.push(
            voxel_game::chat::ChatParameter::from_id(id)
                .ok_or(ConnectionError::Codec("chat decoration parameter"))?,
        );
    }
    let style = voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .map(|component| component.style)
        .ok_or(ConnectionError::Codec("chat decoration style"))?;
    Ok(voxel_game::chat::ChatDecoration {
        translation_key,
        parameters,
        style,
    })
}

/// `ChatType.Bound.STREAM_CODEC`: a holder whose id 0 is a chat type written
/// out (its chat decoration, then its narration), then the names.
pub(super) fn read_chat_bound(input: &mut &[u8]) -> Result<ChatBound, ConnectionError> {
    let holder = match read_varint(input, "chat type")? {
        0 => {
            let chat = read_chat_decoration(input)?;
            read_chat_decoration(input)?;
            ChatHolder::Direct(chat)
        }
        id => ChatHolder::Registry(id - 1),
    };
    let name = read_chat_component(input, "chat sender name")?;
    let target = if read_bool(input, "chat target flag")? {
        Some(read_chat_component(input, "chat target name")?)
    } else {
        None
    };
    Ok(ChatBound {
        holder,
        name,
        target,
    })
}

pub(super) fn skip_bytes(
    input: &mut &[u8],
    count: usize,
    name: &'static str,
) -> Result<(), ConnectionError> {
    if input.len() < count {
        return Err(ConnectionError::Codec(name));
    }
    *input = &input[count..];
    Ok(())
}

/// `ClientboundSystemChatPacket`.
pub(crate) fn parse_system_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "system chat component")?;
    let overlay = read_bool(&mut input, "system chat overlay")?;
    Ok(ChatEvent::System { content, overlay })
}

/// `ClientboundDisguisedChatPacket`.
pub(crate) fn parse_disguised_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "disguised chat message")?;
    let bound = read_chat_bound(&mut input)?;
    Ok(ChatEvent::Disguised { content, bound })
}

pub(super) fn read_packed_signature(input: &mut &[u8]) -> Result<PackedSignature, ConnectionError> {
    match read_varint(input, "packed signature")? {
        0 => Ok(PackedSignature::Full(read_signature(input)?)),
        id => Ok(PackedSignature::Cached(id - 1)),
    }
}

pub(super) fn read_signature(input: &mut &[u8]) -> Result<ChatSignature, ConnectionError> {
    if input.len() < 256 {
        return Err(ConnectionError::Codec("signature"));
    }
    let (signature, rest) = input.split_at(256);
    *input = rest;
    Ok(signature.to_vec())
}

/// `ClientboundPlayerChatPacket`.
pub(crate) fn parse_player_chat_packet(input: &[u8]) -> Result<PlayerChatPacket, ConnectionError> {
    let mut input = input;
    let global_index = read_varint(&mut input, "chat global index")?;
    let sender = voxel_core::binary::read_uuid(&mut input)
        .map_err(|_| ConnectionError::Codec("chat sender"))?;
    read_varint(&mut input, "chat index")?;
    let signature = if read_bool(&mut input, "chat signature flag")? {
        Some(read_signature(&mut input)?)
    } else {
        None
    };
    let signed_text = read_chat_string(&mut input, 256, "chat content")?;
    let timestamp_millis = read_i64(&mut input, "chat timestamp")?;
    skip_bytes(&mut input, 8, "chat salt")?;
    let count = read_varint(&mut input, "chat last seen")?;
    let mut last_seen = Vec::new();
    for _ in 0..count {
        last_seen.push(read_packed_signature(&mut input)?);
    }
    let unsigned = if read_bool(&mut input, "chat unsigned flag")? {
        Some(read_chat_component(&mut input, "chat unsigned content")?)
    } else {
        None
    };
    // `FilterMask`: pass through, fully filtered, or partly (with a bit set).
    let filter = match read_varint(&mut input, "chat filter mask")? {
        1 => FilterMask::FullyFiltered,
        2 => {
            let words = read_varint(&mut input, "chat filter bits")?.max(0) as usize;
            let mut bits = Vec::with_capacity(words);
            for _ in 0..words {
                bits.push(read_i64(&mut input, "chat filter word")? as u64);
            }
            FilterMask::Partial(bits)
        }
        _ => FilterMask::PassThrough,
    };
    let bound = read_chat_bound(&mut input)?;
    Ok(PlayerChatPacket {
        global_index,
        sender: (sender.most, sender.least),
        signature,
        signed_text,
        timestamp_millis,
        last_seen,
        unsigned,
        filter,
        bound,
    })
}

/// `ClientboundDeleteChatPacket`.
pub(crate) fn parse_delete_chat_packet(input: &[u8]) -> Result<PackedSignature, ConnectionError> {
    let mut input = input;
    read_packed_signature(&mut input)
}

/// `ServerboundChatPacket` for a session without a chat key: the message, the
/// time, a salt, no signature and an empty acknowledgement (offset 0, no
/// messages, checksum 1).
pub(crate) fn encode_chat(message: &str, millis: i64, salt: i64, output: &mut Vec<u8>) {
    let units: Vec<u16> = message.encode_utf16().collect();
    let _ = voxel_core::utf8::write_utf8(output, &units, 256);
    output.extend_from_slice(&millis.to_be_bytes());
    output.extend_from_slice(&salt.to_be_bytes());
    output.push(0);
    voxel_core::wire::write_varint(0, output);
    output.extend_from_slice(&[0, 0, 0]);
    output.push(1);
}

pub(super) fn read_lp_vec3(
    input: &mut &[u8],
    name: &'static str,
) -> Result<(f64, f64, f64), ConnectionError> {
    let movement =
        voxel_core::binary::read_lp_vec3(input).map_err(|_| ConnectionError::Codec(name))?;
    Ok((movement.x, movement.y, movement.z))
}

pub(super) fn decode_packed_degrees(value: u8) -> f32 {
    i8::from_ne_bytes([value]) as f32 * 360.0 / 256.0
}

pub(super) fn read_nbt_bytes(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Option<Vec<u8>>, ConnectionError> {
    let before = *input;
    let value = voxel_core::nbt::read_nbt(input).map_err(|_| ConnectionError::Codec(name))?;
    let consumed = before.len() - input.len();
    Ok(value.map(|_| before[..consumed].to_vec()))
}

pub(crate) fn parse_chunk_packet(
    mut input: &[u8],
) -> Result<ChunkPacketObservation, ConnectionError> {
    let x = read_i32(&mut input, "chunk x")?;
    let z = read_i32(&mut input, "chunk z")?;

    let heightmap_count = read_count(&mut input, "heightmap count")?;
    let mut heightmap_observations = Vec::with_capacity(heightmap_count);
    for _ in 0..heightmap_count {
        let type_id = read_varint(&mut input, "heightmap type")?;
        let word_count = read_long_array(&mut input, "heightmap data")?.len();
        heightmap_observations.push(HeightmapObservation {
            type_id,
            word_count,
        });
    }

    let section_data =
        read_length_prefixed_bytes(&mut input, MAX_CHUNK_DATA_LENGTH, "chunk section data")?;
    let section_data_length = section_data.len();
    let section_observations = parse_section_data(section_data)?;

    let block_entity_count = read_count(&mut input, "block entity count")?;
    let mut block_entities = Vec::with_capacity(block_entity_count);
    for _ in 0..block_entity_count {
        let packed_xz = read_byte(&mut input, "block entity packed xz")?;
        let y = i32::from(read_i16(&mut input, "block entity y")?);
        let type_id = read_varint(&mut input, "block entity type")?;
        let tag = read_nbt_bytes(&mut input, "block entity nbt")?;
        let local_x = i32::from(packed_xz >> 4);
        let local_z = i32::from(packed_xz & 0x0f);
        let x = x.wrapping_mul(16).wrapping_add(local_x);
        let z = z.wrapping_mul(16).wrapping_add(local_z);
        let tag_length = tag.as_ref().map_or(0, Vec::len);
        block_entities.push(BlockEntityObservation {
            x,
            y,
            z,
            type_id,
            tag_length,
            applied_to_world: false,
            tag,
        });
    }

    let light_data = parse_light_data(&mut input)?;
    let sky_mask_word_count = light_data.sky_mask.to_longs().len();
    let block_mask_word_count = light_data.block_mask.to_longs().len();
    let empty_sky_mask_word_count = light_data.empty_sky_mask.to_longs().len();
    let empty_block_mask_word_count = light_data.empty_block_mask.to_longs().len();
    let sky_update_lengths = light_data
        .sky_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    let block_update_lengths = light_data
        .block_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();

    if !input.is_empty() {
        return Err(ConnectionError::Codec("chunk trailing payload"));
    }

    Ok(ChunkPacketObservation {
        x,
        z,
        heightmap_count,
        heightmap_observations,
        section_data_length,
        section_observations,
        block_entity_count,
        block_entities,
        sky_mask_word_count,
        block_mask_word_count,
        empty_sky_mask_word_count,
        empty_block_mask_word_count,
        sky_update_count: sky_update_lengths.len(),
        sky_update_lengths,
        block_update_count: block_update_lengths.len(),
        block_update_lengths,
        light_data,
    })
}

pub(crate) fn parse_light_update_packet(
    mut input: &[u8],
) -> Result<LightUpdateObservation, ConnectionError> {
    let x = read_varint(&mut input, "light update chunk x")?;
    let z = read_varint(&mut input, "light update chunk z")?;
    let light_data = parse_light_data(&mut input)?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("light update trailing payload"));
    }
    let sky_mask_word_count = light_data.sky_mask.to_longs().len();
    let block_mask_word_count = light_data.block_mask.to_longs().len();
    let empty_sky_mask_word_count = light_data.empty_sky_mask.to_longs().len();
    let empty_block_mask_word_count = light_data.empty_block_mask.to_longs().len();
    let sky_update_lengths = light_data
        .sky_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    let block_update_lengths = light_data
        .block_updates
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    Ok(LightUpdateObservation {
        x,
        z,
        sky_mask_word_count,
        block_mask_word_count,
        empty_sky_mask_word_count,
        empty_block_mask_word_count,
        sky_update_lengths,
        block_update_lengths,
        applied_sky_sections: 0,
        applied_block_sections: 0,
        queued_for_chunk: false,
        light_data,
    })
}

pub(super) fn parse_light_data(input: &mut &[u8]) -> Result<LightUpdateData, ConnectionError> {
    let sky_mask = read_bitset(input, "sky light mask")?;
    let block_mask = read_bitset(input, "block light mask")?;
    let empty_sky_mask = read_bitset(input, "empty sky light mask")?;
    let empty_block_mask = read_bitset(input, "empty block light mask")?;
    let sky_updates = read_light_update_bytes(input, "sky light updates")?;
    let block_updates = read_light_update_bytes(input, "block light updates")?;
    Ok(LightUpdateData {
        sky_mask,
        block_mask,
        empty_sky_mask,
        empty_block_mask,
        sky_updates,
        block_updates,
    })
}

pub(crate) fn parse_forget_level_chunk_packet(
    mut input: &[u8],
) -> Result<(i32, i32), ConnectionError> {
    let packed = read_i64(&mut input, "forget level chunk position")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "forget level chunk trailing payload",
        ));
    }
    Ok((packed as i32, (packed >> 32) as i32))
}

pub(crate) fn parse_set_chunk_cache_radius_packet(
    mut input: &[u8],
) -> Result<i32, ConnectionError> {
    let radius = read_varint(&mut input, "chunk cache radius")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "chunk cache radius trailing payload",
        ));
    }
    Ok(radius)
}

pub(crate) fn parse_set_chunk_cache_center_packet(
    mut input: &[u8],
) -> Result<(i32, i32), ConnectionError> {
    let x = read_varint(&mut input, "chunk cache center x")?;
    let z = read_varint(&mut input, "chunk cache center z")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "chunk cache center trailing payload",
        ));
    }
    Ok((x, z))
}

pub(crate) fn read_varint(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::read_varint(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_count(input: &mut &[u8], name: &'static str) -> Result<usize, ConnectionError> {
    let count = read_varint(input, name)?;
    usize::try_from(count).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_byte(input: &mut &[u8], name: &'static str) -> Result<u8, ConnectionError> {
    let Some((&byte, rest)) = input.split_first() else {
        return Err(ConnectionError::Codec(name));
    };
    *input = rest;
    Ok(byte)
}

pub(crate) fn read_bool(input: &mut &[u8], name: &'static str) -> Result<bool, ConnectionError> {
    Ok(read_byte(input, name)? != 0)
}

pub(crate) fn read_i64(input: &mut &[u8], name: &'static str) -> Result<i64, ConnectionError> {
    wire::take(input)
        .map(i64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_i16(input: &mut &[u8], name: &'static str) -> Result<i16, ConnectionError> {
    wire::take(input)
        .map(i16::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_i32(input: &mut &[u8], name: &'static str) -> Result<i32, ConnectionError> {
    wire::take(input)
        .map(i32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_f32(input: &mut &[u8], name: &'static str) -> Result<f32, ConnectionError> {
    wire::take(input)
        .map(f32::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}

pub(crate) fn read_f64(input: &mut &[u8], name: &'static str) -> Result<f64, ConnectionError> {
    wire::take(input)
        .map(f64::from_be_bytes)
        .ok_or(ConnectionError::Codec(name))
}
