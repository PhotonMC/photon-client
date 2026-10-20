//! What a test block's screen reads of the block: its block entity's tag and the mode of its state.

use crate::*;

/// A test block as the client has it: its block entity's tag and the mode its block state holds.
pub struct TestBlockView<'a> {
    pub tag: Option<&'a [u8]>,
    /// The `mode` property of the block state, by name.
    pub state_mode: &'static str,
}

impl ClientWorld {
    /// The test block at a position when the client has the block and its entity
    /// (`TestBlockEntity`).
    pub fn test_block_view(&self, pos: (i32, i32, i32)) -> Option<TestBlockView<'_>> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        let info = crate::block::state_or_air(state);
        if info.block().name != "minecraft:test_block" {
            return None;
        }
        let entity = self.block_entity_at(pos.0, pos.1, pos.2)?;
        if photon_data::block_entity_type::name(entity.type_id) != Some("minecraft:test_block") {
            return None;
        }
        let (_, state_mode) = info
            .block()
            .properties_at(u32::from(info.offset))?
            .into_iter()
            .find(|(name, _)| *name == "mode")?;
        Some(TestBlockView {
            tag: entity.tag.as_deref(),
            state_mode,
        })
    }
}
