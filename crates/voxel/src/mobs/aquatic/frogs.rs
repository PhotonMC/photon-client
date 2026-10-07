//! The frog (`FrogRenderer`) and the tadpole (`TadpoleRenderer`): a frog's
//! texture by the variant the server names and its pose from the animations
//! its pose and floating run; a tadpole's tail swinging.

use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::{
    FrogRenderState, QuadrupedState, TadpoleState, mob_vertices,
};
use voxel_render::entity::textures::EntityTextures;
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The registry that numbers the frog variants.
const REGISTRY: &str = "minecraft:frog_variant";
/// The variant a frog wears while the server has not set another.
const DEFAULT_VARIANT: &str = "minecraft:temperate";
const TADPOLE_TEXTURE: &str = "entity/tadpole/tadpole";

fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures frogs and tadpoles wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    voxel_data::entity_type::variants("frog")
        .map(|variant| texture_path(variant.asset))
        .chain([TADPOLE_TEXTURE])
}

/// `FrogRenderer` and `TadpoleRenderer` for the ones the server added.
pub fn remote_frogs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let frog = voxel_data::entity_type::id("minecraft:frog");
    let tadpole = voxel_data::entity_type::id("minecraft:tadpole");
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_frog = Some(entity.type_id) == frog;
        if !is_frog && Some(entity.type_id) != tadpole {
            continue;
        }
        let (layer, texture) = if is_frog {
            // The variant the server named by its place in the registry, or the
            // default while it has said nothing; one the registry lacks is not drawn.
            let name = match entity.variant() {
                None => Some(DEFAULT_VARIANT),
                Some(id) => usize::try_from(id)
                    .ok()
                    .and_then(|id| session.registry_names(REGISTRY).get(id))
                    .map(String::as_str),
            };
            let Some(variant) =
                name.and_then(|name| voxel_data::entity_type::variant("frog", name))
            else {
                continue;
            };
            ("minecraft:frog#main", texture_path(variant.asset))
        } else {
            ("minecraft:tadpole#main", TADPOLE_TEXTURE)
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let age_in_ticks = entity.tick_count() as f32 + partial;
        if is_frog {
            let own = entity.frog().copied().unwrap_or_default();
            model.setup_frog(&FrogRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: voxel_core::math::wrap_degrees_f32(
                        snapshot.head_yaw - snapshot.body_yaw,
                    ),
                    x_rot: snapshot.pitch,
                    walk_position: snapshot.walk_position,
                    walk_speed: snapshot.walk_speed,
                },
                age_in_ticks,
                is_swimming: entity.is_in_water(),
                jump: own.jump,
                croak: own.croak,
                tongue: own.tongue,
                swim_idle: own.swim_idle,
            });
        } else {
            model.setup_tadpole(&TadpoleState {
                age_in_ticks,
                is_in_water: entity.is_in_water(),
            });
        }
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
    }
    vertices
}
