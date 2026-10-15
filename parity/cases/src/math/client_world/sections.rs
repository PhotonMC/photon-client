//! A chunk section as the level packet writes it: block and biome palettes
//! and their packed storage.

use crate::bits::hex;

/// A palette container: its bits per entry, palette and values.
pub type Container = (i64, Vec<i64>, Vec<i64>);

/// A protocol VarInt (the value's low 32 bits).
fn varint(value: i64) -> Vec<u8> {
    let mut value = (value as u64) & 0xffff_ffff;
    let mut encoded = Vec::new();
    while value & !0x7f != 0 {
        encoded.push(((value & 0x7f) | 0x80) as u8);
        value >>= 7;
    }
    encoded.push(value as u8);
    encoded
}

/// `entry_count` zeroes with the given entries set.
pub fn sparse_values(entry_count: usize, assignments: &[(usize, i64)]) -> Vec<i64> {
    let mut values = vec![0; entry_count];
    for (index, value) in assignments {
        assert!(*index < entry_count, "palette assignment is outside its container");
        values[*index] = *value;
    }
    values
}

/// The palette entries a local palette writes before its packed values.
fn local_palette(palette: &[i64]) -> Vec<u8> {
    let mut encoded = varint(palette.len() as i64);
    for value in palette {
        encoded.extend(varint(*value));
    }
    encoded
}

/// `values` packed into longs of `storage_bits`-wide cells, most significant byte first.
fn packed_longs(values: &[i64], entry_count: usize, storage_bits: usize) -> Vec<u8> {
    let values_per_long = 64 / storage_bits;
    let word_count = entry_count.div_ceil(values_per_long);
    let mask = (1u64 << storage_bits) - 1;
    let mut words = vec![0u64; word_count];
    for (index, value) in values.iter().enumerate() {
        let offset = (index % values_per_long) * storage_bits;
        words[index / values_per_long] |= ((*value as u64) & mask) << offset;
    }
    words.into_iter().flat_map(u64::to_be_bytes).collect()
}

/// One palette container: a single value, a local palette or a global one,
/// then the values packed into longs.
fn palette_container(container: &Container, entry_count: usize, block_states: bool) -> Vec<u8> {
    let (bits, palette, values) = container;
    if *bits == 0 {
        assert_eq!(palette.len(), 1, "single-value palette must have one entry");
        return [vec![0u8], varint(palette[0])].concat();
    }
    let is_local = (block_states && *bits <= 8) || (!block_states && *bits <= 3);
    let storage_bits = if block_states && *bits <= 4 { 4 } else { *bits as usize };
    let mut encoded = vec![*bits as u8];
    if is_local {
        encoded.extend(local_palette(palette));
    } else {
        assert!(palette.is_empty(), "global palette must not carry palette entries");
    }
    encoded.extend(packed_longs(values, entry_count, storage_bits));
    encoded
}

/// A section: its counts of non-empty blocks and fluids, then the block
/// states and the biomes.
pub fn section_hex(counts: (i64, i64), blocks: Container, biomes: Container) -> String {
    let mut encoded = Vec::new();
    encoded.extend((counts.0 as i16).to_be_bytes());
    encoded.extend((counts.1 as i16).to_be_bytes());
    encoded.extend(palette_container(&blocks, 4096, true));
    encoded.extend(palette_container(&biomes, 64, false));
    hex(&encoded)
}

/// A 2048-byte light layer with its first byte set.
pub fn light_layer_hex(value: u8) -> String {
    let mut layer = vec![0u8; 2048];
    layer[0] = value;
    hex(&layer)
}
