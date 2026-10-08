use super::*;

impl ClientEntity {
    /// A display entity's data and render state.
    pub fn display(&self) -> Option<&crate::objects::display::DisplayState> {
        self.kind.display()
    }
}

/// `Entity.blockPosition`: the block a position is in.
pub(super) fn block_of(position: (f64, f64, f64)) -> [i32; 3] {
    [position.0, position.1, position.2].map(|value| value.floor() as i32)
}

pub(crate) fn is_vanilla_shulker_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_SHULKER_ENTITY_TYPE_ID
}

/// Which display a type is, if it is one (`Display`).
pub(crate) fn display_kind_of(entity_type_id: i32) -> Option<crate::objects::display::DisplayKind> {
    use crate::objects::display::DisplayKind;
    match photon_data::entity_type::name(entity_type_id)? {
        "minecraft:block_display" => Some(DisplayKind::Block),
        "minecraft:item_display" => Some(DisplayKind::Item),
        "minecraft:text_display" => Some(DisplayKind::Text),
        _ => None,
    }
}

pub(crate) fn is_vanilla_happy_ghast_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID
}
