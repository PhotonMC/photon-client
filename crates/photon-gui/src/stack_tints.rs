//! A stack's components, as the item tint sources read them.

use photon_core::ids::ItemId;
use photon_render::item_model::{CustomModelData, ItemTints};
use photon_world::ClientItemStackState;

/// The tints' view of a stack's components.
pub struct StackTints {
    values: photon_game::item::values::Values,
    custom: Option<Box<CustomModelData>>,
}

/// The tints of a stack's components.
pub fn stack_tints(item: ItemId, stack: &ClientItemStackState) -> StackTints {
    StackTints {
        values: photon_game::item::values::Values::new(item, stack.patch_bytes()),
        custom: crate::stack_custom_model_data(stack),
    }
}

impl ItemTints for StackTints {
    fn dyed_color(&self) -> Option<i32> {
        self.values.dyed_color()
    }

    fn potion_color_or(&self, default: i32) -> i32 {
        self.values.potion_color_or(default)
    }

    fn map_color(&self) -> Option<i32> {
        self.values.map_color()
    }

    fn firework_colors(&self) -> Vec<i32> {
        self.values.firework_colors()
    }

    fn custom_color(&self, index: usize) -> Option<i32> {
        self.custom.as_ref()?.color(index)
    }
}
