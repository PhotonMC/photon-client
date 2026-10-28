use super::*;
use crate::equipment::arm_poses;
use crate::equipment::held_items::HandActions;
use photon_core::ids::ItemId;
use wear::body;

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
    item_in_hand.set_invisible(local.is_some_and(|state| state.is_invisible()));
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
/// empty, and each held stack in its hand, the main hand on the main arm's side. The
/// swing is the main hand's, since which arm swings is not tracked yet.
pub fn hands(
    player: &LocalPlayer,
    item_in_hand: &ItemInHand,
    (held_layers, cast): (&mut [ItemLayerCache; 2], [bool; 2]),
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    (model, view): (&mut PlayerModel, FrameView),
    mut head_skin: impl FnMut(&ClientItemStackState) -> Option<[f32; 4]>,
) -> HandVertices {
    let partial = view.partial as f32;
    let swap = |item: ItemId| renderer.item_swap(item);
    let main_height = item_in_hand.inverse_arm_height(MAIN_HAND, partial, &swap);
    let off_height = item_in_hand.inverse_arm_height(OFF_HAND, partial, &swap);
    let hand_view = hand_view(player, view, main_height);
    let main_item = item_in_hand.item(MAIN_HAND);
    let off_item = item_in_hand.item(OFF_HAND);
    if player.is_scoping() {
        return HandVertices::default();
    }
    let (show_main, show_off) = super::hand_use::selection(player, (main_item, off_item));
    // `ItemInHandRenderer`: an invisible player's bare arm is not drawn.
    let mut hand = if main_item.is_none() && show_main && !item_in_hand.invisible() {
        photon_render::entity::player_renderer::hand_vertices(model, &hand_view)
    } else {
        Vec::new()
    };
    let mut held = Vec::new();
    for (stack, right, attack, height) in [
        (
            main_item.filter(|_| show_main),
            true,
            hand_view.attack,
            main_height,
        ),
        (off_item.filter(|_| show_off), false, 0.0, off_height),
    ] {
        let Some(stack) = stack else {
            continue;
        };
        let cache = &mut held_layers[usize::from(!right)];
        let drawn = item_in_hand_vertices(
            (player, stack),
            (right, attack, height, cast[usize::from(!right)]),
            cache,
            (renderer, tints),
            (&hand_view, partial),
        );
        // A head with a profile is drawn in the skin it names, which the entity pictures hold.
        if let Some(rect) = head_skin(stack) {
            let light = photon_render::block::light::light_uv(hand_view.light);
            hand.extend(wear_skin(drawn, renderer, rect, light));
            continue;
        }
        held_layers[usize::from(!right)].extend_with_glint(stack, drawn, &mut held);
    }
    let glint = held_layers
        .iter_mut()
        .flat_map(|cache| cache.take_glint())
        .collect();
    HandVertices { hand, held, glint }
}

/// One held stack's triangles in the first-person view, in the hand's pose.
fn item_in_hand_vertices(
    (player, stack): (&LocalPlayer, &ClientItemStackState),
    (right, attack, height, cast): (bool, f32, f32, bool),
    cache: &mut ItemLayerCache,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    (hand_view, partial): (&HandView, f32),
) -> Vec<Vertex> {
    cache.set_cast(cast);
    // `ItemInHandRenderer`: the player's id plus the first-person context's ordinal.
    // The side the hand is on: the main hand on the main arm's.
    let arm_right = right != crate::overlays::local_main_arm_left();
    cache.set_seed(player.entity_id().0 + if arm_right { 4 } else { 3 });
    let ticks = player.use_ticks_of(right);
    let hit_feedback = player.ticks_since_kinetic_hit(partial);
    let charge = cache.charge_duration(stack);
    let motion = super::hand_use::motion(stack, (ticks, partial), (right, charge, hit_feedback));
    let layers = cache.get_in_use(stack, ticks, (renderer, tints));
    photon_render::entity::player_renderer::held_item_vertices(
        hand_view,
        layers,
        (arm_right, attack, motion),
        height,
    )
}

/// A head's triangles in the skin its profile names, which the hand's run lights per vertex as
/// it does the bare arm's.
fn wear_skin(
    mut drawn: Vec<Vertex>,
    renderer: &BlockRenderer,
    rect: [f32; 4],
    light: [f32; 2],
) -> Vec<Vertex> {
    crate::equipment::skinned_heads::wear_skin(&mut drawn, renderer, rect);
    for vertex in &mut drawn {
        vertex.light = light;
    }
    drawn
}

/// The first-person view's triangles: the bare arm, the held items, and the items' glints.
#[derive(Default)]
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
    let view_x_rot = player.rotation().1;
    let view_y_rot = player.view_y_rot(partial);
    let right_arm = !crate::overlays::local_main_arm_left();
    // `RIGHT_SLEEVE` is bit 3, `LEFT_SLEEVE` bit 2.
    let sleeve_bit = if right_arm { 3 } else { 2 };
    HandView {
        right_arm,
        sleeve: crate::overlays::local_hidden_layers() & (1 << sleeve_bit) == 0,
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
pub fn hud_fov(player: &LocalPlayer, effect_scale: f32) -> f32 {
    if player.is_eye_in_water() {
        // `fovEffectScale` lerps the narrowing in.
        70.0 * (1.0 + effect_scale * (0.857_142_87 - 1.0))
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
    (line_width, high_contrast): (f32, bool),
) -> (Vec<LineVertex>, usize) {
    if matches!(
        session.game_mode(),
        photon_world::ClientPlayerGameMode::Adventure
            | photon_world::ClientPlayerGameMode::Spectator
    ) {
        return (Vec::new(), 0);
    }
    let Aim::Block { x, y, z, .. } =
        pick(session.world(), player, eye_position(player, view.partial))
    else {
        return (Vec::new(), 0);
    };
    let Some(state) = session.world().block_state_at(x, y, z) else {
        return (Vec::new(), 0);
    };
    let shape = photon_world::outline_shape(state, [x, y, z]);
    use photon_render::block::outline::{
        HIGH_CONTRAST_BACKING, HIGH_CONTRAST_COLOUR, OUTLINE_COLOUR, outline_vertices,
    };
    let block = view.relative([x, y, z]);
    // `submitBlockOutline`: the black backing first, then the line over it.
    let (backing, colour) = if high_contrast {
        (
            outline_vertices(
                &shape,
                block,
                eye,
                HIGH_CONTRAST_BACKING.0,
                HIGH_CONTRAST_BACKING.1,
            ),
            HIGH_CONTRAST_COLOUR,
        )
    } else {
        (Vec::new(), OUTLINE_COLOUR)
    };
    let backing_len = backing.len();
    let mut lines = backing;
    lines.extend(outline_vertices(&shape, block, eye, colour, line_width));
    (lines, backing_len)
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

/// `isUpsideDown` for the local player (the cape shows and the profile is named Dinnerbone or
/// Grumm): the height of its box, its pitch as the model sees it, and the sign its head turn
/// takes (`extractRenderState` mirrors both).
fn local_flip(session: &PlaySession, player: &LocalPlayer) -> (Option<f32>, f32, f32) {
    let (_, x_rot) = player.rotation();
    let named = matches!(session.local_name(), "Dinnerbone" | "Grumm");
    let cape = session
        .world()
        .local_player()
        .is_some_and(|state| state.model_customisation() & 1 != 0);
    if named && cape {
        (Some(player.bb_height()), -x_rot, -1.0)
    } else {
        (None, x_rot, 1.0)
    }
}

/// The local player's riptide spin, by the server's flag: its age, which the spin turns with.
fn local_spin(session: &PlaySession, player: &LocalPlayer, partial: f32) -> Option<f32> {
    let spinning = session
        .world()
        .local_player()
        .is_some_and(|state| state.is_auto_spin_attack());
    spinning.then(|| player.tick_count() as f32 + partial)
}

/// The lying of the local player while it sleeps.
fn local_sleep(
    session: &PlaySession,
    player: &LocalPlayer,
    body_rot: f32,
) -> Option<photon_render::entity::player_renderer::Sleep> {
    let state = session.world().local_player()?;
    (player.pose() == photon_world::ClientEntityPose::Sleeping)
        .then(|| super::remote_placement::in_bed(session.world(), state.sleeping_pos(), body_rot))
}

/// Where the local player's model stands in the frame.
fn local_placement(
    (session, player): (&PlaySession, &LocalPlayer),
    (view, x_rot, upside_down): (FrameView, f32, Option<f32>),
    (pose, glide): (&photon_world::local_player::ModelPose, Option<Glide>),
) -> Placement {
    let feet = player.interpolated_position(view.partial);
    Placement {
        feet: [
            (feet.x - f64::from(view.origin[0])) as f32,
            (feet.y - f64::from(view.origin[1])) as f32,
            (feet.z - f64::from(view.origin[2])) as f32,
        ],
        body_rot: pose.body_rot,
        x_rot,
        swim_amount: pose.swim_amount,
        in_water: player.in_water(),
        visually_swimming: player.is_visually_swimming(),
        crouching: pose.crouching,
        light: view.light,
        skin: view.skin,
        death_time: 0.0,
        red_overlay: false,
        fall_flying: glide.map(|glide| glide.flight),
        auto_spin: local_spin(session, player, view.partial as f32),
        upside_down,
        sleeping: local_sleep(session, player, pose.body_rot),
    }
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
    invisible: bool,
) -> Vec<Vertex> {
    let partial = view.partial as f32;
    let (flip, x_rot, side) = local_flip(session, player);
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
            hit_feedback: player.ticks_since_kinetic_hit(partial),
        },
    );
    let glide = local_glide(player, pose.head_rot, partial);
    let facing = (pose, x_rot, side);
    let state = local_state::local_render_state(
        (player, &facing),
        (&arms, (attack_time, attack_arm), glide),
        (&spears, partial),
    );
    model.setup_anim(&state);
    let placement = local_placement((session, player), (view, x_rot, flip), (&pose, glide));
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
        motions: HandActions::of_local(spears.motions, (session, player), stacks.0),
    };
    hands.collect_outlined(held, crate::mobs::outline::local_holder(session));
    let mut vertices = local_body(session, (model, &placement), invisible);
    let lit = (placement.light, false);
    let skull = armor.skull_vertices(hands.skull(), pose.walk_position, textures, lit);
    vertices.extend(glowing_local(session, skull));
    let dressed = wear(armor, worn, model.slim, textures, (&state, &placement));
    vertices.extend(glowing_local(session, dressed));
    let hang = cape::own_hang(session, player, (view, invisible));
    let spread = player.elytra_rotations(partial);
    let back = cape::back(
        armor,
        (worn.chest, &hang, spread),
        (model, &placement),
        (textures, partial),
    );
    vertices.extend(glowing_local(session, back));
    vertices
}

/// The triangles of the local player's own model, which join the outline when it glows
/// (colour by its profile name).
fn glowing_local(session: &PlaySession, triangles: Vec<Vertex>) -> Vec<Vertex> {
    if let Some(name) = crate::mobs::outline::local_holder(session) {
        crate::mobs::outline::record_holder(name, triangles.clone());
    }
    triangles
}

/// The local player's body: an invisible glowing player is seen by its outline alone.
fn local_body(
    session: &PlaySession,
    (model, placement): (&PlayerModel, &Placement),
    invisible: bool,
) -> Vec<Vertex> {
    let shown = glowing_local(session, body(model, placement, None));
    if invisible { Vec::new() } else { shown }
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
    let gliding = entity.is_fall_flying();
    let speed_value = if gliding {
        arm_poses::glider_speed_value(entity.motion())
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
        hidden_layers: !entity.model_parts(),
        ..Default::default()
    };
    (state, spears)
}

/// What the local player's glide gives its model.
#[derive(Clone, Copy)]
pub(super) struct Glide {
    pub(super) speed_value: f32,
    flight: photon_render::entity::player_renderer::FallFlying,
}

/// The glide of the player's own model, when it glides.
fn local_glide(player: &LocalPlayer, head_rot: f32, partial: f32) -> Option<Glide> {
    if !player.is_fall_flying() {
        return None;
    }
    let delta = player.delta_movement();
    let motion = (delta.x, delta.y, delta.z);
    let (_, x_rot) = player.rotation();
    Some(Glide {
        speed_value: arm_poses::glider_speed_value(motion),
        flight: arm_poses::flight(
            player.fall_flying_ticks(),
            (player.motion_on_previous_tick(), motion),
            (x_rot, head_rot),
            partial,
        )?,
    })
}
