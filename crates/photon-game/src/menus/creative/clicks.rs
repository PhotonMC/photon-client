use super::*;

impl CreativeMenu {
    /// `selectTab`: the tab's stacks, scrolled to the top.
    pub fn select_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.items.clear();
        match CREATIVE_TABS[tab].kind {
            TabKind::Hotbar => {
                // Every saved hotbar is empty: a paper in the diagonal.
                let paper = crate::item::id_of("minecraft:paper").expect("paper is an item");
                for hotbar in 0..COLUMNS {
                    for slot in 0..COLUMNS {
                        self.items.push((slot == hotbar).then(|| Shown {
                            stack: Stack::new(1, paper),
                            locked: true,
                        }));
                    }
                }
            }
            TabKind::Category | TabKind::Search => {
                for entry in CREATIVE_TABS[tab].items {
                    self.items.push(Some(Shown {
                        stack: table_stack(entry, &self.registries),
                        locked: false,
                    }));
                }
            }
            TabKind::Inventory => {}
        }
        self.scroll = 0.0;
        self.scroll_to(0.0);
    }

    /// `Slot.mayPickup` for what `slotClicked` asks it of.
    pub(super) fn may_pickup(&self, target: Target) -> bool {
        match target {
            // `CustomCreativeSlot`: an empty slot may be, a full one unless
            // it is locked.
            Target::Window(slot) => self.window[slot].as_ref().is_none_or(|shown| !shown.locked),
            _ => true,
        }
    }

    /// `CreativeModeInventoryScreen.slotClicked`.
    ///
    /// `can_drop` is whether `canDropItems` allows a drop.
    pub fn slot_clicked(
        &mut self,
        click: Click,
        menu: &mut ContainerMenu,
        player: Player,
        can_drop: bool,
    ) -> Vec<Effect> {
        let Click {
            target,
            slot_id,
            button,
            input,
            outside: clicked_outside,
        } = click;
        let quick_key = input == ClickType::QuickMove;
        let input = if slot_id == -999 && input == ClickType::Pickup {
            ClickType::Throw
        } else {
            input
        };
        let mut effects = Vec::new();
        if input == ClickType::Throw && !can_drop {
            return effects;
        }
        let inventory_tab = self.kind() == TabKind::Inventory;
        if target.is_none() && !inventory_tab && input != ClickType::QuickCraft {
            if let Some(carried) = menu.carried.clone()
                && clicked_outside
            {
                if !can_drop {
                    return effects;
                }
                if button == 0 {
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(carried));
                    menu.carried = None;
                }
                if button == 1
                    && let Some(carried) = menu.carried.as_mut()
                {
                    let removed = carried.split(1);
                    settle(&mut menu.carried);
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(removed));
                }
            }
            return effects;
        }
        if let Some(target) = target
            && !self.may_pickup(target)
        {
            return effects;
        }
        if target == Some(Target::Destroy) && quick_key {
            for index in 0..menu.slots.len() {
                menu.slots[index] = None;
                effects.push(Effect::SetSlot(index as i16, None));
            }
        } else if inventory_tab {
            self.inventory_tab_click(target, (slot_id, button, input), menu, player, &mut effects);
        } else if input != ClickType::QuickCraft
            && let Some(Target::Window(slot)) = target
        {
            self.window_click(slot, button, input, quick_key, menu, &mut effects);
        } else {
            self.picker_click(target, (slot_id, button, input), menu, player, &mut effects);
        }
        effects
    }

    /// The inventory tab: the player's own menu, seen through other places.
    pub(super) fn inventory_tab_click(
        &mut self,
        target: Option<Target>,
        (slot_id, button, input): (i32, i32, ClickType),
        menu: &mut ContainerMenu,
        player: Player,
        effects: &mut Vec<Effect>,
    ) {
        if target == Some(Target::Destroy) {
            menu.carried = None;
        } else if input == ClickType::Throw
            && let Some(Target::Inventory(index)) = target
            && menu.slots[index].is_some()
        {
            let amount = if button == 0 {
                1
            } else {
                menu.slots[index].as_ref().map_or(1, max_stack_size)
            };
            if let Some(dropped) = menu.take_from_slot(index, amount) {
                effects.push(Effect::Swing);
                effects.push(Effect::Drop(dropped));
            }
            effects.push(Effect::SetSlot(index as i16, menu.slots[index].clone()));
        } else if input == ClickType::Throw && slot_id == -999 && menu.carried.is_some() {
            let carried = menu.carried.take().expect("checked above");
            effects.push(Effect::Swing);
            effects.push(Effect::Drop(carried));
        } else {
            let index = match target {
                Some(Target::Inventory(index)) => index as i32,
                _ => slot_id,
            };
            let before = menu.slots.clone();
            let swings = menu.click_local(index, button, input, player);
            effects.extend(std::iter::repeat_n(Effect::Swing, swings));
            broadcast_changes(&before, &menu.slots, effects);
        }
    }

    /// A click on a window slot: stacks are made, not moved.
    pub(super) fn window_click(
        &mut self,
        slot: usize,
        button: i32,
        input: ClickType,
        quick_key: bool,
        menu: &mut ContainerMenu,
        effects: &mut Vec<Effect>,
    ) {
        let clicked = self.window_stack(slot).cloned();
        match input {
            ClickType::Swap => {
                if let Some(clicked) = clicked {
                    // `Inventory.setItem`: the hotbar, or the off hand.
                    let menu_slot = match button {
                        0..=8 => Some(36 + button as usize),
                        40 => Some(45),
                        _ => None,
                    };
                    if let Some(menu_slot) = menu_slot {
                        let before = menu.slots.clone();
                        menu.slots[menu_slot] =
                            Some(clicked.copy_with_count(max_stack_size(&clicked)));
                        broadcast_changes(&before, &menu.slots, effects);
                    }
                }
            }
            ClickType::Clone => {
                if menu.carried.is_none()
                    && let Some(clicked) = clicked
                {
                    menu.carried = Some(clicked.copy_with_count(max_stack_size(&clicked)));
                }
            }
            ClickType::Throw => {
                if let Some(clicked) = clicked {
                    let count = if button == 0 {
                        1
                    } else {
                        max_stack_size(&clicked)
                    };
                    effects.push(Effect::Swing);
                    effects.push(Effect::Drop(clicked.copy_with_count(count)));
                }
            }
            _ => {
                let carried = menu.carried.clone();
                match (carried, clicked) {
                    (Some(mut carried), Some(clicked))
                        if carried.is_same_item_same_components(&clicked) =>
                    {
                        if button == 0 {
                            if quick_key {
                                carried.set_count(max_stack_size(&carried));
                            } else if carried.item_count() < max_stack_size(&carried) {
                                carried.grow(1);
                            }
                        } else {
                            carried.shrink(1);
                        }
                        menu.carried = Some(carried);
                        settle(&mut menu.carried);
                    }
                    (None, Some(clicked)) => {
                        let count = if quick_key {
                            max_stack_size(&clicked)
                        } else {
                            clicked.item_count()
                        };
                        menu.carried = Some(clicked.copy_with_count(count));
                    }
                    _ if button == 0 => menu.carried = None,
                    (Some(mut carried), _) => {
                        carried.shrink(1);
                        menu.carried = Some(carried);
                        settle(&mut menu.carried);
                    }
                    _ => {}
                }
            }
        }
    }

    /// The picker's own slots: the hotbar, and a drag over it. The click runs
    /// in a menu that holds only the hotbar, since the window's slots take no
    /// part in it.
    pub(super) fn picker_click(
        &mut self,
        target: Option<Target>,
        (slot_id, button, input): (i32, i32, ClickType),
        menu: &mut ContainerMenu,
        player: Player,
        effects: &mut Vec<Effect>,
    ) {
        let hotbar = match target {
            Some(Target::Hotbar(slot)) => Some(slot),
            Some(_) => return,
            None => None,
        };
        let old = hotbar.and_then(|slot| menu.slots[36 + slot].clone());
        let before = menu.slots.clone();
        let mut slots = vec![None; menu.slots.len()];
        slots[36..45].clone_from_slice(&menu.slots[36..45]);
        self.picker.load(slots, menu.carried.clone(), menu.state_id);
        let index = hotbar.map_or(slot_id, |slot| 36 + slot as i32);
        let swings = if input == ClickType::QuickMove && (button == 0 || button == 1) {
            // `quickMoveStack` of the picker throws the stack away.
            if let Some(slot) = hotbar {
                self.picker.slots[36 + slot] = None;
            }
            0
        } else {
            self.picker.click_local(index, button, input, player)
        };
        menu.slots[36..45].clone_from_slice(&self.picker.slots[36..45]);
        menu.carried = self.picker.carried.clone();
        effects.extend(std::iter::repeat_n(Effect::Swing, swings));
        if button & 3 == 2 {
            // `getQuickcraftHeader(buttonNum) == 2`, whatever the click was.
            for slot in 0..COLUMNS {
                effects.push(Effect::SetSlot(
                    (36 + slot) as i16,
                    menu.slots[36 + slot].clone(),
                ));
            }
        } else if let Some(slot) = hotbar {
            if input == ClickType::Throw
                && let Some(old) = old
                && menu.carried.is_some()
            {
                let count = if button == 0 { 1 } else { old.item_count() };
                let dropped = old.copy_with_count(count);
                // The old stack is shrunk where it lies.
                if let Some(present) = menu.slots[36 + slot].as_mut() {
                    present.shrink(count);
                    settle(&mut menu.slots[36 + slot]);
                }
                effects.push(Effect::Swing);
                effects.push(Effect::Drop(dropped));
            }
            broadcast_changes(&before, &menu.slots, effects);
        }
    }
}

/// What a click landed on, by what the reference calls its slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A slot of the window, by its place in it.
    Window(usize),
    /// A hotbar slot of the picker's own layout, 0 to 8.
    Hotbar(usize),
    /// The inventory tab's view of a slot of the player's menu.
    Inventory(usize),
    /// The inventory tab's bin.
    Destroy,
}

/// A click as `slotClicked` gets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Click {
    pub target: Option<Target>,
    /// The slot's index, or -999 outside the screen.
    pub slot_id: i32,
    pub button: i32,
    pub input: ClickType,
    /// What `hasClickedOutside` last found.
    pub outside: bool,
}

/// What a click made the client do, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// `Player.drop` on the client: the arm swings, and the swing is sent.
    Swing,
    /// `handleCreativeModeItemAdd`: the slot's new stack.
    SetSlot(i16, Option<Stack>),
    /// `handleCreativeModeItemDrop`: a stack thrown away.
    Drop(Stack),
}

/// `AbstractContainerMenu.broadcastChanges`: the creative listener sends
/// each slot that no longer matches what it held.
pub(super) fn broadcast_changes(
    before: &[Option<Stack>],
    after: &[Option<Stack>],
    effects: &mut Vec<Effect>,
) {
    for (index, (was, now)) in before.iter().zip(after).enumerate() {
        let same = match (was, now) {
            (None, None) => true,
            (Some(a), Some(b)) => a.matches(b),
            _ => false,
        };
        if !same {
            effects.push(Effect::SetSlot(index as i16, now.clone()));
        }
    }
}
