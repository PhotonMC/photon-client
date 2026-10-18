//! What the camera reads of a bed.

use photon_core::direction::Direction;

/// `BedBlock.getBedOrientation`: the way the bed at `(x, y, z)` faces, nothing if there is no bed.
pub fn bed_orientation(
    world: &crate::ClientWorld,
    (x, y, z): (i32, i32, i32),
) -> Option<Direction> {
    use photon_data::block::classes::BlockClass;
    let id = world.block_state_or_air(x, y, z);
    let block = super::state_or_air(id).block();
    if !block.class.lineage().any(|own| own == BlockClass::BedBlock) {
        return None;
    }
    super::rules::property(id, "facing").and_then(Direction::from_name)
}
