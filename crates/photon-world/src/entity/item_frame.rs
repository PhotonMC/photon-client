use super::*;

impl ClientEntity {
    /// Whether this is an item frame or a glow item frame (`ItemFrame`).
    pub fn is_item_frame(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:item_frame" | "minecraft:glow_item_frame")
        )
    }

    /// `ItemFrame.getRotation`.
    pub fn item_frame_rotation(&self) -> i32 {
        self.item_frame_rotation
    }
}
