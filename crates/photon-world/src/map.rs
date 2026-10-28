//! The maps the server has sent (`MapItemSavedData` on the client): each map's colours, scale and
//! decorations, kept by id until the dimension changes.

/// The side of a map in colours.
pub const MAP_SIDE: usize = 128;

pub use photon_core::map_values::{MapDecoration, MapItemDataInput, MapPatch};

/// What the client knows of a map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapData {
    pub scale: i8,
    pub locked: bool,
    colors: Vec<u8>,
    decorations: Vec<MapDecoration>,
    /// How many times the map changed (what is uploaded is good until it does).
    version: u32,
}

impl MapData {
    /// `MapItemSavedData.createForClient`: all colours clear.
    fn new(scale: i8, locked: bool) -> Self {
        Self {
            scale,
            locked,
            colors: vec![0; MAP_SIDE * MAP_SIDE],
            decorations: Vec::new(),
            version: 0,
        }
    }

    /// The packed colours, row by row.
    pub fn colors(&self) -> &[u8] {
        &self.colors
    }

    /// `MapItemSavedData.getDecorations`, in the order the server sent them.
    pub fn decorations(&self) -> &[MapDecoration] {
        &self.decorations
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    /// `ClientboundMapItemDataPacket.applyToMap`: the decorations replace the old, the patch sets
    /// its colours.
    fn apply(&mut self, input: &MapItemDataInput) {
        if let Some(decorations) = &input.decorations {
            self.decorations = decorations
                .iter()
                .cloned()
                .map(|mut decoration| {
                    decoration.rot &= 15;
                    decoration
                })
                .collect();
        }
        if let Some(patch) = &input.patch {
            for x in 0..patch.width {
                for y in 0..patch.height {
                    let (column, row) = (patch.start_x + x, patch.start_y + y);
                    // `setColor`: what lies outside the map is the array's bounds' business; here
                    // it is left out.
                    if column < MAP_SIDE
                        && row < MAP_SIDE
                        && let Some(color) = patch.colors.get(x + y * patch.width)
                    {
                        self.colors[column + row * MAP_SIDE] = *color;
                    }
                }
            }
        }
        self.version = self.version.wrapping_add(1);
    }
}

impl crate::ClientWorld {
    /// `handleMapItemData`: the map is made when it is new, then the packet is applied.
    pub fn apply_map_item_data(&mut self, input: &MapItemDataInput) {
        self.maps
            .entry(input.map_id)
            .or_insert_with(|| MapData::new(input.scale, input.locked))
            .apply(input);
    }

    /// `ClientLevel.getMapData`.
    pub fn map_data(&self, map_id: i32) -> Option<&MapData> {
        self.maps.get(&map_id)
    }
}
