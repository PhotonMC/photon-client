//! `Minecraft.startAttack` for the item in the main hand: a weapon that asks for a minimum attack
//! charge is not swung before it has it, and a `piercing_weapon` stabs instead of hitting what is
//! aimed at (`MultiPlayerGameMode.piercingAttack`).

use super::context::Context;
use super::{Outgoing, PlayerAction};
use crate::item::values::Values;
use photon_core::ids::ItemId;
use photon_data::item::component_types::{MINIMUM_ATTACK_CHARGE, PIERCING_WEAPON};

/// The item's own definition.
fn definition(item: ItemId) -> Option<&'static photon_data::item::definition::Item> {
    photon_data::item::registry::ITEMS.get(usize::try_from(item.0).ok()?)
}

/// `Player.cannotAttackWithItem` with no tolerance: the charge the item asks for is not yet
/// reached by the attack strength, which recovers at the attack speed.
pub(super) fn cannot_attack_with_item(
    values: &Values,
    item: ItemId,
    ticker: i32,
    attack_speed: f64,
) -> bool {
    if !values.has(MINIMUM_ATTACK_CHARGE) {
        return false;
    }
    let required = values
        .raw(MINIMUM_ATTACK_CHARGE)
        .and_then(|bytes| {
            bytes
                .first_chunk::<4>()
                .map(|bytes| f32::from_be_bytes(*bytes))
        })
        .or_else(|| definition(item).map(|item| item.minimum_attack_charge))
        .unwrap_or(0.0);
    let delay = photon_world::local_player::attack_strength_delay(attack_speed);
    required > 0.0 && ticker as f32 / delay < required
}

/// The sound a `piercing_weapon` makes when stabbing: its optional `sound`, a registered one (an
/// inline one in a patch is not carried).
fn piercing_sound(values: &Values, item: ItemId) -> Option<&'static str> {
    let Some(bytes) = values.raw(PIERCING_WEAPON) else {
        return definition(item)?.piercing_sound;
    };
    let mut rest = bytes.get(2..)?;
    if *rest.first()? == 0 {
        return None;
    }
    rest = &rest[1..];
    let holder = photon_core::wire::read_varint(&mut rest).ok()?;
    photon_data::sound_event::name(holder - 1)
}

/// The main-hand stack's components and item, when something is held.
fn held(ctx: &Context<'_>) -> Option<(Values, ItemId)> {
    let held = ctx.miner.main_hand.as_ref()?;
    let item = held.item_registry_id();
    Some((Values::new(item, held.patch_bytes()), item))
}

/// `heldItem.has(PIERCING_WEAPON)`.
pub(super) fn holds_piercing_weapon(ctx: &Context<'_>) -> bool {
    held(ctx).is_some_and(|(values, _)| values.has(PIERCING_WEAPON))
}

/// The held weapon's part of `Minecraft.startAttack`: `Some(false)` when its attack charge is
/// not reached, `Some(true)` after a stab, `None` to go on with a hit or a block break.
pub(super) fn held_weapon(ctx: &mut Context<'_>, out: &mut Vec<Outgoing>) -> Option<bool> {
    let (values, item) = held(ctx)?;
    let ticker = ctx.player.attack_strength_ticker();
    if cannot_attack_with_item(&values, item, ticker, ctx.miner.attack_speed) {
        return Some(false);
    }
    stab(ctx, &values, item, out).then_some(true)
}

/// `MultiPlayerGameMode.piercingAttack` and the swing after it, when the stack is a piercing
/// weapon; whether it stabbed.
pub(super) fn stab(
    ctx: &mut Context<'_>,
    values: &Values,
    item: ItemId,
    out: &mut Vec<Outgoing>,
) -> bool {
    if !values.has(PIERCING_WEAPON) {
        return false;
    }
    out.push(Outgoing::PlayerAction {
        action: PlayerAction::Stab,
        x: 0,
        y: 0,
        z: 0,
        face: 0,
        sequence: 0,
    });
    // `Player.onAttack`, then `PiercingWeapon.makeSound`.
    ctx.player.reset_only_attack_strength();
    if let Some(sound) = piercing_sound(values, item) {
        ctx.player.play_own_sound(sound, (1.0, 1.0));
    }
    ctx.swing(out);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(name: &str) -> (Values, ItemId) {
        let item = photon_data::item::registry::named(name);
        (Values::new(item, &[0, 0]), item)
    }

    /// A spear asks for a minimum attack charge, a stick for none.
    #[test]
    fn a_weapon_with_a_minimum_charge_waits_for_it() {
        let (stick, item) = stack("minecraft:stick");
        assert!(!cannot_attack_with_item(&stick, item, 0, 4.0));
        let (spear, item) = stack("minecraft:iron_spear");
        assert!(cannot_attack_with_item(&spear, item, 0, 4.0));
        assert!(!cannot_attack_with_item(&spear, item, 100, 4.0));
    }

    #[test]
    fn a_spear_pierces_and_a_stick_does_not() {
        assert!(stack("minecraft:iron_spear").0.has(PIERCING_WEAPON));
        assert!(!stack("minecraft:stick").0.has(PIERCING_WEAPON));
    }

    #[test]
    fn a_spear_names_a_registered_stab_sound() {
        let (spear, item) = stack("minecraft:iron_spear");
        let sound = piercing_sound(&spear, item).expect("sound");
        assert!(sound.starts_with("minecraft:item.spear"), "{sound}");
    }
}
