use super::*;

impl ContainerMenu {
    /// `canTakeItemForPickAll`: the result containers of these menus are
    /// skipped when a double click gathers.
    pub fn can_take_for_pick_all(&self, index: usize) -> bool {
        // `MerchantMenu.canTakeItemForPickAll`: from none of them.
        if self.kind == MenuKind::Merchant {
            return false;
        }
        !(matches!(
            self.kind,
            MenuKind::Inventory
                | MenuKind::Crafting
                | MenuKind::Stonecutter
                | MenuKind::Cartography
                | MenuKind::Smithing
        ) && self.kind.result_slot() == Some(index))
    }

    /// The crafting grid's slots, which the result slot uses up.
    pub(super) fn grid(&self) -> std::ops::Range<usize> {
        match self.kind {
            MenuKind::Crafting => 1..10,
            _ => 1..5,
        }
    }

    /// `Slot.isSameInventory`: in the player menu the result, the crafting
    /// grid and the rest are three containers; in the others the menu's own
    /// slots and the player's are two.
    pub fn same_inventory(&self, a: usize, b: usize) -> bool {
        let part = |index: usize| match self.kind {
            MenuKind::Inventory => match index {
                0 => 0,
                1..5 => 1,
                _ => 2,
            },
            MenuKind::Crafting => match index {
                0 => 0,
                1..10 => 1,
                _ => 2,
            },
            MenuKind::Furnace(_) => usize::from(index >= 3),
            MenuKind::Enchantment => usize::from(index >= 2),
            MenuKind::Crafter => match index {
                0..9 => 0,
                45 => 1,
                _ => 2,
            },
            other => {
                if other.result_slot() == Some(index) {
                    1
                } else if index < other.top_slots() {
                    0
                } else {
                    2
                }
            }
        };
        part(a) == part(b)
    }

    /// The armour slot at a menu index, in the player menu.
    pub(super) fn armour_at(&self, index: usize) -> Option<&'static str> {
        if self.kind == MenuKind::Inventory {
            armour_slot(index)
        } else {
            None
        }
    }

    /// An `Inventory` index as this menu's slot: the hotbar and the main
    /// inventory in both, the armour and the off hand in the player menu.
    pub(super) fn inventory_to_menu(&self, index: usize) -> Option<usize> {
        match self.kind {
            MenuKind::Inventory => Some(inventory_to_menu(index)),
            other => {
                let top = other.top_slots();
                match index {
                    0..9 => Some(top + 27 + index),
                    9..36 => Some(top + index - 9),
                    _ => None,
                }
            }
        }
    }

    /// `Slot.getMaxStackSize(stack)`.
    pub fn slot_max_for(&self, index: usize, stack: &Stack) -> i32 {
        let slot_max = if self.armour_at(index).is_some()
            || (matches!(self.kind, MenuKind::Furnace(_)) && index == 1 && is_bucket(stack))
            || (self.kind == MenuKind::Enchantment && index == 0)
            || (self.kind == MenuKind::Brewing && index < 3)
            || (self.kind == MenuKind::Beacon && index == 0)
        {
            1
        } else {
            99
        };
        slot_max.min(max_stack_size(stack))
    }

    /// `Slot.mayPlace`.
    pub fn may_place(&self, index: usize, stack: &Stack) -> bool {
        if self.is_result(index) {
            return false;
        }
        if let Some(allowed) = self.transform_may_place(index, stack) {
            return allowed;
        }
        if matches!(self.kind, MenuKind::Furnace(_)) {
            match index {
                // `FurnaceResultSlot`.
                2 => return false,
                // `FurnaceFuelSlot`.
                1 => return self.is_fuel(stack) || is_bucket(stack),
                _ => {}
            }
        }
        // `ShulkerBoxSlot`: a shulker box does not go inside one.
        if self.kind == MenuKind::ShulkerBox && index < self.kind.top_slots() {
            return can_fit_inside_container_items(stack);
        }
        match self.armour_at(index) {
            Some(slot) => stacking(stack)
                .and_then(|item| item.equippable.as_ref())
                .is_some_and(|equip| equip.slot == slot && equip.player),
            None => true,
        }
    }

    /// `Slot.mayPickup`: every slot but the anvil's result, which needs the
    /// levels it costs; armour locks are not read.
    pub(super) fn may_pickup(&self, index: usize) -> bool {
        if self.kind == MenuKind::Anvil && index == 2 {
            let cost = i32::from(self.data.first().copied().unwrap_or(0));
            return (self.infinite_materials || self.xp_level >= cost) && cost > 0;
        }
        // `NonInteractiveResultSlot`.
        !(self.kind == MenuKind::Crafter && index == 45)
    }

    /// `Slot.allowModification`.
    pub(super) fn allow_modification(&self, index: usize) -> bool {
        self.may_pickup(index)
            && self.slots[index]
                .as_ref()
                .is_none_or(|stack| self.may_place(index, stack))
    }

    /// `AbstractContainerMenu.canItemQuickReplace`.
    pub fn can_item_quick_replace(&self, index: usize, stack: &Stack, ignore_size: bool) -> bool {
        match &self.slots[index] {
            Some(slot) if stack.is_same_item_same_components(slot) => {
                slot.item_count() + if ignore_size { 0 } else { stack.item_count() }
                    <= max_stack_size(stack)
            }
            Some(_) => false,
            None => true,
        }
    }

    pub(super) fn set_by_player(&mut self, index: usize, stack: Option<Stack>) {
        self.slots[index] = stack;
        settle(&mut self.slots[index]);
    }

    /// `Slot.remove`: the result slot gives its whole stack.
    pub(super) fn remove(&mut self, index: usize, amount: i32) -> Option<Stack> {
        if self.is_result(index) {
            return self.slots[index].take();
        }
        let stack = self.slots[index].as_mut()?;
        if amount <= 0 {
            return None;
        }
        let taken = stack.split(amount);
        settle(&mut self.slots[index]);
        Some(taken).filter(|taken| taken.item_count() > 0)
    }

    /// `Slot.tryRemove`.
    pub(super) fn try_remove(&mut self, index: usize, amount: i32, max: i32) -> Option<Stack> {
        if !self.may_pickup(index) {
            return None;
        }
        let count = self.slots[index].as_ref().map_or(0, Stack::item_count);
        if !self.allow_modification(index) && max < count {
            return None;
        }
        let taken = self.remove(index, amount.min(max))?;
        Some(taken)
    }

    /// `Slot.safeTake`.
    pub(super) fn safe_take(
        &mut self,
        index: usize,
        amount: i32,
        max: i32,
        player: Player,
    ) -> Option<Stack> {
        let taken = self.try_remove(index, amount, max)?;
        self.on_take(index, &taken, player);
        Some(taken)
    }

    /// `Slot.safeInsert`: what is left of the stack.
    pub(super) fn safe_insert(
        &mut self,
        index: usize,
        mut input: Stack,
        amount: i32,
    ) -> Option<Stack> {
        if !self.may_place(index, &input) {
            return Some(input);
        }
        let present = self.slots[index].as_ref().map_or(0, Stack::item_count);
        let moved = amount
            .min(input.item_count())
            .min(self.slot_max_for(index, &input) - present);
        if moved <= 0 {
            return Some(input);
        }
        match self.slots[index].as_mut() {
            None => {
                let part = input.split(moved);
                self.set_by_player(index, Some(part));
            }
            Some(slot) if slot.is_same_item_same_components(&input) => {
                input.shrink(moved);
                slot.grow(moved);
            }
            Some(_) => {}
        }
        Some(input).filter(|input| input.item_count() > 0)
    }

    /// `Slot.onTake`: the result slot uses up the crafting grid.
    pub(super) fn on_take(&mut self, index: usize, _taken: &Stack, player: Player) {
        if self.kind == MenuKind::Merchant && index == 2 {
            self.take_merchant_result();
        }
        self.transform_on_take(index, player);
        if !matches!(self.kind, MenuKind::Inventory | MenuKind::Crafting) || !self.is_result(index)
        {
            return;
        }
        // `ResultSlot.onTake` with `CraftingRecipe.defaultCraftingReminder`,
        // row by row through the grid.
        for grid in self.grid() {
            let Some(stack) = self.slots[grid].as_ref() else {
                continue;
            };
            let remainder = stacking(stack)
                .and_then(|item| item.crafting_remainder.as_ref())
                .and_then(|left| {
                    crate::item::id_of(left.item).map(|id| Stack::new(left.count, id))
                });
            let _used = self.remove(grid, 1);
            let Some(mut remainder) = remainder else {
                continue;
            };
            match self.slots[grid].as_ref() {
                None => self.slots[grid] = Some(remainder),
                Some(present) if present.is_same_item_same_components(&remainder) => {
                    remainder.grow(present.item_count());
                    self.slots[grid] = Some(remainder);
                }
                Some(_) => {
                    let mut remainder = Some(remainder);
                    if !self.inventory_add(&mut remainder, player) {
                        self.drop(remainder.as_ref());
                    }
                }
            }
        }
    }

    /// `quickMoveStack` of a generic container menu: out of its own slots into
    /// the player's, from the end, or the player's into its own.
    pub(super) fn quick_move_container(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let top = self.kind.top_slots();
        let (start, end, backwards) = if index < top {
            (top, self.slots.len(), true)
        } else {
            (0, top, false)
        };
        let mut stack = self.slots[index].take().expect("checked above");
        let moved = self.move_item_stack_to(&mut stack, start, end, backwards);
        self.slots[index] = Some(stack);
        settle(&mut self.slots[index]);
        if !moved {
            return None;
        }
        // Only the dispenser's menu gives up when nothing came off the stack.
        if self.kind == MenuKind::Dispenser {
            let left = self.slots[index].as_ref().map_or(0, Stack::item_count);
            if left == clicked.item_count() {
                return None;
            }
            let taken = self.slots[index]
                .clone()
                .unwrap_or_else(|| clicked.copy_with_count(0));
            self.on_take(index, &taken, player);
        }
        Some(clicked)
    }

    /// `CraftingMenu.quickMoveStack`: the result into the player's slots
    /// from the end, the player's into the grid and then across the
    /// inventory and hotbar, the grid back into the player's.
    pub(super) fn quick_move_crafting(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let moved = if index == 0 {
            self.move_item_stack_to(&mut stack, 10, 46, true)
        } else if (10..46).contains(&index) {
            self.move_item_stack_to(&mut stack, 1, 10, false)
                || if index < 37 {
                    self.move_item_stack_to(&mut stack, 37, 46, false)
                } else {
                    self.move_item_stack_to(&mut stack, 10, 37, false)
                }
        } else {
            self.move_item_stack_to(&mut stack, 10, 46, false)
        };
        self.slots[index] = Some(stack);
        settle(&mut self.slots[index]);
        if !moved {
            return None;
        }
        let left = self.slots[index].as_ref().map_or(0, Stack::item_count);
        if left == clicked.item_count() {
            return None;
        }
        let taken = self.slots[index]
            .clone()
            .unwrap_or_else(|| clicked.copy_with_count(0));
        self.on_take(index, &taken, player);
        if index == 0 {
            self.drop(self.slots[index].clone().as_ref());
        }
        Some(clicked)
    }

    /// `AbstractFurnaceMenu.isFuel`.
    pub(super) fn is_fuel(&self, stack: &Stack) -> bool {
        stacking(stack).is_some_and(|item| item.burn_time > 0)
    }

    /// `AbstractFurnaceMenu.canSmelt`.
    pub(super) fn can_smelt(&self, stack: &Stack) -> bool {
        self.accepted
            .first()
            .is_some_and(|inputs| inputs.contains(&stack.item_registry_id()))
    }

    /// `AbstractFurnaceMenu.quickMoveStack`.
    pub(super) fn quick_move_furnace(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let moved = if index == 2 {
            self.move_item_stack_to(&mut stack, 3, 39, true)
        } else if index > 2 {
            if self.can_smelt(&stack) {
                self.move_item_stack_to(&mut stack, 0, 1, false)
            } else if self.is_fuel(&stack) {
                self.move_item_stack_to(&mut stack, 1, 2, false)
            } else if index < 30 {
                self.move_item_stack_to(&mut stack, 30, 39, false)
            } else {
                self.move_item_stack_to(&mut stack, 3, 30, false)
            }
        } else {
            self.move_item_stack_to(&mut stack, 3, 39, false)
        };
        self.slots[index] = Some(stack);
        settle(&mut self.slots[index]);
        if !moved {
            return None;
        }
        let left = self.slots[index].as_ref().map_or(0, Stack::item_count);
        if left == clicked.item_count() {
            return None;
        }
        let taken = self.slots[index]
            .clone()
            .unwrap_or_else(|| clicked.copy_with_count(0));
        self.on_take(index, &taken, player);
        Some(clicked)
    }

    /// `BrewingStandMenu.quickMoveStack`. A move that fuels the stand ends the click (its stack
    /// was changed in place, and the reference returns nothing for it).
    pub(super) fn quick_move_brewing(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let keeps_going = if index >= 5 {
            if self.in_tag(&clicked, "minecraft:brewing_fuel") {
                let fueled = self.move_item_stack_to(&mut stack, 4, 5, false);
                !(fueled
                    || self.may_place(3, &stack)
                        && !self.move_item_stack_to(&mut stack, 3, 4, false))
            } else if self.may_place(3, &stack) {
                self.move_item_stack_to(&mut stack, 3, 4, false)
            } else if self.may_place(0, &clicked) {
                self.move_item_stack_to(&mut stack, 0, 3, false)
            } else if index < 32 {
                self.move_item_stack_to(&mut stack, 32, 41, false)
            } else if index < 41 {
                self.move_item_stack_to(&mut stack, 5, 32, false)
            } else {
                self.move_item_stack_to(&mut stack, 5, 41, false)
            }
        } else {
            self.move_item_stack_to(&mut stack, 5, 41, true)
        };
        self.slots[index] = Some(stack);
        settle(&mut self.slots[index]);
        if !keeps_going {
            return None;
        }
        let left = self.slots[index].as_ref().map_or(0, Stack::item_count);
        if left == clicked.item_count() {
            return None;
        }
        let taken = self.slots[index]
            .clone()
            .unwrap_or_else(|| clicked.copy_with_count(0));
        self.on_take(index, &taken, player);
        Some(clicked)
    }
}
