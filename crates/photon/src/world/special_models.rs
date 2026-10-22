//! A block state drawn on an entity through `BuiltInBlockModels`: the special renderers that
//! stand in for the baked model (a minecart's contents, a block display, an enderman's block).
//! Each is drawn in the state it has in the item form: shut, still, undyed unless the block is.
//! The reference makes each a composite of the baked model and the special one, so the baked
//! model is drawn as well (it is the frame of a bell, and no more than a particle for the rest).

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_render::block_entity::shulker_box;
use photon_render::block_entity::special_model::SpecialModel;
use photon_render::entity::mob_model::mob_vertices_posed;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::scene::Vertex;

use super::block_entities::{
    banner_vertices, bell_vertices, pot_vertices, skull_vertices, statue_vertices,
};
use crate::mobs::MobModels;

/// The vertices of a state's special model, from the pose of the block's corner and lit as the
/// entity is; none for a state that has no special model.
pub(crate) fn special_block_vertices(
    models: &mut MobModels,
    textures: &EntityTextures,
    (state, base, light): (BlockStateId, &Pose, i32),
) -> Option<Vec<Vertex>> {
    let name = photon_world::block::state_or_air(state).block().name;
    let placed = (state, base, light);
    match SpecialModel::of(state)? {
        SpecialModel::Shulker { color } => shulker_box(models, textures, placed, color),
        SpecialModel::Chest { .. } => {
            super::block_entities::chest_state_vertices(models, textures, placed)
        }
        // The petals are drawn from their baked model, and the end cubes and the conduit by the
        // entities that hold them.
        SpecialModel::Empty
        | SpecialModel::Conduit
        | SpecialModel::EndCube { .. }
        | SpecialModel::FlowerBed => None,
        _ => other_special(models, textures, name, placed),
    }
}

/// What a state's properties give a special model.
struct Props(BlockStateId);

impl Props {
    fn get(&self, key: &str) -> Option<&'static str> {
        photon_world::block::rules::property(self.0, key)
    }

    fn facing(&self) -> Option<Direction> {
        Direction::from_name(self.get("facing")?)
    }

    /// The wall a hanging block's `facing` names, none for a standing one with a `rotation`.
    fn wall_and_rotation(&self, wall: bool) -> Option<(Option<Direction>, i32)> {
        if wall {
            Some((Some(self.facing()?), 0))
        } else {
            Some((None, self.get("rotation")?.parse().ok()?))
        }
    }
}

/// The bell, the heads, the banners, the pot, the statue and the book, each as its item shows it.
fn other_special(
    models: &mut MobModels,
    textures: &EntityTextures,
    name: &str,
    (state, base, light): (BlockStateId, &Pose, i32),
) -> Option<Vec<Vertex>> {
    let short = name.strip_prefix("minecraft:")?;
    let placed = (base, light);
    let props = Props(state);
    match short {
        "enchanting_table" => Some(table_book(models, textures, placed)),
        "bell" => Some(bell_vertices(models, textures, placed, (0.0, None))),
        "decorated_pot" => {
            let pot = photon_world::PotView {
                facing: props.facing()?,
                sherds: Default::default(),
                wobble: -1.0,
            };
            Some(pot_vertices(models, textures, placed, &pot))
        }
        _ if short.ends_with("copper_golem_statue") => {
            statue(models, textures, placed, (&props, short))
        }
        _ if short.ends_with("_head") || short.ends_with("_skull") => {
            let (wall, rotation) = props.wall_and_rotation(short.contains("_wall_"))?;
            let skull = photon_world::SkullBlockView {
                name: photon_world::block::state_or_air(state).block().name,
                wall,
                rotation,
                animation: 0.0,
            };
            Some(skull_vertices(models, textures, placed, skull))
        }
        _ => banner(models, textures, placed, (&props, short)),
    }
}

/// `CopperGolemStatueSpecialRenderer`: the pose and weathering the state names.
fn statue(
    models: &mut MobModels,
    textures: &EntityTextures,
    placed: (&Pose, i32),
    (props, short): (&Props, &str),
) -> Option<Vec<Vertex>> {
    let statue = photon_world::CopperStatueView {
        facing: props.facing()?,
        pose: props.get("copper_golem_pose")?.to_owned(),
        weathering: ["exposed", "weathered", "oxidized"]
            .iter()
            .position(|stage| short.contains(stage))
            .map_or(0, |index| index as u8 + 1),
    };
    Some(statue_vertices(models, textures, placed, &statue))
}

/// `BannerSpecialRenderer`: the cloth of the block's dye, with no patterns.
fn banner(
    models: &mut MobModels,
    textures: &EntityTextures,
    (base, light): (&Pose, i32),
    (props, short): (&Props, &str),
) -> Option<Vec<Vertex>> {
    let wall = short.ends_with("_wall_banner");
    let dye = short
        .strip_suffix("_wall_banner")
        .or_else(|| short.strip_suffix("_banner"))?;
    let (wall, rotation) = props.wall_and_rotation(wall)?;
    let banner = photon_world::BannerView {
        color: photon_world::DYES
            .iter()
            .find(|candidate| **candidate == dye)?,
        wall,
        rotation,
        patterns: Vec::new(),
    };
    let mut vertices = Vec::new();
    banner_vertices(models, textures, (base, light, 0.0), &banner, &mut vertices);
    Some(vertices)
}

/// `ShulkerBoxSpecialRenderer`: the box shut, turned to the state's facing.
fn shulker_box(
    models: &mut MobModels,
    textures: &EntityTextures,
    (state, base, light): (BlockStateId, &Pose, i32),
    color: Option<&str>,
) -> Option<Vec<Vertex>> {
    let facing = photon_world::block::rules::property(state, "facing")?;
    let facing = Direction::from_name(facing)?;
    let rect = textures.rect(&shulker_box::texture(color))?;
    let model = models.model(shulker_box::LAYER)?;
    model.setup_shulker_box(0.0);
    let pose = shulker_box::box_pose(base, facing);
    Some(mob_vertices_posed(model, &pose, rect, (light, false)))
}

/// `BuiltInBlockModels.createEnchantingTable`: a shut book, laid on the table that the baked
/// model still draws (the two make one composite model).
fn table_book(
    models: &mut MobModels,
    textures: &EntityTextures,
    (base, light): (&Pose, i32),
) -> Vec<Vertex> {
    use photon_render::block_entity::enchant_table;
    use photon_render::joml::Quaternion;
    let (Some(rect), Some(model)) = (
        textures.rect(enchant_table::TEXTURE),
        models.model(enchant_table::LAYER),
    ) else {
        return Vec::new();
    };
    model.setup_book(0.0, (0.0, 0.0));
    let mut pose = *base;
    pose.translate(0.5, 0.8125, 0.5);
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        180.0,
    )));
    pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
        90.0,
    )));
    mob_vertices_posed(model, &pose, rect, (light, false))
}
