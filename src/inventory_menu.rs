//! The player's own inventory menu as the client predicts a click in it:
//! `AbstractContainerMenu.doClick` with `InventoryMenu`'s slots, and the
//! click packet `MultiPlayerGameMode.handleContainerInput` sends.
//!
//! The menu's slots, by menu index: 0 the crafting result, 1 to 4 the
//! crafting grid, 5 to 8 the armour from head to feet, 9 to 35 the main
//! inventory, 36 to 44 the hotbar and 45 the off hand.
//!
//! Stacks are values here where the reference shares objects; each step
//! writes back what the shared object would have become. Not ported: the
//! bundle's click overrides (a click with a bundle carried or clicked is
//! sent without a prediction), damaged items (a damaged stack counts as
//! undamaged), enchantments that lock armour, and component hashes (a
//! stack with a component patch is hashed as if it had none).

use crate::item_stacking_table::ITEM_STACKING;
use crate::world::ClientItemStackState as Stack;

/// `ContainerInput`, by its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Pickup = 0,
    QuickMove = 1,
    Swap = 2,
    Clone = 3,
    Throw = 4,
    QuickCraft = 5,
    PickupAll = 6,
}

impl ClickType {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "PICKUP" => Self::Pickup,
            "QUICK_MOVE" => Self::QuickMove,
            "SWAP" => Self::Swap,
            "CLONE" => Self::Clone,
            "THROW" => Self::Throw,
            "QUICK_CRAFT" => Self::QuickCraft,
            "PICKUP_ALL" => Self::PickupAll,
            _ => return None,
        })
    }
}

/// What a click needs from the player.
#[derive(Debug, Clone, Copy)]
pub struct Player {
    /// The selected hotbar slot, 0 to 8.
    pub selected: usize,
    /// `hasInfiniteMaterials`: creative.
    pub infinite_materials: bool,
}

/// The number of slots in the menu.
pub const SLOT_COUNT: usize = 46;
const RESULT: usize = 0;
const OFF_HAND: usize = 45;

/// The player's inventory menu: its slots, the carried stack and the drag
/// in progress.
#[derive(Debug, Clone)]
pub struct InventoryMenu {
    pub slots: Vec<Option<Stack>>,
    pub carried: Option<Stack>,
    pub state_id: i32,
    quickcraft_status: i32,
    quickcraft_type: i32,
    quickcraft_slots: Vec<usize>,
    /// Arm swings the click caused: `LivingEntity.drop` swings on the
    /// client instead of dropping.
    swings: usize,
}

impl Default for InventoryMenu {
    fn default() -> Self {
        Self {
            slots: vec![None; SLOT_COUNT],
            carried: None,
            state_id: 0,
            quickcraft_status: 0,
            quickcraft_type: 0,
            quickcraft_slots: Vec::new(),
            swings: 0,
        }
    }
}

fn stacking(stack: &Stack) -> Option<&'static crate::item_stacking_table::ItemStacking> {
    usize::try_from(stack.item_registry_id())
        .ok()
        .and_then(|id| ITEM_STACKING.get(id))
}

/// `ItemStack.getMaxStackSize`.
pub fn max_stack_size(stack: &Stack) -> i32 {
    stacking(stack).map_or(64, |item| item.max_stack_size)
}

/// `ItemStack.isStackable`, taking every stack as undamaged.
fn is_stackable(stack: &Stack) -> bool {
    max_stack_size(stack) > 1
}

/// `LivingEntity.getEquipmentSlotForItem` for a player, by name.
fn equipment_slot(stack: &Stack) -> &'static str {
    stacking(stack)
        .and_then(|item| item.equip)
        .map_or("mainhand", |(slot, _)| slot)
}

/// The armour slot's equipment slot, by menu index.
fn armour_slot(index: usize) -> Option<&'static str> {
    match index {
        5 => Some("head"),
        6 => Some("chest"),
        7 => Some("legs"),
        8 => Some("feet"),
        _ => None,
    }
}

/// `EquipmentSlot.getIndex` of a humanoid armour slot.
fn armour_index(slot: &str) -> Option<usize> {
    match slot {
        "feet" => Some(0),
        "legs" => Some(1),
        "chest" => Some(2),
        "head" => Some(3),
        _ => None,
    }
}

fn overrides_click(stack: Option<&Stack>, carried: bool) -> bool {
    stack.and_then(stacking).is_some_and(|item| {
        if carried {
            item.stacked_on_other != "Item"
        } else {
            item.other_stacked_on_me != "Item"
        }
    })
}

/// A stack emptied in place reads as no stack.
fn settle(stack: &mut Option<Stack>) {
    if stack.as_ref().is_some_and(|stack| stack.item_count() <= 0) {
        *stack = None;
    }
}

fn same_item(a: Option<&Stack>, b: &Stack) -> bool {
    a.is_some_and(|a| a.item_registry_id() == b.item_registry_id())
}

fn same_item_same_components(a: &Stack, b: &Stack) -> bool {
    a.is_same_item_same_components(b)
}

impl InventoryMenu {
    /// The menu's slots and carried stack from the client's copy.
    pub fn with_contents(slots: Vec<Option<Stack>>, carried: Option<Stack>, state_id: i32) -> Self {
        let mut menu = Self {
            slots,
            carried,
            state_id,
            ..Self::default()
        };
        menu.slots.resize(SLOT_COUNT, None);
        menu
    }

    /// New contents from the client's copy; a drag in progress carries on.
    pub fn load(&mut self, slots: Vec<Option<Stack>>, carried: Option<Stack>, state_id: i32) {
        self.slots = slots;
        self.slots.resize(SLOT_COUNT, None);
        self.carried = carried;
        self.state_id = state_id;
    }

    /// `MultiPlayerGameMode.handleContainerInput`: the click as the client
    /// predicts it, then the packets it sends: `(swings, click packet body)`.
    pub fn handle_click(
        &mut self,
        slot: i32,
        button: i32,
        click: ClickType,
        player: Player,
    ) -> (usize, Vec<u8>) {
        let before = self.slots.clone();
        self.swings = 0;
        self.clicked(slot, button, click, player);
        let changed: Vec<(usize, Option<&Stack>)> = (0..SLOT_COUNT)
            .filter(|&index| !match (&before[index], &self.slots[index]) {
                (None, None) => true,
                (Some(a), Some(b)) => a.matches(b),
                _ => false,
            })
            .map(|index| (index, self.slots[index].as_ref()))
            .collect();
        let mut body = Vec::new();
        crate::wire::write_varint(0, &mut body);
        crate::wire::write_varint(self.state_id, &mut body);
        body.extend_from_slice(&(slot as i16).to_be_bytes());
        body.push(button as i8 as u8);
        crate::wire::write_varint(click as i32, &mut body);
        let keys: Vec<i32> = changed.iter().map(|&(index, _)| index as i32).collect();
        let order = fastutil_int_order(&keys);
        crate::wire::write_varint(order.len() as i32, &mut body);
        for key in order {
            body.extend_from_slice(&(key as i16).to_be_bytes());
            write_hashed_stack(&mut body, self.slots[key as usize].as_ref());
        }
        write_hashed_stack(&mut body, self.carried.as_ref());
        (self.swings, body)
    }

    /// `AbstractContainerMenu.doClick`.
    pub fn clicked(&mut self, slot_index: i32, button: i32, click: ClickType, player: Player) {
        // The reference would fail on a slot past the menu's end.
        if slot_index >= SLOT_COUNT as i32 {
            return;
        }
        if click == ClickType::QuickCraft {
            let expected = self.quickcraft_status;
            self.quickcraft_status = button & 3;
            // A step out of order, or nothing to drag, drops the drag.
            if ((expected != 1 || self.quickcraft_status != 2)
                && expected != self.quickcraft_status)
                || self.carried.is_none()
            {
                self.reset_quick_craft();
            } else if self.quickcraft_status == 0 {
                self.quickcraft_type = button >> 2 & 3;
                let valid = match self.quickcraft_type {
                    0 | 1 => true,
                    2 => player.infinite_materials,
                    _ => false,
                };
                if valid {
                    self.quickcraft_status = 1;
                    self.quickcraft_slots.clear();
                } else {
                    self.reset_quick_craft();
                }
            } else if self.quickcraft_status == 1 {
                let Ok(index) = usize::try_from(slot_index) else {
                    return;
                };
                if index >= SLOT_COUNT {
                    return;
                }
                let carried = self.carried.as_ref().expect("checked above");
                if self.can_item_quick_replace(index, carried, true)
                    && self.may_place(index, carried)
                    && (self.quickcraft_type == 2
                        || carried.item_count() > self.quickcraft_slots.len() as i32)
                    && !self.quickcraft_slots.contains(&index)
                {
                    self.quickcraft_slots.push(index);
                }
            } else if self.quickcraft_status == 2 {
                if !self.quickcraft_slots.is_empty() {
                    if self.quickcraft_slots.len() == 1 {
                        let index = self.quickcraft_slots[0];
                        let kind = self.quickcraft_type;
                        self.reset_quick_craft();
                        self.clicked(index as i32, kind, ClickType::Pickup, player);
                        return;
                    }
                    let Some(source) = self
                        .carried
                        .as_ref()
                        .map(|carried| carried.copy_with_count(carried.item_count()))
                    else {
                        self.reset_quick_craft();
                        return;
                    };
                    let mut remaining = source.item_count();
                    let count = self.quickcraft_slots.len() as i32;
                    for index in self.quickcraft_slots.clone() {
                        let carried = self.carried.as_ref().expect("checked above");
                        if self.can_item_quick_replace(index, carried, true)
                            && self.may_place(index, carried)
                            && (self.quickcraft_type == 2 || carried.item_count() >= count)
                        {
                            let carry = self.slots[index].as_ref().map_or(0, Stack::item_count);
                            let max =
                                max_stack_size(&source).min(self.slot_max_for(index, &source));
                            let place = match self.quickcraft_type {
                                0 => (source.item_count() as f32 / count as f32).floor() as i32,
                                1 => 1,
                                2 => max_stack_size(&source),
                                _ => source.item_count(),
                            };
                            let new_count = (place + carry).min(max);
                            remaining -= new_count - carry;
                            self.set_by_player(index, Some(source.copy_with_count(new_count)));
                        }
                    }
                    let mut source = source;
                    source.item_count = remaining;
                    self.carried = Some(source);
                    settle(&mut self.carried);
                }
                self.reset_quick_craft();
            } else {
                self.reset_quick_craft();
            }
        } else if self.quickcraft_status != 0 {
            self.reset_quick_craft();
        } else if matches!(click, ClickType::Pickup | ClickType::QuickMove)
            && (button == 0 || button == 1)
        {
            let primary = button == 0;
            if slot_index == -999 {
                if let Some(carried) = self.carried.as_mut() {
                    if primary {
                        self.carried = None;
                        self.swings += 1;
                    } else {
                        let _dropped = carried.split(1);
                        settle(&mut self.carried);
                        self.swings += 1;
                    }
                }
            } else if click == ClickType::QuickMove {
                let Ok(index) = usize::try_from(slot_index) else {
                    return;
                };
                if !self.may_pickup(index) {
                    return;
                }
                let mut clicked = self.quick_move_stack(index, player);
                while let Some(stack) = &clicked {
                    if !same_item(self.slots[index].as_ref(), stack) {
                        break;
                    }
                    clicked = self.quick_move_stack(index, player);
                }
            } else {
                let Ok(index) = usize::try_from(slot_index) else {
                    return;
                };
                if overrides_click(self.carried.as_ref(), true)
                    || overrides_click(self.slots[index].as_ref(), false)
                {
                    return;
                }
                match (self.slots[index].clone(), self.carried.take()) {
                    (None, Some(carried)) => {
                        let amount = if primary { carried.item_count() } else { 1 };
                        self.carried = self.safe_insert(index, carried, amount);
                    }
                    (None, None) => {}
                    (Some(clicked), carried) => {
                        if !self.may_pickup(index) {
                            self.carried = carried;
                        } else if let Some(mut carried) = carried {
                            if self.may_place(index, &carried) {
                                if same_item_same_components(&clicked, &carried) {
                                    let amount = if primary { carried.item_count() } else { 1 };
                                    self.carried = self.safe_insert(index, carried, amount);
                                } else if carried.item_count() <= self.slot_max_for(index, &carried)
                                {
                                    self.carried = Some(clicked);
                                    self.set_by_player(index, Some(carried));
                                } else {
                                    self.carried = Some(carried);
                                }
                            } else if same_item_same_components(&clicked, &carried) {
                                let room = max_stack_size(&carried) - carried.item_count();
                                if let Some(taken) =
                                    self.try_remove(index, clicked.item_count(), room)
                                {
                                    carried.item_count += taken.item_count();
                                    self.carried = Some(carried);
                                    self.on_take(index, &taken, player);
                                } else {
                                    self.carried = Some(carried);
                                }
                            } else {
                                self.carried = Some(carried);
                            }
                        } else {
                            let amount = if primary {
                                clicked.item_count()
                            } else {
                                (clicked.item_count() + 1) / 2
                            };
                            if let Some(taken) = self.try_remove(index, amount, i32::MAX) {
                                self.carried = Some(taken.clone());
                                self.on_take(index, &taken, player);
                            }
                        }
                    }
                }
            }
        } else if click == ClickType::Swap && ((0..9).contains(&button) || button == 40) {
            let Ok(index) = usize::try_from(slot_index) else {
                return;
            };
            let source_slot = inventory_to_menu(button as usize);
            let source = self.slots[source_slot].clone();
            let target = self.slots[index].clone();
            match (source, target) {
                (None, None) => {}
                (None, Some(target)) => {
                    if self.may_pickup(index) {
                        self.slots[source_slot] = Some(target.clone());
                        self.set_by_player(index, None);
                        self.on_take(index, &target, player);
                    }
                }
                (Some(mut source), None) => {
                    if self.may_place(index, &source) {
                        let max = self.slot_max_for(index, &source);
                        if source.item_count() > max {
                            let part = source.split(max);
                            self.slots[source_slot] = Some(source);
                            self.set_by_player(index, Some(part));
                        } else {
                            self.slots[source_slot] = None;
                            self.set_by_player(index, Some(source));
                        }
                    }
                }
                (Some(mut source), Some(target)) => {
                    if self.may_pickup(index) && self.may_place(index, &source) {
                        let max = self.slot_max_for(index, &source);
                        if source.item_count() > max {
                            let part = source.split(max);
                            self.slots[source_slot] = Some(source);
                            self.set_by_player(index, Some(part));
                            self.on_take(index, &target, player);
                            let mut target = Some(target);
                            if !self.inventory_add(&mut target, player) {
                                self.drop(target.as_ref());
                            }
                        } else {
                            self.slots[source_slot] = Some(target.clone());
                            self.set_by_player(index, Some(source));
                            self.on_take(index, &target, player);
                        }
                    }
                }
            }
        } else if click == ClickType::Clone
            && player.infinite_materials
            && self.carried.is_none()
            && slot_index >= 0
        {
            if let Some(stack) = &self.slots[slot_index as usize] {
                self.carried = Some(stack.copy_with_count(max_stack_size(stack)));
            }
        } else if click == ClickType::Throw && self.carried.is_none() && slot_index >= 0 {
            let index = slot_index as usize;
            let amount = if button == 0 {
                1
            } else {
                self.slots[index].as_ref().map_or(0, Stack::item_count)
            };
            let mut taken = self.safe_take(index, amount, i32::MAX, player);
            self.drop(taken.as_ref());
            if button == 1 {
                while let Some(stack) = &taken {
                    if !same_item(self.slots[index].as_ref(), stack) {
                        break;
                    }
                    taken = self.safe_take(index, amount, i32::MAX, player);
                    self.drop(taken.as_ref());
                }
            }
        } else if click == ClickType::PickupAll && slot_index >= 0 {
            let index = slot_index as usize;
            let Some(mut carried) = self.carried.take() else {
                return;
            };
            if self.slots[index].is_none() || !self.may_pickup(index) {
                let (start, step) = if button == 0 {
                    (0_i32, 1_i32)
                } else {
                    (SLOT_COUNT as i32 - 1, -1)
                };
                for pass in 0..2 {
                    let mut i = start;
                    while i >= 0
                        && i < SLOT_COUNT as i32
                        && carried.item_count() < max_stack_size(&carried)
                    {
                        let target = i as usize;
                        if self.slots[target].is_some()
                            && self.can_item_quick_replace(target, &carried, true)
                            && self.may_pickup(target)
                            && target != RESULT
                        {
                            let stack = self.slots[target].as_ref().expect("checked above");
                            if pass != 0 || stack.item_count() != max_stack_size(stack) {
                                let count = stack.item_count();
                                let room = max_stack_size(&carried) - carried.item_count();
                                if let Some(removed) = self.safe_take(target, count, room, player) {
                                    carried.item_count += removed.item_count();
                                }
                            }
                        }
                        i += step;
                    }
                }
            }
            self.carried = Some(carried);
        }
    }

    fn reset_quick_craft(&mut self) {
        self.quickcraft_status = 0;
        self.quickcraft_slots.clear();
    }

    /// `LivingEntity.drop` on the client: an arm swing for a real stack.
    fn drop(&mut self, stack: Option<&Stack>) {
        if stack.is_some() {
            self.swings += 1;
        }
    }

    /// `Slot.getMaxStackSize(stack)`.
    pub fn slot_max_for(&self, index: usize, stack: &Stack) -> i32 {
        let slot_max = if armour_slot(index).is_some() { 1 } else { 99 };
        slot_max.min(max_stack_size(stack))
    }

    /// `Slot.mayPlace`.
    pub fn may_place(&self, index: usize, stack: &Stack) -> bool {
        if index == RESULT {
            return false;
        }
        match armour_slot(index) {
            Some(slot) => stacking(stack)
                .and_then(|item| item.equip)
                .is_some_and(|(equip, player)| equip == slot && player),
            None => true,
        }
    }

    /// `Slot.mayPickup`: every slot, since armour locks are not read.
    fn may_pickup(&self, _index: usize) -> bool {
        true
    }

    /// `Slot.allowModification`.
    fn allow_modification(&self, index: usize) -> bool {
        self.may_pickup(index)
            && self.slots[index]
                .as_ref()
                .is_none_or(|stack| self.may_place(index, stack))
    }

    /// `AbstractContainerMenu.canItemQuickReplace`.
    pub fn can_item_quick_replace(&self, index: usize, stack: &Stack, ignore_size: bool) -> bool {
        match &self.slots[index] {
            Some(slot) if same_item_same_components(stack, slot) => {
                slot.item_count() + if ignore_size { 0 } else { stack.item_count() }
                    <= max_stack_size(stack)
            }
            Some(_) => false,
            None => true,
        }
    }

    fn set_by_player(&mut self, index: usize, stack: Option<Stack>) {
        self.slots[index] = stack;
        settle(&mut self.slots[index]);
    }

    /// `Slot.remove`: the result slot gives its whole stack.
    fn remove(&mut self, index: usize, amount: i32) -> Option<Stack> {
        if index == RESULT {
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
    fn try_remove(&mut self, index: usize, amount: i32, max: i32) -> Option<Stack> {
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
    fn safe_take(&mut self, index: usize, amount: i32, max: i32, player: Player) -> Option<Stack> {
        let taken = self.try_remove(index, amount, max)?;
        self.on_take(index, &taken, player);
        Some(taken)
    }

    /// `Slot.safeInsert`: what is left of the stack.
    fn safe_insert(&mut self, index: usize, mut input: Stack, amount: i32) -> Option<Stack> {
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
            Some(slot) if same_item_same_components(slot, &input) => {
                input.item_count -= moved;
                slot.item_count += moved;
            }
            Some(_) => {}
        }
        Some(input).filter(|input| input.item_count() > 0)
    }

    /// `Slot.onTake`: the result slot uses up the crafting grid.
    fn on_take(&mut self, index: usize, _taken: &Stack, player: Player) {
        if index != RESULT {
            return;
        }
        // `ResultSlot.onTake` with `CraftingRecipe.defaultCraftingReminder`,
        // row by row through the grid.
        for grid in 1..5 {
            let Some(stack) = self.slots[grid].as_ref() else {
                continue;
            };
            let remainder = stacking(stack)
                .and_then(|item| item.crafting_remainder)
                .and_then(|(name, count)| crate::item::id_of(name).map(|id| Stack::new(count, id)));
            let _used = self.remove(grid, 1);
            let Some(mut remainder) = remainder else {
                continue;
            };
            match self.slots[grid].as_ref() {
                None => self.slots[grid] = Some(remainder),
                Some(present) if same_item_same_components(present, &remainder) => {
                    remainder.item_count += present.item_count();
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

    /// `InventoryMenu.quickMoveStack`.
    fn quick_move_stack(&mut self, index: usize, player: Player) -> Option<Stack> {
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
        if index == RESULT {
            self.drop(self.slots[index].clone().as_ref());
        }
        Some(clicked)
    }

    /// `AbstractContainerMenu.moveItemStackTo`.
    fn move_item_stack_to(
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
                    Some(target) if same_item_same_components(stack, target) => {
                        self.slot_max_for(dest, target)
                    }
                    _ => continue,
                };
                let target = self.slots[dest].as_mut().expect("checked above");
                let total = target.item_count() + stack.item_count();
                if total <= max {
                    stack.item_count = 0;
                    target.item_count = total;
                    changed = true;
                } else if target.item_count() < max {
                    stack.item_count -= max - target.item_count();
                    target.item_count = max;
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
    fn inventory_add(&mut self, stack: &mut Option<Stack>, player: Player) -> bool {
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
                present.item_count += add;
                item.item_count -= add;
            }
            if item.item_count() <= 0 || item.item_count() >= last {
                let added = item.item_count() < last;
                if !added && player.infinite_materials {
                    item.item_count = 0;
                    settle(stack);
                    return true;
                }
                let any = item.item_count() < last;
                settle(stack);
                return any;
            }
        }
    }

    fn slot_with_remaining_space(&self, stack: &Stack, player: Player) -> Option<usize> {
        let has_room = |slot: usize| {
            self.slots[slot].as_ref().is_some_and(|present| {
                same_item_same_components(present, stack)
                    && is_stackable(present)
                    && present.item_count() < 99.min(max_stack_size(present))
            })
        };
        std::iter::once(inventory_to_menu(player.selected))
            .chain(std::iter::once(OFF_HAND))
            .chain((0..36).map(inventory_to_menu))
            .find(|&slot| has_room(slot))
    }

    fn free_slot(&self) -> Option<usize> {
        (0..36)
            .map(inventory_to_menu)
            .find(|&slot| self.slots[slot].is_none())
    }
}

/// An `Inventory` index as the menu's slot.
fn inventory_to_menu(index: usize) -> usize {
    match index {
        0..9 => 36 + index,
        40 => OFF_HAND,
        36..40 => 8 - (index - 36),
        _ => index,
    }
}

/// `HashedStack` as its codec writes it. A component patch is written as
/// none, since the component hashes are not ported.
fn write_hashed_stack(out: &mut Vec<u8>, stack: Option<&Stack>) {
    let Some(stack) = stack else {
        out.push(0);
        return;
    };
    out.push(1);
    crate::wire::write_varint(stack.item_registry_id(), out);
    crate::wire::write_varint(stack.item_count(), out);
    crate::wire::write_varint(0, out);
    crate::wire::write_varint(0, out);
}

/// The order a fastutil `Int2ObjectOpenHashMap` built with its defaults
/// iterates keys put in the given order: the zero key first, then the
/// table from its end.
pub fn fastutil_int_order(keys: &[i32]) -> Vec<i32> {
    fn mix(x: i32) -> i32 {
        let h = x.wrapping_mul(0x9E37_79B9_u32 as i32);
        h ^ ((h as u32) >> 16) as i32
    }
    fn array_size(expected: usize) -> usize {
        ((expected as f64 / 0.75).ceil() as usize)
            .next_power_of_two()
            .max(2)
    }
    fn insert(table: &mut [i32], key: i32) {
        let mask = table.len() - 1;
        let mut pos = mix(key) as usize & mask;
        while table[pos] != 0 {
            pos = (pos + 1) & mask;
        }
        table[pos] = key;
    }
    let mut table = vec![0; array_size(16)];
    let mut has_zero = false;
    let mut size = 0;
    for &key in keys {
        if key == 0 {
            has_zero = true;
        } else {
            insert(&mut table, key);
        }
        let max_fill = ((table.len() as f64 * 0.75).ceil() as usize).min(table.len() - 1);
        let before = size;
        size += 1;
        if before >= max_fill {
            // `rehash`: from the old table's end down into the new one.
            let old = std::mem::replace(&mut table, vec![0; array_size(size + 1)]);
            for &key in old.iter().rev() {
                if key != 0 {
                    insert(&mut table, key);
                }
            }
        }
    }
    let mut order = Vec::with_capacity(keys.len());
    if has_zero {
        order.push(0);
    }
    order.extend(table.iter().rev().copied().filter(|&key| key != 0));
    order
}
