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
    if let Some(vertices) = other_special(models, textures, name, (state, base, light)) {
        return Some(vertices);
    }
    super::block_entities::chest_state_vertices(models, textures, (state, base, light))
}

/// The bell, the heads, the banners and the decorated pot, each as its item shows it.
fn other_special(
    models: &mut MobModels,
    textures: &EntityTextures,
    name: &str,
    (state, base, light): (BlockStateId, &Pose, i32),
) -> Option<Vec<Vertex>> {
    use super::block_entities::{banner_vertices, bell_vertices, pot_vertices, skull_vertices};
    let property = |key| photon_world::block::rules::property(state, key);
    let facing = || photon_core::direction::Direction::from_name(property("facing")?);
    let rotation = || property("rotation")?.parse::<i32>().ok();
    let short = name.strip_prefix("minecraft:")?;
    if short == "bell" {
        return Some(bell_vertices(models, textures, (base, light), (0.0, None)));
    }
    if short == "decorated_pot" {
        let pot = photon_world::PotView {
            facing: facing()?,
            sherds: Default::default(),
            wobble: -1.0,
        };
        return Some(pot_vertices(models, textures, (base, light), &pot));
    }
    if short.ends_with("_head") || short.ends_with("_skull") {
        let wall = short.contains("_wall_");
        let skull = photon_world::SkullBlockView {
            name: photon_world::block::state_or_air(state).block().name,
            wall: if wall { Some(facing()?) } else { None },
            rotation: if wall { 0 } else { rotation()? },
            animation: 0.0,
        };
        return Some(skull_vertices(models, textures, (base, light), skull));
    }
    let wall = short.ends_with("_wall_banner");
    let dye = short
        .strip_suffix("_wall_banner")
        .or_else(|| short.strip_suffix("_banner"))?;
    let banner = photon_world::BannerView {
        color: photon_world::DYES
            .iter()
            .find(|candidate| **candidate == dye)?,
        wall: if wall { Some(facing()?) } else { None },
        rotation: if wall { 0 } else { rotation()? },
        patterns: Vec::new(),
    };
    let mut vertices = Vec::new();
    banner_vertices(models, textures, (base, light, 0.0), &banner, &mut vertices);
    Some(vertices)
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
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    shulker_color(name).is_some()
        || short.ends_with("_chest")
        || short == "chest"
        || matches!(short, "bell" | "decorated_pot")
        || short.ends_with("_head")
        || short.ends_with("_skull")
        || short.ends_with("_banner")
}
