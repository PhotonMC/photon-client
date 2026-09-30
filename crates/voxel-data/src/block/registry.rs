//! Lookups on the block registry ([`super::states`]).
//!
//! Two measured facts keep the registry small: block state numbers run
//! **contiguously in registry order**, and a block's states are the
//! **cartesian product of its properties in order, with the last property
//! varying fastest**. Both were checked against every one of the 1196 blocks
//! and all 32366 states; the tests below keep checking them.

pub use super::states::{BLOCKS, BlockDefinition, PROPERTIES, Property, PropertyKind, TINTS};

/// Look one block up by its identifier.
pub fn block(name: &str) -> Option<&'static BlockDefinition> {
    BLOCKS.iter().find(|block| block.name == name)
}

/// The tint layers a block takes outside a world.
pub fn tints(name: &str) -> &'static [u32] {
    TINTS
        .iter()
        .find(|(block, _)| *block == name)
        .map_or(&[], |(_, layers)| *layers)
}

impl BlockDefinition {
    /// The value each property holds in one of this block's states.
    ///
    /// The offset counts from the block's first state. The properties cycle
    /// with the last one varying fastest, so the offset is read back by
    /// dividing from the end.
    pub fn properties_at(&self, offset: u32) -> Option<Vec<(&'static str, &'static str)>> {
        if offset >= u32::from(self.state_count) {
            return None;
        }
        let mut remaining = offset;
        let mut out = vec![("", ""); self.properties.len()];
        for (slot, index) in self.properties.iter().enumerate().rev() {
            let property = &PROPERTIES[usize::from(*index)];
            let count = property.values.len() as u32;
            out[slot] = (property.name, property.values[(remaining % count) as usize]);
            remaining /= count;
        }
        Some(out)
    }

    /// The state number one set of property values names.
    ///
    /// A property the caller did not name keeps the default state's value, and
    /// a value the property cannot hold gives nothing back.
    pub fn state_id(&self, chosen: &std::collections::BTreeMap<String, String>) -> Option<u32> {
        let mut offset = 0_u32;
        for (slot, index) in self.properties.iter().enumerate() {
            let property = &PROPERTIES[usize::from(*index)];
            let value = match chosen.get(property.name) {
                Some(named) => property.values.iter().position(|value| value == named)?,
                None => usize::from(self.default[slot]),
            };
            offset = offset * property.values.len() as u32 + value as u32;
        }
        Some(self.first_state_id + offset)
    }

    /// The state the block starts in.
    pub fn default_state_id(&self) -> u32 {
        let mut offset = 0_u32;
        for (slot, index) in self.properties.iter().enumerate() {
            let property = &PROPERTIES[usize::from(*index)];
            offset = offset * property.values.len() as u32 + u32::from(self.default[slot]);
        }
        self.first_state_id + offset
    }
}

/// Which block a state number belongs to, by index, and how far into it the
/// state sits.
pub fn block_index_for_state(state_id: u32) -> Option<(usize, u32)> {
    // The numbers run contiguously in registry order, so a binary search over
    // the starting numbers finds the owner without a table of every state.
    let found = BLOCKS.partition_point(|block| block.first_state_id <= state_id);
    let index = found.checked_sub(1)?;
    let offset = state_id - BLOCKS[index].first_state_id;
    (offset < u32::from(BLOCKS[index].state_count)).then_some((index, offset))
}

/// Which block a state number belongs to, and how far into it the state sits.
pub fn block_for_state(state_id: u32) -> Option<(&'static BlockDefinition, u32)> {
    let (index, offset) = block_index_for_state(state_id)?;
    Some((&BLOCKS[index], offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn state_numbers_run_contiguously_in_registry_order() {
        let mut running = 0;
        for block in &BLOCKS {
            assert_eq!(block.first_state_id, running, "gap before {}", block.name);
            running += u32::from(block.state_count);
        }
        assert_eq!(running, 32366);
    }

    #[test]
    fn a_state_number_decodes_back_to_the_values_that_made_it() {
        for block in &BLOCKS {
            for offset in 0..u32::from(block.state_count) {
                let values = block.properties_at(offset).expect("offset is in range");
                let chosen: BTreeMap<String, String> = values
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect();
                assert_eq!(
                    block.state_id(&chosen),
                    Some(block.first_state_id + offset),
                    "{} at {offset}",
                    block.name
                );
            }
        }
    }

    #[test]
    fn a_state_number_finds_the_block_that_owns_it() {
        for block in &BLOCKS {
            for offset in [0, u32::from(block.state_count) - 1] {
                let (found, found_offset) = block_for_state(block.first_state_id + offset)
                    .expect("every number in range has an owner");
                assert_eq!(found.name, block.name);
                assert_eq!(found_offset, offset);
            }
        }
        assert!(block_for_state(32366).is_none());
    }

    #[test]
    fn an_unnamed_property_keeps_the_default() {
        let stairs = block("minecraft:oak_stairs").expect("stairs are in the registry");
        assert_eq!(
            stairs.state_id(&BTreeMap::new()),
            Some(stairs.default_state_id())
        );
        let mut chosen = BTreeMap::new();
        chosen.insert("facing".to_owned(), "nowhere".to_owned());
        assert_eq!(stairs.state_id(&chosen), None);
    }

    #[test]
    fn only_blocks_the_reference_tints_have_layers() {
        assert_eq!(tints("minecraft:oak_leaves"), &[0xFF48_B518]);
        assert_eq!(tints("minecraft:pink_petals").len(), 2);
        assert!(tints("minecraft:stone").is_empty());
    }
}
