//! `SpawnEggItem` as the client predicts it: `EntityType.canSpawn` fails an egg of a type that
//! may not spawn in peaceful, before anything else is asked.

use photon_data::item::component_types::ENTITY_DATA;

use super::UseResult;
use super::context::Context;
use crate::item::values::Values;

/// Whether the egg in `hand` fails: the difficulty is peaceful and its type is not allowed there.
/// An egg with its own `entity_data` is left to the server (its type is read from the
/// component).
pub(super) fn refused(ctx: &Context<'_>, hand: u8) -> bool {
    if ctx.difficulty != 0 {
        return false;
    }
    let Some(stack) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
        .filter(|stack| stack.item_count() > 0)
    else {
        return false;
    };
    let item = stack.item_registry_id();
    let custom = Values::new(item, stack.patch_bytes()).raw(ENTITY_DATA)
        != Values::new(item, &[0, 0]).raw(ENTITY_DATA);
    let Some(kind) =
        crate::item::definition(item).and_then(|row| row.name.strip_suffix("_spawn_egg"))
    else {
        return false;
    };
    !custom && !photon_data::entity_type::allowed_in_peaceful(kind)
}

/// `SpawnEggItem.useOn` on the client: the spawn is the server's, unless the type may not spawn.
pub(super) fn use_on(ctx: &Context<'_>, hand: u8) -> UseResult {
    if refused(ctx, hand) {
        UseResult::Fail
    } else {
        UseResult::Success { swing: true }
    }
}
