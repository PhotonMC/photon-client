//! The menus that turn what is put in them into something else: the
//! stonecutter, grindstone, cartography table, loom, smithing table, anvil and
//! enchanting table. What the client runs of them is the slots' rules, what
//! happens when a result is taken and `quickMoveStack`; the results
//! themselves are made by the server (the client's copy of each menu has no
//! level to make them with, or is predicted as empty), and arrive as slot
//! updates.
//!
//! Not predicted (the server corrects): the grindstone's, the loom's and the
//! anvil's result, which the reference works out on the client too; the
//! anvil's use of its additional item on a repair.

use voxel_data::item::classes::ItemClass;
use voxel_data::item::component_types as ids;

use super::{ContainerMenu, MenuKind, Player, Stack, settle, stacking};

/// Whether the stack's item is a named one.
fn is_item(stack: &Stack, name: &str) -> bool {
    stacking(stack).is_some_and(|item| item.name == name)
}

fn is_class(stack: &Stack, class: ItemClass) -> bool {
    stacking(stack).is_some_and(|item| item.class.is(class))
}

/// `ItemStack.has(component)`, defaults included.
fn has_component(stack: &Stack, component: i32) -> bool {
    crate::item_values::Values::new(stack.item_registry_id(), stack.patch_bytes()).has(component)
}

/// `EnchantmentHelper.hasAnyEnchantments`.
fn has_any_enchantments(stack: &Stack) -> bool {
    let values = crate::item_values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    !values.enchantments(ids::ENCHANTMENTS).is_empty()
        || !values.enchantments(ids::STORED_ENCHANTMENTS).is_empty()
}

impl ContainerMenu {
    /// `ItemStack.is(tag)`.
    fn in_tag(&self, stack: &Stack, tag: &str) -> bool {
        self.item_tags.contains(stack.item_registry_id(), tag)
    }

    /// `Slot.mayPlace` of these menus' slots; `None` for another menu's.
    pub(super) fn transform_may_place(&self, index: usize, stack: &Stack) -> Option<bool> {
        if self.kind.result_slot() == Some(index)
            && !matches!(self.kind, MenuKind::Inventory | MenuKind::Crafting)
            && !matches!(self.kind, MenuKind::Furnace(_))
        {
            return Some(false);
        }
        Some(match (self.kind, index) {
            (MenuKind::Grindstone, 0 | 1) => {
                stacking(stack).is_some_and(|item| item.max_damage > 0)
                    || has_any_enchantments(stack)
            }
            (MenuKind::Cartography, 0) => has_component(stack, ids::MAP_ID),
            (MenuKind::Cartography, 1) => {
                is_item(stack, "minecraft:paper")
                    || is_item(stack, "minecraft:filled_map")
                    || is_item(stack, "minecraft:glass_pane")
            }
            (MenuKind::Smithing, 0..=2) => self
                .accepted
                .get(index)
                .is_some_and(|items| items.contains(&stack.item_registry_id())),
            (MenuKind::Loom, 0) => is_class(stack, ItemClass::BannerItem),
            (MenuKind::Loom, 1) => {
                self.in_tag(stack, "minecraft:loom_dyes") && has_component(stack, ids::DYE)
            }
            (MenuKind::Loom, 2) => {
                self.in_tag(stack, "minecraft:loom_patterns")
                    && has_component(stack, ids::PROVIDES_BANNER_PATTERNS)
            }
            (MenuKind::Enchantment, 1) => is_item(stack, "minecraft:lapis_lazuli"),
            (
                MenuKind::Stonecutter
                | MenuKind::Grindstone
                | MenuKind::Cartography
                | MenuKind::Loom
                | MenuKind::Smithing
                | MenuKind::Anvil
                | MenuKind::Enchantment,
                _,
            ) => true,
            _ => return None,
        })
    }

    /// What taking a result does on the client: the inputs it uses up.
    pub(super) fn transform_on_take(&mut self, index: usize, _player: Player) {
        if self.kind.result_slot() != Some(index) {
            return;
        }
        match self.kind {
            MenuKind::Stonecutter => {
                let _used = self.remove(0, 1);
            }
            MenuKind::Grindstone => {
                self.slots[0] = None;
                self.slots[1] = None;
            }
            MenuKind::Cartography => {
                let _map = self.remove(0, 1);
                let _extra = self.remove(1, 1);
            }
            MenuKind::Loom => {
                let _banner = self.remove(0, 1);
                let _dye = self.remove(1, 1);
                if self.slots[0].is_none() || self.slots[1].is_none() {
                    // `selectedBannerPatternIndex`.
                    if let Some(selected) = self.data.first_mut() {
                        *selected = -1;
                    }
                }
            }
            MenuKind::Smithing => {
                for input in 0..3 {
                    if let Some(stack) = self.slots[input].as_mut() {
                        stack.shrink(1);
                    }
                    settle(&mut self.slots[input]);
                }
            }
            MenuKind::Anvil => {
                // `repairItemCountCost` and `onlyRenaming` are not known
                // here, so the additional item goes whole.
                self.slots[1] = None;
                self.slots[0] = None;
            }
            _ => {}
        }
    }

    /// The generic end of a `quickMoveStack`: the source slot settles, a
    /// stack that did not change comes to nothing, then `onTake`.
    fn finish_quick_move(
        &mut self,
        index: usize,
        stack: Stack,
        clicked: &Stack,
        player: Player,
    ) -> Option<Stack> {
        self.slots[index] = Some(stack);
        settle(&mut self.slots[index]);
        let left = self.slots[index].as_ref().map_or(0, Stack::item_count);
        if left == clicked.item_count() {
            return None;
        }
        let taken = self.slots[index]
            .clone()
            .unwrap_or_else(|| clicked.copy_with_count(0));
        self.on_take(index, &taken, player);
        Some(clicked.clone())
    }

    /// `quickMoveStack` of these menus.
    #[expect(
        clippy::too_many_lines,
        clippy::cognitive_complexity,
        reason = "one arm per menu, read against its Java"
    )]
    pub(super) fn quick_move_transform(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let size = self.slots.len();
        let moved = match self.kind {
            MenuKind::Stonecutter => {
                if index == 1 {
                    self.move_item_stack_to(&mut stack, 2, 38, true)
                } else if index == 0 {
                    self.move_item_stack_to(&mut stack, 2, 38, false)
                } else if self
                    .accepted
                    .first()
                    .is_some_and(|inputs| inputs.contains(&stack.item_registry_id()))
                {
                    self.move_item_stack_to(&mut stack, 0, 1, false)
                } else if index < 29 {
                    self.move_item_stack_to(&mut stack, 29, 38, false)
                } else {
                    self.move_item_stack_to(&mut stack, 2, 29, false)
                }
            }
            MenuKind::Grindstone => {
                if index == 2 {
                    self.move_item_stack_to(&mut stack, 3, 39, true)
                } else if index > 2 {
                    if self.slots[0].is_some() && self.slots[1].is_some() {
                        if index < 30 {
                            self.move_item_stack_to(&mut stack, 30, 39, false)
                        } else {
                            self.move_item_stack_to(&mut stack, 3, 30, false)
                        }
                    } else {
                        self.move_item_stack_to(&mut stack, 0, 2, false)
                    }
                } else {
                    self.move_item_stack_to(&mut stack, 3, 39, false)
                }
            }
            MenuKind::Cartography => {
                if index == 2 {
                    self.move_item_stack_to(&mut stack, 3, 39, true)
                } else if index > 2 {
                    let is_extra = is_item(&stack, "minecraft:paper")
                        || is_item(&stack, "minecraft:filled_map")
                        || is_item(&stack, "minecraft:glass_pane");
                    if has_component(&stack, ids::MAP_ID) {
                        self.move_item_stack_to(&mut stack, 0, 1, false)
                    } else if is_extra {
                        self.move_item_stack_to(&mut stack, 1, 2, false)
                    } else if index < 30 {
                        self.move_item_stack_to(&mut stack, 30, 39, false)
                    } else {
                        self.move_item_stack_to(&mut stack, 3, 30, false)
                    }
                } else {
                    self.move_item_stack_to(&mut stack, 3, 39, false)
                }
            }
            MenuKind::Loom => {
                if index == 3 {
                    self.move_item_stack_to(&mut stack, 4, 40, true)
                } else if index > 3 {
                    if is_class(&stack, ItemClass::BannerItem) {
                        self.move_item_stack_to(&mut stack, 0, 1, false)
                    } else if is_class(&stack, ItemClass::DyeItem) {
                        self.move_item_stack_to(&mut stack, 1, 2, false)
                    } else if has_component(&stack, ids::PROVIDES_BANNER_PATTERNS) {
                        self.move_item_stack_to(&mut stack, 2, 3, false)
                    } else if index < 31 {
                        self.move_item_stack_to(&mut stack, 31, 40, false)
                    } else {
                        self.move_item_stack_to(&mut stack, 4, 31, false)
                    }
                } else {
                    self.move_item_stack_to(&mut stack, 4, 40, false)
                }
            }
            MenuKind::Smithing | MenuKind::Anvil => {
                // `ItemCombinerMenu.quickMoveStack` with its result slot at R.
                let result = self.kind.result_slot().expect("a result slot");
                let can_move_into_inputs = match self.kind {
                    MenuKind::Smithing => (0..3).any(|input| {
                        self.slots[input].is_none()
                            && self
                                .accepted
                                .get(input)
                                .is_some_and(|items| items.contains(&stack.item_registry_id()))
                    }),
                    _ => true,
                };
                if index == result {
                    self.move_item_stack_to(&mut stack, result + 1, size, true)
                } else if index < result {
                    self.move_item_stack_to(&mut stack, result + 1, size, false)
                } else if can_move_into_inputs {
                    self.move_item_stack_to(&mut stack, 0, result, false)
                } else if index < result + 28 {
                    self.move_item_stack_to(&mut stack, result + 28, size, false)
                } else {
                    self.move_item_stack_to(&mut stack, result + 1, result + 28, false)
                }
            }
            MenuKind::Enchantment => {
                if index < 2 {
                    self.move_item_stack_to(&mut stack, 2, 38, true)
                } else if is_item(&stack, "minecraft:lapis_lazuli") {
                    self.move_item_stack_to(&mut stack, 1, 2, true)
                } else {
                    // One of the stack goes into the empty item slot.
                    if self.slots[0].is_some() || !self.may_place(0, &stack) {
                        self.slots[index] = Some(stack);
                        return None;
                    }
                    let single = stack.split(1);
                    self.slots[0] = Some(single);
                    true
                }
            }
            _ => false,
        };
        if !moved {
            self.slots[index] = Some(stack);
            settle(&mut self.slots[index]);
            return None;
        }
        let taken = self.finish_quick_move(index, stack, &clicked, player);
        if taken.is_some() && self.kind == MenuKind::Stonecutter && index == 1 {
            self.drop(self.slots[index].clone().as_ref());
        }
        taken
    }
}
