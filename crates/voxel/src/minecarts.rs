//! The minecarts (`MinecartRenderer`): the model on the rail it is on, rocked by a
//! hit, with the block it carries. Only the old behaviour (the default's) is drawn; a
//! tnt minecart does not flash and a spawner minecart shows no mob in its block yet.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::boat::boat_vertices;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::minecart::{MinecartRenderState, RailPlace, minecart_poses, offset_seed};
use voxel_render::scene::Vertex;

use crate::held_items::HeldItem;
use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};

/// The texture all of them wear (`AbstractMinecartRenderer.MINECART_LOCATION`).
const TEXTURE: &str = "entity/minecart/minecart";

/// The paths (under `textures/`) of the textures the minecarts wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [TEXTURE].into_iter()
}

/// `AbstractMinecartRenderer.oldExtractState`: where the rail puts the minecart, and the
/// rail a little ahead of that and behind it (this alone when the rail ends there).
fn rail_place(world: &voxel_world::ClientWorld, at: (f64, f64, f64)) -> Option<RailPlace> {
    let pos_on_rail = world.minecart_rail_pos(at)?;
    // `getPosOffs(x, y, z, 0.3F)` and `-0.3F`: the offset is a float widened.
    let half = f64::from(0.3_f32);
    Some(RailPlace {
        pos_on_rail,
        front: world
            .minecart_rail_pos_offset(at, half)
            .unwrap_or(pos_on_rail),
        back: world
            .minecart_rail_pos_offset(at, -half)
            .unwrap_or(pos_on_rail),
    })
}

/// `MinecartRenderer` for the minecarts the server added.
pub fn remote_minecarts(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities().filter(|entity| entity.is_minecart()) {
        let Some(name) = voxel_data::entity_type::name(entity.type_id) else {
            continue;
        };
        let layer = format!("{name}#main");
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(TEXTURE),
            models.model(&layer),
        ) else {
            continue;
        };
        let own = entity.minecart().copied().unwrap_or_default();
        let (x, y, z) = snapshot.position;
        let state = MinecartRenderState {
            position: (x, y, z),
            x_rot: entity.x_rot_at(partial),
            y_rot: entity.y_rot_at(partial),
            offset_seed: offset_seed(entity.id()),
            hurt_dir: own.hurt_dir,
            hurt_time: own.hurt_time as f32 - partial,
            damage_time: (own.damage - partial).max(0.0),
            display_offset: entity.minecart_display_offset(),
            rail: rail_place(world, (x, y, z)),
        };
        let feet = [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ];
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let mut base = voxel_render::joml::Pose::default();
        base.translate(feet[0], feet[1], feet[2]);
        let poses = minecart_poses(&base, &state);
        vertices.extend(boat_vertices(model, &poses.model, light, rect));
        if let Some(block) = entity.minecart_display_block() {
            held.push(HeldItem::block(block, poses.block, light, 0.0));
        }
    }
    vertices
}
