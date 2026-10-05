//! The wind charges (`WindChargeRenderer`) and the dragon's fireball (`DragonFireballRenderer`):
//! the charge a spinning model whose wind scrolls along its texture, the fireball a quad that
//! faces the camera, fully lit.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::block_light::light_uv;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::joml::Pose;
use voxel_render::mob_model::mob_vertices_scrolled_posed;
use voxel_render::name_tag::camera_orientation;
use voxel_render::projectile::{dragon_fireball_quad, dragon_fireball_triangles};
use voxel_render::scene::Vertex;

use crate::mobs::MobModels;
use crate::overlays::{FrameView, full_block_light, packed_light};

const WIND_CHARGE_LAYER: &str = "minecraft:wind_charge#main";
const WIND_CHARGE: &str = "entity/projectiles/wind_charge";
const DRAGON_FIREBALL: &str = "entity/enderdragon/dragon_fireball";

/// The paths (under `textures/`) of the textures these projectiles wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [WIND_CHARGE, DRAGON_FIREBALL].into_iter()
}

/// The wind charges and dragon fireballs the server added.
pub fn remote_charges(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let orientation = camera_orientation(view.rotation.0, view.rotation.1);
    let mut vertices = Vec::new();
    for entity in world.entities() {
        let name = voxel_data::entity_type::name(entity.type_id);
        let is_fireball = name == Some("minecraft:dragon_fireball");
        if !is_fireball
            && !matches!(
                name,
                Some("minecraft:wind_charge" | "minecraft:breeze_wind_charge")
            )
        {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
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
        if is_fireball {
            let Some(rect) = textures.rect(DRAGON_FIREBALL) else {
                continue;
            };
            // `DragonFireballRenderer.getBlockLightLevel`: fifteen.
            let light = light_uv(full_block_light(light));
            let quad = dragon_fireball_quad(&base, &orientation);
            vertices.extend(dragon_fireball_triangles(&quad, rect, light));
            continue;
        }
        let (Some(rect), Some(model)) =
            (textures.rect(WIND_CHARGE), models.model(WIND_CHARGE_LAYER))
        else {
            continue;
        };
        let age = entity.tick_count() as f32 + partial;
        model.setup_wind_charge(age);
        // `WindChargeRenderer.xOffset`: three hundredths of the age, within one texture.
        let scroll = (age * 0.03) % 1.0;
        vertices.extend(mob_vertices_scrolled_posed(
            model,
            &base,
            light,
            (rect, scroll),
        ));
    }
    vertices
}
