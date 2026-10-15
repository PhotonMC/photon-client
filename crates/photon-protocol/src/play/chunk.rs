use super::*;

pub fn read_light_update_bytes(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Vec<Vec<u8>>, ConnectionError> {
    let count = read_count(input, name)?;
    let mut updates = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = read_length_prefixed_bytes(input, 2_048, name)?;
        updates.push(bytes.to_vec());
    }
    Ok(updates)
}

pub fn parse_section_data(
    mut input: &[u8],
) -> Result<Vec<ChunkSectionObservation>, ConnectionError> {
    let mut sections = Vec::new();
    while !input.is_empty() {
        let non_empty_block_count = read_i16(&mut input, "section non-empty block count")?;
        let fluid_count = read_i16(&mut input, "section fluid count")?;
        let block_states = parse_palette_container(&mut input, 4_096, true)?;
        let biomes = parse_palette_container(&mut input, 64, false)?;
        sections.push(ChunkSectionObservation {
            non_empty_block_count,
            fluid_count,
            block_state_bits_per_entry: block_states.bits_per_entry,
            block_state_palette_entry_count: block_states.palette_entry_count,
            block_state_palette_ids: block_states.palette_ids,
            block_state_storage_word_count: block_states.storage_word_count,
            block_state_first_values: block_states.first_values,
            block_state_values: block_states.values,
            block_state_distinct_value_count: block_states.distinct_value_count,
            biome_bits_per_entry: biomes.bits_per_entry,
            biome_palette_entry_count: biomes.palette_entry_count,
            biome_palette_ids: biomes.palette_ids,
            biome_storage_word_count: biomes.storage_word_count,
            biome_first_values: biomes.first_values,
            biome_values: biomes.values,
            biome_distinct_value_count: biomes.distinct_value_count,
        });
    }
    Ok(sections)
}

/// Decode the section payload with the same parser used by chunk packets.
pub fn decode_chunk_section_data(input: &[u8]) -> Result<Vec<ChunkSectionObservation>, String> {
    parse_section_data(input).map_err(|error| error.to_string())
}

pub(super) fn parse_palette_container(
    input: &mut &[u8],
    entry_count: usize,
    block_states: bool,
) -> Result<PaletteObservation, ConnectionError> {
    let bits_per_entry = read_byte(
        input,
        if block_states {
            "block state bits"
        } else {
            "biome bits"
        },
    )?;
    if bits_per_entry > 32 {
        return Err(ConnectionError::Codec(if block_states {
            "block state bits"
        } else {
            "biome bits"
        }));
    }

    let (palette_capacity, storage_bits) = if bits_per_entry == 0 {
        (Some(1), 0)
    } else if block_states && bits_per_entry <= 4 {
        (Some(1_usize << 4), 4)
    } else if (block_states && bits_per_entry <= 8) || (!block_states && bits_per_entry <= 3) {
        (Some(1_usize << bits_per_entry), bits_per_entry)
    } else {
        (None, bits_per_entry)
    };

    let palette_ids = if bits_per_entry == 0 {
        let value = read_varint(
            input,
            if block_states {
                "block state palette value"
            } else {
                "biome palette value"
            },
        )?;
        vec![value]
    } else if let Some(capacity) = palette_capacity {
        let name = if block_states {
            "block state palette"
        } else {
            "biome palette"
        };
        let count = read_count(input, name)?;
        if count > capacity {
            return Err(ConnectionError::Codec(name));
        }
        let mut palette_ids = Vec::with_capacity(count);
        for _ in 0..count {
            palette_ids.push(read_varint(
                input,
                if block_states {
                    "block state palette value"
                } else {
                    "biome palette value"
                },
            )?);
        }
        palette_ids
    } else {
        Vec::new()
    };

    let storage_word_count = if storage_bits == 0 {
        0
    } else {
        let values_per_long = 64 / usize::from(storage_bits);
        entry_count.div_ceil(values_per_long)
    };
    let storage = read_fixed_long_array(
        input,
        storage_word_count,
        if block_states {
            "block state storage"
        } else {
            "biome storage"
        },
    )?;

    let values = decode_palette_values(
        &storage,
        entry_count,
        storage_bits,
        &palette_ids,
        if block_states {
            "block state value"
        } else {
            "biome value"
        },
    )?;
    let first_values = values.iter().copied().take(4).collect();
    let distinct_value_count = values.iter().copied().collect::<BTreeSet<_>>().len();

    Ok(PaletteObservation {
        bits_per_entry,
        palette_entry_count: palette_ids.len(),
        palette_ids,
        storage_word_count,
        first_values,
        values,
        distinct_value_count,
    })
}

pub fn read_fixed_long_array(
    input: &mut &[u8],
    word_count: usize,
    name: &'static str,
) -> Result<Vec<u64>, ConnectionError> {
    let byte_count = word_count
        .checked_mul(8)
        .ok_or(ConnectionError::Codec(name))?;
    if input.len() < byte_count {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(byte_count);
    *input = rest;
    let mut words = Vec::with_capacity(word_count);
    let (chunks, remainder) = head.as_chunks::<8>();
    debug_assert!(remainder.is_empty());
    for word in chunks {
        words.push(u64::from_be_bytes(*word));
    }
    Ok(words)
}

pub fn decode_palette_values(
    storage: &[u64],
    entry_count: usize,
    storage_bits: u8,
    palette_ids: &[i32],
    name: &'static str,
) -> Result<Vec<i32>, ConnectionError> {
    if storage_bits == 0 {
        let Some(&value) = palette_ids.first() else {
            return Err(ConnectionError::Codec(name));
        };
        return Ok(vec![value; entry_count]);
    }

    let bits = usize::from(storage_bits);
    let values_per_long = 64 / bits;
    let mask = if bits == 32 {
        u64::from(u32::MAX)
    } else {
        (1_u64 << bits) - 1
    };
    let mut values = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let Some(&word) = storage.get(index / values_per_long) else {
            return Err(ConnectionError::Codec(name));
        };
        let palette_index = ((word >> ((index % values_per_long) * bits)) & mask) as usize;
        let value = if palette_ids.is_empty() {
            i32::try_from(palette_index).map_err(|_| ConnectionError::Codec(name))?
        } else {
            *palette_ids
                .get(palette_index)
                .ok_or(ConnectionError::Codec(name))?
        };
        values.push(value);
    }
    Ok(values)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PaletteObservation {
    pub(super) bits_per_entry: u8,
    pub(super) palette_entry_count: usize,
    pub(super) palette_ids: Vec<i32>,
    pub(super) storage_word_count: usize,
    pub(super) first_values: Vec<i32>,
    pub(super) values: Vec<i32>,
    pub(super) distinct_value_count: usize,
}

/// Wire-level fields observed from one clientbound chunk-unload packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkUnloadObservation {
    pub x: i32,
    pub z: i32,
    pub removed_from_world: bool,
    pub tracked_entity_count: usize,
    pub ticking_entity_count: usize,
    pub stopped_entity_count: usize,
}

/// Wire-level fields decoded from one clientbound light-update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightUpdateObservation {
    pub x: i32,
    pub z: i32,
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_lengths: Vec<usize>,
    pub applied_sky_sections: usize,
    pub applied_block_sections: usize,
    pub queued_for_chunk: bool,
    pub light_data: LightUpdateData,
}

/// Structural fields decoded from one clientbound level-chunk-with-light packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPacketObservation {
    pub x: i32,
    pub z: i32,
    pub heightmap_count: usize,
    pub heightmap_observations: Vec<HeightmapObservation>,
    pub section_data_length: usize,
    pub section_observations: Vec<ChunkSectionObservation>,
    pub block_entity_count: usize,
    pub block_entities: Vec<BlockEntityObservation>,
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_count: usize,
    pub block_update_lengths: Vec<usize>,
    pub light_data: LightUpdateData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightUpdateData {
    pub sky_mask: BitSet,
    pub block_mask: BitSet,
    pub empty_sky_mask: BitSet,
    pub empty_block_mask: BitSet,
    pub sky_updates: Vec<Vec<u8>>,
    pub block_updates: Vec<Vec<u8>>,
}

/// Wire-level fields observed from one chunk heightmap entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightmapObservation {
    pub type_id: i32,
    pub word_count: usize,
}

/// Structural fields decoded from one serialized 16-by-16 chunk section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSectionObservation {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub block_state_bits_per_entry: u8,
    pub block_state_palette_entry_count: usize,
    pub block_state_palette_ids: Vec<i32>,
    pub block_state_storage_word_count: usize,
    pub block_state_first_values: Vec<i32>,
    pub block_state_values: Vec<i32>,
    pub block_state_distinct_value_count: usize,
    pub biome_bits_per_entry: u8,
    pub biome_palette_entry_count: usize,
    pub biome_palette_ids: Vec<i32>,
    pub biome_storage_word_count: usize,
    pub biome_first_values: Vec<i32>,
    pub biome_values: Vec<i32>,
    pub biome_distinct_value_count: usize,
}

pub fn parse_chunk_packet(mut input: &[u8]) -> Result<ChunkPacketObservation, ConnectionError> {
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

pub fn parse_light_update_packet(
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

pub fn parse_forget_level_chunk_packet(mut input: &[u8]) -> Result<(i32, i32), ConnectionError> {
    let packed = read_i64(&mut input, "forget level chunk position")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "forget level chunk trailing payload",
        ));
    }
    Ok((packed as i32, (packed >> 32) as i32))
}

pub fn parse_set_chunk_cache_radius_packet(mut input: &[u8]) -> Result<i32, ConnectionError> {
    let radius = read_varint(&mut input, "chunk cache radius")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "chunk cache radius trailing payload",
        ));
    }
    Ok(radius)
}

pub fn parse_set_chunk_cache_center_packet(
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

impl Decode for ChunkPacketObservation {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        parse_chunk_packet(payload)
    }
}

impl Decode for LightUpdateObservation {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        parse_light_update_packet(payload)
    }
}

wrapped_packet!(
    /// `ClientboundForgetLevelChunkPacket`: the chunk's position.
    ForgetLevelChunk((i32, i32)) = parse_forget_level_chunk_packet
);
wrapped_packet!(
    /// `ClientboundSetChunkCacheRadiusPacket`.
    SetChunkCacheRadius(i32) = parse_set_chunk_cache_radius_packet
);
wrapped_packet!(
    /// `ClientboundSetChunkCacheCenterPacket`.
    SetChunkCacheCenter((i32, i32)) = parse_set_chunk_cache_center_packet
);

/// `ClientboundChunkBatchFinishedPacket`: the batch size it carries is not read.
pub struct ChunkBatchFinished;

impl Decode for ChunkBatchFinished {
    fn decode(_payload: &[u8]) -> Result<Self, ConnectionError> {
        Ok(Self)
    }
}
