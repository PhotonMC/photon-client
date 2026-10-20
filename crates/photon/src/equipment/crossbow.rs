//! What a crossbow holds and how long it takes to charge, which its item model reads
//! (`Charge`, `CrossbowPull`).

use photon_data::item::component_types::{CHARGED_PROJECTILES, ENCHANTMENTS};
use photon_game::item::values::Values;
use photon_net::connection::PlaySession;
use photon_render::item_model::Charge;
use photon_world::ClientItemStackState;

/// `Charge.get`: nothing when the stack holds no projectile, a rocket when any of them is a
/// firework rocket, an arrow otherwise.
pub fn charge_type(stack: &ClientItemStackState) -> Charge {
    let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
    let Some(mut input) = values.raw(CHARGED_PROJECTILES) else {
        return Charge::None;
    };
    let count = photon_core::wire::read_varint(&mut input).unwrap_or(0);
    if count <= 0 {
        return Charge::None;
    }
    let rocket = photon_game::item::id_of("minecraft:firework_rocket");
    let mut found_rocket = false;
    for _ in 0..count {
        // `ItemStack.STREAM_CODEC`: the count, the item, and the component patch.
        let (Ok(amount), Ok(item)) = (
            photon_core::wire::read_varint(&mut input),
            photon_core::wire::read_varint(&mut input),
        ) else {
            break;
        };
        if amount <= 0 || photon_game::item_components::read_patch(&mut input).is_err() {
            break;
        }
        found_rocket |= rocket.is_some_and(|rocket| rocket.0 == item);
    }
    if found_rocket {
        Charge::Rocket
    } else {
        Charge::Arrow
    }
}

/// The registry id of the quick charge enchantment, which the server's registry order sets.
pub fn quick_charge_id(session: &PlaySession) -> Option<i32> {
    session
        .registry_names("minecraft:enchantment")
        .iter()
        .position(|name| name == "minecraft:quick_charge")
        .and_then(|index| i32::try_from(index).ok())
}

/// `CrossbowItem.getChargeDuration`: a second and a quarter, less a quarter per level of quick
/// charge, in ticks. (The reference reads the enchantment's charge time effect; its vanilla
/// definition is this formula.)
pub fn charge_duration(quick_charge: Option<i32>, stack: &ClientItemStackState) -> i32 {
    let level = Values::new(stack.item_registry_id(), stack.patch_bytes())
        .enchantments(ENCHANTMENTS)
        .into_iter()
        .find(|(id, _)| Some(*id) == quick_charge)
        .map_or(0, |(_, level)| level);
    let seconds = if level > 0 {
        1.25 - 0.25 * level as f32
    } else {
        1.25
    };
    (seconds * 20.0).floor() as i32
}
