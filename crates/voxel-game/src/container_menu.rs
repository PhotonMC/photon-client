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
    /// `BrewingStandMenu`: three bottles, the ingredient and the fuel.
    Brewing,
    /// `BeaconMenu`: the payment slot.
    Beacon,
    /// `CrafterMenu`: the three by three grid, whose slots can be switched off, and, after the
    /// player's own 36, the result it shows and gives nothing of.
    Crafter,
    /// `LecternMenu`: the book, and no inventory under it.
    Lectern,
    /// `MerchantMenu`: two payments and what they buy.
    Merchant,
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
            7 => Self::Crafter,
            9 => Self::Beacon,
            11 => Self::Brewing,
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
            17 => Self::Lectern,
            19 => Self::Merchant,
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
            Self::Brewing => 5,
            Self::Beacon => 1,
            Self::Crafter => 9,
            Self::Lectern => 1,
            Self::Merchant => 3,
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
            Self::Crafter => Some(45),
            Self::Merchant => Some(2),
            _ => None,
        }
    }

    /// Every slot of the menu.
    pub fn slot_count(self) -> usize {
        match self {
            Self::Inventory => SLOT_COUNT,
            Self::Crafter => 46,
            Self::Lectern => 1,
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
    /// What the merchant of a trading menu offers, and the offer the payments are now matched to.
    pub offers: std::sync::Arc<Vec<crate::merchant::MerchantOffer>>,
    /// `MerchantContainer.selectionHint`: the offer the player chose, tried first.
    pub selection_hint: usize,
    /// The merchant's level and experience, and what its screen shows of them.
    pub trader: crate::merchant::Trader,
    active_offer: Option<usize>,
    /// `MerchantContainer.futureXp`: what the trade would give the merchant.
    pub future_xp: i32,
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
            offers: std::sync::Arc::default(),
            selection_hint: 0,
            trader: crate::merchant::Trader::default(),
            active_offer: None,
            future_xp: 0,
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
        assert_eq!(MenuKind::from_menu_type(9), Some(MenuKind::Beacon));
        assert_eq!(MenuKind::from_menu_type(17), Some(MenuKind::Lectern));
        assert_eq!(MenuKind::Lectern.slot_count(), 1);
        assert_eq!(MenuKind::from_menu_type(19), Some(MenuKind::Merchant));
        // The mounts' menus are not modelled yet (they are not in the registry of menus).
        assert_eq!(MenuKind::from_menu_type(40), None);
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

    #[test]
    fn a_grindstone_refuses_a_tool_that_cannot_break() {
        let menu = ContainerMenu::for_menu(MenuKind::Grindstone, 4, vec![None; 39], None, 0);
        let plain = stack("minecraft:iron_pickaxe", 1);
        let patch = crate::item_components::Patch {
            added: vec![(voxel_data::item::component_types::UNBREAKABLE, Vec::new())],
            removed: Vec::new(),
        };
        let unbreakable = Stack::from_wire(
            1,
            plain.item_registry_id(),
            vec![voxel_data::item::component_types::UNBREAKABLE],
            Vec::new(),
        )
        .with_patch_bytes(patch.trusted());
        assert!(menu.may_place(0, &plain));
        assert!(!menu.may_place(0, &unbreakable));
    }
}

mod container_menu_click;
mod container_menu_move_ops;
mod container_menu_quick;
