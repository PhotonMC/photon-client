use super::*;

/// What an item frame keeps: `ItemFrame.getRotation`, the eighths of a turn the item in it is
/// turned.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemFrameState {
    pub(crate) rotation: i32,
}

pub(crate) fn is_item_frame_type(type_id: EntityTypeId) -> bool {
    matches!(
        photon_data::entity_type::name(type_id),
        Some("minecraft:item_frame" | "minecraft:glow_item_frame")
    )
}

impl ClientEntity {
    /// Whether this is an item frame or a glow item frame (`ItemFrame`).
    pub fn is_item_frame(&self) -> bool {
        is_item_frame_type(self.type_id)
    }
}

impl EntityView<'_> {
    /// `ItemFrame.getRotation`.
    pub fn item_frame_rotation(&self) -> i32 {
        self.kind.item_frame().map_or(0, |frame| frame.rotation)
    }
}
