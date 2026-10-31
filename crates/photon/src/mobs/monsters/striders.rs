//! The strider (`StriderRenderer`): its texture (the cold one while it
//! suffocates), its pose and its saddle (`SimpleEquipmentLayer`).
//!
//! A suffocating strider also shakes in the reference (`isShaking`): the
//! client world decides that and `placement` draws it.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{QuadrupedState, StriderState};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::armor::Armor;
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The strider's textures: warm, then cold; grown, then baby.
const TEXTURES: [[&str; 2]; 2] = [
    ["entity/strider/strider", "entity/strider/strider_baby"],
    [
        "entity/strider/strider_cold",
        "entity/strider/strider_cold_baby",
    ],
];

/// The paths (under `textures/`) of the textures a strider wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter().flatten()
}

/// `StriderRenderer` for the striders the server added.
pub fn remote_striders(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(strider) = photon_data::entity_type::id("minecraft:strider") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs_where(|candidate| candidate == strider) {
        let baby = entity.is_baby();
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let state = StriderState {
            quadruped: QuadrupedState {
                head_y_rot: photon_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: entity.tick_count() as f32 + partial,
            ridden: entity.is_vehicle(),
            baby,
        };
        let (Some(rect), Some(model)) = (
            textures.rect(TEXTURES[usize::from(entity.is_suffocating())][usize::from(baby)]),
            models.model(if baby {
                "minecraft:strider_baby#main"
            } else {
                "minecraft:strider#main"
            }),
        ) else {
            continue;
        };
        model.setup_strider(&state);
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
        // The saddle goes on a grown strider's own model, posed as it is.
        if let (false, Some(stack)) = (baby, entity.equipment(EquipmentSlot::Saddle)) {
            vertices.extend(armor.simple_layer(
                stack,
                ("saddle", "strider_saddle", "minecraft:strider#saddle"),
                textures,
                &|model| model.setup_strider(&state),
                &|model, rect, colour| {
                    crate::mobs::outline::coloured(entity, model, &placed, (rect, colour))
                },
            ));
        }
    }
}
