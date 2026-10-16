//! Lookups on the block registry ([`super::blocks`]).
//!
//! The reference numbers block states as it registers blocks: each block's
//! states in turn, in registry order, and within a block the cartesian
//! product of its properties with the last varying fastest. The numbers are
//! worked out the same way once, on first use.

use std::collections::HashMap;
use std::sync::OnceLock;

use photon_core::ids::BlockStateId;

pub use super::blocks::BLOCKS;
pub use super::definition::{Block, Property, PropertyKind};
pub use super::tints::TINTS;

/// Where each block's states start, which block owns each state, and each
/// block by name.
struct Layout {
    first_state: Vec<u32>,
    owner: Vec<u16>,
    by_name: HashMap<&'static str, usize>,
}

fn layout() -> &'static Layout {
    static LAYOUT: OnceLock<Layout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        let mut first_state = Vec::with_capacity(BLOCKS.len());
        let mut owner = Vec::new();
        for (index, block) in BLOCKS.iter().enumerate() {
            first_state.push(owner.len() as u32);
            let index = u16::try_from(index).expect("the registry has fewer than 65536 blocks");
            owner.extend(std::iter::repeat_n(index, block.state_count() as usize));
        }
        let by_name = BLOCKS
            .iter()
            .enumerate()
            .map(|(index, block)| (block.name, index))
            .collect();
        Layout {
            first_state,
            owner,
            by_name,
        }
    })
}

/// How many block states the registry has.
pub fn state_count() -> u32 {
    layout().owner.len() as u32
}

/// Look one block up by its identifier.
pub fn block(name: &str) -> Option<&'static Block> {
    block_index(name).map(|index| &BLOCKS[index])
}

/// A block's place in the registry, by its identifier.
pub fn block_index(name: &str) -> Option<usize> {
    layout().by_name.get(name).copied()
}

/// The tint layers a block takes outside a world.
pub fn tints(name: &str) -> &'static [u32] {
    TINTS
        .iter()
        .find(|(block, _)| *block == name)
        .map_or(&[], |(_, layers)| *layers)
}

impl Block {
    /// This block's place in the registry.
    pub fn index(&self) -> usize {
        let offset = (self as *const Self as usize).wrapping_sub(BLOCKS.as_ptr() as usize);
        let index = offset / std::mem::size_of::<Self>();
        assert!(
            index < BLOCKS.len() && std::ptr::eq(&BLOCKS[index], self),
            "a block of the registry"
        );
        index
    }

    /// How many states the block has: the product of its properties' value
    /// counts.
    pub fn state_count(&self) -> u32 {
        self.properties
            .iter()
            .map(|property| property.values.len() as u32)
            .product()
    }

    /// The number of the block's first state.
    pub fn first_state_id(&self) -> BlockStateId {
        BlockStateId(layout().first_state[self.index()] as i32)
    }

    /// How far the default state sits from the block's first.
    pub fn default_offset(&self) -> u32 {
        (self.default_state_id().0 - self.first_state_id().0) as u32
    }

    /// The state `offset` places after the block's first.
    pub fn state_at(&self, offset: u32) -> BlockStateId {
        BlockStateId(self.first_state_id().0 + offset as i32)
    }

    /// The value each property holds in one of this block's states.
    ///
    /// The offset counts from the block's first state. The properties cycle
    /// with the last one varying fastest, so the offset is read back by
    /// dividing from the end.
    pub fn properties_at(&self, offset: u32) -> Option<Vec<(&'static str, &'static str)>> {
        if offset >= self.state_count() {
            return None;
        }
        let mut remaining = offset;
        let mut out = vec![("", ""); self.properties.len()];
        for (slot, property) in self.properties.iter().enumerate().rev() {
            let count = property.values.len() as u32;
            out[slot] = (property.name, property.values[(remaining % count) as usize]);
            remaining /= count;
        }
        Some(out)
    }

    /// `state.getValue(property)` for the state at `offset`; `None` when the
    /// block does not have the property (`hasProperty` is false).
    pub fn value(&self, offset: u32, property: &Property) -> Option<&'static str> {
        let slot = self
            .properties
            .iter()
            .position(|own| std::ptr::eq(*own, property))?;
        let below: u32 = self.properties[slot + 1..]
            .iter()
            .map(|later| later.values.len() as u32)
            .product();
        let count = property.values.len() as u32;
        Some(property.values[(offset / below % count) as usize])
    }

    /// A boolean property's value; `false` when the block does not have it.
    pub fn flag(&self, offset: u32, property: &Property) -> bool {
        self.value(offset, property) == Some("true")
    }

    /// The default state's value of each property, as an index into its values.
    fn default_value(&self, slot: usize) -> usize {
        let property = self.properties[slot];
        property
            .values
            .iter()
            .position(|value| *value == self.default[slot])
            .expect("a default is one of its property's values")
    }

    /// The state number one set of property values names.
    ///
    /// A property the caller did not name keeps the default state's value, and
    /// a value the property cannot hold gives nothing back.
    pub fn state_id(
        &self,
        chosen: &std::collections::BTreeMap<String, String>,
    ) -> Option<BlockStateId> {
        let mut offset = 0_u32;
        for (slot, property) in self.properties.iter().enumerate() {
            let value = match chosen.get(property.name) {
                Some(named) => property.values.iter().position(|value| value == named)?,
                None => self.default_value(slot),
            };
            offset = offset * property.values.len() as u32 + value as u32;
        }
        Some(BlockStateId(self.first_state_id().0 + offset as i32))
    }

    /// The state the block starts in.
    pub fn default_state_id(&self) -> BlockStateId {
        let mut offset = 0_u32;
        for (slot, property) in self.properties.iter().enumerate() {
            offset = offset * property.values.len() as u32 + self.default_value(slot) as u32;
        }
        BlockStateId(self.first_state_id().0 + offset as i32)
    }
}

/// Which block a state number belongs to, by index, and how far into it the
/// state sits.
pub fn block_index_for_state(state: BlockStateId) -> Option<(usize, u32)> {
    let layout = layout();
    let state_id = u32::try_from(state.0).ok()?;
    let index = usize::from(*layout.owner.get(state_id as usize)?);
    Some((index, state_id - layout.first_state[index]))
}

/// Which block a state number belongs to, and how far into it the state sits.
pub fn block_for_state(state: BlockStateId) -> Option<(&'static Block, u32)> {
    let (index, offset) = block_index_for_state(state)?;
    Some((&BLOCKS[index], offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn the_registry_numbers_every_state() {
        assert_eq!(state_count(), 32366);
        assert_eq!(
            block("minecraft:stone").map(Block::default_state_id),
            Some(BlockStateId(1))
        );
        let stairs = block("minecraft:oak_stairs").expect("stairs are in the registry");
        assert_eq!(stairs.default_state_id(), BlockStateId(3918));
    }

    #[test]
    fn a_state_number_decodes_back_to_the_values_that_made_it() {
        for block in &BLOCKS {
            for offset in 0..block.state_count() {
                let values = block.properties_at(offset).expect("offset is in range");
                let chosen: BTreeMap<String, String> = values
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect();
                assert_eq!(
                    block.state_id(&chosen),
                    Some(block.state_at(offset)),
                    "{} at {offset}",
                    block.name
                );
            }
        }
    }

    #[test]
    fn a_state_number_finds_the_block_that_owns_it() {
        for block in &BLOCKS {
            for offset in [0, block.state_count() - 1] {
                let (found, found_offset) = block_for_state(block.state_at(offset))
                    .expect("every number in range has an owner");
                assert_eq!(found.name, block.name);
                assert_eq!(found_offset, offset);
            }
        }
        assert!(block_for_state(BlockStateId(32366)).is_none());
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
