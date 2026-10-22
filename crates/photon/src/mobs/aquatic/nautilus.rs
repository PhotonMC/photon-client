//! The nautilus and the zombie nautilus (`NautilusRenderer`,
//! `ZombieNautilusRenderer`): a nautilus's texture and model grown or baby, a
//! zombie nautilus's texture and model by the variant the server names; each
//! posed from its look and stride.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{
    MobPlacement, NautilusState, QuadrupedState, mob_vertices, mob_vertices_coloured,
};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::{ClientEntity, EntitySnapshot, EquipmentSlot};

use crate::equipment::armor::Armor;
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
    photon_data::entity_type::variants("zombie_nautilus")
        .map(|variant| texture_path(variant.asset))
        .chain([TEXTURE, BABY_TEXTURE])
}

/// `NautilusRenderer` and `ZombieNautilusRenderer` for the ones the server added.
pub fn remote_nautiluses(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let nautilus = photon_data::entity_type::id("minecraft:nautilus");
    let zombie = photon_data::entity_type::id("minecraft:zombie_nautilus");
    for entity in world.drawn_mobs() {
        let is_zombie = Some(entity.type_id) == zombie;
        if !is_zombie && Some(entity.type_id) != nautilus {
            continue;
        }
        let Some((layer, texture)) = appearance(entity, is_zombie, &|id| {
            session.registry_names(REGISTRY).get(id).map(String::as_str)
        }) else {
            continue;
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let state = pose_state(
            entity,
            &snapshot,
            snapshot.head_yaw - snapshot.body_yaw,
            partial,
        );
        model.setup_nautilus(&state);
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        worn_layers(armor, entity, textures, &state, &placed, vertices);
    }
}

/// The model layer and texture of a nautilus or zombie nautilus, or none for a zombie variant
/// the registry lacks. `variant_name` names a zombie variant by its place in the registry.
fn appearance<'a>(
    entity: &ClientEntity,
    is_zombie: bool,
    variant_name: &dyn Fn(usize) -> Option<&'a str>,
) -> Option<(&'static str, &'static str)> {
    if !is_zombie {
        return Some(if entity.is_baby() {
            ("minecraft:nautilus_baby#main", BABY_TEXTURE)
        } else {
            ("minecraft:nautilus#main", TEXTURE)
        });
    }
    // The variant the server named, or the default while it has said nothing.
    let name = match entity.variant() {
        None => Some(DEFAULT_VARIANT),
        Some(id) => usize::try_from(id).ok().and_then(variant_name),
    };
    let variant = photon_data::entity_type::variant("zombie_nautilus", name?)?;
    let layer = if variant.model == "warm" {
        "minecraft:zombie_nautilus_coral#main"
    } else {
        "minecraft:zombie_nautilus#main"
    };
    Some((layer, texture_path(variant.asset)))
}

/// What the model is posed from; `head_yaw` is the head's turn against the body.
fn pose_state(
    entity: &ClientEntity,
    snapshot: &EntitySnapshot,
    head_yaw: f32,
    partial: f32,
) -> NautilusState {
    NautilusState {
        quadruped: QuadrupedState {
            head_y_rot: photon_core::math::wrap_degrees_f32(head_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        },
        age_in_ticks: entity.tick_count() as f32 + partial,
        has_body_armor: entity.equipment(EquipmentSlot::Body).is_some(),
    }
}

/// A grown nautilus's saddle and armor (`SimpleEquipmentLayer`; a baby has neither), each on a
/// model of its own, posed as the nautilus is.
fn worn_layers(
    armor: &mut Armor,
    entity: &ClientEntity,
    textures: &EntityTextures,
    state: &NautilusState,
    placed: &MobPlacement,
    vertices: &mut Vec<Vertex>,
) {
    if entity.is_baby() {
        return;
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
            &|model| model.setup_nautilus(state),
            &|model, rect, colour| mob_vertices_coloured(model, placed, rect, colour),
        ));
    }
}

/// A nautilus or zombie nautilus in a screen's box (`InventoryScreen.
/// extractEntityInInventoryFollowsMouse`), with its saddle and armor; none for another entity.
pub fn gui_preview(
    world: &photon_world::ClientWorld,
    registry_names: &[String],
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    entity: &ClientEntity,
    view: &photon_render::entity::player_renderer::GuiEntityView,
) -> Option<Vec<Vertex>> {
    let is_zombie =
        Some(entity.type_id) == photon_data::entity_type::id("minecraft:zombie_nautilus");
    if !is_zombie && Some(entity.type_id) != photon_data::entity_type::id("minecraft:nautilus") {
        return None;
    }
    let (layer, texture) = appearance(entity, is_zombie, &|id| {
        registry_names.get(id).map(String::as_str)
    })?;
    // `createRenderState(entity, 1.0F)`.
    let snapshot = world.entity_snapshot(entity.id(), 1.0)?;
    let (rect, model) = (textures.rect(texture)?, models.model(layer)?);
    let state = pose_state(entity, &snapshot, view.head_yaw(), 1.0);
    model.setup_nautilus(&state);
    let placed = MobPlacement {
        body_rot: view.body_rot(),
        scale: 1.0,
        flip_degrees: photon_render::entity::mob_model::FLIP_DEGREES,
        ..MobPlacement::default()
    };
    let mut vertices = mob_vertices(model, &placed, rect);
    worn_layers(armor, entity, textures, &state, &placed, &mut vertices);
    Some(vertices)
}
