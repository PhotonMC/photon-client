//! The swirl around a player who is spinning with a riptide trident.

use super::Armor;
use super::parrots::PlaceAt;
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

impl Armor {
    /// `SpinAttackEffectLayer` with `SpinAttackEffectModel.setupAnim`: two boxes around the
    /// player, each turning about the vertical at its own speed with the player's age in ticks.
    pub fn spin_effect(
        &mut self,
        age_in_ticks: f32,
        textures: &EntityTextures,
        place: PlaceAt,
    ) -> Vec<Vertex> {
        let Some(rect) = textures.rect(crate::overlays::SPIN_ATTACK_TEXTURE) else {
            return Vec::new();
        };
        let Some(model) = self.baked(concat!("minecraft:", "spin_attack#main")) else {
            return Vec::new();
        };
        model.setup_spin_attack(age_in_ticks);
        place(model, [0.0; 3], rect)
    }
}
