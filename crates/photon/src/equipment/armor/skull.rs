//! The skull a head stack shows.

use super::Armor;
use crate::equipment::held_items::WornSkull;
use photon_render::entity::mob_model::MobModel;
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

impl Armor {
    /// `CustomHeadLayer`'s skull: the skull's model, posed by the wearer's walk (a piglin's ears and a
    /// dragon's jaw follow it), in its texture, from the pose the layer leaves.
    pub fn skull_vertices(
        &mut self,
        skull: Option<WornSkull>,
        animation_pos: f32,
        textures: &EntityTextures,
        (light, red_overlay): (i32, bool),
    ) -> Vec<Vertex> {
        let Some(WornSkull {
            kind,
            pose,
            profile,
        }) = skull
        else {
            return Vec::new();
        };
        // A head with a profile wears its skin once that is resolved, which the frame after the
        // head was first drawn does; until then and without a profile, the default one.
        let skin = profile.and_then(|profile| self.head_skins.rect(profile));
        let Some(rect) = skin.or_else(|| textures.rect(kind.texture())) else {
            return Vec::new();
        };
        if !self.models.contains_key(kind.layer()) {
            let Some(layer) = photon_data::entity_model::layer(kind.layer()) else {
                return Vec::new();
            };
            self.models
                .insert(kind.layer().to_owned(), MobModel::bake(layer));
        }
        let model = self
            .models
            .get_mut(kind.layer())
            .expect("the model was just built");
        model.setup_skull(kind, animation_pos);
        photon_render::entity::mob_model::mob_vertices_posed(
            model,
            &pose,
            rect,
            (light, red_overlay),
        )
    }
}
