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

pub(super) const OFF_HAND: usize = 45;

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
