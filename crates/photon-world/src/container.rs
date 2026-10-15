//! `ClientWorld` container layer split out of the former single-file module.

use photon_core::ids::ItemId;
use std::collections::BTreeMap;

/// Raw item-stack state retained by the client inventory copy.
#[derive(Debug, Clone)]
pub struct ClientItemStackState {
    pub(crate) item_count: i32,
    pub(crate) item_registry_id: ItemId,
    pub(crate) positive_component_type_ids: Vec<i32>,
    pub(crate) negative_component_type_ids: Vec<i32>,
    /// The component patch as the wire carried it, for comparing stacks.
    pub(crate) patch_bytes: Vec<u8>,
    /// Which Java `ItemStack` object this is: each decoded stack is a new
    /// one, and changing a stack in place (`shrink`) keeps it. The hand
    /// compares held stacks by object.
    pub(crate) identity: u64,
    /// `BundleContents.selectedItem`: the item of a bundle's contents the
    /// player has scrolled to, or -1. The client's own: the wire does not
    /// carry it, and the stacks compare without it.
    pub(crate) bundle_selected: i32,
}

static NEXT_IDENTITY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Stacks are compared as observed: count, item and which components the
/// patch adds or removes. The patch bytes only serve
/// [`ClientItemStackState::is_same_item_same_components`].
impl PartialEq for ClientItemStackState {
    fn eq(&self, other: &Self) -> bool {
        self.item_count == other.item_count
            && self.item_registry_id == other.item_registry_id
            && self.positive_component_type_ids == other.positive_component_type_ids
            && self.negative_component_type_ids == other.negative_component_type_ids
    }
}

impl Eq for ClientItemStackState {}

impl ClientItemStackState {
    pub fn from_wire(
        item_count: i32,
        item_registry_id: ItemId,
        positive_component_type_ids: Vec<i32>,
        negative_component_type_ids: Vec<i32>,
    ) -> Self {
        Self {
            item_count,
            item_registry_id,
            positive_component_type_ids,
            negative_component_type_ids,
            patch_bytes: Vec::new(),
            identity: NEXT_IDENTITY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            bundle_selected: -1,
        }
    }

    /// The bundle contents' selected item, or -1.
    pub fn bundle_selected(&self) -> i32 {
        self.bundle_selected
    }

    /// The contents set again with this selected item.
    pub fn set_bundle_selected(&mut self, selected: i32) {
        self.bundle_selected = selected;
    }

    /// A stack of an item with no component patch.
    pub fn new(item_count: i32, item_registry_id: ItemId) -> Self {
        Self::from_wire(item_count, item_registry_id, Vec::new(), Vec::new())
    }

    /// `ItemStack.set`/`remove` on the patch: the same stack (object) with
    /// this patch.
    pub fn set_patch(&mut self, patch: &crate::item_components::Patch) {
        self.positive_component_type_ids = patch.added.iter().map(|(id, _)| *id).collect();
        self.negative_component_type_ids.clone_from(&patch.removed);
        self.patch_bytes = if patch.added.is_empty() && patch.removed.is_empty() {
            Vec::new()
        } else {
            patch.trusted()
        };
    }

    /// The stack's component patch, split; empty for one with none.
    pub fn patch(&self) -> crate::item_components::Patch {
        crate::item_components::read_patch(&mut self.patch_bytes.as_slice()).unwrap_or(
            crate::item_components::Patch {
                added: Vec::new(),
                removed: Vec::new(),
            },
        )
    }

    /// What the patch says of a component: `Some(Some(value))` it sets it,
    /// `Some(None)` it removes it, `None` the item's own stands.
    pub fn patched(&self, id: i32) -> Option<Option<Vec<u8>>> {
        let patch = self.patch();
        if patch.removed.contains(&id) {
            return Some(None);
        }
        patch
            .added
            .into_iter()
            .find(|(added, _)| *added == id)
            .map(|(_, value)| Some(value))
    }

    /// An empty patch is kept as no bytes, however it was written.
    pub fn with_patch_bytes(mut self, patch_bytes: Vec<u8>) -> Self {
        self.patch_bytes = if patch_bytes == [0, 0] {
            Vec::new()
        } else {
            patch_bytes
        };
        self
    }

    /// `ItemStack.isSameItemSameComponents`: the same item with the same
    /// component patch; the count does not matter.
    pub fn is_same_item_same_components(&self, other: &Self) -> bool {
        self.item_registry_id == other.item_registry_id && self.patch_bytes == other.patch_bytes
    }

    /// `ItemStack.copyWithCount`: a new stack of the same item and patch.
    pub fn copy_with_count(&self, count: i32) -> Self {
        Self {
            item_count: count,
            identity: NEXT_IDENTITY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            ..self.clone()
        }
    }

    /// `ItemStack.split`: up to `amount` taken off into a new stack.
    pub fn split(&mut self, amount: i32) -> Self {
        let taken = amount.min(self.item_count);
        self.item_count -= taken;
        self.copy_with_count(taken)
    }

    /// `ItemStack.matches`: the same count, item and patch.
    pub fn matches(&self, other: &Self) -> bool {
        self.item_count == other.item_count && self.is_same_item_same_components(other)
    }

    /// Whether two stacks are the same Java object.
    pub fn is_same_object(&self, other: &Self) -> bool {
        self.identity == other.identity
    }

    /// `ItemStack.matchesIgnoringComponents` with the swap animation's
    /// ignored components (damage): the same object, or the same count,
    /// item and patch. The patch is compared whole, damage included.
    pub fn matches_for_swap(&self, other: &Self) -> bool {
        self.is_same_object(other)
            || (self.item_count == other.item_count
                && self.item_registry_id == other.item_registry_id
                && self.patch_bytes == other.patch_bytes)
    }

    /// The component patch as the wire carries it, from the positive
    /// component count on; empty for a stack made here without one.
    pub fn patch_bytes(&self) -> &[u8] {
        &self.patch_bytes
    }

    /// `ItemStack.setCount`.
    pub fn set_count(&mut self, count: i32) {
        self.item_count = count;
    }

    /// `ItemStack.grow`.
    pub fn grow(&mut self, amount: i32) {
        self.item_count += amount;
    }

    /// `ItemStack.shrink`.
    pub fn shrink(&mut self, amount: i32) {
        self.item_count -= amount;
    }

    pub fn item_count(&self) -> i32 {
        self.item_count
    }

    pub fn item_registry_id(&self) -> ItemId {
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
