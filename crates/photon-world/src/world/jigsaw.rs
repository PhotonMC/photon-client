//! What a jigsaw block's screen reads of the block: its block entity's tag and which way it faces.

use crate::*;

/// A jigsaw block as the client has it: its block entity's tag, and whether it faces up or down.
pub struct JigsawBlockView<'a> {
    pub tag: Option<&'a [u8]>,
    pub vertical: bool,
}

impl ClientWorld {
    /// The jigsaw block at a position when the client has the block and its entity
    /// (`JigsawBlockEntity`). The block faces up or down when the front of its `orientation` does
    /// (`JigsawBlock.getFrontFacing(state).getAxis().isVertical()`).
    pub fn jigsaw_block_view(&self, pos: (i32, i32, i32)) -> Option<JigsawBlockView<'_>> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        let info = crate::block::state_or_air(state);
        if info.block().name != "minecraft:jigsaw" {
            return None;
        }
        let entity = self.block_entity_at(pos.0, pos.1, pos.2)?;
        if photon_data::block_entity_type::name(entity.type_id) != Some("minecraft:jigsaw") {
            return None;
        }
        let (_, orientation) = info
            .block()
            .properties_at(u32::from(info.offset))?
            .into_iter()
            .find(|(name, _)| *name == "orientation")?;
        Some(JigsawBlockView {
            tag: entity.tag.as_deref(),
            vertical: orientation.starts_with("up_") || orientation.starts_with("down_"),
        })
    }
}
