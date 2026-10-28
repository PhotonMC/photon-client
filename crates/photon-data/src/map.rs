//! The map's tables (`MapColor`, `MapDecorationType`): what each packed colour looks like and
//! which decoration types the server numbers.

/// A map decoration type (`MapDecorationType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapDecorationType {
    pub name: &'static str,
    /// The sprite's name (`minecraft:player`) in the map decorations atlas.
    pub asset: &'static str,
    /// The sprite's path under `textures/` (`map/decorations/player`).
    pub texture: &'static str,
    pub show_on_item_frame: bool,
    pub map_color: i32,
    pub track_count: bool,
}

/// The decoration type the server numbers `id`.
pub fn decoration_type(id: i32) -> Option<&'static MapDecorationType> {
    usize::try_from(id)
        .ok()
        .and_then(|index| crate::tables::map_data::MAP_DECORATION_TYPES.get(index))
}

/// Every decoration type, in the server's numbering.
pub fn decoration_types() -> &'static [MapDecorationType] {
    &crate::tables::map_data::MAP_DECORATION_TYPES
}

/// `MapColor.getColorFromPackedId`: the ARGB colour of a packed map colour.
pub fn packed_color(packed: u8) -> u32 {
    crate::tables::map_data::MAP_COLORS[usize::from(packed)]
}

/// The RGBA bytes a map's texture holds for a packed colour.
pub fn packed_rgba(packed: u8) -> [u8; 4] {
    let argb = packed_color(packed);
    [
        (argb >> 16) as u8,
        (argb >> 8) as u8,
        argb as u8,
        (argb >> 24) as u8,
    ]
}

/// `BlockState.getMapColor(..).id` of a state of a block.
pub fn state_map_color(block: &str, state: u32) -> u8 {
    use crate::tables::map_colors::{BLOCK_MAP_COLORS, STATE_MAP_COLORS};
    if let Ok(index) = STATE_MAP_COLORS.binary_search_by_key(&state, |(id, _)| *id) {
        return STATE_MAP_COLORS[index].1;
    }
    BLOCK_MAP_COLORS
        .iter()
        .find(|(name, _)| *name == block)
        .map_or(0, |(_, color)| *color)
}

/// `MapColor.col`: a map colour's own RGB, at its highest brightness.
pub fn base_color(id: u8) -> i32 {
    (packed_color(id.wrapping_mul(4).wrapping_add(2)) & 0xFF_FFFF) as i32
}

/// `FallingBlock.getDustColor` of a falling block; none for another block.
pub fn dust_color(block: &str) -> Option<i32> {
    crate::tables::map_colors::DUST_COLORS
        .iter()
        .find(|(name, _)| *name == block)
        .map(|(_, color)| *color)
}

/// Every block's map colour id, the states with another, and the falling blocks' dust colours.
pub type MapColorTables = (
    &'static [(&'static str, u8)],
    &'static [(u32, u8)],
    &'static [(&'static str, i32)],
);

/// The map colour tables, as captured.
pub fn map_color_tables() -> MapColorTables {
    use crate::tables::map_colors::{BLOCK_MAP_COLORS, DUST_COLORS, STATE_MAP_COLORS};
    (&BLOCK_MAP_COLORS, &STATE_MAP_COLORS, &DUST_COLORS)
}
