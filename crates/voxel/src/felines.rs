//! The cat and the ocelot (`CatRenderer`, `OcelotRenderer`, `CatCollarLayer`):
//! the texture of the cat's variant, the pose of each, how far a cat has lain
//! down, and the collar of a tame cat.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{FelineState, QuadrupedState, mob_vertices, mob_vertices_coloured};
use voxel_render::scene::Vertex;
use voxel_world::ClientEntity;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const REGISTRY: &str = "minecraft:cat_variant";
/// The variant of a cat the server has said nothing of (`Cat.DEFAULT_VARIANT`).
const DEFAULT_VARIANT: &str = "minecraft:black";
/// The ocelot's textures, grown and baby (`OcelotRenderer`).
const OCELOT: [&str; 2] = ["entity/cat/ocelot", "entity/cat/ocelot_baby"];
/// The collar's textures, grown and baby (`CatCollarLayer`).
const COLLARS: [&str; 2] = ["entity/cat/cat_collar", "entity/cat/cat_collar_baby"];

fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures a cat or an ocelot wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    voxel_data::entity_type::variants("cat")
        .flat_map(|variant| {
            [
                texture_path(variant.asset),
                texture_path(variant.baby_asset),
            ]
        })
        .chain(OCELOT)
        .chain(COLLARS)
}

/// The texture of a cat's variant, as the server named it by its place in the
/// registry (the default while it has said nothing); none for one the registry lacks.
fn cat_texture(session: &PlaySession, entity: &ClientEntity) -> Option<&'static str> {
    let name = match entity.variant() {
        None => Some(DEFAULT_VARIANT),
        Some(id) => usize::try_from(id)
            .ok()
            .and_then(|id| session.registry_names(REGISTRY).get(id))
            .map(String::as_str),
    };
    let variant = voxel_data::entity_type::variant("cat", name?)?;
    Some(texture_path(if entity.is_baby() {
        variant.baby_asset
    } else {
        variant.asset
    }))
}

/// `CatRenderer` and `OcelotRenderer` for the cats and ocelots the server added.
pub fn remote_felines(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let (Some(cat), Some(ocelot)) = (
        voxel_data::entity_type::id("minecraft:cat"),
        voxel_data::entity_type::id("minecraft:ocelot"),
    ) else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world
        .entities()
        .filter(|entity| entity.type_id == cat || entity.type_id == ocelot)
    {
        let is_cat = entity.type_id == cat;
        let baby = entity.is_baby();
        let Some(texture) = (if is_cat {
            cat_texture(session, entity)
        } else {
            Some(OCELOT[usize::from(baby)])
        }) else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let name = match (is_cat, baby) {
            (true, false) => "minecraft:cat#main",
            (true, true) => "minecraft:cat_baby#main",
            (false, false) => "minecraft:ocelot#main",
            (false, true) => "minecraft:ocelot_baby#main",
        };
        let (Some(rect), true) = (textures.rect(texture), models.model(name).is_some()) else {
            continue;
        };
        // An ocelot neither sits nor lies down: only a cat has that state.
        let cat_state = is_cat.then(|| entity.cat());
        let state = FelineState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            crouching: entity.is_crouching(),
            sprinting: entity.is_sprinting(),
            sitting: is_cat && entity.is_sitting(),
            baby,
            lie_down_amount: cat_state.map_or(0.0, |cat| cat.lie_down_amount(partial)),
            lie_down_amount_tail: cat_state.map_or(0.0, |cat| cat.lie_down_amount_tail(partial)),
            relax_state_one_amount: cat_state
                .map_or(0.0, |cat| cat.relax_state_one_amount(partial)),
            age_scale: if baby { 0.5 } else { 1.0 },
        };
        let mut placed = placement(world, &snapshot, view, entity.scale());
        if let Some(cat) = cat_state {
            placed.cat_lying(
                state.lie_down_amount,
                cat.is_lying_on_top_of_sleeping_player(),
            );
        }
        if let Some(model) = models.model(name) {
            model.setup_feline(&state);
            vertices.extend(mob_vertices(model, &placed, rect));
        }
        // The collar of a tame cat, on a model of its own posed the same way.
        if is_cat
            && entity.is_tame()
            && !entity.is_invisible()
            && let Some(rect) = textures.rect(COLLARS[usize::from(baby)])
            && let Some(colour) = voxel_data::entity_type::dye_texture_colour(
                usize::try_from(entity.collar_dye()).unwrap_or(14),
            )
            && let Some(model) = models.model(if baby {
                "minecraft:cat_baby#collar"
            } else {
                "minecraft:cat#collar"
            })
        {
            model.setup_feline(&state);
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
