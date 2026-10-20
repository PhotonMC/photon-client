//! Which worn stacks let the player glide (`LivingEntity.canGlideUsing`).

use photon_core::wire::read_varint;
use photon_world::ClientItemStackState as Stack;
use photon_world::item_components::ids;

/// An int component of the stack: its patch's value, else the item's own.
fn int_component(stack: &Stack, id: i32, own: i32) -> i32 {
    match stack.patched(id) {
        Some(Some(value)) => read_varint(&mut value.as_slice()).unwrap_or(own),
        Some(None) => 0,
        None => own,
    }
}

/// `ItemStack.nextDamageWillBreak`: a damageable stack one point from its end.
fn next_damage_will_break(stack: &Stack, item_durability: i32) -> bool {
    let max = int_component(stack, ids::MAX_DAMAGE, item_durability);
    let unbreakable = stack
        .patched(ids::UNBREAKABLE)
        .is_some_and(|value| value.is_some());
    if max <= 0 || unbreakable {
        return false;
    }
    let damage = int_component(stack, ids::DAMAGE, 0).clamp(0, max);
    damage >= max - 1
}

/// `LivingEntity.canGlideUsing`: the stack has the glider component, is worn in the slot its
/// equippable names, and its next point of damage would not break it.
pub fn can_glide_using(stack: &Stack, slot: &str) -> bool {
    let item = super::definition(stack.item_registry_id());
    let glider = match stack.patched(ids::GLIDER) {
        Some(value) => value.is_some(),
        None => item.is_some_and(|item| item.glider),
    };
    if !glider {
        return false;
    }
    let Some(equippable) =
        super::values::equippable_of(stack.item_registry_id(), stack.patch_bytes())
    else {
        return false;
    };
    equippable.slot == slot
        && !next_damage_will_break(stack, item.map_or(0, |item| item.max_damage))
}

/// The inventory slot of each equipment slot, by `EquipmentSlot` name; the main hand is the
/// selected hotbar slot.
fn slots(selected: i32) -> [(&'static str, i16); 6] {
    [
        ("mainhand", 36 + selected as i16),
        ("offhand", 45),
        ("feet", 8),
        ("legs", 7),
        ("chest", 6),
        ("head", 5),
    ]
}

/// `ClientWorld::local_hand_state` with what the item rules add: whether a stack lets the
/// player glide.
pub fn hand_state(
    world: &photon_world::ClientWorld,
    selected: i32,
) -> photon_world::local_player::HandState {
    let mut state = world.local_hand_state(selected);
    if let Some(player) = world.local_player() {
        state.glider = any_can_glide(|slot| player.inventory_slot(slot), selected);
    }
    state
}

/// Whether any worn or held stack lets the player glide.
pub fn any_can_glide<'a>(stack_at: impl Fn(i16) -> Option<&'a Stack>, selected: i32) -> bool {
    slots(selected)
        .into_iter()
        .any(|(name, slot)| stack_at(slot).is_some_and(|stack| can_glide_using(stack, name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(name: &str, patch: Vec<u8>) -> Stack {
        let id = super::super::id_of(name).expect("item");
        Stack::new(1, id).with_patch_bytes(patch)
    }

    fn damaged(damage: i32) -> Vec<u8> {
        // One added component, none removed: the damage.
        let mut patch = vec![1, 0];
        photon_core::wire::write_varint(ids::DAMAGE, &mut patch);
        photon_core::wire::write_varint(damage, &mut patch);
        patch
    }

    #[test]
    fn an_elytra_glides_only_in_the_chest_slot() {
        let elytra = stack("minecraft:elytra", Vec::new());
        assert!(can_glide_using(&elytra, "chest"));
        assert!(!can_glide_using(&elytra, "head"));
        assert!(!can_glide_using(&elytra, "mainhand"));
    }

    #[test]
    fn a_stack_one_point_from_breaking_does_not_glide() {
        assert!(can_glide_using(
            &stack("minecraft:elytra", damaged(430)),
            "chest"
        ));
        assert!(!can_glide_using(
            &stack("minecraft:elytra", damaged(431)),
            "chest"
        ));
    }

    #[test]
    fn other_items_and_empty_slots_do_not_glide() {
        let plate = stack("minecraft:diamond_chestplate", Vec::new());
        assert!(!can_glide_using(&plate, "chest"));
        assert!(!any_can_glide(|_| None, 0));
    }

    #[test]
    fn a_worn_elytra_lets_the_player_glide() {
        let elytra = stack("minecraft:elytra", Vec::new());
        assert!(any_can_glide(|slot| (slot == 6).then_some(&elytra), 0));
    }
}
