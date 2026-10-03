//! The copper golem (`CopperGolemRenderer`): its texture by how far it has
//! oxidized, its glowing eyes (`LivingEntityEmissiveLayer`), and its pose, the
//! walk (with arms raised while it holds something), the idle head spin and the
//! chest animations.
//!
//! What it holds, the block on its antenna and a head item are not drawn yet.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{
    CopperGolemModelState, QuadrupedState, mob_vertices, mob_vertices_glow,
};
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const LAYER: &str = "minecraft:copper_golem#main";

/// `CopperGolemOxidationLevels`: the body's and the eyes' textures, from
/// unaffected to oxidized.
const TEXTURES: [(&str, &str); 4] = [
    (
        "entity/copper_golem/copper_golem",
        "entity/copper_golem/copper_golem_eyes",
    ),
    (
        "entity/copper_golem/copper_golem_exposed",
        "entity/copper_golem/copper_golem_eyes_exposed",
    ),
    (
        "entity/copper_golem/copper_golem_weathered",
        "entity/copper_golem/copper_golem_eyes_weathered",
    ),
    (
        "entity/copper_golem/copper_golem_oxidized",
        "entity/copper_golem/copper_golem_eyes_oxidized",
    ),
];

/// The paths (under `textures/`) of the textures a copper golem wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter().flat_map(|(body, eyes)| [body, eyes])
}

/// `CopperGolemRenderer` for the copper golems the server added.
pub fn remote_copper_golems(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let Some(golem) = voxel_data::entity_type::id("minecraft:copper_golem") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    let mut glow = Vec::new();
    for entity in world.entities().filter(|entity| entity.type_id == golem) {
        let (body, eyes) = TEXTURES[usize::from(entity.copper_golem_weather().min(3))];
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(body),
            models.model(LAYER),
        ) else {
            continue;
        };
        model.setup_copper_golem(&CopperGolemModelState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: snapshot.age_in_ticks,
            holds_item: entity.equipment(EquipmentSlot::MainHand).is_some()
                || entity.equipment(EquipmentSlot::OffHand).is_some(),
            animations: entity
                .copper_golem()
                .map(|golem| golem.playing)
                .unwrap_or_default(),
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        // The eyes: the same model, as posed, in the glowing texture.
        if let Some(eyes) = textures.rect(eyes) {
            glow.extend(mob_vertices_glow(model, &placed, eyes));
        }
    }
    models.glow.extend(glow);
    vertices
}
