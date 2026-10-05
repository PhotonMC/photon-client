//! The minecarts (`MinecartRenderer`): the model on the rail it is on, rocked by a
//! hit, with the block it carries. Only the old behaviour (the default's) is drawn; a
//! tnt minecart does not flash and a spawner minecart shows no mob in its block yet.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::boat::boat_vertices;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::minecart::{
    MinecartRenderState, RailPlace, minecart_poses, offset_seed, primed_tnt_pose, tnt_is_lit,
    tnt_swell_amount,
};
use voxel_render::mob_model::overlay_code;
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
        // `TntMinecartRenderer.extractRenderState`: the fuse left, less the part tick (plus one), if lit.
        let tnt_fuse = (name == "minecraft:tnt_minecart").then(|| {
            if own.fuse() > -1 {
                own.fuse() as f32 - partial + 1.0
            } else {
                -1.0
            }
        });
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
            block_swell: tnt_fuse
                .filter(|fuse| *fuse > -1.0 && *fuse < 10.0)
                .map(tnt_swell_amount),
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
            // A tnt minecart's block flashes white while it is lit (`submitWhiteSolidBlock`).
            let white = tnt_fuse.is_some_and(tnt_is_lit);
            let overlay = if white { overlay_code(false, 1.0) } else { 0.0 };
            held.push(HeldItem::block(block, poses.block, light, overlay));
        }
    }
    vertices
}

/// `TntRenderer` for the blocks of primed tnt the server added: the block it shows (the tnt block's
/// own), swollen as its fuse runs out and flashing white while it burns.
pub fn remote_primed_tnt(session: &PlaySession, (view, held): (FrameView, &mut Vec<HeldItem>)) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().filter(|entity| entity.is_primed_tnt()) {
        let (Some(snapshot), Some(own)) = (
            world.entity_snapshot(entity.id(), partial),
            entity.primed_tnt(),
        ) else {
            continue;
        };
        let Some(block) = own.block_state() else {
            continue;
        };
        // `TntRenderer.extractRenderState`: the fuse left, less the part tick (plus one).
        let fuse = own.fuse() as f32 - partial + 1.0;
        let (x, y, z) = snapshot.position;
        let mut base = voxel_render::joml::Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        // `Entity.getLightProbePosition`: the eyes.
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let overlay = if tnt_is_lit(fuse) {
            overlay_code(false, 1.0)
        } else {
            0.0
        };
        held.push(HeldItem::block(
            block,
            primed_tnt_pose(&base, fuse),
            light,
            overlay,
        ));
    }
}
