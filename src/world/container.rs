//! `ClientWorld` container layer split out of the former single-file module.



use std::collections::BTreeMap;



/// Raw item-stack state retained by the client inventory copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientItemStackState {
    pub(crate) item_count: i32,
    pub(crate) item_registry_id: i32,
    pub(crate) positive_component_type_ids: Vec<i32>,
    pub(crate) negative_component_type_ids: Vec<i32>,
}

impl ClientItemStackState {
    pub(crate) fn from_wire(
        item_count: i32,
        item_registry_id: i32,
        positive_component_type_ids: Vec<i32>,
        negative_component_type_ids: Vec<i32>,
    ) -> Self {
        Self {
            item_count,
            item_registry_id,
            positive_component_type_ids,
            negative_component_type_ids,
        }
    }

    pub fn item_count(&self) -> i32 {
        self.item_count
    }

    pub fn item_registry_id(&self) -> i32 {
        self.item_registry_id
    }

    pub fn positive_component_type_ids(&self) -> &[i32] {
        &self.positive_component_type_ids
    }

    pub fn negative_component_type_ids(&self) -> &[i32] {
        &self.negative_component_type_ids
    }
}

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
        }
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

pub(crate) fn player_inventory_menu_slot(slot: i32) -> Option<i16> {
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
