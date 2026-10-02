//! The mobs the frame draws: which entity types have a model and a texture,
//! and where the server's mobs stand between two ticks.

use std::collections::HashMap;

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{MobModel, MobPlacement, QuadrupedState, ZombieState, mob_vertices};
use voxel_render::player_model::PlayerRenderState;
use voxel_render::scene::Vertex;

use crate::overlays::{FrameView, packed_light};

/// A mob that wears a variant the server names: its entity type, the kind
/// of variant, and the registry that numbers the variants.
struct MobKind {
    type_name: &'static str,
    variant_kind: &'static str,
    registry: &'static str,
}

/// The mobs drawn so far.
const KINDS: [MobKind; 2] = [
    MobKind {
        type_name: "minecraft:pig",
        variant_kind: "pig",
        registry: "minecraft:pig_variant",
    },
    MobKind {
        type_name: "minecraft:cow",
        variant_kind: "cow",
        registry: "minecraft:cow_variant",
    },
];

/// A zombie of the family `ZombieModel` poses: its entity type, its model
/// layers and textures (grown and baby), the layer drawn over it (the
/// drowned's) and its scale.
struct ZombieKind {
    type_name: &'static str,
    adult_layer: &'static str,
    baby_layer: &'static str,
    adult_texture: &'static str,
    baby_texture: &'static str,
    /// The outer layer's model layers and textures, if it has one.
    outer: Option<[&'static str; 4]>,
    drowned: bool,
    scale: f32,
}

/// The zombies drawn so far.
const ZOMBIES: [ZombieKind; 4] = [
    ZombieKind {
        type_name: "minecraft:zombie",
        adult_layer: "minecraft:zombie#main",
        baby_layer: "minecraft:zombie_baby#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie_baby",
        outer: None,
        drowned: false,
        scale: 1.0,
    },
    ZombieKind {
        type_name: "minecraft:husk",
        adult_layer: "minecraft:husk#main",
        baby_layer: "minecraft:husk_baby#main",
        adult_texture: "entity/zombie/husk",
        baby_texture: "entity/zombie/husk_baby",
        outer: None,
        drowned: false,
        scale: 1.0,
    },
    ZombieKind {
        type_name: "minecraft:drowned",
        adult_layer: "minecraft:drowned#main",
        baby_layer: "minecraft:drowned_baby#main",
        adult_texture: "entity/zombie/drowned",
        baby_texture: "entity/zombie/drowned_baby",
        outer: Some([
            "minecraft:drowned#outer",
            "minecraft:drowned_baby#outer",
            "entity/zombie/drowned_outer_layer",
            "entity/zombie/drowned_outer_layer_baby",
        ]),
        drowned: true,
        scale: 1.0,
    },
    // `EntityRenderers`: the giant's renderer is made with scale six.
    ZombieKind {
        type_name: "minecraft:giant",
        adult_layer: "minecraft:giant#main",
        baby_layer: "minecraft:giant#main",
        adult_texture: "entity/zombie/zombie",
        baby_texture: "entity/zombie/zombie",
        outer: None,
        drowned: false,
        scale: 6.0,
    },
];

/// The variant a pig or a cow wears while the server has not set another
/// (`PigVariants.DEFAULT`, `CowVariants.TEMPERATE`).
const DEFAULT_VARIANT: &str = "minecraft:temperate";

/// The model layer a variant's model family asks for (`PigRenderer.models`,
/// `CowRenderer.models`): a pig's baby has one model whatever the family.
fn layer_name(kind: &MobKind, model: &str, baby: bool) -> String {
    match (kind.variant_kind, model, baby) {
        ("pig", _, true) => "minecraft:pig_baby#main".to_owned(),
        ("pig", "cold", false) => "minecraft:cold_pig#main".to_owned(),
        ("pig", _, false) => "minecraft:pig#main".to_owned(),
        (cow, model, baby) => {
            let family = match model {
                "normal" => String::new(),
                other => format!("{other}_"),
            };
            let baby = if baby { "_baby" } else { "" };
            format!("minecraft:{family}{cow}{baby}#main")
        }
    }
}

/// An asset id of the data pack as the path under `textures/`.
fn texture_path(asset: &'static str) -> &'static str {
    asset.strip_prefix("minecraft:").unwrap_or(asset)
}

/// The paths (under `textures/`) of the textures the mobs wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    let variants = KINDS
        .iter()
        .flat_map(|kind| voxel_data::entity_type::variants(kind.variant_kind))
        .flat_map(|variant| {
            [
                texture_path(variant.asset),
                texture_path(variant.baby_asset),
            ]
        });
    let zombies = ZOMBIES.iter().flat_map(|kind| {
        [kind.adult_texture, kind.baby_texture].into_iter().chain(
            kind.outer
                .into_iter()
                .flat_map(|outer| [outer[2], outer[3]]),
        )
    });
    variants.chain(zombies)
}

/// The baked models of the mobs, which are posed anew for each one drawn.
pub struct MobModels {
    kinds: HashMap<i32, &'static MobKind>,
    zombies: HashMap<i32, &'static ZombieKind>,
    models: HashMap<String, MobModel>,
}

impl MobModels {
    pub fn new() -> Self {
        Self {
            kinds: KINDS
                .iter()
                .filter_map(|kind| Some((voxel_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            zombies: ZOMBIES
                .iter()
                .filter_map(|kind| Some((voxel_data::entity_type::id(kind.type_name)?, kind)))
                .collect(),
            models: HashMap::new(),
        }
    }

    fn model(&mut self, layer: &str) -> Option<&mut MobModel> {
        if !self.models.contains_key(layer) {
            let baked = MobModel::bake(voxel_data::entity_model::layer(layer)?);
            self.models.insert(layer.to_owned(), baked);
        }
        self.models.get_mut(layer)
    }
}

impl Default for MobModels {
    fn default() -> Self {
        Self::new()
    }
}

/// `LivingEntityRenderer` for the mobs the server added: each posed from
/// where it is between two ticks.
pub fn remote_mobs(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(kind) = models.kinds.get(&entity.type_id).copied() else {
            continue;
        };
        // The variant the server named by its place in the registry, or the
        // default while it has said nothing; one the registry lacks is not drawn.
        let name = match entity.variant() {
            None => Some(DEFAULT_VARIANT),
            Some(id) => usize::try_from(id)
                .ok()
                .and_then(|id| session.registry_names(kind.registry).get(id))
                .map(String::as_str),
        };
        let Some(variant) =
            name.and_then(|name| voxel_data::entity_type::variant(kind.variant_kind, name))
        else {
            continue;
        };
        let layer = layer_name(kind, variant.model, entity.is_baby());
        let texture = texture_path(if entity.is_baby() {
            variant.baby_asset
        } else {
            variant.asset
        });
        let (Some(snapshot), Some(rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
        ) else {
            continue;
        };
        let Some(model) = models.model(&layer) else {
            continue;
        };
        model.setup_quadruped(&QuadrupedState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
        });
        let (x, y, z) = snapshot.position;
        vertices.extend(mob_vertices(
            model,
            &MobPlacement {
                feet: [
                    (x - f64::from(view.origin[0])) as f32,
                    (y - f64::from(view.origin[1])) as f32,
                    (z - f64::from(view.origin[2])) as f32,
                ],
                body_rot: snapshot.body_yaw,
                scale: 1.0,
                light: packed_light(world, Vec3::new(x, y + 0.5, z)),
                death_time: snapshot.death_time,
                flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
                red_overlay: snapshot.red_overlay,
            },
            rect,
        ));
    }
    vertices.extend(remote_zombies(session, models, textures, view));
    vertices
}

/// `LivingEntityRenderer` for the zombies the server added, each posed by
/// `ZombieModel.setupAnim` from where it is between two ticks. The held item
/// (and so the attack swing and the arm pose it asks for) is not read yet.
fn remote_zombies(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let Some(kind) = models.zombies.get(&entity.type_id).copied() else {
            continue;
        };
        let baby = entity.is_baby();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let state = ZombieState {
            humanoid: PlayerRenderState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
                age_in_ticks: entity.tick_count() as f32 + partial,
                swim_amount: snapshot.swim_amount,
                ..Default::default()
            },
            aggressive: entity.is_aggressive(),
            drowned: kind.drowned,
        };
        let (x, y, z) = snapshot.position;
        let placement = MobPlacement {
            feet: [
                (x - f64::from(view.origin[0])) as f32,
                (y - f64::from(view.origin[1])) as f32,
                (z - f64::from(view.origin[2])) as f32,
            ],
            body_rot: snapshot.body_yaw,
            scale: kind.scale,
            light: packed_light(world, Vec3::new(x, y + 0.5, z)),
            death_time: snapshot.death_time,
            flip_degrees: voxel_render::mob_model::FLIP_DEGREES,
            red_overlay: snapshot.red_overlay,
        };
        let layers = [
            Some((
                if baby {
                    kind.baby_layer
                } else {
                    kind.adult_layer
                },
                if baby {
                    kind.baby_texture
                } else {
                    kind.adult_texture
                },
            )),
            kind.outer.map(|outer| {
                (
                    if baby { outer[1] } else { outer[0] },
                    if baby { outer[3] } else { outer[2] },
                )
            }),
        ];
        for (layer, texture) in layers.into_iter().flatten() {
            let (Some(rect), Some(model)) = (textures.rect(texture), models.model(layer)) else {
                continue;
            };
            model.setup_zombie(&state);
            vertices.extend(mob_vertices(model, &placement, rect));
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant of the data pack asks for model layers the game builds,
    /// adult and baby, and the textures it names are the ones the frame loads.
    #[test]
    fn every_variant_has_its_layers_and_textures() {
        let loaded: Vec<&str> = textures().collect();
        for kind in &KINDS {
            for variant in voxel_data::entity_type::variants(kind.variant_kind) {
                for baby in [false, true] {
                    let layer = layer_name(kind, variant.model, baby);
                    assert!(voxel_data::entity_model::layer(&layer).is_some(), "{layer}");
                }
                assert!(loaded.contains(&texture_path(variant.asset)));
                assert!(loaded.contains(&texture_path(variant.baby_asset)));
            }
        }
        assert!(voxel_data::entity_type::variant("pig", DEFAULT_VARIANT).is_some());
        assert!(voxel_data::entity_type::variant("cow", DEFAULT_VARIANT).is_some());
    }
}
