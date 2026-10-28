//! `ClientWorld` container layer split out of the former single-file module.

use std::collections::BTreeMap;

pub use photon_data::item_stack::ClientItemStackState;

/// Raw state for the currently open non-player container menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientContainerState {
    pub(crate) container_id: i32,
    pub(crate) menu_type_id: i32,
    pub(crate) title_nbt: Vec<u8>,
    pub(crate) slot_count: usize,
    pub(crate) top_slot_count: Option<usize>,
    pub(crate) state_id: i32,
    pub(crate) slots: BTreeMap<i16, Option<ClientItemStackState>>,
    pub(crate) carried_item: Option<ClientItemStackState>,
    /// The menu's `ContainerData`, by index.
    pub(crate) data: BTreeMap<i16, i16>,
}

impl ClientContainerState {
    pub(crate) fn new(container_id: i32, menu_type_id: i32, title_nbt: &[u8]) -> Self {
        Self {
            container_id,
            menu_type_id,
            title_nbt: title_nbt.to_vec(),
            slot_count: 0,
            top_slot_count: menu_top_slot_count(menu_type_id),
            state_id: 0,
            slots: BTreeMap::new(),
            carried_item: None,
            data: BTreeMap::new(),
        }
    }

    /// A `ContainerData` value; zero until the server has sent it.
    pub fn data(&self, index: i16) -> i16 {
        self.data.get(&index).copied().unwrap_or(0)
    }

    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    pub fn menu_type_id(&self) -> i32 {
        self.menu_type_id
    }

    pub fn title_nbt(&self) -> &[u8] {
        &self.title_nbt
    }

    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub fn top_slot_count(&self) -> Option<usize> {
        self.top_slot_count
    }

    pub fn state_id(&self) -> i32 {
        self.state_id
    }

    pub fn slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }
}

pub(crate) fn player_container_menu_slot(slot: i32) -> Option<i16> {
    match slot {
        0..=8 => Some((36 + slot) as i16),
        9..=35 => Some(slot as i16),
        _ => None,
    }
}

pub(crate) const VANILLA_MENU_TOP_SLOT_COUNTS: [usize; 25] = [
    9, 18, 27, 36, 45, 54, 9, 10, 3, 1, 3, 5, 10, 2, 3, 3, 5, 1, 4, 3, 27, 4, 3, 3, 2,
];

pub(crate) fn menu_top_slot_count(menu_type_id: i32) -> Option<usize> {
    usize::try_from(menu_type_id)
        .ok()
        .and_then(|id| VANILLA_MENU_TOP_SLOT_COUNTS.get(id).copied())
}
