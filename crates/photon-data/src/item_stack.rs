//! The raw item stack the client keeps: item, count and component patch.

use photon_core::ids::ItemId;

mod facts;

/// Raw item-stack state retained by the client inventory copy.
#[derive(Debug, Clone)]
pub struct ClientItemStackState {
    pub item_count: i32,
    pub item_registry_id: ItemId,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    /// The component patch as the wire carried it, for comparing stacks.
    pub patch_bytes: Vec<u8>,
    /// Which Java `ItemStack` object this is: each decoded stack is a new
    /// one, and changing a stack in place (`shrink`) keeps it. The hand
    /// compares held stacks by object.
    pub identity: u64,
    /// `BundleContents.selectedItem`: the item of a bundle's contents the
    /// player has scrolled to, or -1. The client's own: the wire does not
    /// carry it, and the stacks compare without it.
    pub bundle_selected: i32,
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
