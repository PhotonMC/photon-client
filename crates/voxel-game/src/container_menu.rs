//! A menu as the client predicts a click in it: `AbstractContainerMenu.doClick`
//! with the slots of the player's own inventory menu or of a generic
//! container's (chests, dispensers, hoppers, shulker boxes), and the click
//! packet `MultiPlayerGameMode.handleContainerInput` sends.
//!
//! The player menu's slots, by menu index: 0 the crafting result, 1 to 4 the
//! crafting grid, 5 to 8 the armour from head to feet, 9 to 35 the main
//! inventory, 36 to 44 the hotbar and 45 the off hand. A container menu has
//! its own slots first, then the 27 of the main inventory and the 9 of the
//! hotbar; its off hand is not a slot, so a swap with it is not predicted.
//!
//! Stacks are values here where the reference shares objects; each step
//! writes back what the shared object would have become. Not ported: the
//! bundle's click overrides (a click with a bundle carried or clicked is
//! sent without a prediction), damaged items (a damaged stack counts as
//! undamaged), enchantments that lock armour, and component hashes (a
//! stack with a component patch is hashed as if it had none).

use voxel_data::item::classes::{ItemClass, ItemMethod};
use voxel_data::item::registry::{ITEMS, Item};
use voxel_world::ClientItemStackState as Stack;

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

/// The number of slots in the player's inventory menu.
pub const SLOT_COUNT: usize = 46;
const OFF_HAND: usize = 45;

/// Which menu this is, and so how its slots behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    /// `InventoryMenu`.
    Inventory,
    /// `ChestMenu` with this many rows of nine (also the barrel and the
    /// ender chest).
    Chest(usize),
    /// `DispenserMenu`: three by three (also the dropper).
    Dispenser,
    /// `HopperMenu`: five slots.
    Hopper,
    /// `ShulkerBoxMenu`: 27 slots that refuse shulker boxes.
    ShulkerBox,
    /// `CraftingMenu`: the result, then the three by three grid.
    Crafting,
    /// `AbstractFurnaceMenu`: the ingredient, the fuel and the result.
    Furnace(Smelter),
    /// `StonecutterMenu`: the input and the result.
    Stonecutter,
    /// `GrindstoneMenu`: two inputs and the result.
    Grindstone,
    /// `CartographyTableMenu`: the map, the additional item and the result.
    Cartography,
    /// `LoomMenu`: the banner, the dye, the pattern and the result.
    Loom,
    /// `SmithingMenu`: the template, the base, the addition and the result.
    Smithing,
    /// `AnvilMenu`: two inputs and the result.
    Anvil,
    /// `EnchantmentMenu`: the item and the lapis.
    Enchantment,
}

/// Which furnace a [`MenuKind::Furnace`] is: each has its own screen and
/// its own set of inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Smelter {
    Furnace,
    BlastFurnace,
    Smoker,
}

impl Smelter {
    /// The recipe property set of the items it accepts.
    pub fn input_set(self) -> &'static str {
        match self {
            Self::Furnace => "minecraft:furnace_input",
            Self::BlastFurnace => "minecraft:blast_furnace_input",
            Self::Smoker => "minecraft:smoker_input",
        }
    }
}

impl MenuKind {
    /// The menu a `MenuType` registry id opens, for those modelled.
    pub fn from_menu_type(id: i32) -> Option<Self> {
        Some(match id {
            0..=5 => Self::Chest(id as usize + 1),
            6 => Self::Dispenser,
            8 => Self::Anvil,
            10 => Self::Furnace(Smelter::BlastFurnace),
            12 => Self::Crafting,
            13 => Self::Enchantment,
            14 => Self::Furnace(Smelter::Furnace),
            15 => Self::Grindstone,
            18 => Self::Loom,
            21 => Self::Smithing,
            22 => Self::Furnace(Smelter::Smoker),
            23 => Self::Cartography,
            24 => Self::Stonecutter,
            16 => Self::Hopper,
            20 => Self::ShulkerBox,
            _ => return None,
        })
    }

    /// The menu's own slots, before the player's 36.
    pub fn top_slots(self) -> usize {
        match self {
            Self::Inventory => 0,
            Self::Chest(rows) => rows * 9,
            Self::Dispenser => 9,
            Self::Hopper => 5,
            Self::ShulkerBox => 27,
            Self::Crafting => 10,
            Self::Furnace(_) | Self::Grindstone | Self::Cartography | Self::Anvil => 3,
            Self::Stonecutter | Self::Enchantment => 2,
            Self::Loom | Self::Smithing => 4,
        }
    }

    /// The recipe property sets the menu's slots take their items from, in
    /// the order of [`ContainerMenu::accepted`].
    pub fn input_sets(self) -> &'static [&'static str] {
        match self {
            Self::Furnace(smelter) => match smelter {
                Smelter::Furnace => &["minecraft:furnace_input"],
                Smelter::BlastFurnace => &["minecraft:blast_furnace_input"],
                Smelter::Smoker => &["minecraft:smoker_input"],
            },
            Self::Smithing => &[
                "minecraft:smithing_template",
                "minecraft:smithing_base",
                "minecraft:smithing_addition",
            ],
            _ => &[],
        }
    }

    /// The menu's result slot, for the menus that have one.
    pub fn result_slot(self) -> Option<usize> {
        match self {
            Self::Inventory | Self::Crafting => Some(0),
            Self::Furnace(_) => Some(2),
            Self::Stonecutter => Some(1),
            Self::Grindstone | Self::Cartography | Self::Anvil => Some(2),
            Self::Loom | Self::Smithing => Some(3),
            _ => None,
        }
    }

    /// Every slot of the menu.
    pub fn slot_count(self) -> usize {
        match self {
            Self::Inventory => SLOT_COUNT,
            other => other.top_slots() + 36,
        }
    }
}

/// The player's inventory menu: its slots, the carried stack and the drag
/// in progress.
#[derive(Debug, Clone)]
pub struct ContainerMenu {
    pub kind: MenuKind,
    /// The id the server gave the menu; the player's own is 0.
    pub container_id: i32,
    pub slots: Vec<Option<Stack>>,
    pub carried: Option<Stack>,
    pub state_id: i32,
    /// The item sets the menu's slots take, from what the server sent (see
    /// [`MenuKind::input_sets`]).
    pub accepted: Vec<std::sync::Arc<std::collections::BTreeSet<i32>>>,
    /// The item tags the server sent, for the slots that ask for one.
    pub item_tags: std::sync::Arc<voxel_world::block::tags::ItemTags>,
    /// The banner patterns the loom offers, by registry id.
    pub patterns: Vec<i32>,
    /// How many layers the banner in the loom already has.
    pub banner_layers: usize,
    /// What the stonecutter offers for its input: the item each entry shows,
    /// or -1 for one that names none.
    pub options: Vec<i32>,
    /// The player's experience level, which the anvil asks for.
    pub xp_level: i32,
    /// Whether the last click was made with infinite materials (creative).
    infinite_materials: bool,
    /// The item in the stonecutter's input the last time its list was made.
    last_input: Option<i32>,
    /// The menu's `ContainerData` (a furnace's burn and cook progress).
    pub data: Vec<i16>,
    quickcraft_status: i32,
    quickcraft_type: i32,
    quickcraft_slots: Vec<usize>,
    /// Arm swings the click caused: `LivingEntity.drop` swings on the
    /// client instead of dropping.
    swings: usize,
}

impl Default for ContainerMenu {
    fn default() -> Self {
        Self {
            kind: MenuKind::Inventory,
            container_id: 0,
            slots: vec![None; SLOT_COUNT],
            carried: None,
            state_id: 0,
            accepted: Vec::new(),
            item_tags: std::sync::Arc::default(),
            patterns: Vec::new(),
            banner_layers: 0,
            options: Vec::new(),
            xp_level: 0,
            infinite_materials: false,
            last_input: None,
            data: Vec::new(),
            quickcraft_status: 0,
            quickcraft_type: 0,
            quickcraft_slots: Vec::new(),
            swings: 0,
        }
    }
}

fn stacking(stack: &Stack) -> Option<&'static Item> {
    usize::try_from(stack.item_registry_id())
        .ok()
        .and_then(|id| ITEMS.get(id))
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
        .and_then(|item| item.equippable.as_ref())
        .map_or("mainhand", |equip| equip.slot)
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
            item.class.owner_of(ItemMethod::OverrideStackedOnOther) != ItemClass::Item
        } else {
            item.class.owner_of(ItemMethod::OverrideOtherStackedOnMe) != ItemClass::Item
        }
    })
}

/// A stack emptied in place reads as no stack.
pub(crate) fn settle(stack: &mut Option<Stack>) {
    if stack.as_ref().is_some_and(|stack| stack.item_count() <= 0) {
        *stack = None;
    }
}

fn same_item(a: Option<&Stack>, b: &Stack) -> bool {
    a.is_some_and(|a| a.item_registry_id() == b.item_registry_id())
}

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
        voxel_core::wire::write_varint(self.container_id, &mut body);
        voxel_core::wire::write_varint(self.state_id, &mut body);
        body.extend_from_slice(&(slot as i16).to_be_bytes());
        body.push(button as i8 as u8);
        voxel_core::wire::write_varint(click as i32, &mut body);
        let keys: Vec<i32> = changed.iter().map(|&(index, _)| index as i32).collect();
        let order = fastutil_int_order(&keys);
        voxel_core::wire::write_varint(order.len() as i32, &mut body);
        for key in order {
            body.extend_from_slice(&(key as i16).to_be_bytes());
            write_hashed_stack(&mut body, self.slots[key as usize].as_ref());
        }
        write_hashed_stack(&mut body, self.carried.as_ref());
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
    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    fn do_click(&mut self, slot_index: i32, button: i32, click: ClickType, player: Player) {
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

    /// `canTakeItemForPickAll`: the result containers of these menus are
    /// skipped when a double click gathers.
    pub fn can_take_for_pick_all(&self, index: usize) -> bool {
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
    fn grid(&self) -> std::ops::Range<usize> {
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
    fn armour_at(&self, index: usize) -> Option<&'static str> {
        if self.kind == MenuKind::Inventory {
            armour_slot(index)
        } else {
            None
        }
    }

    /// An `Inventory` index as this menu's slot: the hotbar and the main
    /// inventory in both, the armour and the off hand in the player menu.
    fn inventory_to_menu(&self, index: usize) -> Option<usize> {
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
    fn may_pickup(&self, index: usize) -> bool {
        if self.kind == MenuKind::Anvil && index == 2 {
            let cost = i32::from(self.data.first().copied().unwrap_or(0));
            return (self.infinite_materials || self.xp_level >= cost) && cost > 0;
        }
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
            Some(slot) if stack.is_same_item_same_components(slot) => {
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
            Some(slot) if slot.is_same_item_same_components(&input) => {
                input.shrink(moved);
                slot.grow(moved);
            }
            Some(_) => {}
        }
        Some(input).filter(|input| input.item_count() > 0)
    }

    /// `Slot.onTake`: the result slot uses up the crafting grid.
    fn on_take(&mut self, index: usize, _taken: &Stack, player: Player) {
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
    fn quick_move_container(&mut self, index: usize, player: Player) -> Option<Stack> {
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
    fn quick_move_crafting(&mut self, index: usize, player: Player) -> Option<Stack> {
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
    fn is_fuel(&self, stack: &Stack) -> bool {
        stacking(stack).is_some_and(|item| item.burn_time > 0)
    }

    /// `AbstractFurnaceMenu.canSmelt`.
    fn can_smelt(&self, stack: &Stack) -> bool {
        self.accepted
            .first()
            .is_some_and(|inputs| inputs.contains(&stack.item_registry_id()))
    }

    /// `AbstractFurnaceMenu.quickMoveStack`.
    fn quick_move_furnace(&mut self, index: usize, player: Player) -> Option<Stack> {
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

    /// `quickMoveStack` of the menu.
    fn quick_move_stack(&mut self, index: usize, player: Player) -> Option<Stack> {
        match self.kind {
            MenuKind::Furnace(_) => self.quick_move_furnace(index, player),
            MenuKind::Stonecutter
            | MenuKind::Grindstone
            | MenuKind::Cartography
            | MenuKind::Loom
            | MenuKind::Smithing
            | MenuKind::Anvil
            | MenuKind::Enchantment => self.quick_move_transform(index, player),
            MenuKind::Inventory => self.quick_move_inventory(index, player),
            MenuKind::Crafting => self.quick_move_crafting(index, player),
            _ => self.quick_move_container(index, player),
        }
    }

    /// `InventoryMenu.quickMoveStack`.
    fn quick_move_inventory(&mut self, index: usize, player: Player) -> Option<Stack> {
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

    fn slot_with_remaining_space(&self, stack: &Stack, player: Player) -> Option<usize> {
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

    fn free_slot(&self) -> Option<usize> {
        (0..36)
            .filter_map(|index| self.inventory_to_menu(index))
            .find(|&slot| self.slots[slot].is_none())
    }
}

/// `FurnaceFuelSlot.isBucket`.
fn is_bucket(stack: &Stack) -> bool {
    stacking(stack).is_some_and(|item| item.name == "minecraft:bucket")
}

/// `Item.canFitInsideContainerItems`: every item but a shulker box.
fn can_fit_inside_container_items(stack: &Stack) -> bool {
    !stacking(stack)
        .and_then(|item| item.block)
        .and_then(voxel_data::block::registry::block)
        .is_some_and(|block| {
            block
                .class
                .is(voxel_data::block::classes::BlockClass::ShulkerBoxBlock)
        })
}

/// An `Inventory` index as the player menu's slot.
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
    voxel_core::wire::write_varint(stack.item_registry_id(), out);
    voxel_core::wire::write_varint(stack.item_count(), out);
    voxel_core::wire::write_varint(0, out);
    voxel_core::wire::write_varint(0, out);
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

mod transform;

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: Player = Player {
        selected: 0,
        infinite_materials: false,
    };

    fn stack(name: &str, count: i32) -> Stack {
        Stack::new(count, crate::item::id_of(name).expect("an item"))
    }

    fn chest(rows: usize) -> ContainerMenu {
        ContainerMenu::for_menu(MenuKind::Chest(rows), 7, Vec::new(), None, 3)
    }

    #[test]
    fn menu_types_name_their_layouts() {
        assert_eq!(MenuKind::from_menu_type(2), Some(MenuKind::Chest(3)));
        assert_eq!(
            MenuKind::from_menu_type(2).map(MenuKind::slot_count),
            Some(63)
        );
        assert_eq!(MenuKind::from_menu_type(16), Some(MenuKind::Hopper));
        assert_eq!(MenuKind::from_menu_type(9), None);
    }

    #[test]
    fn a_chest_click_carries_the_servers_container_id() {
        let mut menu = chest(3);
        menu.slots[0] = Some(stack("minecraft:stone", 5));
        let (_, body) = menu.handle_click(0, 0, ClickType::Pickup, PLAYER);
        // Container id, then state id.
        assert_eq!(&body[..2], &[7, 3]);
        assert_eq!(menu.carried.as_ref().map(Stack::item_count), Some(5));
    }

    #[test]
    fn shift_clicking_moves_between_the_chest_and_the_player_slots() {
        let mut menu = chest(3);
        menu.slots[0] = Some(stack("minecraft:stone", 5));
        menu.clicked(0, 0, ClickType::QuickMove, PLAYER);
        assert!(menu.slots[0].is_none());
        // Into the hotbar's end first, the way `moveItemStackTo` goes back.
        assert_eq!(menu.slots[62].as_ref().map(Stack::item_count), Some(5));
        menu.clicked(62, 0, ClickType::QuickMove, PLAYER);
        assert_eq!(menu.slots[0].as_ref().map(Stack::item_count), Some(5));
        assert!(menu.slots[62].is_none());
    }

    #[test]
    fn a_shulker_box_does_not_go_inside_one() {
        let mut menu = ContainerMenu::for_menu(MenuKind::ShulkerBox, 1, Vec::new(), None, 0);
        let shulker = stack("minecraft:shulker_box", 1);
        let stone = stack("minecraft:stone", 1);
        assert!(!menu.may_place(0, &shulker));
        assert!(menu.may_place(0, &stone));
        // The player's own slots take it.
        assert!(menu.may_place(27, &shulker));
        menu.slots[27] = Some(shulker);
        menu.clicked(27, 0, ClickType::QuickMove, PLAYER);
        assert!(menu.slots[0].is_none());
    }

    #[test]
    fn a_swap_with_the_hotbar_uses_the_containers_hotbar_slots() {
        let mut menu = chest(1);
        menu.slots[0] = Some(stack("minecraft:stone", 2));
        menu.clicked(0, 3, ClickType::Swap, PLAYER);
        assert!(menu.slots[0].is_none());
        assert_eq!(
            menu.slots[9 + 27 + 3].as_ref().map(Stack::item_count),
            Some(2)
        );
    }
}
