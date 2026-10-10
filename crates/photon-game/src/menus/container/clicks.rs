use super::*;

impl ContainerMenu {
    /// The player menu's slots and carried stack from the client's copy.
    pub fn with_contents(slots: Vec<Option<Stack>>, carried: Option<Stack>, state_id: i32) -> Self {
        Self::for_menu(MenuKind::Inventory, 0, slots, carried, state_id)
    }

    /// A menu of `kind` the server opened as `container_id`, with the
    /// client's copy of its slots and carried stack.
    pub fn for_menu(
        kind: MenuKind,
        container_id: i32,
        slots: Vec<Option<Stack>>,
        carried: Option<Stack>,
        state_id: i32,
    ) -> Self {
        let mut menu = Self {
            kind,
            container_id,
            slots,
            carried,
            state_id,
            ..Self::default()
        };
        menu.slots.resize(kind.slot_count(), None);
        menu
    }

    /// New contents from the client's copy; a drag in progress carries on.
    pub fn load(&mut self, slots: Vec<Option<Stack>>, carried: Option<Stack>, state_id: i32) {
        self.slots = slots;
        self.slots.resize(self.kind.slot_count(), None);
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
        self.sounds.clear();
        self.clicked(slot, button, click, player);
        let changed: Vec<(usize, Option<&Stack>)> = (0..self.slots.len())
            .filter(|&index| !match (&before[index], &self.slots[index]) {
                (None, None) => true,
                (Some(a), Some(b)) => a.matches(b),
                _ => false,
            })
            .map(|index| (index, self.slots[index].as_ref()))
            .collect();
        let mut body = Vec::new();
        photon_core::wire::write_varint(self.container_id, &mut body);
        photon_core::wire::write_varint(self.state_id, &mut body);
        body.extend_from_slice(&(slot as i16).to_be_bytes());
        body.push(button as i8 as u8);
        photon_core::wire::write_varint(click as i32, &mut body);
        let keys: Vec<i32> = changed.iter().map(|&(index, _)| index as i32).collect();
        let order = fastutil_int_order(&keys);
        photon_core::wire::write_varint(order.len() as i32, &mut body);
        for key in order {
            body.extend_from_slice(&(key as i16).to_be_bytes());
            write_hashed_stack(
                &mut body,
                self.slots[key as usize].as_ref(),
                &self.enchantments,
            );
        }
        write_hashed_stack(&mut body, self.carried.as_ref(), &self.enchantments);
        (self.swings, body)
    }

    /// `doClick` alone, for a screen that sends no click packet: how many
    /// times the click swung the arm.
    pub fn click_local(
        &mut self,
        slot: i32,
        button: i32,
        click: ClickType,
        player: Player,
    ) -> usize {
        self.swings = 0;
        self.sounds.clear();
        self.clicked(slot, button, click, player);
        self.swings
    }

    /// `Slot.remove`: up to `amount` taken out of a slot.
    pub fn take_from_slot(&mut self, index: usize, amount: i32) -> Option<Stack> {
        self.remove(index, amount)
    }

    /// `AbstractContainerMenu.doClick`, then `slotsChanged` if an input slot
    /// of the menu changed under it.
    pub fn clicked(&mut self, slot_index: i32, button: i32, click: ClickType, player: Player) {
        let inputs = self.input_slots();
        let before: Vec<Option<Stack>> = self.slots[inputs.clone()].to_vec();
        self.do_click(slot_index, button, click, player);
        let changed = self.slots[inputs]
            .iter()
            .zip(&before)
            .any(|(now, was)| match (now, was) {
                (None, None) => false,
                (Some(a), Some(b)) => !a.matches(b),
                _ => true,
            });
        if changed {
            self.inputs_changed();
        }
    }

    /// `AbstractContainerMenu.doClick`.
    pub(super) fn do_click(
        &mut self,
        slot_index: i32,
        button: i32,
        click: ClickType,
        player: Player,
    ) {
        self.infinite_materials = player.infinite_materials;
        // The reference would fail on a slot past the menu's end.
        if slot_index >= self.slots.len() as i32 {
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
                if index >= self.slots.len() {
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
                    source.set_count(remaining);
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
                if self.bundle_click(index, primary, player) {
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
                                if clicked.is_same_item_same_components(&carried) {
                                    let amount = if primary { carried.item_count() } else { 1 };
                                    self.carried = self.safe_insert(index, carried, amount);
                                } else if carried.item_count() <= self.slot_max_for(index, &carried)
                                {
                                    self.carried = Some(clicked);
                                    self.set_by_player(index, Some(carried));
                                } else {
                                    self.carried = Some(carried);
                                }
                            } else if clicked.is_same_item_same_components(&carried) {
                                let room = max_stack_size(&carried) - carried.item_count();
                                if let Some(taken) =
                                    self.try_remove(index, clicked.item_count(), room)
                                {
                                    carried.grow(taken.item_count());
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
            // The off hand is not a slot of a container menu.
            let Some(source_slot) = self.inventory_to_menu(button as usize) else {
                return;
            };
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
                    (self.slots.len() as i32 - 1, -1)
                };
                for pass in 0..2 {
                    let mut i = start;
                    while i >= 0
                        && i < self.slots.len() as i32
                        && carried.item_count() < max_stack_size(&carried)
                    {
                        let target = i as usize;
                        if self.slots[target].is_some()
                            && self.can_item_quick_replace(target, &carried, true)
                            && self.may_pickup(target)
                            && self.can_take_for_pick_all(target)
                        {
                            let stack = self.slots[target].as_ref().expect("checked above");
                            if pass != 0 || stack.item_count() != max_stack_size(stack) {
                                let count = stack.item_count();
                                let room = max_stack_size(&carried) - carried.item_count();
                                if let Some(removed) = self.safe_take(target, count, room, player) {
                                    carried.grow(removed.item_count());
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

    pub(super) fn reset_quick_craft(&mut self) {
        self.quickcraft_status = 0;
        self.quickcraft_slots.clear();
    }

    /// `LivingEntity.drop` on the client: an arm swing for a real stack.
    pub(super) fn drop(&mut self, stack: Option<&Stack>) {
        if stack.is_some() {
            self.swings += 1;
        }
    }

    /// The sounds the last click played, by event name.
    pub fn sounds(&self) -> &[&'static str] {
        &self.sounds
    }

    /// Whether the last click was made with infinite materials.
    pub fn creative(&self) -> bool {
        self.infinite_materials
    }

    /// Whether the player may take what is in the result slot.
    pub fn can_pick_up_result(&self) -> bool {
        self.kind
            .result_slot()
            .is_none_or(|slot| self.may_pickup(slot))
    }

    /// Whether this is the player menu's crafting result slot.
    pub fn is_result(&self, index: usize) -> bool {
        !matches!(self.kind, MenuKind::Furnace(_) | MenuKind::Loom)
            && self.kind.result_slot() == Some(index)
    }
}
