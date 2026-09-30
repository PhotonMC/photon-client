//! What is drawn over the world's blocks from the player's side: the hands
//! and what they hold, the aimed block's outline, the cracks in blocks being
//! broken, and the player's own model.

use voxel_core::math::Vec3;
use voxel_game::game_mode::{Aim, MAIN_HAND, OFF_HAND};
use voxel_game::item_in_hand::ItemInHand;
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::outline::LineVertex;
use voxel_render::player_model::{PlayerModel, PlayerRenderState};
use voxel_render::player_renderer::{HandView, Placement};
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::local_player::LocalPlayer;

use crate::hud_input::attack_speed;
use crate::item_layers::ItemLayerCache;
use crate::view::{eye_position, pick};

/// Where the frame is drawn from: how far between ticks, the light at the
/// eyes (packed block and sky light), and the point meshes are measured
/// from.
#[derive(Clone, Copy)]
pub struct FrameView {
    pub partial: f64,
    pub light: i32,
    pub origin: [i32; 3],
}

impl FrameView {
    /// A block's corner relative to the mesh origin.
    fn relative(&self, [x, y, z]: [i32; 3]) -> [f32; 3] {
        [
            (x - self.origin[0]) as f32,
            (y - self.origin[1]) as f32,
            (z - self.origin[2]) as f32,
        ]
    }
}

/// `ItemInHandRenderer.tick`: the hands follow the stacks and rise back as
/// the attack strength recovers.
pub fn tick_hands(item_in_hand: &mut ItemInHand, session: &PlaySession, renderer: &BlockRenderer) {
    let world = session.world();
    let local = world.local_player();
    let attack_speed = attack_speed(local);
    let swap = session
        .player()
        .map_or(1.0, |player| player.item_swap_scale(1.0, attack_speed));
    let stack = |slot: i16| {
        local
            .and_then(|state| state.inventory_slot(slot))
            .filter(|stack| stack.item_count() > 0)
    };
    let selected = session.status().selected_slot as i16;
    item_in_hand.tick(stack(36 + selected), stack(45), swap, false, &|item| {
        renderer.item_swap(item)
    });
}

/// Packed block and sky light at a point.
pub fn packed_light(world: &voxel_world::ClientWorld, at: Vec3) -> i32 {
    let [x, y, z] = [at.x, at.y, at.z].map(|value| value.floor() as i32);
    (i32::from(world.block_brightness(x, y, z)) << 4)
        | (i32::from(world.sky_brightness(x, y, z)) << 20)
}

/// `ItemInHandRenderer.submitHandsWithItems`: the arm when the main hand is
/// empty, and each held stack in its hand, the main hand on the right. The
/// swing is the main hand's, since which arm swings is not tracked yet.
pub fn hands(
    player: &LocalPlayer,
    item_in_hand: &ItemInHand,
    held_layers: &mut [ItemLayerCache; 2],
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    model: &mut PlayerModel,
    view: FrameView,
) -> (Vec<Vertex>, Vec<Vertex>) {
    let partial = view.partial as f32;
    let (walked, bob) = player.view_bob(partial);
    let (x_bob, y_bob) = player.hand_bob(partial);
    let (view_y_rot, view_x_rot) = player.rotation();
    let swap = |item: i32| renderer.item_swap(item);
    let main_height = item_in_hand.inverse_arm_height(MAIN_HAND, partial, &swap);
    let off_height = item_in_hand.inverse_arm_height(OFF_HAND, partial, &swap);
    let hand_view = HandView {
        bob_walked: walked,
        bob,
        view_x_rot,
        view_y_rot,
        x_bob,
        y_bob,
        inverse_arm_height: main_height,
        attack: player.attack_anim(partial),
        light: view.light,
    };
    let main_item = item_in_hand.item(MAIN_HAND);
    let off_item = item_in_hand.item(OFF_HAND);
    let hand = if main_item.is_none() {
        voxel_render::player_renderer::hand_vertices(model, &hand_view)
    } else {
        Vec::new()
    };
    let mut held = Vec::new();
    for (stack, right, attack, height) in [
        (main_item, true, hand_view.attack, main_height),
        (off_item, false, 0.0, off_height),
    ] {
        let Some(stack) = stack else {
            continue;
        };
        let layers = held_layers[usize::from(!right)].get(stack, renderer, tints);
        held.extend(voxel_render::player_renderer::held_item_vertices(
            &hand_view, layers, right, attack, height,
        ));
    }
    (hand, held)
}

/// `Camera.calculateHudFov`: the hand's field of view is the default,
/// narrowed in water, and not widened by speed.
pub fn hud_fov(player: &LocalPlayer) -> f32 {
    if player.is_eye_in_water() {
        70.0 * 0.857_142_87
    } else {
        70.0
    }
}

/// `extractBlockOutline`: the block aimed at this frame, unless the player
/// cannot build (adventure and spectator, whose exceptions are not ported).
pub fn outline(
    session: &PlaySession,
    player: &LocalPlayer,
    view: FrameView,
    eye: [f32; 3],
    line_width: f32,
) -> Vec<LineVertex> {
    if matches!(
        session.game_mode(),
        voxel_world::ClientPlayerGameMode::Adventure | voxel_world::ClientPlayerGameMode::Spectator
    ) {
        return Vec::new();
    }
    let Aim::Block { x, y, z, .. } =
        pick(session.world(), player, eye_position(player, view.partial))
    else {
        return Vec::new();
    };
    let Some(state) = session.world().block_state_at(x, y, z) else {
        return Vec::new();
    };
    let shape = voxel_world::outline_shape(state as u32, [x, y, z]);
    voxel_render::outline::outline_vertices(
        &shape,
        view.relative([x, y, z]),
        eye,
        voxel_render::outline::OUTLINE_COLOUR,
        line_width,
    )
}

/// `extractBlockDestroyAnimation`: each cracked block near the camera, at
/// the furthest stage anyone has it.
pub fn cracks(
    session: &PlaySession,
    renderer: &BlockRenderer,
    camera: Vec3,
    view: FrameView,
) -> Vec<Vertex> {
    let mut cracked: std::collections::BTreeMap<(i32, i32, i32), i32> =
        std::collections::BTreeMap::new();
    for progress in session.world().destroying_blocks().values() {
        let centre = [progress.x, progress.y, progress.z].map(|v| f64::from(v) + 0.5);
        let distance = (centre[0] - camera.x).powi(2)
            + (centre[1] - camera.y).powi(2)
            + (centre[2] - camera.z).powi(2);
        if distance <= 1024.0 {
            let stage = cracked
                .entry((progress.x, progress.y, progress.z))
                .or_insert(-1);
            *stage = (*stage).max(progress.stage);
        }
    }
    let mut breaking = Vec::new();
    for (&(x, y, z), &stage) in &cracked {
        let Some(state) = session.world().block_state_at(x, y, z) else {
            continue;
        };
        let (quads, offset) = renderer.block_quads(state as u32, [x, y, z]);
        breaking.extend(voxel_render::breaking::breaking_vertices(
            &quads,
            offset,
            view.relative([x, y, z]),
            stage,
        ));
    }
    breaking
}

/// The player's own model where it stands, posed for the frame.
pub fn player_model(player: &LocalPlayer, model: &mut PlayerModel, view: FrameView) -> Vec<Vertex> {
    let feet = player.interpolated_position(view.partial);
    let partial = view.partial as f32;
    let (_, x_rot) = player.rotation();
    let pose = player.model_pose(partial);
    model.setup_anim(&PlayerRenderState {
        head_y_rot: voxel_core::math::wrap_degrees_f32(pose.head_rot - pose.body_rot),
        x_rot,
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + partial,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time: player.attack_anim(partial),
        ..Default::default()
    });
    voxel_render::player_renderer::player_vertices(
        model,
        &Placement {
            feet: [
                (feet.x - f64::from(view.origin[0])) as f32,
                (feet.y - f64::from(view.origin[1])) as f32,
                (feet.z - f64::from(view.origin[2])) as f32,
            ],
            body_rot: pose.body_rot,
            x_rot,
            swim_amount: pose.swim_amount,
            in_water: player.in_water(),
            visually_swimming: pose.pose == voxel_world::ClientEntityPose::Swimming,
            crouching: pose.crouching,
            light: view.light,
        },
    )
}
