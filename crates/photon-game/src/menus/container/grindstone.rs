//! `GrindstoneMenu.computeResult`, which the client runs too: one
//! enchanted item loses all but its curses, two of an item are merged with
//! a bonus to their durability.

use super::components::{
    damage, enchantments_for_crafting, is_damageable, level, max_damage, set_damage, set_int,
    transmute, update_enchantments, upgrade,
};
use super::transform::has_any_enchantments;
use super::*;
use photon_world::item_components::ids;

impl ContainerMenu {
    /// `computeResult` of the two inputs.
    pub(super) fn grindstone_result(&self) -> Option<Stack> {
        let (input, additional) = (self.slots[0].as_ref(), self.slots[1].as_ref());
        let count = |stack: Option<&Stack>| stack.map_or(0, Stack::item_count);
        if input.is_none() && additional.is_none() || count(input) > 1 || count(additional) > 1 {
            return None;
        }
        match (input, additional) {
            (Some(input), Some(additional)) => self.merge_items(input, additional),
            (Some(item), None) | (None, Some(item)) => {
                if !has_any_enchantments(item) {
                    return None;
                }
                Some(self.remove_non_curses_from(item.copy_with_count(item.item_count())))
            }
            (None, None) => None,
        }
    }

    /// `mergeItems`.
    fn merge_items(&self, input: &Stack, additional: &Stack) -> Option<Stack> {
        if input.item_registry_id() != additional.item_registry_id() {
            return None;
        }
        let durability = max_damage(input).max(max_damage(additional));
        let remaining1 = max_damage(input) - damage(input);
        let remaining2 = max_damage(additional) - damage(additional);
        let remaining = remaining1 + remaining2 + durability * 5 / 100;
        let mut count = 1;
        if !is_damageable(input) {
            if max_stack_size(input) < 2 || !input.matches(additional) {
                return None;
            }
            count = 2;
        }
        let mut made = input.copy_with_count(count);
        if is_damageable(&made) {
            set_int(&mut made, ids::MAX_DAMAGE, durability);
            set_damage(&mut made, (durability - remaining).max(0));
        }
        // `mergeEnchantsFrom`: a curse the item has already is not raised.
        let from = enchantments_for_crafting(additional);
        update_enchantments(&mut made, |enchantments| {
            for (id, value) in from {
                if !self.traits(id).curse || level(enchantments, id) == 0 {
                    upgrade(enchantments, id, value);
                }
            }
        });
        Some(self.remove_non_curses_from(made))
    }

    /// `removeNonCursesFrom`: a book left with none is a plain book, and the
    /// repair cost restarts from the curses that stay.
    fn remove_non_curses_from(&self, mut item: Stack) -> Stack {
        let left = update_enchantments(&mut item, |enchantments| {
            enchantments.retain(|&(id, _)| self.traits(id).curse);
        });
        let book = crate::item::id_of("minecraft:book");
        if Some(item.item_registry_id()) == crate::item::id_of("minecraft:enchanted_book")
            && left.is_empty()
            && let Some(book) = book
        {
            item = transmute(&item, book);
        }
        let repair_cost = left
            .iter()
            .fold(0_i32, |cost, _| cost.saturating_mul(2).saturating_add(1));
        set_int(&mut item, ids::REPAIR_COST, repair_cost);
        item
    }
}
