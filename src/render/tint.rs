//! Block tints: the colour a quad's texture is multiplied by.
//!
//! A face carries a tint slot rather than a colour. What the slot resolves to
//! depends on where the block is: inside a world it is averaged from the
//! biomes around it, and outside one — in a hand, in an inventory, or in a
//! probe like ours — it is a fixed value per block.

/// The tint a mangrove's leaves take, which is a fixed value rather than a
/// colour map reading.
pub const FOLIAGE_MANGROVE: u32 = 0xFF92_C648;

/// A lily pad's tint inside a world, which is fixed rather than biome
/// averaged; it is the one block whose two sides differ by more than the
/// lookup.
pub const LILY_PAD_IN_WORLD: u32 = 0xFF20_8030;

/// Java `ARGB.multiply`: fold two packed colours channel by channel.
///
/// White on either side is returned untouched rather than multiplied, which
/// matters because the division below loses a step: multiplying by white would
/// otherwise turn `0xfe` into `0xfd`.
pub fn multiply(lhs: u32, rhs: u32) -> u32 {
    if lhs == 0xFFFF_FFFF {
        return rhs;
    }
    if rhs == 0xFFFF_FFFF {
        return lhs;
    }
    let channel = |shift: u32| ((lhs >> shift) & 0xFF) * ((rhs >> shift) & 0xFF) / 255;
    (channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

/// Which of the three colour maps a tint reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColourMap {
    Grass,
    Foliage,
    DryFoliage,
}

impl ColourMap {
    /// What the lookup falls back to when the climate lands off the map.
    pub fn default_colour(self) -> u32 {
        match self {
            // Magenta, the reference's usual mark for something missing.
            Self::Grass => 0xFFFF_00FF,
            Self::Foliage => 0xFF48_B518,
            Self::DryFoliage => 0xFF5C_3C32,
        }
    }
}

/// A climate lands outside the colour map.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColourMapOutOfBounds;

/// Java `ColorMapColorUtil.get`: index a 256 by 256 colour map by climate.
///
/// Rainfall is scaled by temperature first, so the same rainfall reads a
/// different row in a cold biome than in a warm one. Both axes are inverted
/// and truncated toward zero, and the two indices are combined with an `or`
/// rather than an addition — so a temperature outside zero to one does not
/// merely walk off the row, it bleeds into the other axis.
///
/// The reference guards the upper bound and returns the map's default, but
/// leaves the lower bound unguarded and throws. That asymmetry is reported
/// here rather than reproduced as a panic.
pub fn colour_map_lookup(
    map: ColourMap,
    pixels: &[u32],
    temperature: f64,
    rain: f64,
) -> Result<u32, ColourMapOutOfBounds> {
    let rain = rain * temperature;
    let x = ((1.0 - temperature) * 255.0) as i32;
    let y = ((1.0 - rain) * 255.0) as i32;
    let index = (y << 8) | x;
    if index >= pixels.len() as i32 {
        return Ok(map.default_colour());
    }
    if index < 0 {
        return Err(ColourMapOutOfBounds);
    }
    Ok(pixels[index as usize])
}

/// The fixed colour a block's tint slot takes outside a world.
///
/// These are the `color(state)` side of the reference's tint sources, which
/// is the side a block drawn on its own uses. The biome-averaged side needs a
/// world to read from and is not built yet.
///
/// Every value here is one of the pinned named constants. The table that maps
/// a block to one of them is read from `BlockColors.createDefault` and is not
/// itself pinned, because asking the reference for it needs its registry
/// bootstrap; the blocks whose tint is a bare literal rather than a named
/// constant are left out rather than guessed at.
pub fn constant_tint(block_path: &str) -> Option<u32> {
    Some(match block_path {
        "spruce_leaves" => 0xFF61_9961,
        "birch_leaves" => 0xFF80_A755,
        "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves" | "vine"
        | "mangrove_leaves" => 0xFF48_B518,
        "leaf_litter" => 0xFF5C_3C32,
        "lily_pad" => 0xFF71_C35C,
        _ => return None,
    })
}
