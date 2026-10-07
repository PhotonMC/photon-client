//! The arrows (`ArrowRenderer`, `TippableArrowRenderer`, `SpectralArrowRenderer`): the
//! crossed planes of the model along the way the arrow flies, shaking for a few ticks
//! once it has stuck. The tipped arrow's texture differs. The trident (`ThrownTridentRenderer`) is
//! drawn here too, upright along its flight; its enchantment glint is not drawn yet.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::entity::mob_model::ArrowRenderState;
use voxel_render::entity::textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::objects::boat::boat_vertices;
use voxel_render::objects::projectile::{arrow_pose, cull_back_faces, trident_pose};
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

const LAYER: &str = "minecraft:arrow#main";
const ARROW: &str = "entity/projectiles/arrow";
const TIPPED: &str = "entity/projectiles/arrow_tipped";
const SPECTRAL: &str = "entity/projectiles/arrow_spectral";
const TRIDENT_LAYER: &str = "minecraft:trident#main";
const TRIDENT: &str = "entity/trident/trident";

/// The paths (under `textures/`) of the textures the arrows wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [ARROW, TIPPED, SPECTRAL, TRIDENT].into_iter()
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
    for entity in world.entities().all() {
        let texture = match voxel_data::entity_type::name(entity.type_id) {
            Some("minecraft:arrow") => {
                if entity.arrow().is_some_and(|own| own.is_tipped()) {
                    TIPPED
                } else {
                    ARROW
                }
            }
            Some("minecraft:spectral_arrow") => SPECTRAL,
            Some("minecraft:trident") => TRIDENT,
            _ => continue,
        };
        let trident = texture == TRIDENT;
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(texture),
            models.model(if trident { TRIDENT_LAYER } else { LAYER }),
        ) else {
            continue;
        };
        // `ArrowRenderer.extractRenderState`: the turns between two ticks and the shake, less the part tick.
        let shake = entity.arrow().map_or(0, |own| own.shake_time()) as f32 - partial;
        if trident {
            model.setup_trident();
        } else {
            model.setup_arrow(&ArrowRenderState { shake });
        }
        let (x, y, z) = snapshot.position;
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let (y_rot, x_rot) = (entity.y_rot_at(partial), entity.x_rot_at(partial));
        let pose = if trident {
            trident_pose(&base, y_rot, x_rot)
        } else {
            arrow_pose(&base, y_rot, x_rot)
        };
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let drawn = boat_vertices(model, &pose, light, rect);
        // The arrow's pipeline culls back faces; the trident's does not.
        vertices.extend(if trident {
            drawn
        } else {
            cull_back_faces(drawn, view.camera)
        });
    }
    vertices
}
