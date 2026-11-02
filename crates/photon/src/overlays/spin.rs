use super::*;
use photon_render::entity::player_renderer::Placement;

/// `SpinAttackEffectLayer`: the swirl of a player who spins with a riptide trident, drawn
/// without the overlay.
pub(super) fn spin_effect(
    armor: &mut Armor,
    textures: &photon_render::entity::textures::EntityTextures,
    placement: &Placement,
) -> Vec<Vertex> {
    let Some(age) = placement.auto_spin else {
        return Vec::new();
    };
    let calm = Placement {
        red_overlay: false,
        ..*placement
    };
    armor.spin_effect(age, textures, &|model, offset, rect| {
        photon_render::entity::player_renderer::layer_vertices_at(
            model,
            (&calm, offset),
            rect,
            None,
        )
    })
}
