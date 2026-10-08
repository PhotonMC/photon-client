use super::*;
use photon_world::item_components::ids;

impl ContainerMenu {
    /// An `Inventory` index as this menu's slot: the hotbar and the main
    /// inventory in both, the armour and the off hand in the player menu.
    pub(super) fn inventory_to_menu(&self, index: usize) -> Option<usize> {
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
            || (self.kind == MenuKind::Brewing && index < 3)
            || (self.kind == MenuKind::Beacon && index == 0)
        {
            1
        } else {
            99
        };
        slot_max.min(max_stack_size(stack))
    }
}

pub(super) fn stacking(stack: &Stack) -> Option<&'static Item> {
    usize::try_from(stack.item_registry_id())
        .ok()
        .and_then(|id| ITEMS.get(id))
}

/// `ItemStack.getMaxStackSize`: the patch's `max_stack_size`, or the item's.
pub fn max_stack_size(stack: &Stack) -> i32 {
    match stack.patched(ids::MAX_STACK_SIZE) {
        Some(Some(value)) => varint_of(&value).unwrap_or(1),
        Some(None) => 1,
        None => stacking(stack).map_or(64, |item| item.max_stack_size),
    }
}

/// The varint a component's value is.
fn varint_of(value: &[u8]) -> Option<i32> {
    photon_core::wire::read_varint(&mut &value[..]).ok()
}

/// `ItemStack.isDamaged`.
pub fn is_damaged(stack: &Stack) -> bool {
    components::is_damageable(stack) && components::damage(stack) > 0
}

/// The network ids of the stack's `enchantments` (the items' own have none).
pub(super) fn enchantment_ids(stack: &Stack) -> impl Iterator<Item = i32> {
    let value = stack
        .patched(ids::ENCHANTMENTS)
        .flatten()
        .unwrap_or_default();
    let mut input = value.as_slice();
    let count = photon_core::wire::read_varint(&mut input).unwrap_or(0);
    let mut found = Vec::new();
    for _ in 0..count {
        let (Ok(id), Ok(_level)) = (
            photon_core::wire::read_varint(&mut input),
            photon_core::wire::read_varint(&mut input),
        ) else {
            break;
        };
        found.push(id);
    }
    found.into_iter()
}

/// `ItemStack.isStackable`.
pub(super) fn is_stackable(stack: &Stack) -> bool {
    max_stack_size(stack) > 1 && !is_damaged(stack)
}

/// `LivingEntity.getEquipmentSlotForItem` for a player, by name.
pub(super) fn equipment_slot(stack: &Stack) -> &'static str {
    stacking(stack)
        .and_then(|item| item.equippable.as_ref())
        .map_or("mainhand", |equip| equip.slot)
}

/// The armour slot's equipment slot, by menu index.
pub(super) fn armour_slot(index: usize) -> Option<&'static str> {
    match index {
        5 => Some("head"),
        6 => Some("chest"),
        7 => Some("legs"),
        8 => Some("feet"),
        _ => None,
    }
}

/// `EquipmentSlot.getIndex` of a humanoid armour slot.
pub(super) fn armour_index(slot: &str) -> Option<usize> {
    match slot {
        "feet" => Some(0),
        "legs" => Some(1),
        "chest" => Some(2),
        "head" => Some(3),
        _ => None,
    }
}

/// A stack emptied in place reads as no stack.
pub(crate) fn settle(stack: &mut Option<Stack>) {
    if stack.as_ref().is_some_and(|stack| stack.item_count() <= 0) {
        *stack = None;
    }
}

pub(super) fn same_item(a: Option<&Stack>, b: &Stack) -> bool {
    a.is_some_and(|a| a.item_registry_id() == b.item_registry_id())
}

/// `FurnaceFuelSlot.isBucket`.
pub(super) fn is_bucket(stack: &Stack) -> bool {
    stacking(stack).is_some_and(|item| item.name == "minecraft:bucket")
}

/// `Item.canFitInsideContainerItems`: every item but a shulker box.
pub(super) fn can_fit_inside_container_items(stack: &Stack) -> bool {
    !stacking(stack)
        .and_then(|item| item.block)
        .and_then(photon_data::block::registry::block)
        .is_some_and(|block| {
            block
                .class
                .is(photon_data::block::classes::BlockClass::ShulkerBoxBlock)
        })
}

/// An `Inventory` index as the player menu's slot.
pub(super) fn inventory_to_menu(index: usize) -> usize {
    match index {
        0..9 => 36 + index,
        40 => OFF_HAND,
        36..40 => 8 - (index - 36),
        _ => index,
    }
}

/// `HashedStack` as its codec writes it: the added components with their
/// hashes, then the removed ones, each in type id order (the reference's
/// identity map and set have no order to follow; the server reads a map).
/// An added component whose hash is not ported is left out, which the
/// server reads as a changed stack.
pub(super) fn write_hashed_stack(
    out: &mut Vec<u8>,
    stack: Option<&Stack>,
    enchantments: &[String],
) {
    use photon_world::item_components::hash::component_hash;
    let Some(stack) = stack else {
        out.push(0);
        return;
    };
    out.push(1);
    photon_core::wire::write_varint(stack.item_registry_id(), out);
    photon_core::wire::write_varint(stack.item_count(), out);
    let mut patch = stack.patch();
    patch.added.sort_by_key(|(id, _)| *id);
    patch.removed.sort_unstable();
    let added: Vec<(i32, i32)> = patch
        .added
        .iter()
        .filter_map(|(id, value)| Some((*id, component_hash(*id, value, enchantments)?)))
        .collect();
    photon_core::wire::write_varint(added.len() as i32, out);
    for (id, hash) in added {
        photon_core::wire::write_varint(id, out);
        out.extend_from_slice(&hash.to_be_bytes());
    }
    photon_core::wire::write_varint(patch.removed.len() as i32, out);
    for id in patch.removed {
        photon_core::wire::write_varint(id, out);
    }
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
            added: vec![(photon_data::item::component_types::UNBREAKABLE, Vec::new())],
            removed: Vec::new(),
        };
        let unbreakable = Stack::from_wire(
            1,
            plain.item_registry_id(),
            vec![photon_data::item::component_types::UNBREAKABLE],
            Vec::new(),
        )
        .with_patch_bytes(patch.trusted());
        assert!(menu.may_place(0, &plain));
        assert!(!menu.may_place(0, &unbreakable));
    }
}
