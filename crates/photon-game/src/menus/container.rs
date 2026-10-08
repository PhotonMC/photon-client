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
//! undamaged) and enchantments that lock armour. Component hashes are
//! ported for the components [`hash`](photon_world::item_components::hash)
//! names.

use photon_data::item::classes::{ItemClass, ItemMethod};
use photon_data::item::registry::{ITEMS, Item};
use photon_world::ClientItemStackState as Stack;

mod transform;

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
    pub item_tags: std::sync::Arc<photon_world::block::tags::ItemTags>,
    /// The banner patterns the loom offers, by registry id.
    pub patterns: Vec<i32>,
    /// How many layers the banner in the loom already has.
    pub banner_layers: usize,
    /// What the stonecutter offers for its input: the item each entry shows,
    /// or -1 for one that names none.
    pub options: Vec<i32>,
    /// The player's experience level, which the anvil asks for.
    pub xp_level: i32,
    /// The enchantment registry's entries by network id, which the click
    /// packet's component hashes name.
    pub enchantments: std::sync::Arc<Vec<String>>,
    /// Whether the last click was made with infinite materials (creative).
    pub(super) infinite_materials: bool,
    /// The item in the stonecutter's input the last time its list was made.
    pub(super) last_input: Option<i32>,
    /// The menu's `ContainerData` (a furnace's burn and cook progress).
    pub data: Vec<i16>,
    /// What the merchant of a trading menu offers, and the offer the payments are now matched to.
    pub offers: std::sync::Arc<Vec<crate::menus::merchant::MerchantOffer>>,
    /// `MerchantContainer.selectionHint`: the offer the player chose, tried first.
    pub selection_hint: usize,
    /// The merchant's level and experience, and what its screen shows of them.
    pub trader: crate::menus::merchant::Trader,
    pub(super) active_offer: Option<usize>,
    /// `MerchantContainer.futureXp`: what the trade would give the merchant.
    pub future_xp: i32,
    pub(super) quickcraft_status: i32,
    pub(super) quickcraft_type: i32,
    pub(super) quickcraft_slots: Vec<usize>,
    /// Arm swings the click caused: `LivingEntity.drop` swings on the
    /// client instead of dropping.
    pub(super) swings: usize,
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
            enchantments: std::sync::Arc::default(),
            infinite_materials: false,
            last_input: None,
            data: Vec::new(),
            offers: std::sync::Arc::default(),
            selection_hint: 0,
            trader: crate::menus::merchant::Trader::default(),
            active_offer: None,
            future_xp: 0,
            quickcraft_status: 0,
            quickcraft_type: 0,
            quickcraft_slots: Vec::new(),
            swings: 0,
        }
    }
}

mod clicks;
mod kinds;
pub use kinds::*;
mod quick_move;
mod slots;
mod stacks;
pub use stacks::*;
