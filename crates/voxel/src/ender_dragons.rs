//! The ender dragon (`EnderDragonRenderer`): its model posed from the flight
//! history, turned and tilted by it, with its eyes over it.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::dragon_rays::dragon_rays;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::joml::{Pose, Quaternion};
use voxel_render::mob_model::{
    EnderDragonRenderState, mob_vertices_dissolving, mob_vertices_glow_posed, mob_vertices_posed,
};
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

const LAYER: &str = "minecraft:ender_dragon#main";
const TEXTURE: &str = "entity/enderdragon/dragon";
const EYES_TEXTURE: &str = "entity/enderdragon/dragon_eyes";
/// The mask a dying dragon dissolves by (`DRAGON_EXPLODING_LOCATION`).
const EXPLODING_TEXTURE: &str = "entity/enderdragon/dragon_exploding";

/// The paths (under `textures/`) of the textures ender dragons wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE, EYES_TEXTURE, EXPLODING_TEXTURE].into_iter()
}

/// `EnderDragonRenderer.extractRenderState`, the part the model reads.
fn render_state(
    world: &voxel_world::ClientWorld,
    entity: &voxel_world::ClientEntity,
    partial: f32,
) -> Option<EnderDragonRenderState> {
    let own = entity.ender_dragon()?;
    let phase = own.phase();
    Some(EnderDragonRenderState {
        flap_time: own.flap_time_at(partial),
        history: own.history.clone(),
        partial_ticks: if entity.is_dead_or_dying() {
            0.0
        } else {
            partial
        },
        is_landing_or_taking_off: phase.is_landing_or_taking_off(),
        is_sitting: phase.is_sitting(),
        distance_to_egg: world.end_podium_distance_squared(entity.position()),
    })
}

/// `EnderDragonRenderer.submit`'s pose: turned after the history's turn a few ticks back,
/// tipped by how much it climbs, flipped upright and lowered.
fn pose_of(state: &EnderDragonRenderState, feet: [f32; 3]) -> Pose {
    let at = |delay: i32| state.history.get_at(delay, state.partial_ticks);
    let y_rot = at(7).y_rot;
    let climb = (at(5).y - at(10).y) as f32;
    let radians = Quaternion::degrees_to_radians;
    let mut pose = Pose::default();
    pose.translate(feet[0], feet[1], feet[2]);
    pose.rotate(&Quaternion::rotation_y(radians(-y_rot)));
    pose.rotate(&Quaternion::rotation_x(radians(climb * 10.0)));
    pose.translate(0.0, 0.0, 1.0);
    pose.scale(-1.0, -1.0, 1.0);
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// `EnderDragonRenderer` for the ender dragons the server added.
pub fn remote_ender_dragons(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(dragon) = voxel_data::entity_type::id("minecraft:ender_dragon") else {
        return Vec::new();
    };
    let mut vertices = Vec::new();
    let (mut glow, mut dissolving, mut rays) = (Vec::new(), Vec::new(), Vec::new());
    let mut mask_offset = None;
    for entity in world.entities().filter(|entity| entity.type_id == dragon) {
        let (Some(snapshot), Some(rect), Some(eyes), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(TEXTURE),
            textures.rect(EYES_TEXTURE),
            models.model(LAYER),
        ) else {
            continue;
        };
        let Some(state) = render_state(world, entity, partial) else {
            continue;
        };
        model.setup_ender_dragon(&state);
        let (x, y, z) = snapshot.position;
        let feet = [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ];
        let pose = pose_of(&state, feet);
        // `Entity.getLightProbePosition`: the eyes.
        let eye = f64::from(entity.eye_height());
        let light = packed_light(world, Vec3::new(x, y + eye, z));
        // `EnderDragonRenderState.deathTime`: how far through its dying, with the partial tick.
        let death_time = match entity.ender_dragon().map(|own| own.dragon_death_time()) {
            Some(ticks) if ticks > 0 => ticks as f32 + partial,
            _ => 0.0,
        };
        if death_time > 0.0 {
            // The body dissolves away by the mask, no hurt overlay; the rays leave its middle.
            let alpha = 1.0 - death_time / 200.0;
            if let Some(mask) = textures.rect(EXPLODING_TEXTURE) {
                mask_offset = Some([mask[0] - rect[0], mask[1] - rect[1]]);
            }
            dissolving.extend(mob_vertices_dissolving(model, &pose, rect, (light, alpha)));
            let mut rays_pose = pose;
            rays_pose.translate(0.0, -1.0, -2.0);
            rays.extend(
                dragon_rays(rays_pose, death_time / 200.0)
                    .into_iter()
                    .map(ray_vertex),
            );
        } else {
            vertices.extend(mob_vertices_posed(
                model,
                &pose,
                rect,
                (light, snapshot.red_overlay),
            ));
        }
        glow.extend(mob_vertices_glow_posed(model, &pose, eyes));
    }
    models.glow.extend(glow);
    models.dissolve.extend(dissolving);
    models.rays.extend(rays);
    if let Some(offset) = mask_offset {
        models.dissolve_mask_offset = offset;
    }
    vertices
}

/// A ray's vertex for the renderer: its colour (`0xAARRGGBB`) in the tint.
fn ray_vertex(vertex: voxel_render::dragon_rays::RayVertex) -> Vertex {
    let channel = |shift: u32| ((vertex.colour >> shift) & 0xFF) as f32 / 255.0;
    Vertex {
        position: vertex.position,
        uv: [0.0; 2],
        shade: 0.0,
        tint: [channel(16), channel(8), channel(0), channel(24)],
        light: [0.0; 2],
    }
}
