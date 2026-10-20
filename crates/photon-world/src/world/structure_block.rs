//! What a structure block's screen reads of the block: its block entity's tag.

use crate::*;

impl ClientWorld {
    /// The block entity tag of the structure block at a position, when the client has the block and
    /// its entity (`StructureBlockEntity`); the tag itself is `None` while the entity has no data.
    pub fn structure_block_view(&self, pos: (i32, i32, i32)) -> Option<Option<&[u8]>> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if crate::block::state_or_air(state).block().name != "minecraft:structure_block" {
            return None;
        }
        let entity = self.block_entity_at(pos.0, pos.1, pos.2)?;
        (photon_data::block_entity_type::name(entity.type_id) == Some("minecraft:structure_block"))
            .then_some(entity.tag.as_deref())
    }
}
