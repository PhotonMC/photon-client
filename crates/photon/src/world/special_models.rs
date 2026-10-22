//! A block state drawn on an entity through `BuiltInBlockModels`: the special renderers that
//! stand in for the baked model (a minecart's contents, a block display, an enderman's block).
//! Each is drawn in the state it has in the item form: shut, still, undyed unless the block is.

use photon_core::ids::BlockStateId;
use photon_render::block_entity::shulker_box;
use photon_render::entity::mob_model::mob_vertices_posed;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::scene::Vertex;

use crate::mobs::MobModels;

/// The vertices of a state's special model, from the pose of the block's corner and lit as the
/// entity is; none for a state that has no special model.
pub(crate) fn special_block_vertices(
    models: &mut MobModels,
    textures: &EntityTextures,
    (state, base, light): (BlockStateId, &Pose, i32),
) -> Option<Vec<Vertex>> {
    let name = photon_world::block::state_or_air(state).block().name;
    if let Some(color) = shulker_color(name) {
        return shulker_box(models, textures, (state, base, light), color);
    }
    super::block_entities::chest_state_vertices(models, textures, (state, base, light))
}

/// The dye of a shulker box block (`Some(None)` for the undyed one), none for any other block.
fn shulker_color(name: &str) -> Option<Option<&str>> {
    let name = name.strip_prefix("minecraft:")?;
    if name == "shulker_box" {
        return Some(None);
    }
    name.strip_suffix("_shulker_box").map(Some)
}

/// `ShulkerBoxSpecialRenderer`: the box shut, turned to the state's facing.
fn shulker_box(
    models: &mut MobModels,
    textures: &EntityTextures,
    (state, base, light): (BlockStateId, &Pose, i32),
    color: Option<&str>,
) -> Option<Vec<Vertex>> {
    let facing = photon_world::block::rules::property(state, "facing")?;
    let facing = photon_core::direction::Direction::from_name(facing)?;
    let rect = textures.rect(&shulker_box::texture(color))?;
    let model = models.model(shulker_box::LAYER)?;
    model.setup_shulker_box(0.0);
    let pose = shulker_box::box_pose(base, facing);
    Some(mob_vertices_posed(model, &pose, rect, (light, false)))
}

/// Whether a state has a special model, which draws in place of its baked one.
pub(crate) fn is_special_block(state: BlockStateId) -> bool {
    let name = photon_world::block::state_or_air(state).block().name;
    shulker_color(name).is_some() || name == "minecraft:chest" || name.ends_with("_chest")
}
