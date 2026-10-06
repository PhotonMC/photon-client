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
        .and_then(|index| crate::generated::map_data::MAP_DECORATION_TYPES.get(index))
}

/// Every decoration type, in the server's numbering.
pub fn decoration_types() -> &'static [MapDecorationType] {
    &crate::generated::map_data::MAP_DECORATION_TYPES
}

/// `MapColor.getColorFromPackedId`: the ARGB colour of a packed map colour.
pub fn packed_color(packed: u8) -> u32 {
    crate::generated::map_data::MAP_COLORS[usize::from(packed)]
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
