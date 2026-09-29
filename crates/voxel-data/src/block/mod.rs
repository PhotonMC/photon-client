//! Per block and per block state, in registry order.

pub mod behaviour;
pub mod collision;
pub mod destroy;
pub mod face_occlusion;
pub mod fluid_flow;
pub mod fluid_overlay;
pub mod light;
pub mod motion_blocking;
pub mod movement;
pub mod offset;
pub mod outline;
pub mod registry;
pub mod shape;

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
