//! The durability and size of a stack, as the item models read them.

use photon_game::menus::container::{damage, is_damaged, max_damage, max_stack_size};
use photon_render::item_model::StackNumbers;
use photon_world::ClientItemStackState;

/// What the `damaged`, `damage` and `count` item model properties read of a stack.
pub fn stack_numbers(stack: &ClientItemStackState) -> StackNumbers {
    StackNumbers {
        damaged: is_damaged(stack),
        damage: damage(stack),
        max_damage: max_damage(stack),
        count: stack.item_count(),
        max_count: max_stack_size(stack),
    }
}
