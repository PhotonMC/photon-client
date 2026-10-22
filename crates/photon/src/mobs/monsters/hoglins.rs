//! The hoglin and the zoglin (`HoglinRenderer`, `ZoglinRenderer`): their
//! textures, grown or baby, and the pose of the swing of an attack.
//!
//! A hoglin that is converting to a zoglin (`Hoglin.isConverting`) shakes in
//! the reference: the client world decides that (`ClientEntity::is_shaking`)
//! and `placement` turns it into the body's shudder.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::HoglinState;
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The hoglins' textures and model layers, grown and baby.
const HOGLINS: [(&str, [&str; 2], [&str; 2]); 2] = [
    (
        "minecraft:hoglin",
        ["entity/hoglin/hoglin", "entity/hoglin/hoglin_baby"],
        ["minecraft:hoglin#main", "minecraft:hoglin_baby#main"],
    ),
    (
        "minecraft:zoglin",
        ["entity/hoglin/zoglin", "entity/hoglin/zoglin_baby"],
        ["minecraft:zoglin#main", "minecraft:zoglin_baby#main"],
    ),
];

/// The paths (under `textures/`) of the textures the hoglins wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    HOGLINS.into_iter().flat_map(|(_, textures, _)| textures)
}

/// `HoglinRenderer` and `ZoglinRenderer` for the hoglins the server added.
pub fn remote_hoglins(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for (name, texture_paths, layers) in HOGLINS {
        let Some(type_id) = photon_data::entity_type::id(name) else {
            continue;
        };
        for entity in world
            .entities()
            .drawn_with_invisible()
            .filter(|entity| entity.type_id == type_id)
        {
            let baby = entity.is_baby();
            let index = usize::from(baby);
            let (Some(snapshot), Some(rect), Some(model)) = (
                world.entity_snapshot(entity.id(), partial),
                textures.rect(texture_paths[index]),
                models.model(layers[index]),
            ) else {
                continue;
            };
            model.setup_hoglin(&HoglinState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
                attack_ticks: entity.hoglin_attack_ticks(),
                baby,
            });
            vertices.extend(crate::mobs::visible_body(
                entity,
                model,
                (&placement(world, &snapshot, view, entity.scale()), rect),
            ));
        }
    }
}
