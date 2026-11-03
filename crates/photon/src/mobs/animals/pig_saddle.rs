//! `PigRenderer`'s saddle (`SimpleEquipmentLayer`): a grown pig's saddle drawn in the pig's own
//! model layer, posed as the pig is; a baby wears none.

use photon_render::entity::mob_model::{MobPlacement, QuadrupedState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::{EntitySnapshot, EntityView, EquipmentSlot};

use crate::equipment::armor::Armor;

/// The saddle's triangles for `entity`, none unless it is a saddled grown pig.
pub(crate) fn pig_saddle(
    entity: EntityView<'_>,
    (armor, textures): (&mut Armor, &EntityTextures),
    (snapshot, placed): (&EntitySnapshot, &MobPlacement),
) -> Vec<Vertex> {
    if entity.type_id != photon_data::entity_type::ids::PIG || entity.is_baby() {
        return Vec::new();
    }
    let Some(stack) = entity.equipment(EquipmentSlot::Saddle) else {
        return Vec::new();
    };
    let look = QuadrupedState {
        head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
        x_rot: snapshot.pitch,
        walk_position: snapshot.walk_position,
        walk_speed: snapshot.walk_speed,
    };
    armor.simple_layer(
        stack,
        (
            "saddle",
            "pig_saddle",
            photon_data::entity_model::ids::PIG_SADDLE,
        ),
        textures,
        &|model| model.setup_quadruped(&look),
        &|model, rect, colour| {
            crate::mobs::outline::coloured(entity, model, placed, (rect, colour))
        },
    )
}
