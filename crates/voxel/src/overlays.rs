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
use crate::view::eye_position;
use voxel_game::pick::pick;

/// Where the frame is drawn from: how far between ticks, the light at the
/// eyes (packed block and sky light), and the point meshes are measured
/// from.
#[derive(Clone, Copy)]
pub struct FrameView {
    pub partial: f64,
    pub light: i32,
    pub origin: [i32; 3],
    /// Where the skin the local player wears is in the entity textures.
    pub skin: voxel_render::entity_textures::TextureRect,
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
        skin: view.skin,
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
            skin: view.skin,
        },
    )
}

/// The player models the frame draws for the players the server added: one
/// wide-armed and one slim, which each remote player is posed in turn.
pub struct RemoteModels {
    wide: PlayerModel,
    slim: PlayerModel,
}

impl RemoteModels {
    pub fn new() -> Self {
        Self {
            wide: PlayerModel::new(false),
            slim: PlayerModel::new(true),
        }
    }
}

impl Default for RemoteModels {
    fn default() -> Self {
        Self::new()
    }
}

/// `LivingEntityRenderer` for the other players: each in its default skin
/// (a profile's own would be downloaded), posed from where it is between
/// two ticks, as the first-person renderer is for the local player's own
/// model.
pub fn remote_players(
    session: &PlaySession,
    models: &mut RemoteModels,
    textures: &voxel_render::entity_textures::EntityTextures,
    view: FrameView,
) -> Vec<Vertex> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    for entity in world.entities() {
        // A player who only watches, or who is invisible, is not drawn.
        if !entity.is_player() || entity.is_spectator() || entity.is_invisible() {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let skin = voxel_render::player_renderer::default_skin(entity.uuid);
        let Some(rect) = textures.rect(skin.texture) else {
            continue;
        };
        let model = if skin.slim {
            &mut models.slim
        } else {
            &mut models.wide
        };
        let crouching = snapshot.pose == voxel_world::ClientEntityPose::Crouching;
        model.setup_anim(&PlayerRenderState {
            head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
            x_rot: snapshot.pitch,
            walk_position: snapshot.walk_position,
            walk_speed: snapshot.walk_speed,
            age_in_ticks: entity.tick_count() as f32 + partial,
            swim_amount: snapshot.swim_amount,
            crouching,
            ..Default::default()
        });
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
        vertices.extend(voxel_render::player_renderer::player_vertices(
            model,
            &Placement {
                feet: [
                    (x - f64::from(view.origin[0])) as f32,
                    (y - f64::from(view.origin[1])) as f32,
                    (z - f64::from(view.origin[2])) as f32,
                ],
                body_rot: snapshot.body_yaw,
                x_rot: snapshot.pitch,
                swim_amount: snapshot.swim_amount,
                in_water: false,
                visually_swimming: snapshot.pose == voxel_world::ClientEntityPose::Swimming,
                crouching,
                light: packed_light(world, eye),
                skin: rect,
            },
        ));
    }
    vertices
}

/// Where the name tags are seen from.
pub struct TagCamera {
    pub position: Vec3,
    /// The camera relative to the mesh origin.
    pub relative: [f32; 3],
    /// The camera's yaw and pitch.
    pub rotation: (f32, f32),
    /// The local player watches from the spectator's seat.
    pub spectating: bool,
}

/// `EntityRenderer.submitNameDisplay` for the other players: the text seen
/// through blocks, and the text tested against the depth, each as triangles
/// from the mesh origin. The background is the options' default for chat
/// only, as the option that changes it is not carried.
pub fn name_tags(
    session: &PlaySession,
    (font, atlas): (&voxel_render::gui::font::Font, &voxel_render::gui::GuiAtlas),
    camera: &TagCamera,
    view: FrameView,
) -> (Vec<Vertex>, Vec<Vertex>) {
    use voxel_render::name_tag::{DisplayMode, Viewer, camera_orientation, name_tag, submits};
    let world = session.world();
    let partial = view.partial as f32;
    let viewer = Viewer {
        camera_position: camera.position,
        spectator: camera.spectating,
        hud_hidden: false,
    };
    let orientation = camera_orientation(camera.rotation.0, camera.rotation.1);
    let (mut see_through, mut normal) = (Vec::new(), Vec::new());
    for entity in world.entities() {
        if !entity.is_player() || entity.is_spectator() {
            continue;
        }
        // A player the server did not tell of has no name to show.
        let Some(name) = session.player_name(entity.uuid) else {
            continue;
        };
        let Some(tag) = name_tag(entity, name, viewer, partial) else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let crouching = snapshot.pose == voxel_world::ClientEntityPose::Crouching;
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
        let placement = Vec3::new(
            x - camera.position.x,
            y - camera.position.y,
            z - camera.position.z,
        );
        for submit in submits(
            &tag,
            placement,
            font.width(&tag.text),
            packed_light(world, eye),
            &orientation,
            0.25,
        ) {
            let vertices = voxel_render::name_tag::submit_vertices(
                &submit,
                &tag.text,
                font,
                atlas,
                camera.relative,
            );
            match submit.mode {
                DisplayMode::SeeThrough => see_through.extend(vertices),
                DisplayMode::Normal => normal.extend(vertices),
            }
        }
    }
    (see_through, normal)
}
