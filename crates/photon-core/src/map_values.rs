//! The values of a map packet.

/// A decoration on a map (`MapDecoration`): its type's number, where, how turned and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    pub type_id: i32,
    pub x: i8,
    pub y: i8,
    /// Sixteenths of a turn (`rot & 15`).
    pub rot: i8,
    /// The name as the network carried the component, if it has one.
    pub name: Option<Vec<u8>>,
}

/// A rectangle of colours (`MapItemSavedData.MapPatch`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPatch {
    pub start_x: usize,
    pub start_y: usize,
    pub width: usize,
    pub height: usize,
    pub colors: Vec<u8>,
}

/// `ClientboundMapItemDataPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapItemDataInput {
    pub map_id: i32,
    pub scale: i8,
    pub locked: bool,
    pub decorations: Option<Vec<MapDecoration>>,
    pub patch: Option<MapPatch>,
}
