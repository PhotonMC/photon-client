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
    values
        .raw(CHARGED_PROJECTILES)
        .map_or(Charge::None, charge_of)
}

/// The charge `charged_projectiles`' value gives: see [`charge_type`].
fn charge_of(mut input: &[u8]) -> Charge {
    let count = photon_core::wire::read_varint(&mut input).unwrap_or(0);
    if count <= 0 {
        return Charge::None;
    }
    let rocket = photon_game::item::id_of("minecraft:firework_rocket");
    let mut found_rocket = false;
    for _ in 0..count {
        // `ItemStackTemplate.STREAM_CODEC`: the item, the count and the component patch.
        let (Ok(item), Ok(amount)) = (
            photon_core::wire::read_varint(&mut input),
            photon_core::wire::read_varint(&mut input),
        ) else {
            break;
        };
        // The item decides, so a patch this client cannot read still counts.
        found_rocket |= rocket.is_some_and(|rocket| rocket.0 == item);
        if amount <= 0 || photon_game::item_components::read_patch(&mut input).is_err() {
            break;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut value: i32) -> Vec<u8> {
        let mut bytes = Vec::new();
        while value >= 0x80 {
            bytes.push((value & 0x7f) as u8 | 0x80);
            value >>= 7;
        }
        bytes.push(value as u8);
        bytes
    }

    /// A projectile list: the count, then each stack's item, count and an empty patch.
    fn list(items: &[&str]) -> Vec<u8> {
        let mut bytes = varint(items.len() as i32);
        for name in items {
            bytes.extend(varint(photon_game::item::id_of(name).unwrap().0));
            bytes.push(1);
            bytes.extend([0, 0]);
        }
        bytes
    }

    #[test]
    fn a_rocket_among_the_projectiles_makes_the_charge_a_rocket() {
        assert_eq!(charge_of(&[0]), Charge::None);
        assert_eq!(charge_of(&list(&["minecraft:arrow"])), Charge::Arrow);
        let rocket = "minecraft:firework_rocket";
        assert_eq!(charge_of(&list(&[rocket])), Charge::Rocket);
        assert_eq!(
            charge_of(&list(&["minecraft:arrow", rocket])),
            Charge::Rocket
        );
    }
}
