//! The block, the water and the fire laid over the view from inside
//! (`ScreenEffectRenderer.submit`), as triangles in the hand's view space.

use photon_core::ids::BlockStateId;
use photon_net::connection::PlaySession;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::scene::Vertex;
use photon_render::screen_effect::{BLOCK_COLOR, EffectVertex, block_quad, fire_quads, water_quad};
use photon_world::ClientEntityPose;
use photon_world::local_player::LocalPlayer;

/// The triangles of each effect: the block the eyes are in, the water and the fire.
#[derive(Default)]
pub struct ScreenEffects {
    pub block: Vec<Vertex>,
    pub water: Vec<Vertex>,
    pub fire: Vec<Vertex>,
}

/// Two triangles for each quad of four corners. The corners are lit as brightly as they can be.
fn triangles(corners: &[EffectVertex]) -> Vec<Vertex> {
    let vertex = |corner: &EffectVertex| {
        let [a, r, g, b] = (corner.color as u32)
            .to_be_bytes()
            .map(|byte| f32::from(byte) / 255.0);
        Vertex {
            position: corner.position,
            uv: corner.uv,
            shade: 1.0,
            tint: [r, g, b, a],
            light: [240.0, 240.0],
        }
    };
    corners
        .chunks(4)
        .flat_map(|quad| [0, 1, 2, 0, 2, 3].map(|i| vertex(&quad[i])))
        .collect()
}

/// `[u0, u1, v0, v1]` of a sprite in the block atlas, from where the atlas has it.
fn corners(renderer: &BlockRenderer, name: &str) -> Option<[f32; 4]> {
    let [u0, v0, u1, v1] = renderer.sprite_rect(name)?;
    Some([u0, u1, v0, v1])
}

/// What covers the view of a player who looks out of their own eyes.
pub fn screen_effects(
    session: &PlaySession,
    player: &LocalPlayer,
    renderer: &BlockRenderer,
) -> ScreenEffects {
    let mut effects = ScreenEffects::default();
    if player.pose() == ClientEntityPose::Sleeping {
        return effects;
    }
    let world = session.world();
    if let Some(state) = player.view_blocking_state(world)
        && let Some(sprite) = renderer
            .particle_sprite(BlockStateId(state))
            .and_then(|name| corners(renderer, name))
    {
        effects.block = triangles(&block_quad(sprite, BLOCK_COLOR));
    }
    if player.spectating() {
        return effects;
    }
    if player.is_eye_in_water() {
        let (y_rot, x_rot) = player.rotation();
        let eyes = player.position();
        let brightness =
            world.eye_brightness([eyes.x, eyes.y + f64::from(player.eye_height()), eyes.z]);
        effects.water = triangles(&water_quad(y_rot, x_rot, brightness));
    }
    let burning = world
        .local_player()
        .is_some_and(photon_world::ClientPlayerState::is_on_fire);
    if burning && let Some(sprite) = corners(renderer, "minecraft:block/fire_1") {
        effects.fire = triangles(&fire_quads(sprite));
    }
    effects
}
