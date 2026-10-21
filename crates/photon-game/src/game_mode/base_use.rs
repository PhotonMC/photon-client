//! `Item.use` for an item whose class does not override it: the default stack's components decide
//! (`consumable`, then a swappable `equippable`, then `blocks_attacks`, then `kinetic_weapon`).

use super::UseResult;
use super::context::Context;
use crate::item::values::Values;
use photon_data::item::component_types::{
    BLOCKS_ATTACKS, CONSUMABLE, EQUIPPABLE, FOOD, KINETIC_WEAPON,
};
use photon_world::block::rules;

/// The items whose default `food` component can be eaten at full hunger (`alwaysEdible`).
const ALWAYS_EDIBLE: [&str; 5] = [
    "minecraft:chorus_fruit",
    "minecraft:enchanted_golden_apple",
    "minecraft:golden_apple",
    "minecraft:honey_bottle",
    "minecraft:suspicious_stew",
];

/// `Consumable.consumeTicks` of the stack: its component's seconds, else the item's.
fn consume_ticks(values: &Values, item: photon_core::ids::ItemId) -> Option<i32> {
    let seconds = match values
        .raw(CONSUMABLE)
        .and_then(|bytes| bytes.first_chunk::<4>())
    {
        Some(bytes) => f32::from_be_bytes(*bytes),
        None => rules::item(item)?.consumable?.seconds,
    };
    Some((seconds * 20.0) as i32)
}

/// `FoodProperties.canAlwaysEat` of the stack's `food` component: its own, else the item's.
fn can_always_eat(values: &Values, item: photon_core::ids::ItemId) -> bool {
    match values.raw(FOOD) {
        // The nutrition (a varint), the saturation, then the flag.
        Some(bytes) => {
            let mut rest = bytes;
            photon_core::wire::read_varint(&mut rest).is_ok() && rest.get(4).copied() == Some(1)
        }
        None => ALWAYS_EDIBLE
            .iter()
            .any(|name| crate::item::id_of(name) == Some(item)),
    }
}

/// `Player.canEat`: the player cannot be hurt, or the food always can be eaten, or some hunger is
/// left.
fn can_eat(ctx: &Context<'_>, always: bool) -> bool {
    let food = ctx.world.local_player().map_or(20, |state| state.food());
    ctx.player.abilities().invulnerable() || always || food < 20
}

/// `Consumable.startConsuming`: refused when the food cannot be eaten, a use that goes on over
/// time otherwise.
fn start_consuming(
    ctx: &mut Context<'_>,
    hand: u8,
    values: &Values,
    item: photon_core::ids::ItemId,
) -> UseResult {
    if values.has(FOOD) && !can_eat(ctx, can_always_eat(values, item)) {
        return UseResult::Fail;
    }
    // Consumed at once (no ticks): the eaten item's effects and its leftover are not predicted.
    if consume_ticks(values, item).is_some_and(|ticks| ticks > 0) {
        ctx.player.start_using_item(hand != super::MAIN_HAND);
    }
    UseResult::Success { swing: false }
}

/// The client's prediction of `Item.use` for the stack in a hand. A use that starts is started
/// here, as `LivingEntity.startUsingItem` does for the local player, so that the server's flag
/// arrives to a use already begun.
pub(super) fn base_item_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let Some(stack) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
    else {
        return UseResult::Pass;
    };
    let item = stack.item_registry_id();
    let values = Values::new(item, stack.patch_bytes());
    if values.has(CONSUMABLE) {
        return start_consuming(ctx, hand, &values, item);
    }
    // A swappable equippable changes the slots, which is not predicted yet.
    if values.has(EQUIPPABLE) && rules::item(item).is_none_or(|row| row.use_does_nothing()) {
        return UseResult::Success { swing: false };
    }
    if values.has(BLOCKS_ATTACKS) || values.has(KINETIC_WEAPON) {
        ctx.player.start_using_item(hand != super::MAIN_HAND);
        return UseResult::Success { swing: false };
    }
    UseResult::Pass
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(name: &str) -> (Values, photon_core::ids::ItemId) {
        let item = photon_data::item::registry::named(name);
        (Values::new(item, &[0, 0]), item)
    }

    /// `Foods`: the apples, the chorus fruit, the honey bottle and the stew can be eaten at full
    /// hunger; bread cannot.
    #[test]
    fn some_default_food_is_always_edible() {
        for name in ["minecraft:golden_apple", "minecraft:chorus_fruit"] {
            let (values, item) = values(name);
            assert!(can_always_eat(&values, item), "{name}");
        }
        let (values, item) = values("minecraft:bread");
        assert!(!can_always_eat(&values, item));
    }

    /// `Consumable.consumeTicks`: 1.6 seconds of bread, 2 of a honey bottle.
    #[test]
    fn a_consumable_takes_its_seconds_in_ticks() {
        let (bread, item) = values("minecraft:bread");
        assert_eq!(consume_ticks(&bread, item), Some(32));
        let (honey, item) = values("minecraft:honey_bottle");
        assert_eq!(consume_ticks(&honey, item), Some(40));
    }
}
