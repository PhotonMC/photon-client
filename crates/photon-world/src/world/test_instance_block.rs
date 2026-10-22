//! What a test instance block's screen reads of the block: its block entity's tag.

use crate::*;

impl ClientWorld {
    /// The block entity tag of the test instance block at a position (`TestInstanceBlockEntity`)
    /// when the client has the entity: the outer `Option` says whether the block is there.
    pub fn test_instance_block_view(&self, pos: (i32, i32, i32)) -> Option<Option<&[u8]>> {
        let entity = self.block_entity_at(pos.0, pos.1, pos.2)?;
        if photon_data::block_entity_type::name(entity.type_id)
            != Some("minecraft:test_instance_block")
        {
            return None;
        }
        Some(entity.tag.as_deref())
    }
}
