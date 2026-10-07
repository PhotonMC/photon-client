pub(crate) fn entity_chunk_key(position: (f64, f64, f64)) -> (i32, i32) {
    (
        voxel_core::math::floor_f64(position.0).div_euclid(16),
        voxel_core::math::floor_f64(position.2).div_euclid(16),
    )
}

pub(crate) fn entity_section_key(position: (f64, f64, f64)) -> (i32, i32, i32) {
    (
        entity_section_coord(position.0),
        entity_section_coord(position.1),
        entity_section_coord(position.2),
    )
}

pub(crate) fn entity_section_coord(position: f64) -> i32 {
    voxel_core::math::floor_f64(position).div_euclid(16)
}

/// The block a grounded entity stands on, and whether the last look for one
/// found none (`Entity.mainSupportingBlockPos`, `onGroundNoBlocks`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Support {
    pub(crate) main_block: Option<voxel_core::binary::BlockPos>,
    pub(crate) no_blocks: bool,
}

/// What the client keeps of where an entity is, besides the position itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tracking {
    pub(crate) support: Support,
    /// Where the server's relative position packets count from
    /// (`Entity.packetPositionCodec`): the last position a packet gave, which
    /// is not the entity's own once the client moves it.
    pub(crate) packet_base_bits: [u64; 3],
}

impl Tracking {
    pub(crate) fn at(x: f64, y: f64, z: f64) -> Self {
        Self {
            support: Support::default(),
            packet_base_bits: [x.to_bits(), y.to_bits(), z.to_bits()],
        }
    }
}
