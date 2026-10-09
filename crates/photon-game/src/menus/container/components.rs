//! The components the result menus read and change, on the stack's patch:
//! `ItemStack.get`, `set` and `remove`, where a value the item has itself
//! drops out of the patch (`PatchedDataComponentMap.applyPatch`), and
//! `ItemEnchantments.Mutable`.

use super::*;
use photon_world::item_components::ids;

fn varint(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    photon_core::wire::write_varint(value, &mut out);
    out
}

fn varint_of(value: &[u8]) -> Option<i32> {
    photon_core::wire::read_varint(&mut &value[..]).ok()
}

/// The item's own value of a component, as the wire writes it.
fn prototype(item: i32, id: i32) -> Option<Vec<u8>> {
    let durability = usize::try_from(item)
        .ok()
        .and_then(|index| ITEMS.get(index))
        .map_or(0, |item| item.max_damage);
    match id {
        // Every item's: no enchantments, no repair cost.
        ids::ENCHANTMENTS | ids::REPAIR_COST => Some(varint(0)),
        ids::STORED_ENCHANTMENTS
            if Some(item) == crate::item::id_of("minecraft:enchanted_book") =>
        {
            Some(varint(0))
        }
        ids::REPAIRABLE => crate::item::repairable::repairable_value(item),
        ids::MAX_DAMAGE => (durability > 0).then(|| varint(durability)),
        ids::DAMAGE => (durability > 0).then(|| varint(0)),
        _ => crate::item::values::Values::new(item, &[])
            .raw(id)
            .map(<[u8]>::to_vec),
    }
}

/// `ItemStack.get`: the patch's value, or the item's unless the patch removes it.
pub(super) fn get(stack: &Stack, id: i32) -> Option<Vec<u8>> {
    match stack.patched(id) {
        Some(value) => value,
        None => prototype(stack.item_registry_id(), id),
    }
}

/// `ItemStack.has`.
pub(super) fn has(stack: &Stack, id: i32) -> bool {
    get(stack, id).is_some()
}

/// `ItemStack.set`: a value the item has itself leaves the patch.
pub(super) fn set(stack: &mut Stack, id: i32, value: Vec<u8>) {
    let mut patch = stack.patch();
    patch.removed.retain(|removed| *removed != id);
    let own = prototype(stack.item_registry_id(), id).as_deref() == Some(value.as_slice());
    match patch.added.iter().position(|(added, _)| *added == id) {
        Some(place) if own => {
            patch.added.remove(place);
        }
        Some(place) => patch.added[place].1 = value,
        None if own => {}
        None => patch.added.push((id, value)),
    }
    stack.set_patch(&patch);
}

/// `ItemStack.remove`: an item's own component is marked removed.
pub(super) fn remove(stack: &mut Stack, id: i32) {
    let mut patch = stack.patch();
    patch.added.retain(|(added, _)| *added != id);
    if prototype(stack.item_registry_id(), id).is_some() && !patch.removed.contains(&id) {
        patch.removed.push(id);
    }
    stack.set_patch(&patch);
}

/// `ItemStack.transmuteCopy`: another item with this stack's patch, laid
/// over the new item's own components.
pub(super) fn transmute(stack: &Stack, item: i32) -> Stack {
    let mut made = Stack::new(stack.item_count(), item);
    let patch = stack.patch();
    for (id, value) in patch.added {
        set(&mut made, id, value);
    }
    for id in patch.removed {
        remove(&mut made, id);
    }
    made
}

/// `ItemStack.getMaxDamage`.
pub fn max_damage(stack: &Stack) -> i32 {
    get(stack, ids::MAX_DAMAGE)
        .and_then(|value| varint_of(&value))
        .unwrap_or(0)
}

/// `ItemStack.getDamageValue`.
pub fn damage(stack: &Stack) -> i32 {
    get(stack, ids::DAMAGE)
        .and_then(|value| varint_of(&value))
        .unwrap_or(0)
        .clamp(0, max_damage(stack))
}

/// `ItemStack.setDamageValue`.
pub(super) fn set_damage(stack: &mut Stack, damage: i32) {
    let max = max_damage(stack);
    set(stack, ids::DAMAGE, varint(damage.clamp(0, max)));
}

/// `ItemStack.isDamageableItem`.
pub(super) fn is_damageable(stack: &Stack) -> bool {
    has(stack, ids::MAX_DAMAGE) && !has(stack, ids::UNBREAKABLE) && has(stack, ids::DAMAGE)
}

/// `ItemStack.getOrDefault` of an int component.
pub(super) fn int_or(stack: Option<&Stack>, id: i32, default: i32) -> i32 {
    stack
        .and_then(|stack| get(stack, id))
        .and_then(|value| varint_of(&value))
        .unwrap_or(default)
}

/// `ItemStack.set(REPAIR_COST, ...)`.
pub(super) fn set_int(stack: &mut Stack, id: i32, value: i32) {
    set(stack, id, varint(value));
}

/// `EnchantmentHelper.getComponentType`: a book's are stored.
pub(super) fn enchantment_type(stack: &Stack) -> i32 {
    if Some(stack.item_registry_id()) == crate::item::id_of("minecraft:enchanted_book") {
        ids::STORED_ENCHANTMENTS
    } else {
        ids::ENCHANTMENTS
    }
}

/// `ItemEnchantments`: (network id, level) pairs.
pub(super) type Enchantments = Vec<(i32, i32)>;

fn read_enchantments(value: &[u8]) -> Enchantments {
    let mut input = value;
    let count = photon_core::wire::read_varint(&mut input).unwrap_or(0);
    let mut entries = Vec::new();
    for _ in 0..count {
        let (Ok(id), Ok(level)) = (
            photon_core::wire::read_varint(&mut input),
            photon_core::wire::read_varint(&mut input),
        ) else {
            break;
        };
        entries.push((id, level));
    }
    entries
}

/// `EnchantmentHelper.getEnchantmentsForCrafting`.
pub(super) fn enchantments_for_crafting(stack: &Stack) -> Enchantments {
    get(stack, enchantment_type(stack))
        .map(|value| read_enchantments(&value))
        .unwrap_or_default()
}

/// `EnchantmentHelper.updateEnchantments`: the changed enchantments, set
/// back; none for a stack without the component.
pub(super) fn update_enchantments(
    stack: &mut Stack,
    change: impl FnOnce(&mut Enchantments),
) -> Enchantments {
    let kind = enchantment_type(stack);
    let Some(value) = get(stack, kind) else {
        return Vec::new();
    };
    let mut enchantments = read_enchantments(&value);
    change(&mut enchantments);
    let mut out = varint(enchantments.len() as i32);
    for &(id, level) in &enchantments {
        out.extend(varint(id));
        out.extend(varint(level));
    }
    set(stack, kind, out);
    enchantments
}

/// `ItemEnchantments.Mutable.getLevel`.
pub(super) fn level(enchantments: &Enchantments, id: i32) -> i32 {
    enchantments
        .iter()
        .find(|(entry, _)| *entry == id)
        .map_or(0, |(_, level)| *level)
}

/// `ItemEnchantments.Mutable.upgrade`: the higher level stays.
pub(super) fn upgrade(enchantments: &mut Enchantments, id: i32, level: i32) {
    if level <= 0 {
        return;
    }
    let level = level.min(255);
    match enchantments.iter_mut().find(|(entry, _)| *entry == id) {
        Some(entry) => entry.1 = entry.1.max(level),
        None => enchantments.push((id, level)),
    }
}
