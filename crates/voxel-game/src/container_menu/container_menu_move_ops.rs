use super::*;

impl ContainerMenu {
    /// `BeaconMenu.quickMoveStack`: a single payment item goes to the empty payment slot.
    pub(super) fn quick_move_beacon(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let keeps_going = if index == 0 {
            self.move_item_stack_to(&mut stack, 1, 37, true)
        } else if self.slots[0].is_none() && self.may_place(0, &stack) && stack.item_count() == 1 {
            self.move_item_stack_to(&mut stack, 0, 1, false)
        } else if index < 28 {
            self.move_item_stack_to(&mut stack, 28, 37, false)
        } else if index < 37 {
            self.move_item_stack_to(&mut stack, 1, 28, false)
        } else {
            self.move_item_stack_to(&mut stack, 1, 37, false)
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

    /// `CrafterMenu.quickMoveStack`: the grid's items go to the inventory, the others to the grid.
    pub(super) fn quick_move_crafter(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let keeps_going = if index < 9 {
            self.move_item_stack_to(&mut stack, 9, 45, true)
        } else {
            self.move_item_stack_to(&mut stack, 0, 9, false)
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

    /// `MerchantMenu.quickMoveStack`.
    pub(super) fn quick_move_merchant(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let keeps_going = if index == 2 {
            self.move_item_stack_to(&mut stack, 3, 39, true)
        } else if index != 0 && index != 1 {
            if index < 30 {
                self.move_item_stack_to(&mut stack, 30, 39, false)
            } else {
                // The hotbar's slots go on into the inventory; a slot past it stays.
                index >= 39 || self.move_item_stack_to(&mut stack, 3, 30, false)
            }
        } else {
            self.move_item_stack_to(&mut stack, 3, 39, false)
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

    /// `MerchantContainer.updateSellItem`: the offer the payments buy, and its result.
    pub fn update_sell_item(&mut self) {
        use crate::merchant::recipe_for;
        self.active_offer = None;
        let (buy_a, buy_b) = if self.slots[0].is_none() {
            (self.slots[1].clone(), None)
        } else {
            (self.slots[0].clone(), self.slots[1].clone())
        };
        if buy_a.is_none() {
            self.slots[2] = None;
            self.future_xp = 0;
            return;
        }
        if self.offers.is_empty() {
            return;
        }
        let offers = self.offers.clone();
        let mut found = recipe_for(&offers, buy_a.as_ref(), buy_b.as_ref(), self.selection_hint);
        if found.is_none_or(|index| offers[index].is_out_of_stock()) {
            self.active_offer = found;
            found = recipe_for(&offers, buy_b.as_ref(), buy_a.as_ref(), self.selection_hint);
        }
        match found.filter(|index| !offers[*index].is_out_of_stock()) {
            Some(index) => {
                self.active_offer = Some(index);
                self.slots[2] = Some(offers[index].result.clone());
                self.future_xp = offers[index].xp;
            }
            None => {
                self.slots[2] = None;
                self.future_xp = 0;
            }
        }
    }

    /// `MerchantMenu.tryMoveItems`: the payments go back to the inventory, and the ones the chosen
    /// offer wants come from it.
    pub fn try_move_items(&mut self, trade: usize) {
        let Some(offer) = self.offers.get(trade).cloned() else {
            return;
        };
        for slot in 0..2 {
            if let Some(mut stack) = self.slots[slot].take() {
                let moved = self.move_item_stack_to(&mut stack, 3, 39, true);
                self.slots[slot] = Some(stack);
                settle(&mut self.slots[slot]);
                if !moved {
                    return;
                }
            }
        }
        if self.slots[0].is_none() && self.slots[1].is_none() {
            self.fill_payment(0, &offer.cost_a);
            if let Some(cost) = &offer.cost_b {
                self.fill_payment(1, cost);
            }
        }
        self.update_sell_item();
    }

    /// `MerchantMenu.moveFromInventoryToPaymentSlot`.
    pub(super) fn fill_payment(&mut self, payment: usize, cost: &crate::merchant::ItemCost) {
        for index in 3..39 {
            let Some(inventory) = self.slots[index].as_ref() else {
                continue;
            };
            if !cost.test(Some(inventory)) {
                continue;
            }
            let current = self.slots[payment].as_ref();
            if current.is_some_and(|stack| !stack.is_same_item_same_components(inventory)) {
                continue;
            }
            let held = current.map_or(0, Stack::item_count);
            let max = max_stack_size(inventory);
            let moved = (max - held).min(inventory.item_count());
            let new = inventory.copy_with_count(held + moved);
            if let Some(stack) = self.slots[index].as_mut() {
                stack.shrink(moved);
            }
            settle(&mut self.slots[index]);
            self.slots[payment] = Some(new);
            if held + moved >= max {
                break;
            }
        }
    }

    /// `MerchantResultSlot.onTake`: the offer the payments bought takes its price from them.
    pub(super) fn take_merchant_result(&mut self) {
        let Some(offer) = self
            .active_offer
            .and_then(|index| self.offers.get(index).cloned())
        else {
            return;
        };
        let mut buy_a = self.slots[0].take();
        let mut buy_b = self.slots[1].take();
        if offer.take(&mut buy_a, &mut buy_b) || offer.take(&mut buy_b, &mut buy_a) {
            self.slots[0] = buy_a;
            self.slots[1] = buy_b;
            self.update_sell_item();
        } else {
            self.slots[0] = buy_a;
            self.slots[1] = buy_b;
        }
    }

    /// `quickMoveStack` of the menu.
    pub(super) fn quick_move_stack(&mut self, index: usize, player: Player) -> Option<Stack> {
        match self.kind {
            MenuKind::Furnace(_) => self.quick_move_furnace(index, player),
            MenuKind::Stonecutter
            | MenuKind::Grindstone
            | MenuKind::Cartography
            | MenuKind::Loom
            | MenuKind::Smithing
            | MenuKind::Anvil
            | MenuKind::Enchantment => self.quick_move_transform(index, player),
            MenuKind::Brewing => self.quick_move_brewing(index, player),
            MenuKind::Beacon => self.quick_move_beacon(index, player),
            MenuKind::Crafter => self.quick_move_crafter(index, player),
            // `LecternMenu.quickMoveStack` moves nothing.
            MenuKind::Lectern => None,
            MenuKind::Merchant => self.quick_move_merchant(index, player),
            MenuKind::Inventory => self.quick_move_inventory(index, player),
            MenuKind::Crafting => self.quick_move_crafting(index, player),
            _ => self.quick_move_container(index, player),
        }
    }

    /// `InventoryMenu.quickMoveStack`.
    pub(super) fn quick_move_inventory(&mut self, index: usize, player: Player) -> Option<Stack> {
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let equip = equipment_slot(&clicked);
        let armour = armour_index(equip);
        let range = if index == 0 {
            (9, 45, true)
        } else if (1..9).contains(&index) {
            (9, 45, false)
        } else if let Some(armour) = armour.filter(|armour| self.slots[8 - armour].is_none()) {
            (8 - armour, 9 - armour, false)
        } else if equip == "offhand" && self.slots[OFF_HAND].is_none() {
            (45, 46, false)
        } else if (9..36).contains(&index) {
            (36, 45, false)
        } else if (36..45).contains(&index) {
            (9, 36, false)
        } else {
            (9, 45, false)
        };
        let mut stack = self.slots[index].take().expect("checked above");
        let moved = self.move_item_stack_to(&mut stack, range.0, range.1, range.2);
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
        if self.is_result(index) {
            self.drop(self.slots[index].clone().as_ref());
        }
        Some(clicked)
    }

    /// `AbstractContainerMenu.moveItemStackTo`.
    pub(super) fn move_item_stack_to(
        &mut self,
        stack: &mut Stack,
        start: usize,
        end: usize,
        backwards: bool,
    ) -> bool {
        let mut changed = false;
        let order: Vec<usize> = if backwards {
            (start..end).rev().collect()
        } else {
            (start..end).collect()
        };
        if is_stackable(stack) {
            for &dest in &order {
                if stack.item_count() <= 0 {
                    break;
                }
                let max = match &self.slots[dest] {
                    Some(target) if stack.is_same_item_same_components(target) => {
                        self.slot_max_for(dest, target)
                    }
                    _ => continue,
                };
                let target = self.slots[dest].as_mut().expect("checked above");
                let total = target.item_count() + stack.item_count();
                if total <= max {
                    stack.set_count(0);
                    target.set_count(total);
                    changed = true;
                } else if target.item_count() < max {
                    stack.shrink(max - target.item_count());
                    target.set_count(max);
                    changed = true;
                }
            }
        }
        if stack.item_count() > 0 {
            for &dest in &order {
                if self.slots[dest].is_none() && self.may_place(dest, stack) {
                    let max = self.slot_max_for(dest, stack);
                    let part = stack.split(stack.item_count().min(max));
                    self.set_by_player(dest, Some(part));
                    changed = true;
                    break;
                }
            }
        }
        changed
    }

    /// `Inventory.add` for an undamaged stack: into stacks with room, the
    /// selected slot and the off hand first, then into free slots. Whether
    /// any of it went in.
    pub(super) fn inventory_add(&mut self, stack: &mut Option<Stack>, player: Player) -> bool {
        let Some(item) = stack.as_mut() else {
            return false;
        };
        loop {
            let last = item.item_count();
            let slot = self
                .slot_with_remaining_space(item, player)
                .or_else(|| self.free_slot());
            if let Some(slot) = slot {
                let present = self.slots[slot].get_or_insert_with(|| item.copy_with_count(0));
                let room = 99.min(max_stack_size(present)) - present.item_count();
                let add = item.item_count().min(room);
                present.grow(add);
                item.shrink(add);
            }
            if item.item_count() <= 0 || item.item_count() >= last {
                let added = item.item_count() < last;
                if !added && player.infinite_materials {
                    item.set_count(0);
                    settle(stack);
                    return true;
                }
                let any = item.item_count() < last;
                settle(stack);
                return any;
            }
        }
    }

    pub(super) fn slot_with_remaining_space(&self, stack: &Stack, player: Player) -> Option<usize> {
        let has_room = |slot: usize| {
            self.slots[slot].as_ref().is_some_and(|present| {
                present.is_same_item_same_components(stack)
                    && is_stackable(present)
                    && present.item_count() < 99.min(max_stack_size(present))
            })
        };
        let off_hand = (self.kind == MenuKind::Inventory).then_some(OFF_HAND);
        self.inventory_to_menu(player.selected)
            .into_iter()
            .chain(off_hand)
            .chain((0..36).filter_map(|index| self.inventory_to_menu(index)))
            .find(|&slot| has_room(slot))
    }

    pub(super) fn free_slot(&self) -> Option<usize> {
        (0..36)
            .filter_map(|index| self.inventory_to_menu(index))
            .find(|&slot| self.slots[slot].is_none())
    }
}
