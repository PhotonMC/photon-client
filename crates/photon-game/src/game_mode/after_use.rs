//! `ItemStack.applyAfterUseComponentSideEffects`: what a used stack's `use_remainder` and
//! `use_cooldown` do once the use is over, after an instant use and after a finished one.

use photon_world::local_player::LocalPlayer;
use photon_world::{ClientItemStackState, ClientWorld};

use crate::item::cooldowns::ItemCooldowns;
use crate::item::values::Values;

/// The remainder and the cooldown of `before`, the stack in menu slot `.0` as it was before its
/// use, after the use took one from it.
pub(super) fn apply(
    world: &mut ClientWorld,
    cooldowns: &mut ItemCooldowns,
    (slot, instabuild): (i16, bool),
    before: &ClientItemStackState,
) {
    let values = Values::new(before.item_registry_id(), before.patch_bytes());
    if let Some(template) = values.raw(photon_data::item::component_types::USE_REMAINDER) {
        convert_into_remainder(world, (slot, instabuild), before.item_count(), template);
    }
    let Some(seconds) = values
        .raw(photon_data::item::component_types::USE_COOLDOWN)
        .and_then(|bytes| bytes.first_chunk::<4>())
        .map(|bytes| f32::from_be_bytes(*bytes))
    else {
        return;
    };
    cooldowns.add(
        crate::item::cooldowns::group(before),
        (seconds * 20.0) as i32,
    );
}

/// `UseRemainder.convertIntoRemainder` for the stack in `slot` that was just used up by one: an
/// emptied stack becomes the remainder. Extra remainders are the server's to hand out
/// (`handleExtraItemsCreatedOnUse` does nothing on the client).
fn convert_into_remainder(
    world: &mut ClientWorld,
    (slot, instabuild): (i16, bool),
    count_before: i32,
    template: &[u8],
) {
    let used_up_count = world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .map_or(0, |stack| stack.item_count());
    if instabuild || used_up_count >= count_before {
        return;
    }
    if used_up_count <= 0
        && let Some(fresh) = remainder_stack(template)
    {
        world.put_local_player_slot(slot, fresh);
    }
}

/// `ItemStackTemplate.create`: the template's item, count and component patch as a stack.
fn remainder_stack(mut template: &[u8]) -> Option<ClientItemStackState> {
    let item = photon_core::ids::ItemId(photon_core::wire::read_varint(&mut template).ok()?);
    let count = photon_core::wire::read_varint(&mut template).ok()?;
    let patch = photon_world::item_components::read_patch(&mut template).ok()?;
    let mut stack = ClientItemStackState::from_wire(count, item, Vec::new(), Vec::new());
    stack.set_patch(&patch);
    Some(stack)
}

/// `LivingEntity.completeUsingItem` on the server's entity event 9 for the local player: the use
/// ends with the consumable's effects (see `LocalPlayer::complete_using_item`), a consumed stack
/// loses one, and the remainder and cooldown of any finished use apply.
pub fn finish_use(
    world: &mut ClientWorld,
    player: &mut LocalPlayer,
    cooldowns: &mut ItemCooldowns,
    selected_slot: i32,
) {
    let slot = if player.used_item_is_off_hand() {
        45
    } else {
        (36 + selected_slot) as i16
    };
    let before = world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .cloned();
    let Some(consumed) = player.complete_using_item() else {
        return;
    };
    let Some(before) = before else {
        return;
    };
    let instabuild = player.abilities().instabuild();
    if consumed && !instabuild {
        world.shrink_local_player_slot(slot, 1);
    }
    apply(world, cooldowns, (slot, instabuild), &before);
}
