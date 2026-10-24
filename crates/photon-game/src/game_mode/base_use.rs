//! `Item.use` for an item whose class does not override it: the default stack's components decide
//! (`consumable`, then a swappable `equippable`, then `blocks_attacks`, then `kinetic_weapon`).

use super::UseResult;
use super::context::Context;
use crate::item::values::Values;
use photon_data::item::component_types::{
    BLOCKS_ATTACKS, CHARGED_PROJECTILES, CONSUMABLE, EQUIPPABLE, FOOD, KINETIC_WEAPON,
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

/// What the prediction of `Item.use` comes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BaseUse {
    /// Nothing happens, and the next hand tries.
    Pass,
    /// Refused, and the next hand tries.
    Fail,
    /// A use starts that goes on until released or finished.
    Start,
    /// What the item does is not predicted (a swap with the equipment slot, a meal eaten at once):
    /// the click counts as done.
    Unported,
}

/// `Item.use` of the stack: the components decide, in order `consumable`, a swappable
/// `equippable`, `blocks_attacks`, `kinetic_weapon`. `can_eat` is `Player.canEat` for food that
/// cannot always be eaten.
fn decide(
    values: &Values,
    item: photon_core::ids::ItemId,
    can_eat: impl Fn(bool) -> bool,
) -> BaseUse {
    if values.has(CONSUMABLE) {
        if values.has(FOOD) && !can_eat(can_always_eat(values, item)) {
            return BaseUse::Fail;
        }
        // Consumed at once (no ticks): the eaten item's effects and its leftover are not predicted.
        return match consume_ticks(values, item) {
            Some(ticks) if ticks > 0 => BaseUse::Start,
            _ => BaseUse::Unported,
        };
    }
    if swappable(values, item) {
        return BaseUse::Unported;
    }
    if values.has(BLOCKS_ATTACKS) || values.has(KINETIC_WEAPON) {
        return BaseUse::Start;
    }
    BaseUse::Pass
}

/// Whether the stack has a swappable `equippable`: the item's own, unless the patch touches the
/// component, whose value is not decoded here (a patched one counts as swappable).
fn swappable(values: &Values, item: photon_core::ids::ItemId) -> bool {
    if !values.has(EQUIPPABLE) {
        return false;
    }
    let defaults = Values::new(item, &[0, 0]);
    values.raw(EQUIPPABLE) != defaults.raw(EQUIPPABLE)
        || rules::item(item)
            .and_then(|row| row.equippable)
            .is_some_and(|equippable| equippable.swappable)
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
    // `Player.canEat`: the player cannot be hurt, or the food always can be eaten, or some hunger
    // is left.
    let food = ctx.world.local_player().map_or(20, |state| state.food());
    let invulnerable = ctx.player.abilities().invulnerable();
    let result = decide(&values, item, |always| invulnerable || always || food < 20);
    match result {
        BaseUse::Pass => UseResult::Pass,
        BaseUse::Fail => UseResult::Fail,
        BaseUse::Start => {
            ctx.player.start_using_item(hand != super::MAIN_HAND);
            if let Some(sound) = kinetic_start_sound(&values, item) {
                ctx.player.play_own_sound(sound, (1.0, 1.0));
            }
            UseResult::Success { swing: false }
        }
        BaseUse::Unported => UseResult::Success { swing: false },
    }
}

/// `KineticWeapon.makeSound`, which `Item.use` makes for a kinetic weapon that does not block
/// attacks and is not consumed.
fn kinetic_start_sound(values: &Values, item: photon_core::ids::ItemId) -> Option<&'static str> {
    if values.has(CONSUMABLE) || swappable(values, item) || values.has(BLOCKS_ATTACKS) {
        return None;
    }
    if !values.has(KINETIC_WEAPON) {
        return None;
    }
    rules::item(item)?.kinetic_sound
}

/// `Player.getProjectile` for the arrows: one in any slot of the inventory, armor or off hand
/// (not the crafting grid), which the item tag `arrows` names; or the player's materials are
/// infinite. A crossbow also takes a firework rocket in a hand.
fn has_projectile(ctx: &Context<'_>, rockets: bool) -> bool {
    let tags = ctx.world.block_state_registry().item_tags();
    let rocket = crate::item::id_of("minecraft:firework_rocket");
    let found = ctx.world.local_player().is_some_and(|state| {
        let held = |slot| {
            state
                .inventory_slot(slot)
                .filter(|stack| stack.item_count() > 0)
        };
        let in_hand = rockets
            && [45, 36 + ctx.selected_slot as i16].into_iter().any(|slot| {
                held(slot).is_some_and(|stack| Some(stack.item_registry_id()) == rocket)
            });
        in_hand
            || (5..=45).any(|slot| {
                held(slot).is_some_and(|stack| {
                    tags.contains(stack.item_registry_id().0, "minecraft:arrows")
                })
            })
    });
    found || ctx.instabuild()
}

/// `BowItem.use`: refused without a projectile, else the draw starts.
fn bow_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    if !has_projectile(ctx, false) {
        return UseResult::Fail;
    }
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    UseResult::Success { swing: false }
}

/// `CrossbowItem.use`: a charged crossbow shoots (not predicted), else it is refused without a
/// projectile or the charge starts.
fn crossbow_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let charged = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
        .is_some_and(|stack| {
            let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
            values.raw(CHARGED_PROJECTILES).is_some_and(|mut bytes| {
                photon_core::wire::read_varint(&mut bytes).is_ok_and(|count| count > 0)
            })
        });
    if charged {
        return UseResult::Success { swing: false };
    }
    if !has_projectile(ctx, true) {
        return UseResult::Fail;
    }
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    UseResult::Success { swing: false }
}

/// `use` of the thrown items: the server spawns the projectile (and the throw sound reaches the
/// client from it), so the client only uses one up (`ItemStack.consume`) and swings.
fn throw_item(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    if !ctx.instabuild() {
        let slot = ctx.hand_slot(hand);
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    UseResult::Success { swing: true }
}

/// `ItemStack.use` after an instantly used item (use duration 0): its `use_cooldown` begins, for
/// the stack as it was before. Other items are approximated as instant when no use started; the
/// ender eye is instant although it starts one.
pub(super) fn after_instant_use(
    ctx: &mut Context<'_>,
    before: Option<&photon_world::ClientItemStackState>,
    result: UseResult,
) {
    let Some(before) = before else {
        return;
    };
    let eye = rules::item_use_owner(before.item_registry_id())
        == photon_data::item::classes::ItemClass::EnderEyeItem;
    if !matches!(result, UseResult::Success { .. }) || (ctx.player.is_using_item() && !eye) {
        return;
    }
    let values = Values::new(before.item_registry_id(), before.patch_bytes());
    let Some(seconds) = values
        .raw(photon_data::item::component_types::USE_COOLDOWN)
        .and_then(|bytes| bytes.first_chunk::<4>())
        .map(|bytes| f32::from_be_bytes(*bytes))
    else {
        return;
    };
    ctx.cooldowns.add(
        crate::item::cooldowns::group(before),
        (seconds * 20.0) as i32,
    );
}

/// `Item.use` for the classes that override it and are predicted: the bow, the crossbow, lily pad
/// and frogspawn, the spyglass (which starts using at once, with its sound), buckets, bottles,
/// thrown items and the server-answered utilities of `utility`. `None` for the rest.
pub(super) fn class_item_use(
    ctx: &mut Context<'_>,
    hand: u8,
    class: photon_data::item::classes::ItemClass,
) -> Option<UseResult> {
    use photon_data::item::classes::ItemClass;
    match class {
        ItemClass::BowItem => Some(bow_use(ctx, hand)),
        ItemClass::CrossbowItem => Some(crossbow_use(ctx, hand)),
        ItemClass::PlaceOnWaterBlockItem => {
            let (item, _) = ctx.held(hand)?;
            Some(super::placing::place_on_water(ctx, hand, item))
        }
        ItemClass::BucketItem => {
            let (item, _) = ctx.held(hand)?;
            Some(super::buckets::bucket_use(ctx, hand, item))
        }
        ItemClass::EnderEyeItem
        | ItemClass::BoatItem
        | ItemClass::SpawnEggItem
        | ItemClass::FishingRodItem
        | ItemClass::EmptyMapItem
        | ItemClass::TridentItem
        | ItemClass::InstrumentItem
        | ItemClass::BottleItem => Some(super::utility::use_of(ctx, hand, class)),
        ItemClass::SnowballItem
        | ItemClass::EggItem
        | ItemClass::EnderpearlItem
        | ItemClass::ExperienceBottleItem
        | ItemClass::WindChargeItem
        | ItemClass::ThrowablePotionItem
        | ItemClass::SplashPotionItem
        | ItemClass::LingeringPotionItem => Some(throw_item(ctx, hand)),
        ItemClass::SpyglassItem => {
            ctx.player
                .play_own_sound("minecraft:item.spyglass.use", (1.0, 1.0));
            ctx.player.start_using_item(hand != super::MAIN_HAND);
            Some(UseResult::Success { swing: false })
        }
        _ => None,
    }
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

    fn decided(name: &str, food: i32, invulnerable: bool) -> BaseUse {
        let (values, item) = values(name);
        decide(&values, item, |always| invulnerable || always || food < 20)
    }

    /// `Consumable.startConsuming` and the rest of `Item.use`, by the default stacks.
    #[test]
    fn the_default_use_of_each_kind_of_item() {
        assert_eq!(decided("minecraft:bread", 20, false), BaseUse::Fail);
        assert_eq!(decided("minecraft:bread", 19, false), BaseUse::Start);
        assert_eq!(decided("minecraft:bread", 20, true), BaseUse::Start);
        assert_eq!(decided("minecraft:golden_apple", 20, false), BaseUse::Start);
        assert_eq!(decided("minecraft:shield", 20, false), BaseUse::Start);
        assert_eq!(decided("minecraft:iron_spear", 20, false), BaseUse::Start);
        assert_eq!(
            decided("minecraft:iron_helmet", 20, false),
            BaseUse::Unported
        );
        assert_eq!(decided("minecraft:stick", 20, false), BaseUse::Pass);
        // A carved pumpkin is worn, but not by using it.
        assert_eq!(
            decided("minecraft:carved_pumpkin", 20, false),
            BaseUse::Pass
        );
    }

    /// `Item.use` makes the kinetic weapon's sound as the use starts; a shield and bread make none.
    #[test]
    fn a_spear_sounds_as_its_use_starts() {
        let (spear, item) = values("minecraft:iron_spear");
        let sound = kinetic_start_sound(&spear, item).expect("sound");
        assert!(sound.starts_with("minecraft:item.spear"), "{sound}");
        for name in ["minecraft:shield", "minecraft:bread", "minecraft:stick"] {
            let (values, item) = values(name);
            assert_eq!(kinetic_start_sound(&values, item), None, "{name}");
        }
    }
}
