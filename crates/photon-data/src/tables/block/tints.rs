//! The tint each block's layers take outside a world, as packed colours.
//!
//! Only the blocks the reference registers a tint source for appear here.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

#[rustfmt::skip]
pub static TINTS: [(&str, &[u32]); 28] = [
    ("minecraft:grass_block", &[0xFF7CBD6B]),
    ("minecraft:water", &[0xFFFFFFFF]),
    ("minecraft:oak_leaves", &[0xFF48B518]),
    ("minecraft:spruce_leaves", &[0xFF619961]),
    ("minecraft:birch_leaves", &[0xFF80A755]),
    ("minecraft:jungle_leaves", &[0xFF48B518]),
    ("minecraft:acacia_leaves", &[0xFF48B518]),
    ("minecraft:dark_oak_leaves", &[0xFF48B518]),
    ("minecraft:mangrove_leaves", &[0xFF48B518]),
    ("minecraft:short_grass", &[0xFF7CBD6B]),
    ("minecraft:fern", &[0xFF7CBD6B]),
    ("minecraft:bush", &[0xFF7CBD6B]),
    ("minecraft:redstone_wire", &[0xFF4C0000]),
    ("minecraft:sugar_cane", &[0xFFFFFFFF]),
    ("minecraft:attached_pumpkin_stem", &[0xFFE0C71C]),
    ("minecraft:attached_melon_stem", &[0xFFE0C71C]),
    ("minecraft:pumpkin_stem", &[0xFF00FF00]),
    ("minecraft:melon_stem", &[0xFF00FF00]),
    ("minecraft:vine", &[0xFF48B518]),
    ("minecraft:lily_pad", &[0xFF71C35C]),
    ("minecraft:water_cauldron", &[0xFFFFFFFF]),
    ("minecraft:potted_fern", &[0xFF7CBD6B]),
    ("minecraft:tall_grass", &[0xFF7CBD6B]),
    ("minecraft:large_fern", &[0xFF7CBD6B]),
    ("minecraft:bubble_column", &[0xFFFFFFFF]),
    ("minecraft:pink_petals", &[0xFFFFFFFF, 0xFF7CBD6B]),
    ("minecraft:wildflowers", &[0xFFFFFFFF, 0xFF7CBD6B]),
    ("minecraft:leaf_litter", &[0xFF5C3C32]),
];
