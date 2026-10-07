use super::*;

pub(crate) fn parse_player_position_packet(
    input: &[u8],
) -> Result<PlayerPositionPacket, ConnectionError> {
    let mut input = input;
    let packet = PlayerPositionPacket {
        teleport_id: read_varint(&mut input, "player position teleport id")?,
        x: read_f64(&mut input, "player position x")?,
        y: read_f64(&mut input, "player position y")?,
        z: read_f64(&mut input, "player position z")?,
        delta_x: read_f64(&mut input, "player position delta x")?,
        delta_y: read_f64(&mut input, "player position delta y")?,
        delta_z: read_f64(&mut input, "player position delta z")?,
        y_rot: read_f32(&mut input, "player position y rotation")?,
        x_rot: read_f32(&mut input, "player position x rotation")?,
        relative_flags: read_i32(&mut input, "player position relative flags")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player position trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn resolve_player_position(
    packet: PlayerPositionPacket,
    current: Option<(f64, f64, f64, f32, f32)>,
) -> (f64, f64, f64, f32, f32) {
    let current = current.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));
    let x = if packet.relative_flags & PLAYER_RELATIVE_X_FLAG != 0 {
        current.0 + packet.x
    } else {
        packet.x
    };
    let y = if packet.relative_flags & PLAYER_RELATIVE_Y_FLAG != 0 {
        current.1 + packet.y
    } else {
        packet.y
    };
    let z = if packet.relative_flags & PLAYER_RELATIVE_Z_FLAG != 0 {
        current.2 + packet.z
    } else {
        packet.z
    };
    let y_rot = if packet.relative_flags & PLAYER_RELATIVE_Y_ROT_FLAG != 0 {
        current.3 + packet.y_rot
    } else {
        packet.y_rot
    };
    let x_rot = if packet.relative_flags & PLAYER_RELATIVE_X_ROT_FLAG != 0 {
        (current.4 + packet.x_rot).clamp(-90.0, 90.0)
    } else {
        packet.x_rot.clamp(-90.0, 90.0)
    };
    (x, y, z, y_rot, x_rot)
}

pub(crate) fn read_long_array(
    input: &mut &[u8],
    name: &'static str,
) -> Result<Vec<i64>, ConnectionError> {
    voxel_core::binary::read_long_array(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_bitset(
    input: &mut &[u8],
    name: &'static str,
) -> Result<BitSet, ConnectionError> {
    voxel_core::binary::read_bitset(input).map_err(|_| ConnectionError::Codec(name))
}

pub(crate) fn read_length_prefixed_bytes<'a>(
    input: &mut &'a [u8],
    max_length: usize,
    name: &'static str,
) -> Result<&'a [u8], ConnectionError> {
    let length = read_count(input, name)?;
    if length > max_length || input.len() < length {
        return Err(ConnectionError::Codec(name));
    }
    let (head, rest) = input.split_at(length);
    *input = rest;
    Ok(head)
}

pub(crate) fn read_light_update_bytes(
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

pub(crate) fn parse_section_data(
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

pub(crate) fn read_fixed_long_array(
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

pub(crate) fn decode_palette_values(
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
