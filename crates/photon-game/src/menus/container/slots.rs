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
        if matches!(self.kind, MenuKind::Mount(_)) {
            return self.mount_may_place(index, stack);
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
        // `ArmorSlot`: armour with an enchantment that prevents changing it
        // stays on but in creative.
        if self.mount_keeps(index) {
            return false;
        }
        if self.armour_at(index).is_some()
            && !self.infinite_materials
            && self.slots[index].as_ref().is_some_and(|stack| {
                enchantment_ids(stack).any(|id| self.traits(id).prevents_armor_change)
            })
        {
            return false;
        }
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
}
