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
    if !block
        .class
        .lineage()
        .any(|own| own == BlockClass::AbstractBedBlock)
    {
        return None;
    }
    super::rules::property(id, "facing").and_then(Direction::from_name)
}

/// `LivingEntity.setPosToBed`: where a sleeper lies over the bed at `bed`. 26.2 puts it a fixed
/// height over the block; 26.3 asks the bed for its sleep height (the top of its shape, and for
/// a straw bed the top of its foot half), and a block that gives none leaves the sleeper where
/// it is (`None`).
pub fn sleeping_position(
    world: &crate::ClientWorld,
    bed: photon_core::binary::BlockPos,
) -> Option<(f64, f64, f64)> {
    let height = if photon_core::versions::rules()
        .world
        .sleep_height_comes_from_the_bed
    {
        sleep_height(world, bed)? + 0.125
    } else {
        0.6875
    };
    Some((
        f64::from(bed.x) + 0.5,
        f64::from(bed.y) + height,
        f64::from(bed.z) + 0.5,
    ))
}

/// `AbstractBedBlock.getSleepHeight`: the top of the bed's shape. A straw bed's head half
/// answers for the foot half it is joined to, and has none without one.
fn sleep_height(world: &crate::ClientWorld, bed: photon_core::binary::BlockPos) -> Option<f64> {
    use photon_data::block::classes::BlockClass;
    let id = world.block_state_or_air(bed.x, bed.y, bed.z);
    let state = super::state_or_air(id);
    let class = state.block().class;
    if !class
        .lineage()
        .any(|own| own == BlockClass::AbstractBedBlock)
    {
        return None;
    }
    let mut shape_of = id;
    if class == BlockClass::StrawBedBlock && super::rules::property(id, "part") == Some("head") {
        // `getConnectedDirection` of a head half: opposite its facing.
        let facing = super::rules::property(id, "facing").and_then(Direction::from_name)?;
        let (dx, dy, dz) = facing.opposite().step();
        let foot = world.block_state_or_air(bed.x + dx, bed.y + dy, bed.z + dz);
        let joined = super::state_or_air(foot).block == state.block
            && super::rules::property(foot, "part") == Some("foot");
        if !joined {
            return None;
        }
        shape_of = foot;
    }
    let shape = super::states().outline(super::state_or_air(shape_of));
    (!shape.is_empty()).then(|| shape.max(photon_core::direction::Axis::Y))
}
