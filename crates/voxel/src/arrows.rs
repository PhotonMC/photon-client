//! The arrows (`ArrowRenderer`, `TippableArrowRenderer`, `SpectralArrowRenderer`): the
//! crossed planes of the model along the way the arrow flies, shaking for a few ticks
//! once it has stuck. The tipped arrow's texture differs; the trident is not drawn yet.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::boat::boat_vertices;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::mob_model::ArrowRenderState;
use voxel_render::projectile::{arrow_pose, cull_back_faces};
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

const LAYER: &str = "minecraft:arrow#main";
const ARROW: &str = "entity/projectiles/arrow";
const TIPPED: &str = "entity/projectiles/arrow_tipped";
const SPECTRAL: &str = "entity/projectiles/arrow_spectral";

/// The paths (under `textures/`) of the textures the arrows wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [ARROW, TIPPED, SPECTRAL].into_iter()
}

/// `ArrowRenderer` for the arrows and spectral arrows the server added.
pub fn remote_arrows(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let texture = match voxel_data::entity_type::name(entity.type_id) {
            Some("minecraft:arrow") => {
                if entity.arrow().is_some_and(|own| own.is_tipped()) {
                    TIPPED
                } else {
                    ARROW
                }
            }
            Some("minecraft:spectral_arrow") => SPECTRAL,
            _ => continue,
        };
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(LAYER),
        ) else {
            continue;
        };
        // `ArrowRenderer.extractRenderState`: the turns between two ticks and the shake, less the part tick.
        let shake = entity.arrow().map_or(0, |own| own.shake_time()) as f32 - partial;
        model.setup_arrow(&ArrowRenderState { shake });
        let (x, y, z) = snapshot.position;
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let pose = arrow_pose(&base, entity.y_rot_at(partial), entity.x_rot_at(partial));
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        vertices.extend(cull_back_faces(
            boat_vertices(model, &pose, light, rect),
            view.camera,
        ));
    }
    vertices
}
