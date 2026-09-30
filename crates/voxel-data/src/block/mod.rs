//! Per block and per block state, in registry order.

pub mod behaviour;
pub mod blocks;
pub mod classes;
pub mod collision;
pub mod definition;
pub mod destroy;
pub mod face_occlusion;
pub mod fluid_flow;
pub mod fluid_overlay;
pub mod light;
pub mod motion_blocking;
pub mod movement;
pub mod offset;
pub mod outline;
pub mod properties;
pub mod registry;
pub mod shape;
pub mod tints;

/// A per-block list holds either one value shared by every state or one value
/// per state: the value for the state at `offset` in its block.
pub fn per_state<T: Copy>(values: &[T], offset: u32) -> Option<T> {
    if values.len() == 1 {
        Some(values[0])
    } else {
        values.get(offset as usize).copied()
    }
}

/// A state's value in a per-block table; `None` outside the registry.
pub fn state_value<T: Copy>(per_block: &[&[T]], state: u32) -> Option<T> {
    let (block, offset) = registry::block_index_for_state(state)?;
    per_state(per_block[block], offset)
}

/// A state's lighting traits; a state outside the registry lights as air.
pub fn light_traits(state: u32) -> &'static light::LightTraits {
    &light::TRAITS[usize::from(state_value(&light::BLOCKS, state).unwrap_or(0))]
}

/// A state's destroy traits; a state outside the registry breaks as air.
pub fn destroy_traits(state: u32) -> &'static destroy::DestroyTraits {
    &destroy::TRAITS[usize::from(state_value(&destroy::BLOCKS, state).unwrap_or(0))]
}

/// Bit `index` of a row of a bit matrix, which the tables write as text of
/// `0` and `1`, as the reference's capture has it.
fn bit(row: &str, index: usize) -> bool {
    row.as_bytes()[index] == b'1'
}

/// Whether face occlusion shape `shape` has any part `occluder` does not
/// cover (`Shapes.joinIsNotEmpty` with `ONLY_FIRST`).
pub fn shape_joins(shape: u8, occluder: u8) -> bool {
    bit(
        face_occlusion::JOINS[usize::from(shape)],
        usize::from(occluder),
    )
}

/// `Shapes.faceShapeOccludes`: whether face occlusion shape `shape` is
/// covered by `occluder`.
pub fn shape_covers(shape: u8, occluder: u8) -> bool {
    bit(
        face_occlusion::COVERS[usize::from(shape)],
        usize::from(occluder),
    )
}

impl behaviour::BlockBehaviour {
    /// `BlockState.canBeReplaced()` of the state at `offset` in the block.
    pub fn replaceable(&self, offset: u32) -> bool {
        bit(self.replaceable, offset as usize)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn air_is_the_first_trait_of_every_per_state_table() {
        // The lookups fall back to trait zero for unknown states, which the
        // old code reached through block zero, state zero: air.
        assert_eq!(super::light::BLOCKS[0], &[0]);
        assert_eq!(super::destroy::BLOCKS[0], &[0]);
    }
}
