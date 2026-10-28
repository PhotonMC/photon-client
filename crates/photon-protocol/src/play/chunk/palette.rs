//! The paletted containers of a chunk section: block states and biomes, a palette and packed
//! storage.

use super::super::*;
use std::collections::BTreeSet;

/// The codec error labels of one paletted container kind.
struct ContainerLabels {
    bits: &'static str,
    palette: &'static str,
    palette_value: &'static str,
    storage: &'static str,
    value: &'static str,
}

const BLOCK_STATE_LABELS: ContainerLabels = ContainerLabels {
    bits: "block state bits",
    palette: "block state palette",
    palette_value: "block state palette value",
    storage: "block state storage",
    value: "block state value",
};

const BIOME_LABELS: ContainerLabels = ContainerLabels {
    bits: "biome bits",
    palette: "biome palette",
    palette_value: "biome palette value",
    storage: "biome storage",
    value: "biome value",
};

/// The palette capacity (`None` when ids are stored directly) and the bits each stored entry
/// takes, for a container kind and the bits-per-entry byte the server sent.
fn palette_layout(bits_per_entry: u8, block_states: bool) -> (Option<usize>, u8) {
    if bits_per_entry == 0 {
        (Some(1), 0)
    } else if block_states && bits_per_entry <= 4 {
        (Some(1_usize << 4), 4)
    } else if (block_states && bits_per_entry <= 8) || (!block_states && bits_per_entry <= 3) {
        (Some(1_usize << bits_per_entry), bits_per_entry)
    } else {
        (None, bits_per_entry)
    }
}

/// The palette entries of a container: one for a single value, a counted list when the palette
/// has a capacity, and none when ids are stored directly.
fn read_palette_ids(
    input: &mut &[u8],
    bits_per_entry: u8,
    palette_capacity: Option<usize>,
    labels: &ContainerLabels,
) -> Result<Vec<i32>, ConnectionError> {
    if bits_per_entry == 0 {
        return Ok(vec![read_varint(input, labels.palette_value)?]);
    }
    let Some(capacity) = palette_capacity else {
        return Ok(Vec::new());
    };
    let count = read_count(input, labels.palette)?;
    if count > capacity {
        return Err(ConnectionError::Codec(labels.palette));
    }
    let mut palette_ids = Vec::with_capacity(count);
    for _ in 0..count {
        palette_ids.push(read_varint(input, labels.palette_value)?);
    }
    Ok(palette_ids)
}

pub(super) fn parse_palette_container(
    input: &mut &[u8],
    entry_count: usize,
    block_states: bool,
    registries: &VersionRegistries,
) -> Result<PaletteObservation, ConnectionError> {
    let labels = if block_states {
        &BLOCK_STATE_LABELS
    } else {
        &BIOME_LABELS
    };
    let bits_per_entry = read_byte(input, labels.bits)?;
    if bits_per_entry > 32 {
        return Err(ConnectionError::Codec(labels.bits));
    }

    let (palette_capacity, storage_bits) = palette_layout(bits_per_entry, block_states);
    let palette_ids = read_palette_ids(input, bits_per_entry, palette_capacity, labels)?;

    let storage_word_count = if storage_bits == 0 {
        0
    } else {
        let values_per_long = 64 / usize::from(storage_bits);
        entry_count.div_ceil(values_per_long)
    };
    let storage = read_fixed_long_array(input, storage_word_count, labels.storage)?;

    let mut values = decode_palette_values(
        &storage,
        entry_count,
        storage_bits,
        &palette_ids,
        labels.value,
    )?;
    let mut palette_ids = palette_ids;
    if block_states {
        let canonical = |id: &mut i32| *id = registries.block_states.canonical_or_unknown(*id);
        palette_ids.iter_mut().for_each(canonical);
        values.iter_mut().for_each(canonical);
    }
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
