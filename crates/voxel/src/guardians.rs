//! The guardian and the elder guardian (`GuardianRenderer`,
//! `ElderGuardianRenderer`): the model posed from the look and the
//! animations of its tail and spikes, its eye turned toward what it looks at:
//! what it aims at, or else the player the camera follows.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::mob_model::{GuardianLook, GuardianState, mob_vertices};
use voxel_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const TEXTURE: &str = "entity/guardian/guardian";
const ELDER_TEXTURE: &str = "entity/guardian/guardian_elder";

/// The paths (under `textures/`) of the textures guardians wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE, ELDER_TEXTURE].into_iter()
}

/// `Entity.calculateViewVector`: where a turn and a pitch (in degrees) point.
fn view_vector(x_rot: f32, y_rot: f32) -> [f64; 3] {
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    let (pitch, yaw) = (x_rot * degrees, -y_rot * degrees);
    let cos = |value: f32| f64::from(voxel_core::math::mth_cos_f64(f64::from(value)));
    let sin = |value: f32| f64::from(voxel_core::math::mth_sin_f64(f64::from(value)));
    let (h, i, j, k) = (cos(yaw), sin(yaw), cos(pitch), sin(pitch));
    [i * j, -k, h * j]
}

/// `getEyePosition` of the entity the guardian looks at (`getEntityToLookAt`):
/// what its beam aims at, or else the camera's entity, the local player.
fn look_at_position(session: &PlaySession, target: Option<i32>, partial: f32) -> Option<[f64; 3]> {
    let world = session.world();
    let player = session.player()?;
    let local_eye = || {
        let position = player.interpolated_position(f64::from(partial));
        [
            position.x,
            position.y + f64::from(player.eye_height()),
            position.z,
        ]
    };
    let Some(id) = target else {
        return Some(local_eye());
    };
    if id == player.entity_id() {
        return Some(local_eye());
    }
    let entity = world.entity_at(id)?;
    let snapshot = world.entity_snapshot(id, partial)?;
    let (_, _, eye) = voxel_data::entity_type::pick(entity.type_id)?.size;
    let (x, y, z) = snapshot.position;
    Some([x, y + f64::from(eye * entity.scale()), z])
}

/// `GuardianRenderer` and `ElderGuardianRenderer` for the ones the server added.
pub fn remote_guardians(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let guardian = voxel_data::entity_type::id("minecraft:guardian");
    let elder = voxel_data::entity_type::id("minecraft:elder_guardian");
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let is_elder = Some(entity.type_id) == elder;
        if !is_elder && Some(entity.type_id) != guardian {
            continue;
        }
        let (layer, texture) = if is_elder {
            ("minecraft:elder_guardian#main", ELDER_TEXTURE)
        } else {
            ("minecraft:guardian#main", TEXTURE)
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let (_, _, eye_height) =
            voxel_data::entity_type::pick(entity.type_id).map_or((0.0, 0.0, 0.0), |pick| pick.size);
        let (x, y, z) = snapshot.position;
        let eye_position = [x, y + f64::from(eye_height * entity.scale()), z];
        let look = look_at_position(session, entity.guardian_attack_target(), partial).map(
            |look_at_position| GuardianLook {
                eye_position,
                look_direction: view_vector(snapshot.pitch, snapshot.yaw),
                look_at_position,
            },
        );
        let own = entity.guardian();
        model.setup_guardian(&GuardianState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            age_in_ticks: entity.tick_count() as f32 + partial,
            spikes_animation: own.map_or(0.0, |own| own.spikes_animation(partial)),
            tail_animation: own.map_or(0.0, |own| own.tail_animation(partial)),
            look,
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
    }
    vertices
}
