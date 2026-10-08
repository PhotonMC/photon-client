//! What each spear is made with (`Item.Properties.spear`): how long its stab swings, and the
//! timings its kinetic weapon component has. The client reads them to pose a spear in use and
//! to time its swing; the speed thresholds and sounds the same component carries are not drawn.

/// A spear's own numbers, in ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spear {
    pub name: &'static str,
    /// `SwingAnimation.duration`: how long the stab swings.
    pub swing_ticks: i32,
    /// `KineticWeapon.delayTicks`: how long the spear takes to be raised.
    pub delay_ticks: i32,
    /// The `maxDurationTicks` of the dismount, knockback and damage conditions.
    pub dismount_ticks: i32,
    pub knockback_ticks: i32,
    pub damage_ticks: i32,
}

/// `KineticWeapon.forwardMovement`: how far the spear is thrust, the same for every spear.
pub const FORWARD_MOVEMENT: f32 = 0.38;

/// The seven spears, as `Items` makes them: each time in seconds times twenty, cut to a whole
/// number of ticks.
pub const SPEARS: [Spear; 7] = [
    spear("minecraft:wooden_spear", 13, 15, 100, 200, 300),
    spear("minecraft:stone_spear", 15, 14, 90, 180, 275),
    spear("minecraft:copper_spear", 17, 13, 80, 165, 250),
    spear("minecraft:iron_spear", 19, 12, 50, 135, 225),
    spear("minecraft:golden_spear", 19, 14, 70, 170, 275),
    spear("minecraft:diamond_spear", 21, 10, 60, 130, 200),
    spear("minecraft:netherite_spear", 23, 8, 50, 110, 175),
];

const fn spear(
    name: &'static str,
    swing_ticks: i32,
    delay_ticks: i32,
    dismount_ticks: i32,
    knockback_ticks: i32,
    damage_ticks: i32,
) -> Spear {
    Spear {
        name,
        swing_ticks,
        delay_ticks,
        dismount_ticks,
        knockback_ticks,
        damage_ticks,
    }
}

/// The spear an item is, by its registry name.
pub fn spear_of(name: &str) -> Option<&'static Spear> {
    SPEARS.iter().find(|spear| spear.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kinetic_item_is_a_spear() {
        for item in crate::item::items::ITEMS
            .iter()
            .filter(|item| item.kinetic_weapon)
        {
            assert!(spear_of(item.name).is_some(), "{}", item.name);
        }
        assert_eq!(
            crate::item::items::ITEMS
                .iter()
                .filter(|item| item.kinetic_weapon)
                .count(),
            SPEARS.len()
        );
    }
}
