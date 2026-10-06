//! What is drawn over the world's blocks from the player's side: the hands
//! and what they hold, the aimed block's outline, the cracks in blocks being
//! broken, and the player's own model.

use voxel_core::math::Vec3;
use voxel_game::game_mode::{Aim, MAIN_HAND, OFF_HAND};
use voxel_game::item_in_hand::ItemInHand;
use voxel_net::connection::PlaySession;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::outline::LineVertex;
use voxel_render::player_model::{Arm, ArmPose, PlayerModel, PlayerRenderState};
use voxel_render::player_renderer::{HandView, Placement};
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::local_player::LocalPlayer;
use voxel_world::{ClientItemStackState, EquipmentSlot};

use crate::armor::{Armor, Worn};
use crate::held_items::{HEAD_TRANSFORMS, Hands, HeldItem};
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
    /// Where the camera is, from that origin.
    pub camera: [f32; 3],
    /// The camera's yaw and pitch, in degrees.
    pub rotation: (f32, f32),
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

/// A packed light with its block light raised to fifteen (what an entity whose `getBlockLightLevel`
/// is fifteen is lit by), its sky light kept.
pub fn full_block_light(light: i32) -> i32 {
    (light & !0xFFFF) | (15 << 4)
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
pub fn player_model(
    player: &LocalPlayer,
    model: &mut PlayerModel,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    stacks: (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    (armor, worn, textures): (
        &mut Armor,
        Worn,
        &voxel_render::entity_textures::EntityTextures,
    ),
) -> Vec<Vertex> {
    let feet = player.interpolated_position(view.partial);
    let partial = view.partial as f32;
    let (_, x_rot) = player.rotation();
    let pose = player.model_pose(partial);
    let state = PlayerRenderState {
        head_y_rot: voxel_core::math::wrap_degrees_f32(pose.head_rot - pose.body_rot),
        x_rot,
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + partial,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time: player.attack_anim(partial),
        right_arm_pose: held_arm_pose(stacks.0),
        left_arm_pose: held_arm_pose(stacks.1),
        ..Default::default()
    };
    model.setup_anim(&state);
    let placement = Placement {
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
        death_time: 0.0,
        red_overlay: false,
    };
    Hands {
        base: &voxel_render::player_renderer::placement_pose(&placement),
        root: &model.root,
        hand_shift: if model.slim { 0.5 } else { 0.0 },
        baby: false,
        light: placement.light,
        right: stacks.0,
        left: stacks.1,
        head: worn.head_item(armor),
        head_transforms: HEAD_TRANSFORMS,
    }
    .collect(held);
    let mut vertices = voxel_render::player_renderer::player_vertices(model, &placement);
    vertices.extend(wear(
        armor,
        worn,
        model.slim,
        textures,
        (&state, &placement),
    ));
    vertices.extend(armor.wings(
        worn.chest,
        (false, pose.crouching, player.elytra_rotations(partial)),
        textures,
        &|model, rect, colour| {
            voxel_render::player_renderer::layer_vertices(model, &placement, rect, colour)
        },
    ));
    vertices
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

/// The model state of a remote player: its arms and what they use, its riding and gliding, and
/// how it walks, swims and swings.
fn remote_render_state(
    session: &PlaySession,
    (entity, snapshot): (&voxel_world::ClientEntity, &voxel_world::EntitySnapshot),
    partial: f32,
) -> PlayerRenderState {
    let world = session.world();
    let crouching = snapshot.pose == voxel_world::ClientEntityPose::Crouching;
    let main_left = entity.main_arm_is_left();
    let arms = crate::arm_poses::arm_state(session, entity, partial);
    // `HumanoidMobRenderer.extractHumanoidRenderState`: a glider's limbs swing slower the
    // faster it flies.
    let gliding = entity.is_fall_flying();
    let speed_value = if gliding {
        let (mx, my, mz) = entity.motion();
        let squared = (mx * mx + my * my + mz * mz) as f32 / 0.2;
        (squared * (squared * squared)).max(1.0)
    } else {
        1.0
    };
    PlayerRenderState {
        right_arm_pose: arms.right,
        left_arm_pose: arms.left,
        is_using_item: arms.using_item,
        use_item_main_hand: arms.use_main_hand,
        ticks_using_item: arms.ticks_using_item,
        max_crossbow_charge_duration: arms.max_crossbow_charge,
        is_passenger: world.entity_vehicle_id(entity.id()).is_some(),
        is_fall_flying: gliding,
        speed_value,
        main_arm: if main_left { Arm::Left } else { Arm::Right },
        head_y_rot: voxel_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
        x_rot: snapshot.pitch,
        walk_position: snapshot.walk_position,
        walk_speed: snapshot.walk_speed,
        age_in_ticks: entity.tick_count() as f32 + partial,
        swim_amount: snapshot.swim_amount,
        crouching,
        attack_time: snapshot.attack_time,
        attack_arm: if snapshot.attack_arm_left {
            Arm::Left
        } else {
            Arm::Right
        },
        ..Default::default()
    }
}

/// `LivingEntityRenderer` for the other players: each in its default skin
/// (a profile's own would be downloaded), posed from where it is between
/// two ticks, as the first-person renderer is for the local player's own
/// model.
pub fn remote_players(
    session: &PlaySession,
    (models, armor): (&mut RemoteModels, &mut Armor),
    textures: &voxel_render::entity_textures::EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
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
        let (right, left) = (
            entity.item_held_by_arm(false),
            entity.item_held_by_arm(true),
        );
        let state = remote_render_state(session, (entity, &snapshot), partial);
        model.setup_anim(&state);
        let (x, y, z) = snapshot.position;
        let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
        let placement = Placement {
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
            death_time: snapshot.death_time,
            red_overlay: snapshot.red_overlay,
        };
        let worn = Worn {
            head: entity.equipment(EquipmentSlot::Head),
            chest: entity.equipment(EquipmentSlot::Chest),
            legs: entity.equipment(EquipmentSlot::Legs),
            feet: entity.equipment(EquipmentSlot::Feet),
        };
        Hands {
            base: &voxel_render::player_renderer::placement_pose(&placement),
            root: &model.root,
            hand_shift: if model.slim { 0.5 } else { 0.0 },
            baby: false,
            light: placement.light,
            right,
            left,
            head: worn.head_item(armor),
            head_transforms: HEAD_TRANSFORMS,
        }
        .collect(held);
        vertices.extend(voxel_render::player_renderer::player_vertices(
            model, &placement,
        ));
        vertices.extend(wear(
            armor,
            worn,
            model.slim,
            textures,
            (&state, &placement),
        ));
        vertices.extend(armor.wings(
            worn.chest,
            (false, crouching, entity.elytra_rotations(partial)),
            textures,
            &|model, rect, colour| {
                voxel_render::player_renderer::layer_vertices(model, &placement, rect, colour)
            },
        ));
    }
    vertices
}

/// What a player wears, drawn over its model: the armor models of its build,
/// posed as it is.
fn wear(
    armor: &mut Armor,
    worn: Worn,
    slim: bool,
    textures: &voxel_render::entity_textures::EntityTextures,
    (state, placement): (&PlayerRenderState, &Placement),
) -> Vec<Vertex> {
    let prefix = if slim {
        "minecraft:player_slim"
    } else {
        "minecraft:player"
    };
    armor.vertices(
        worn,
        (prefix, false),
        textures,
        &|model| model.setup_humanoid(state),
        &|model, rect, colour| {
            voxel_render::player_renderer::layer_vertices(model, placement, rect, colour)
        },
    )
}

/// `AvatarRenderer.getArmPose` of an arm that is not in use: a held stack
/// raises the arm a little.
fn held_arm_pose(stack: Option<&voxel_world::ClientItemStackState>) -> ArmPose {
    if stack.is_some() {
        ArmPose::Item
    } else {
        ArmPose::Empty
    }
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

/// The path (under `textures/`) of the experience orbs' texture.
pub const ORB_TEXTURE: &str = "entity/experience/experience_orb";

/// `ExperienceOrbRenderer` for the orbs the server added: a quad each, turned
/// to the camera, with the icon its value picks.
pub fn experience_orbs(
    session: &PlaySession,
    textures: &voxel_render::entity_textures::EntityTextures,
    camera: &TagCamera,
    view: FrameView,
) -> Vec<Vertex> {
    let Some(rect) = textures.rect(ORB_TEXTURE) else {
        return Vec::new();
    };
    let Some(orb_type) = voxel_data::entity_type::id("minecraft:experience_orb") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let orientation =
        voxel_render::name_tag::camera_orientation(camera.rotation.0, camera.rotation.1);
    let mut vertices = Vec::new();
    for entity in world.entities().filter(|entity| entity.type_id == orb_type) {
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        // `getBlockLightLevel`: seven brighter than the block it is in.
        let light = packed_light(world, Vec3::new(x, y + 0.425, z));
        let block = ((light >> 4 & 15) + 7).min(15);
        let light = (light & !(15 << 4)) | block << 4;
        let corners = voxel_render::xp_orb::corners(
            voxel_render::xp_orb::icon(entity.orb_value().unwrap_or(0)),
            entity.tick_count() as f32 + partial,
            [
                (x - camera.position.x) as f32,
                (y - camera.position.y) as f32,
                (z - camera.position.z) as f32,
            ],
            &orientation,
        );
        vertices.extend(voxel_render::xp_orb::vertices(
            &corners,
            rect,
            camera.relative,
            light,
        ));
    }
    vertices
}

/// What casts a shadow: where it is drawn and where it ticked (the camera's
/// distance is measured to the latter), and its renderer's radius and strength.
struct Caster {
    drawn: (f64, f64, f64),
    ticked: (f64, f64, f64),
    radius: f32,
    strength: f32,
}

/// The shadow texture's path (under `textures/`).
pub const SHADOW_TEXTURE: &str = "misc/shadow";

/// `EntityRenderer.extractShadow` and `ShadowFeatureRenderer` for the
/// entities the frame draws: the shadow each casts on the blocks below it.
/// `shadowRadius` and `shadowStrength` are the renderers': a mob's is its
/// age's share of 0.7, a player's 0.5, a dropped item's and an orb's 0.15
/// at three quarters of the strength. `local` is the player's own place, when
/// the camera shows them.
pub fn entity_shadows(
    session: &PlaySession,
    textures: &voxel_render::entity_textures::EntityTextures,
    camera: &TagCamera,
    local: Option<Vec3>,
    view: FrameView,
) -> Vec<Vertex> {
    use voxel_world::ShadowInput;
    let Some(rect) = textures.rect(SHADOW_TEXTURE) else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let id = |name: &str| voxel_data::entity_type::id(name);
    let (pig, cow, item, orb) = (
        id("minecraft:pig"),
        id("minecraft:cow"),
        id("minecraft:item"),
        id("minecraft:experience_orb"),
    );
    let mut casters: Vec<Caster> = Vec::new();
    for entity in world.entities() {
        let type_id = Some(entity.type_id);
        let (radius, strength) = if entity.is_player() {
            if entity.is_spectator() || entity.is_invisible() {
                continue;
            }
            (0.5, 1.0)
        } else if type_id == pig || type_id == cow {
            (if entity.is_baby() { 0.35 } else { 0.7 }, 1.0)
        } else if entity.is_cube_mob() {
            // `AbstractCubeMobRenderer.getShadowRadius`.
            (entity.cube_size() as f32 * 0.25, 1.0)
        } else if type_id == item || type_id == orb {
            (0.15, 0.75)
        } else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        casters.push(Caster {
            drawn: snapshot.position,
            ticked: entity.position(),
            radius,
            strength,
        });
    }
    if let Some(at) = local {
        casters.push(Caster {
            drawn: (at.x, at.y, at.z),
            ticked: (at.x, at.y, at.z),
            radius: 0.5,
            strength: 1.0,
        });
    }
    let mut vertices = Vec::new();
    for Caster {
        drawn,
        ticked,
        radius,
        strength,
    } in casters
    {
        let dx = camera.position.x - ticked.0;
        let dy = camera.position.y - ticked.1;
        let dz = camera.position.z - ticked.2;
        let radius = radius.min(32.0);
        let pieces = world.entity_shadow_pieces(ShadowInput {
            position: drawn,
            radius,
            strength,
            distance_sq: dx * dx + dy * dy + dz * dz,
            // The sky's dimming (`gameplay/sky_light_level`) is not carried.
            sky_darken: 0,
        });
        let translation = [
            (drawn.0 - camera.position.x) as f32,
            (drawn.1 - camera.position.y) as f32,
            (drawn.2 - camera.position.z) as f32,
        ];
        vertices.extend(voxel_render::shadow::vertices(
            &pieces,
            radius,
            translation,
            rect,
            camera.relative,
        ));
    }
    vertices
}
