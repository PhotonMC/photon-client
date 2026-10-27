//! `Equippable.swapWithEquipmentSlot` as the client predicts it: using a wearable item swaps it
//! with what is worn in its slot.

use photon_data::item::component_types::EQUIPPABLE;

use photon_world::ClientItemStackState;

use super::UseResult;
use super::context::Context;
use crate::item::values::Values;

/// The inventory menu slot of an equipment slot a player has (`Player.canUseSlot`): the armor
/// slots. `None` for the slots a player has none of (body, saddle) and for the hand slots, which
/// are not predicted here.
fn menu_slot(name: &str) -> Option<i16> {
    match name {
        "head" => Some(5),
        "chest" => Some(6),
        "legs" => Some(7),
        "feet" => Some(8),
        _ => None,
    }
}

/// Whether the item's own `equippable` is the one the stack has, and the slot it names.
pub(super) fn default_slot(
    values: &Values,
    item: photon_core::ids::ItemId,
) -> Option<&'static str> {
    let defaults = Values::new(item, &[0, 0]);
    if values.raw(EQUIPPABLE) != defaults.raw(EQUIPPABLE) {
        return None;
    }
    let equippable = photon_world::block::rules::item(item)?.equippable?;
    (equippable.swappable && equippable.player).then_some(equippable.slot)
}

/// `Equippable.swapWithEquipmentSlot` for the stack in `hand`, whose slot is `slot`. A worn item
/// with enchantments may carry the curse that stops the change, which is not read, so the result
/// then stays unpredicted (`None`).
pub(super) fn swap(ctx: &mut Context<'_>, hand: u8, slot: &str) -> Option<UseResult> {
    let Some(worn_slot) = menu_slot(slot) else {
        // A slot the player has none of: `canUseSlot` fails and the use passes.
        return matches!(slot, "body" | "saddle").then_some(UseResult::Pass);
    };
    let hand_slot = ctx.hand_slot(hand);
    let state = ctx.world.local_player()?;
    let held = state
        .inventory_slot(hand_slot)
        .filter(|s| s.item_count() > 0)?
        .clone();
    let worn = state
        .inventory_slot(worn_slot)
        .filter(|s| s.item_count() > 0)
        .cloned();
    let creative = ctx.instabuild();
    if let Some(refused) = refusal(worn.as_ref(), &held, creative) {
        return refused;
    }
    let to_equipment = held.copy_with_count(1);
    if held.item_count() <= 1 {
        ctx.world.put_local_player_slot(worn_slot, to_equipment);
        match worn {
            Some(worn) => ctx.world.put_local_player_slot(hand_slot, worn),
            // Creative keeps the stack it wears a copy of; anyone else's hand is emptied.
            None if !creative => ctx.world.shrink_local_player_slot(hand_slot, 1),
            None => {}
        }
        return Some(UseResult::Success { swing: true });
    }
    if !creative {
        ctx.world.shrink_local_player_slot(hand_slot, 1);
    }
    ctx.world.put_local_player_slot(worn_slot, to_equipment);
    if let Some(worn) = worn {
        super::buckets::add_to_inventory(ctx, worn);
    }
    Some(UseResult::Success { swing: true })
}

/// Whether the swap stops before it starts: `Some(None)` where it is not predicted (a worn
/// enchanted item may be cursed with binding), `Some(Some(Fail))` where the same stack is worn.
fn refusal(
    worn: Option<&ClientItemStackState>,
    held: &ClientItemStackState,
    creative: bool,
) -> Option<Option<UseResult>> {
    let worn = worn?;
    let values = Values::new(worn.item_registry_id(), worn.patch_bytes());
    let enchanted = !values
        .enchantments(photon_world::item_components::ids::ENCHANTMENTS)
        .is_empty();
    if enchanted && !creative {
        return Some(None);
    }
    worn.is_same_item_same_components(held)
        .then_some(Some(UseResult::Fail))
}
