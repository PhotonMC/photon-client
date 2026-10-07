//! The small projectiles drawn as models: the shulker's bullet (`ShulkerBulletRenderer`, a
//! tumbling core in a translucent shell), the llama's spit (`LlamaSpitRenderer`) and the evoker's
//! fangs (`EvokerFangsRenderer`, hidden until they bite), and the leash knot on a fence
//! (`LeashKnotRenderer`).

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::mob_vertices_blended;
use voxel_render::entity::textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::objects::boat::boat_vertices;
use voxel_render::objects::projectile::{
    evoker_fangs_pose, leash_knot_pose, llama_spit_pose, shulker_bullet_poses,
};
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, full_block_light, packed_light};

const BULLET_LAYER: &str = "minecraft:shulker_bullet#main";
const BULLET: &str = "entity/shulker/spark";
const SPIT_LAYER: &str = "minecraft:llama_spit#main";
const SPIT: &str = "entity/llama/llama_spit";
const KNOT_LAYER: &str = "minecraft:leash_knot#main";
const KNOT: &str = "entity/lead_knot/lead_knot";
const FANGS_LAYER: &str = "minecraft:evoker_fangs#main";
const FANGS: &str = "entity/illager/evoker_fangs";

/// `ShulkerBulletRenderer`'s shell colour, `0x26FFFFFF`: its alpha byte.
const SHELL_ALPHA: u8 = 0x26;

/// The paths (under `textures/`) of the textures these projectiles wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [BULLET, SPIT, FANGS, KNOT].into_iter()
}

/// The shulker bullets, llama spit and evoker fangs the server added.
pub fn remote_bullets(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().drawn() {
        let (layer, texture) = match voxel_data::entity_type::name(entity.type_id) {
            Some("minecraft:shulker_bullet") => (BULLET_LAYER, BULLET),
            Some("minecraft:llama_spit") => (SPIT_LAYER, SPIT),
            Some("minecraft:evoker_fangs") => (FANGS_LAYER, FANGS),
            Some("minecraft:leash_knot") => (KNOT_LAYER, KNOT),
            _ => continue,
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(layer),
        ) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let (y_rot, x_rot) = (entity.y_rot_at(partial), entity.x_rot_at(partial));
        if texture == BULLET {
            let age = entity.tick_count() as f32 + partial;
            model.setup_shulker_bullet(y_rot, x_rot);
            let [core, shell] = shulker_bullet_poses(&base, age);
            // `ShulkerBulletRenderer.getBlockLightLevel`: fifteen.
            let light = full_block_light(light);
            vertices.extend(boat_vertices(model, &core, light, rect));
            vertices.extend(mob_vertices_blended(
                model,
                &shell,
                rect,
                (light, SHELL_ALPHA),
                view.camera,
            ));
        } else if texture == KNOT {
            model.setup_leash_knot();
            let pose = leash_knot_pose(&base);
            vertices.extend(boat_vertices(model, &pose, light, rect));
        } else if texture == SPIT {
            model.setup_llama_spit();
            let pose = llama_spit_pose(&base, y_rot, x_rot);
            vertices.extend(boat_vertices(model, &pose, light, rect));
        } else {
            // `EvokerFangsRenderer.extractRenderState`: the turn is the current one, not between two.
            let progress = entity
                .evoker_fangs()
                .map_or(0.0, |own| own.animation_progress(partial));
            if progress == 0.0 {
                continue;
            }
            model.setup_evoker_fangs(progress);
            let pose = evoker_fangs_pose(&base, entity.rotation().0);
            vertices.extend(boat_vertices(model, &pose, light, rect));
        }
    }
}
