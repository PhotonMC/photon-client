//! What the view lands on within 20 blocks (`Entity.pick(20.0, 0.0F, withLiquids)`), for the
//! targeted block and fluid entries.

use photon_core::math::Vec3;
use photon_world::ClientWorld;

use super::TargetedBlock;

/// The range of the pick.
const RANGE: f64 = 20.0;

/// The block outline the view meets, and the block or fluid it meets when fluids count too, each
/// with what the level holds there. `eye` is the eyes a tick ago.
pub fn targets(
    world: &ClientWorld,
    eye: Vec3,
    (yaw, pitch): (f32, f32),
) -> (Option<TargetedBlock>, Option<TargetedBlock>) {
    let look = Vec3::direction_from_rotation_components(pitch, yaw);
    let to = Vec3::new(
        eye.x + look.x * RANGE,
        eye.y + look.y * RANGE,
        eye.z + look.z * RANGE,
    );
    let block = world.clip_outlines(eye, to).and_then(|hit| {
        let at = [hit.block_pos.x, hit.block_pos.y, hit.block_pos.z];
        let state = world.block_state_or_air(at[0], at[1], at[2]);
        let tags = world.block_state_registry().state_tags(state);
        TargetedBlock::of((at[0], at[1], at[2]), state).map(|block| block.with_tags(tags))
    });
    let fluid = world.clip_outlines_and_fluids(eye, to).map(|hit| {
        let at = [hit.block_pos.x, hit.block_pos.y, hit.block_pos.z];
        let state = world.fluid_at(at);
        TargetedBlock {
            position: (at[0], at[1], at[2]),
            name: state.registered_name().to_owned(),
            properties: state
                .properties()
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
            tags: world.block_state_registry().fluid_tags_of(state.fluid),
        }
    });
    (block, fluid)
}
