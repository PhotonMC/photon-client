use super::*;

impl CreativeMenu {
    /// A menu on a tab.
    pub fn new(tab: usize, registries: std::sync::Arc<Registries>) -> Self {
        let mut menu = Self {
            tab,
            items: Vec::new(),
            window: vec![None; WINDOW],
            picker: ContainerMenu::default(),
            scroll: 0.0,
            registries,
        };
        menu.select_tab(tab);
        menu
    }

    pub fn tab(&self) -> usize {
        self.tab
    }

    pub fn kind(&self) -> TabKind {
        CREATIVE_TABS[self.tab].kind
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    pub fn set_scroll(&mut self, scroll: f32) {
        self.scroll = scroll;
        self.scroll_to(scroll);
    }

    /// The stack in a window slot.
    pub fn window_stack(&self, slot: usize) -> Option<&Stack> {
        self.window[slot].as_ref().map(|shown| &shown.stack)
    }

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

    /// `calculateRowCount`: the rows past the five the window shows.
    pub(super) fn row_count(&self) -> i32 {
        (self.items.len() as i32 + 8) / 9 - 5
    }

    /// `getRowIndexForScroll`.
    pub fn row_index_for_scroll(&self, scroll: f32) -> i32 {
        ((scroll * self.row_count() as f32 + 0.5) as i32).max(0)
    }

    /// `getScrollForRowIndex`.
    pub fn scroll_for_row_index(&self, row: i32) -> f32 {
        (row as f32 / self.row_count() as f32).clamp(0.0, 1.0)
    }

    /// `subtractInputFromScroll`: the wheel's turn as a step of the scroll.
    pub fn subtract_input_from_scroll(&self, scroll: f32, input: f64) -> f32 {
        (scroll - (input / f64::from(self.row_count())) as f32).clamp(0.0, 1.0)
    }

    /// `scrollTo`: the window shows the rows from the scroll's row.
    pub fn scroll_to(&mut self, scroll: f32) {
        let row = self.row_index_for_scroll(scroll);
        for y in 0..ROWS {
            for x in 0..COLUMNS {
                let index = x as i32 + (y as i32 + row) * 9;
                self.window[x + y * COLUMNS] = usize::try_from(index)
                    .ok()
                    .and_then(|index| self.items.get(index))
                    .cloned()
                    .flatten();
            }
        }
    }

    /// The search tab's stacks, by place, shown in place of the tab's own, as
    /// a search finds them, scrolled to the top.
    pub fn show_items(&mut self, places: &[u16]) {
        let items = search_tab().items;
        self.items = places
            .iter()
            .map(|&place| {
                Some(Shown {
                    stack: table_stack(&items[usize::from(place)], &self.registries),
                    locked: false,
                })
            })
            .collect();
        self.scroll = 0.0;
        self.scroll_to(0.0);
    }

    /// `canScroll` of the menu: more stacks than the window holds.
    pub fn can_scroll(&self) -> bool {
        self.items.len() > WINDOW
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
            self.inventory_tab_click(target, slot_id, button, input, menu, player, &mut effects);
        } else if input != ClickType::QuickCraft
            && let Some(Target::Window(slot)) = target
        {
            self.window_click(slot, button, input, quick_key, menu, &mut effects);
        } else {
            self.picker_click(target, slot_id, button, input, menu, player, &mut effects);
        }
        effects
    }

    /// The inventory tab: the player's own menu, seen through other places.
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    pub(super) fn inventory_tab_click(
        &mut self,
        target: Option<Target>,
        slot_id: i32,
        button: i32,
        input: ClickType,
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
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    pub(super) fn picker_click(
        &mut self,
        target: Option<Target>,
        slot_id: i32,
        button: i32,
        input: ClickType,
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
