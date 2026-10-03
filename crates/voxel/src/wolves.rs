//! The wolf (`WolfRenderer`, `WolfCollarLayer`): the texture of its variant
//! and state, its pose, and the collar of a tame one.
//!
//! The wolf's armor layer and the darkening of a wet wolf are not drawn: the
//! equipment is not read for mobs yet and whether a mob is in water is not
//! tracked.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{QuadrupedState, WolfState, mob_vertices, mob_vertices_coloured};
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const REGISTRY: &str = "minecraft:wolf_variant";
/// The variant of a wolf the server has said nothing of (`WolfVariants.DEFAULT`).
const DEFAULT_VARIANT: &str = "minecraft:pale";
/// The collar's textures, grown and baby (`WolfCollarLayer`).
const COLLARS: [&str; 2] = ["entity/wolf/wolf_collar", "entity/wolf/wolf_collar_baby"];

fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures a wolf wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    voxel_data::entity_type::wolf_variants()
        .flat_map(|variant| variant.assets.into_iter().map(texture_path))
        .chain(COLLARS)
}

/// `WolfRenderer` and `WolfCollarLayer` for the wolves the server added.
pub fn remote_wolves(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let Some(wolf) = voxel_data::entity_type::id("minecraft:wolf") else {
        return Vec::new();
    };
    let world = session.world();
    let (partial, game_time) = (view.partial as f32, world.game_time());
    let mut vertices = Vec::new();
    for entity in world.entities().filter(|entity| entity.type_id == wolf) {
        let name = match entity.variant() {
            None => Some(DEFAULT_VARIANT),
            Some(id) => usize::try_from(id)
                .ok()
                .and_then(|id| session.registry_names(REGISTRY).get(id))
                .map(String::as_str),
        };
        let Some(variant) = name.and_then(voxel_data::entity_type::wolf_variant) else {
            continue;
        };
        let baby = entity.is_baby();
        let tame = entity.is_tame();
        let angry = entity.is_angry(game_time);
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (Some(rect), Some(model)) = (
            textures.rect(texture_path(variant.texture(baby, tame, angry))),
            models.model(if baby {
                "minecraft:wolf_baby#main"
            } else {
                "minecraft:wolf#main"
            }),
        ) else {
            continue;
        };
        let own = entity.wolf();
        model.setup_wolf(&WolfState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            angry,
            sitting: entity.is_sitting(),
            baby,
            tail_angle: entity.wolf_tail_angle(game_time),
            head_roll_angle: own.head_roll_angle(partial),
            shake_anim: own.shake_anim(partial),
            age_scale: if baby { 0.5 } else { 1.0 },
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        // The collar of a tame wolf, over the model as it is posed.
        if tame
            && !entity.is_invisible()
            && let Some(rect) = textures.rect(COLLARS[usize::from(baby)])
            && let Some(colour) = voxel_data::entity_type::dye_texture_colour(
                usize::try_from(entity.collar_dye()).unwrap_or(14),
            )
        {
            vertices.extend(mob_vertices_coloured(
                model,
                &placed,
                rect,
                Some(colour & 0x00FF_FFFF),
            ));
        }
    }
    vertices
}
