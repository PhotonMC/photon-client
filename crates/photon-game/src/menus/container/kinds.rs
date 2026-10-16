use super::Mount;

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
    /// `HorseInventoryMenu` or `NautilusInventoryMenu`: the saddle, the body armour and the
    /// mount's chest.
    Mount(Mount),
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

/// The menu each `MenuType` opens, by its registry name.
const MENUS: [(&str, MenuKind); 25] = [
    ("minecraft:generic_9x1", MenuKind::Chest(1)),
    ("minecraft:generic_9x2", MenuKind::Chest(2)),
    ("minecraft:generic_9x3", MenuKind::Chest(3)),
    ("minecraft:generic_9x4", MenuKind::Chest(4)),
    ("minecraft:generic_9x5", MenuKind::Chest(5)),
    ("minecraft:generic_9x6", MenuKind::Chest(6)),
    ("minecraft:generic_3x3", MenuKind::Dispenser),
    ("minecraft:crafter_3x3", MenuKind::Crafter),
    ("minecraft:anvil", MenuKind::Anvil),
    ("minecraft:beacon", MenuKind::Beacon),
    (
        "minecraft:blast_furnace",
        MenuKind::Furnace(Smelter::BlastFurnace),
    ),
    ("minecraft:brewing_stand", MenuKind::Brewing),
    ("minecraft:crafting", MenuKind::Crafting),
    ("minecraft:enchantment", MenuKind::Enchantment),
    ("minecraft:furnace", MenuKind::Furnace(Smelter::Furnace)),
    ("minecraft:grindstone", MenuKind::Grindstone),
    ("minecraft:hopper", MenuKind::Hopper),
    ("minecraft:lectern", MenuKind::Lectern),
    ("minecraft:loom", MenuKind::Loom),
    ("minecraft:merchant", MenuKind::Merchant),
    ("minecraft:shulker_box", MenuKind::ShulkerBox),
    ("minecraft:smithing", MenuKind::Smithing),
    ("minecraft:smoker", MenuKind::Furnace(Smelter::Smoker)),
    ("minecraft:cartography_table", MenuKind::Cartography),
    ("minecraft:stonecutter", MenuKind::Stonecutter),
];

impl MenuKind {
    /// The menu a `MenuType` registry id opens, for those modelled.
    pub fn from_menu_type(id: i32) -> Option<Self> {
        let name = photon_data::menu_type::name(id)?;
        MENUS
            .iter()
            .find(|(menu, _)| *menu == name)
            .map(|(_, kind)| *kind)
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
            Self::Mount(mount) => 2 + usize::from(mount.columns) * 3,
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
