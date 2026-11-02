//! The parrots a player carries on its shoulders.

use super::Armor;
use photon_render::entity::mob_model::{MobModel, ParrotPose, ParrotRenderState, QuadrupedState};
use photon_render::entity::player_model::PlayerRenderState;
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_render::scene::Vertex;

/// Turns a posed parrot into triangles, moved by an offset in the pose the layers share.
pub type PlaceAt<'a> = &'a dyn Fn(&MobModel, [f32; 3], TextureRect) -> Vec<Vertex>;

impl Armor {
    /// `ParrotOnShoulderLayer`: a parrot on each shoulder that carries one (`left`, `right`), in
    /// its variant's texture, sitting up to the shoulder in the pose the player's layers share
    /// (lower when the player crouches) and looking where the player does.
    pub fn shoulder_parrots(
        &mut self,
        [left, right]: [Option<i32>; 2],
        state: &PlayerRenderState,
        textures: &EntityTextures,
        place: PlaceAt,
    ) -> Vec<Vertex> {
        let Some(model) = self.baked(concat!("minecraft:", "parrot#main")) else {
            return Vec::new();
        };
        // The reference's fresh render state leaves the wing flap at rest.
        model.setup_parrot(&ParrotRenderState {
            quadruped: QuadrupedState {
                head_y_rot: state.head_y_rot,
                x_rot: state.x_rot,
                walk_position: state.walk_position,
                walk_speed: state.walk_speed,
            },
            age_in_ticks: state.age_in_ticks,
            flap_angle: 0.0,
            pose: ParrotPose::OnShoulder,
        });
        let height = if state.crouching { -1.3 } else { -1.5 };
        let mut vertices = Vec::new();
        for (variant, side) in [(left, 0.4), (right, -0.4)] {
            let Some(variant) = variant else { continue };
            let path = photon_world::mobs::parrot::variant_texture(variant);
            if let Some(rect) = textures.rect(path) {
                vertices.extend(place(model, [side, height, 0.0], rect));
            }
        }
        vertices
    }
}
