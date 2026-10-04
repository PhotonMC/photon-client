//! The nautilus and the zombie nautilus (`NautilusRenderer`,
//! `ZombieNautilusRenderer`): a nautilus's texture and model grown or baby, a
//! zombie nautilus's texture and model by the variant the server names; each
//! posed from its look and stride.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{NautilusState, QuadrupedState, mob_vertices, mob_vertices_coloured};
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::armor::Armor;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The registry that numbers the zombie nautilus variants.
const REGISTRY: &str = "minecraft:zombie_nautilus_variant";
/// The variant a zombie nautilus wears while the server has not set another.
const DEFAULT_VARIANT: &str = "minecraft:temperate";
const TEXTURE: &str = "entity/nautilus/nautilus";
const BABY_TEXTURE: &str = "entity/nautilus/nautilus_baby";

fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures nautiluses wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    voxel_data::entity_type::variants("zombie_nautilus")
        .map(|variant| texture_path(variant.asset))
        .chain([TEXTURE, BABY_TEXTURE])
}

/// `NautilusRenderer` and `ZombieNautilusRenderer` for the ones the server added.
pub fn remote_nautiluses(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let nautilus = voxel_data::entity_type::id("minecraft:nautilus");
    let zombie = voxel_data::entity_type::id("minecraft:zombie_nautilus");
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_zombie = Some(entity.type_id) == zombie;
        if !is_zombie && Some(entity.type_id) != nautilus {
            continue;
        }
        let (layer, texture) = if is_zombie {
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
                name.and_then(|name| voxel_data::entity_type::variant("zombie_nautilus", name))
            else {
                continue;
            };
            let layer = if variant.model == "warm" {
                "minecraft:zombie_nautilus_coral#main"
            } else {
                "minecraft:zombie_nautilus#main"
            };
            (layer, texture_path(variant.asset))
        } else if entity.is_baby() {
            ("minecraft:nautilus_baby#main", BABY_TEXTURE)
        } else {
            ("minecraft:nautilus#main", TEXTURE)
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let state = NautilusState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            has_body_armor: entity.equipment(EquipmentSlot::Body).is_some(),
        };
        model.setup_nautilus(&state);
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        // A grown nautilus's saddle and armor (`SimpleEquipmentLayer`; a baby has
        // neither), each on a model of its own, posed as the nautilus is.
        if entity.is_baby() {
            continue;
        }
        for (slot, stack, layer_type, model) in [
            (
                "saddle",
                entity.equipment(EquipmentSlot::Saddle),
                "nautilus_saddle",
                "minecraft:nautilus#saddle",
            ),
            (
                "body",
                entity.equipment(EquipmentSlot::Body),
                "nautilus_body",
                "minecraft:nautilus_armor#main",
            ),
        ] {
            let Some(stack) = stack else {
                continue;
            };
            vertices.extend(armor.simple_layer(
                stack,
                (slot, layer_type, model),
                textures,
                &|model| model.setup_nautilus(&state),
                &|model, rect, colour| mob_vertices_coloured(model, &placed, rect, colour),
            ));
        }
    }
    vertices
}
