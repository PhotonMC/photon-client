//! The boats, chest boats and rafts (`BoatRenderer`, `RaftRenderer`): the
//! model of the wood the boat is made of, turned, rocked by a hit, shaken by
//! a bubble column and with its paddles rowing. The water patch that keeps the
//! water out of the hull is not drawn.

use photon_net::connection::PlaySession;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::boat::{BoatRenderState, boat_pose, boat_vertices};
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The water patch's model layer (`ModelLayers.BOAT_WATER_PATCH`).
const WATER_PATCH_LAYER: &str = "minecraft:boat#water_patch";

/// The textures (under `textures/`) of a boat and a chest boat of each wood (and
/// the bamboo's rafts).
const TEXTURES: [&str; 20] = [
    "entity/boat/acacia",
    "entity/boat/bamboo",
    "entity/boat/birch",
    "entity/boat/cherry",
    "entity/boat/dark_oak",
    "entity/boat/jungle",
    "entity/boat/mangrove",
    "entity/boat/oak",
    "entity/boat/pale_oak",
    "entity/boat/spruce",
    "entity/chest_boat/acacia",
    "entity/chest_boat/bamboo",
    "entity/chest_boat/birch",
    "entity/chest_boat/cherry",
    "entity/chest_boat/dark_oak",
    "entity/chest_boat/jungle",
    "entity/chest_boat/mangrove",
    "entity/chest_boat/oak",
    "entity/chest_boat/pale_oak",
    "entity/chest_boat/spruce",
];

/// The model layer and the texture (under `textures/`) of a boat of this type
/// (`minecraft:oak_chest_boat`, `minecraft:bamboo_raft`...), as
/// `ModelLayers.createBoatModelName` and the renderers' texture path name them.
fn model_of(type_name: &str) -> Option<(String, String)> {
    let name = type_name.strip_prefix("minecraft:")?;
    let chest = name.contains("_chest_");
    let wood = name
        .strip_suffix("_chest_boat")
        .or_else(|| name.strip_suffix("_chest_raft"))
        .or_else(|| name.strip_suffix("_boat"))
        .or_else(|| name.strip_suffix("_raft"))?;
    let folder = if chest { "chest_boat" } else { "boat" };
    Some((
        format!("minecraft:{folder}/{wood}#main"),
        format!("entity/{folder}/{wood}"),
    ))
}

/// The paths (under `textures/`) of the textures the boats wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter()
}

/// `AbstractBoatRenderer` for the boats the server added.
pub fn remote_boats(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn_of_kind(|entity| entity.is_boat()) {
        let Some((layer, texture)) =
            photon_data::entity_type::name(entity.type_id).and_then(model_of)
        else {
            continue;
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot_of(entity, partial),
            textures.rect(&texture),
            models.model(&layer),
        ) else {
            continue;
        };
        let own = entity.boat().copied().unwrap_or_default();
        let state = BoatRenderState {
            y_rot: entity.y_rot_at(partial),
            hurt_dir: own.hurt_dir,
            hurt_time: own.hurt_time as f32 - partial,
            damage_time: (own.damage - partial).max(0.0),
            bubble_angle: own.bubble_angle(partial),
            is_under_water: own.is_under_water,
            rowing_time_left: own.rowing_time(0, entity.boat_paddle_rows(0), partial),
            rowing_time_right: own.rowing_time(1, entity.boat_paddle_rows(1), partial),
        };
        model.setup_boat(&state);
        let placed = placement(world, &snapshot, view, entity.scale());
        let mut base = Pose::default();
        base.translate(placed.feet[0], placed.feet[1], placed.feet[2]);
        let pose = boat_pose(&base, &state);
        // A glowing boat's body joins the outline.
        vertices.extend(crate::mobs::with_outline(
            entity,
            boat_vertices(model, &pose, placed.light, rect),
        ));
        // `BoatRenderer.submitTypeAdditions`: the patch that keeps the water out of the hull,
        // unless the boat is under water; the rafts have none.
        let raft = photon_data::entity_type::name(entity.type_id)
            .is_some_and(|name| name.contains("_raft") || name.contains("_chest_raft"));
        if !raft
            && !state.is_under_water
            && let Some(patch) = models.model(WATER_PATCH_LAYER)
        {
            let mut mask = boat_vertices(patch, &pose, placed.light, rect);
            on_opaque_texels(&mut mask, rect);
            models.water_masks.extend(mask);
        }
    }
}

/// The water patch's layer has no texture size (`LayerDefinition.create(mesh, 0, 0)`; the mask
/// shader never reads it), so its coordinates are not numbers. The entity shader discards the
/// transparent, so they go to a texel of the boat's own planks.
fn on_opaque_texels(vertices: &mut [Vertex], rect: [f32; 4]) {
    let at = |from: f32, to: f32| from + (to - from) * (8.0 / 128.0);
    let uv = [at(rect[0], rect[2]), at(rect[1], rect[3])];
    for vertex in vertices {
        vertex.uv = uv;
    }
}

#[cfg(test)]
mod tests {
    use super::{TEXTURES, model_of};

    fn of(name: &str) -> (String, String) {
        model_of(name).unwrap_or_default()
    }

    #[test]
    fn a_boat_wears_the_texture_of_its_layer() {
        assert_eq!(
            of("minecraft:oak_boat"),
            ("minecraft:boat/oak#main".into(), "entity/boat/oak".into())
        );
        assert_eq!(
            of("minecraft:dark_oak_chest_boat"),
            (
                "minecraft:chest_boat/dark_oak#main".into(),
                "entity/chest_boat/dark_oak".into()
            )
        );
    }

    #[test]
    fn rafts_use_the_bamboo_layers() {
        assert_eq!(of("minecraft:bamboo_raft").1, "entity/boat/bamboo");
        assert_eq!(
            of("minecraft:bamboo_chest_raft").0,
            "minecraft:chest_boat/bamboo#main"
        );
    }

    #[test]
    fn every_wood_has_its_texture_listed() {
        for wood in [
            "acacia", "birch", "cherry", "dark_oak", "jungle", "mangrove", "oak", "pale_oak",
            "spruce",
        ] {
            for kind in ["boat", "chest_boat"] {
                let name = format!("minecraft:{wood}_{kind}");
                assert!(TEXTURES.contains(&of(&name).1.as_str()), "{name}");
            }
        }
        assert_eq!(model_of("minecraft:pig"), None);
    }
}
