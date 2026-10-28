//! What a damage type does to the one it hurts (`DamageType.effects`), and the sound the player
//! makes for it (`DamageEffects.sound`).

use crate::tables::damage_effects::{DAMAGE_EFFECT_SOUNDS, DAMAGE_TYPE_EFFECTS};

/// Every effect and the sound it makes, by its serialized name.
pub fn effect_sounds() -> impl Iterator<Item = (&'static str, &'static str)> {
    DAMAGE_EFFECT_SOUNDS.iter().copied()
}

/// Every damage type and its effect, by the type's registry name.
pub fn type_effects() -> impl Iterator<Item = (&'static str, &'static str)> {
    DAMAGE_TYPE_EFFECTS.iter().copied()
}

/// The sound the player makes when hurt by the damage type of this registry name
/// (`Player.getHurtSound`); the default type's (`hurt`) for a name that is not one the
/// reference knows.
pub fn hurt_sound(damage_type: &str) -> &'static str {
    let effect = DAMAGE_TYPE_EFFECTS
        .iter()
        .find(|(name, _)| *name == damage_type)
        .map_or("hurt", |(_, effect)| *effect);
    DAMAGE_EFFECT_SOUNDS
        .iter()
        .find(|(name, _)| *name == effect)
        .map_or("minecraft:entity.player.hurt", |(_, sound)| *sound)
}
