//! The click a crafter's grid slot makes when it is switched on or off.

/// The event `CrafterScreen.updateSlotState` plays on the player.
pub const CRAFTER_SLOT_SOUND: &str = "minecraft:ui.button.click";

/// The volume and pitch of that sound: a slot switched on is higher than one switched off.
pub fn crafter_slot_sound_levels(enabled: bool) -> (f32, f32) {
    (0.4, if enabled { 1.0 } else { 0.75 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_switched_off_sounds_lower() {
        assert_eq!(crafter_slot_sound_levels(true), (0.4, 1.0));
        assert_eq!(crafter_slot_sound_levels(false), (0.4, 0.75));
    }
}
