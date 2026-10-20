use super::*;
use crate::equipment::arm_poses;
use photon_core::ids::ItemId;

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
pub fn packed_light(world: &photon_world::ClientWorld, at: Vec3) -> i32 {
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
) -> HandVertices {
    let partial = view.partial as f32;
    let swap = |item: ItemId| renderer.item_swap(item);
    let main_height = item_in_hand.inverse_arm_height(MAIN_HAND, partial, &swap);
    let off_height = item_in_hand.inverse_arm_height(OFF_HAND, partial, &swap);
    let hand_view = hand_view(player, view, main_height);
    let main_item = item_in_hand.item(MAIN_HAND);
    let off_item = item_in_hand.item(OFF_HAND);
    let hand = if main_item.is_none() {
        photon_render::entity::player_renderer::hand_vertices(model, &hand_view)
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
        let drawn = photon_render::entity::player_renderer::held_item_vertices(
            &hand_view, layers, right, attack, height,
        );
        held_layers[usize::from(!right)].extend_with_glint(stack, drawn, &mut held);
    }
    let glint = held_layers
        .iter_mut()
        .flat_map(|cache| cache.take_glint())
        .collect();
    HandVertices { hand, held, glint }
}

/// The first-person view's triangles: the bare arm, the held items, and the items' glints.
pub struct HandVertices {
    pub hand: Vec<Vertex>,
    pub held: Vec<Vertex>,
    pub glint: Vec<Vertex>,
}

/// The pose inputs of the first-person view this frame.
fn hand_view(player: &LocalPlayer, view: FrameView, inverse_arm_height: f32) -> HandView {
    let partial = view.partial as f32;
    let (walked, bob) = player.view_bob(partial);
    // `bobView` off leaves the hand without the sway.
    let bob = if view.view_bobbing { bob } else { 0.0 };
    let (x_bob, y_bob) = player.hand_bob(partial);
    let (view_y_rot, view_x_rot) = player.rotation();
    HandView {
        bob_walked: walked,
        bob,
        view_x_rot,
        view_y_rot,
        x_bob,
        y_bob,
        inverse_arm_height,
        attack: player.attack_anim(partial),
        light: view.light,
        skin: view.skin,
        hurt: {
            let (hurt_time, hurt_duration, hurt_dir, dying, death_time) = player.hurt_view(partial);
            photon_render::camera::HurtView {
                hurt_time,
                hurt_duration,
                hurt_dir,
                dying,
                death_time,
                tilt_strength: view.damage_tilt,
            }
        },
    }
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
        photon_world::ClientPlayerGameMode::Adventure
            | photon_world::ClientPlayerGameMode::Spectator
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
    let shape = photon_world::outline_shape(state, [x, y, z]);
    photon_render::block::outline::outline_vertices(
        &shape,
        view.relative([x, y, z]),
        eye,
        photon_render::block::outline::OUTLINE_COLOUR,
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
        let (quads, offset) = renderer.block_quads(state, [x, y, z]);
        breaking.extend(photon_render::block::breaking::breaking_vertices(
            &quads,
            offset,
            view.relative([x, y, z]),
            stage,
        ));
    }
    breaking
}

/// The poses of the local player's arms and the arm that swings: the main arm is the right, and
/// the off hand's is the one that swings when it acted.
pub(super) fn local_arms(
    session: &PlaySession,
    player: &LocalPlayer,
    stacks: (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    partial: f32,
) -> (arm_poses::ArmState, Arm) {
    let arms = arm_poses::arm_state_of(
        session,
        stacks,
        &arm_poses::ArmSource {
            main_arm_left: false,
            swinging: player.is_swinging(),
            using_item: player.is_using_item(),
            used_off_hand: player.used_item_is_off_hand(),
            ticks_using_item: player.ticks_using_item(partial),
        },
    );
    let attack_arm = if player.swinging_off_hand() {
        Arm::Left
    } else {
        Arm::Right
    };
    (arms, attack_arm)
}

/// The player's own model where it stands, posed for the frame.
pub fn player_model(
    (session, player): (&PlaySession, &LocalPlayer),
    model: &mut PlayerModel,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    stacks: (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    (armor, worn, textures): (
        &mut Armor,
        Worn,
        &photon_render::entity::textures::EntityTextures,
    ),
) -> Vec<Vertex> {
    let feet = player.interpolated_position(view.partial);
    let partial = view.partial as f32;
    let (_, x_rot) = player.rotation();
    let pose = player.model_pose(partial);
    let attack_time = player.attack_anim(partial);
    let (arms, attack_arm) = local_arms(session, player, stacks, partial);
    let arm_poses = (arms.right, arms.left);
    let spears = arm_poses::stacks_spear_view(
        stacks,
        arm_poses::SpearRest {
            main_arm: Arm::Right,
            attack: (attack_time, attack_arm),
            arm_poses,
            using: (arms.using_item, arms.use_main_hand, arms.ticks_using_item),
            hit_feedback: 0.0,
        },
    );
    let state = PlayerRenderState {
        head_y_rot: photon_core::math::wrap_degrees_f32(pose.head_rot - pose.body_rot),
        x_rot,
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + partial,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time,
        attack_arm,
        right_arm_pose: arm_poses.0,
        left_arm_pose: arm_poses.1,
        is_using_item: arms.using_item,
        use_item_main_hand: arms.use_main_hand,
        ticks_using_item: arms.ticks_using_item,
        max_crossbow_charge_duration: arms.max_crossbow_charge,
        swing_animation: spears.swing_animation,
        right_spear: spears.right,
        left_spear: spears.left,
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
        visually_swimming: pose.pose == photon_world::ClientEntityPose::Swimming,
        crouching: pose.crouching,
        light: view.light,
        skin: view.skin,
        death_time: 0.0,
        red_overlay: false,
        fall_flying: None,
        auto_spin: None,
    };
    let hands = Hands {
        base: &photon_render::entity::player_renderer::placement_pose(&placement),
        root: &model.root,
        hand_shift: if model.slim { 0.5 } else { 0.0 },
        baby: false,
        light: placement.light,
        right: stacks.0,
        left: stacks.1,
        head: worn.head_item(armor),
        head_transforms: HEAD_TRANSFORMS,
        eye: None,
        motions: spears.motions,
    };
    hands.collect(held);
    let skull = hands.skull();
    let mut vertices = photon_render::entity::player_renderer::player_vertices(model, &placement);
    vertices.extend(armor.skull_vertices(
        skull,
        pose.walk_position,
        textures,
        (placement.light, false),
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
        (false, pose.crouching, player.elytra_rotations(partial)),
        textures,
        &|model, rect, colour| {
            photon_render::entity::player_renderer::layer_vertices(model, &placement, rect, colour)
        },
    ));
    vertices
}

/// The model state of a remote player: its arms and what they use, its riding and gliding, and
/// how it walks, swims and swings.
pub(super) fn remote_render_state(
    session: &PlaySession,
    (entity, snapshot): (&photon_world::ClientEntity, &photon_world::EntitySnapshot),
    partial: f32,
) -> (
    PlayerRenderState,
    photon_render::entity::held_item::SpearView,
) {
    let world = session.world();
    let crouching = snapshot.pose == photon_world::ClientEntityPose::Crouching;
    let main_left = entity.main_arm_is_left();
    let arms = arm_poses::arm_state(session, entity, partial);
    let attack_arm = if snapshot.attack_arm_left {
        Arm::Left
    } else {
        Arm::Right
    };
    let spears = arm_poses::entity_spear_view(
        entity,
        (snapshot.attack_time, attack_arm),
        (arms.right, arms.left),
        (world.game_time(), partial),
    );
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
    let state = PlayerRenderState {
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
        head_y_rot: photon_core::math::wrap_degrees_f32(snapshot.head_yaw - snapshot.body_yaw),
        x_rot: snapshot.pitch,
        walk_position: snapshot.walk_position,
        walk_speed: snapshot.walk_speed,
        age_in_ticks: entity.tick_count() as f32 + partial,
        swim_amount: snapshot.swim_amount,
        crouching,
        attack_time: snapshot.attack_time,
        attack_arm,
        swing_animation: spears.swing_animation,
        right_spear: spears.right,
        left_spear: spears.left,
        ..Default::default()
    };
    (state, spears)
}
