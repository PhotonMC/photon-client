//! The end crystal (`EndCrystalRenderer`): the glass floating over its base,
//! and the beam to the block it is bound to, when it has one.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::light::light_uv;
use photon_render::entity::mob_model::{EndCrystalRenderState, end_crystal_y, mob_vertices_posed};
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::crystal_beam::{crystal_beam, crystal_beam_triangles};
use photon_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

pub(crate) const LAYER: &str = "minecraft:end_crystal#main";
const TEXTURE: &str = "entity/end_crystal/end_crystal";
/// The texture of the beams, of a crystal's and of a dragon's.
pub(crate) const BEAM_TEXTURE: &str = "entity/end_crystal/end_crystal_beam";

/// The paths (under `textures/`) of the textures end crystals and their beams wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE, BEAM_TEXTURE].into_iter()
}

/// `EndCrystalRenderer` for the end crystals the server added.
pub fn remote_end_crystals(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    let Some(crystal) = photon_data::entity_type::id("minecraft:end_crystal") else {
        return;
    };
    for entity in world
        .entities()
        .drawn()
        .filter(|entity| entity.type_id == crystal)
    {
        let (Some(snapshot), Some(rect), Some(beam_rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(TEXTURE),
            textures.rect(BEAM_TEXTURE),
            models.model(LAYER),
        ) else {
            continue;
        };
        let own = entity.end_crystal().copied();
        // `EndCrystalRenderState.ageInTicks`.
        let age = own.map_or(0, |own| own.time()) as f32 + partial;
        model.setup_end_crystal(&EndCrystalRenderState {
            age_in_ticks: age,
            shows_bottom: own.is_none_or(|own| own.shows_bottom()),
        });
        let (x, y, z) = snapshot.position;
        let feet = [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ];
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let mut pose = Pose::default();
        pose.translate(feet[0], feet[1], feet[2]);
        pose.scale(2.0, 2.0, 2.0);
        pose.translate(0.0, -0.5, 0.0);
        vertices.extend(mob_vertices_posed(model, &pose, rect, (light, false)));
        if let Some(target) = own.and_then(|own| own.beam_target()) {
            // The block's centre, from where the crystal is.
            let offset = (
                f64::from(target.x) + 0.5 - x,
                f64::from(target.y) + 0.5 - y,
                f64::from(target.z) + 0.5 - z,
            );
            let mut pose = Pose::default();
            pose.translate(feet[0], feet[1], feet[2]);
            pose.translate(offset.0 as f32, offset.1 as f32, offset.2 as f32);
            let (dx, dy, dz) = (offset.0 as f32, offset.1 as f32, offset.2 as f32);
            let beam = crystal_beam(pose, [-dx, -dy + end_crystal_y(age), -dz], age);
            vertices.extend(crystal_beam_triangles(&beam, beam_rect, light_uv(light)));
        }
    }
}
