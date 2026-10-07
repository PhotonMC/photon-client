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

/// The fixed colours a block's tint slots take outside a world.
///
/// These are the `color(state)` side of the reference's tint sources, which is
/// the side a block drawn on its own uses. The table is the reference's own,
/// read from its registry and compared on every parity run, so a block with no
/// tint source comes back with no layers rather than with a guess.
///
/// The biome-averaged side needs a world to read from and is not built yet.
pub fn constant_tints(block_name: &str) -> &'static [u32] {
    voxel_data::block::registry::tints(block_name)
}

/// The tint of a block whose colour follows its state rather than its biome:
/// redstone wire by its power, and a growing melon or pumpkin stem by its
/// age. Other blocks have none.
pub fn state_tint(
    block_name: &str,
    properties: &std::collections::BTreeMap<String, String>,
) -> Option<u32> {
    let value = |name: &str| properties.get(name)?.parse::<i32>().ok();
    match block_name {
        "minecraft:redstone_wire" => Some(redstone_colour(value("power")?)),
        "minecraft:melon_stem" | "minecraft:pumpkin_stem" => {
            let age = value("age")?;
            Some(argb(255, age * 32, 255 - age * 8, age * 4))
        }
        _ => None,
    }
}

/// `RedStoneWireBlock`'s colour for a power level: red rises with power,
/// green and blue only near the top.
fn redstone_colour(power: i32) -> u32 {
    let power = power as f32 / 15.0;
    let red = power * 0.6 + if power > 0.0 { 0.4 } else { 0.3 };
    let green = (power * power * 0.7 - 0.5).clamp(0.0, 1.0);
    let blue = (power * power * 0.6 - 0.7).clamp(0.0, 1.0);
    let channel = |value: f32| (value * 255.0).floor() as i32;
    argb(channel(1.0), channel(red), channel(green), channel(blue))
}

/// `ARGB.color`: pack four channels, each already in range.
pub(crate) fn argb(alpha: i32, red: i32, green: i32, blue: i32) -> u32 {
    ((alpha as u32 & 0xFF) << 24)
        | ((red as u32 & 0xFF) << 16)
        | ((green as u32 & 0xFF) << 8)
        | (blue as u32 & 0xFF)
}

#[cfg(test)]
mod state_tint_tests {
    use super::*;

    #[test]
    fn default_states_match_the_reference_table() {
        let unpowered = [("power".to_owned(), "0".to_owned())].into_iter().collect();
        assert_eq!(
            state_tint("minecraft:redstone_wire", &unpowered),
            constant_tints("minecraft:redstone_wire").first().copied()
        );
        let young = [("age".to_owned(), "0".to_owned())].into_iter().collect();
        assert_eq!(
            state_tint("minecraft:melon_stem", &young),
            constant_tints("minecraft:melon_stem").first().copied()
        );
        let full = [("power".to_owned(), "15".to_owned())]
            .into_iter()
            .collect();
        assert_eq!(
            state_tint("minecraft:redstone_wire", &full),
            Some(0xFFFF_3200)
        );
    }
}
