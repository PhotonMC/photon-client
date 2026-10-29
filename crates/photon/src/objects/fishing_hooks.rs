//! The fishing bobbers (`FishingHookRenderer`): the bobber's sprite, facing the camera, and the
//! line from it to the hand of the player who cast it. A bobber whose owner is not a player
//! known to this client is not drawn.

use photon_core::ids::EntityId;
use photon_core::math::{Vec3, mth_lerp_f32, mth_sin_f64};
use photon_data::item::classes::ItemClass;
use photon_net::connection::PlaySession;
use photon_render::block::outline::LineVertex;
use photon_render::entity::name_tag::camera_orientation;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::fishing::{
    Holder, first_person_hand_position, hand_position, line_offset, near_plane, string_strips,
    string_vertices,
};
use photon_render::objects::projectile::{fishing_hook_quad, sprite_quad_triangles};
use photon_render::scene::Vertex;
use photon_world::local_player::LocalPlayer;
use photon_world::{ClientItemStackState, EntityView, EquipmentSlot};

use crate::overlays::{FrameView, packed_light};
use crate::view::{CameraType, eye_position};

const HOOK: &str = "entity/fishing/fishing_hook";

/// The paths (under `textures/`) of the textures the bobbers wear.
pub fn textures() -> impl Iterator<Item = &'static str> {
    [HOOK].into_iter()
}

/// What the frame knows that the hand's place depends on.
pub struct HookView<'a> {
    pub view: FrameView,
    pub camera_type: CameraType,
    /// `Options.fov`, the slider's whole degrees.
    pub option_fov: i32,
    pub aspect: f32,
    pub line_width: f32,
    /// The stack in the local player's main hand.
    pub main_hand: Option<&'a ClientItemStackState>,
}

/// `FishingRodItem`: whether the stack is a rod.
pub(crate) fn is_rod(stack: Option<&ClientItemStackState>) -> bool {
    stack.is_some_and(|stack| {
        photon_world::block::rules::item(stack.item_registry_id())
            .is_some_and(|item| item.class == ItemClass::FishingRodItem)
    })
}

/// `FishingHookRenderer.getHoldingArm`: the main arm for a rod in the main hand, else the other.
fn holds_with_right_arm(main_arm_left: bool, rod_in_main_hand: bool) -> bool {
    if rod_in_main_hand {
        !main_arm_left
    } else {
        main_arm_left
    }
}

/// `FishingRodCast`: which of a player's arms hold a rod whose bobber is out, the right arm's
/// first. `player.fishing` is the bobber that names the player its owner.
pub fn casting_arms(
    session: &PlaySession,
    owner: EntityId,
    (main_arm_left, rod_in_main_hand): (bool, bool),
) -> [bool; 2] {
    let out = session.world().entities().all().any(|entity| {
        entity.data == owner.0
            && photon_data::entity_type::name(entity.type_id) == Some("minecraft:fishing_bobber")
    });
    let right = holds_with_right_arm(main_arm_left, rod_in_main_hand);
    [out && right, out && !right]
}

/// [`casting_arms`] of an entity.
pub fn entity_casts(session: &PlaySession, entity: EntityView<'_>) -> [bool; 2] {
    let rod = is_rod(entity.equipment(EquipmentSlot::MainHand));
    casting_arms(session, entity.id(), (entity.main_arm_is_left(), rod))
}

/// [`casting_arms`] of the local player, whose main arm is the right (`ClientInformation`).
pub fn local_casts(
    session: &PlaySession,
    player: &LocalPlayer,
    main_hand: Option<&ClientItemStackState>,
) -> [bool; 2] {
    casting_arms(session, player.entity_id(), (false, is_rod(main_hand)))
}

/// `Mth.sin(Mth.sqrt(attackAnim) * PI)`.
fn swing_sine(attack_anim: f32) -> f32 {
    let root = f64::from(attack_anim).sqrt() as f32;
    mth_sin_f64(f64::from(root * std::f64::consts::PI as f32))
}

/// The bobbers and their lines: the sprites for the entity shader, and the lines for the line shader.
pub fn fishing_hooks(
    session: &PlaySession,
    (player, textures): (&LocalPlayer, &EntityTextures),
    hook_view: &HookView,
) -> (Vec<Vertex>, Vec<LineVertex>) {
    let world = session.world();
    let view = hook_view.view;
    let partial = view.partial as f32;
    let orientation = camera_orientation(view.rotation.0, view.rotation.1);
    let (mut sprites, mut lines) = (Vec::new(), Vec::new());
    for entity in world.entities().drawn() {
        if photon_data::entity_type::name(entity.type_id) != Some("minecraft:fishing_bobber") {
            continue;
        }
        let (Some(snapshot), Some(rect)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(HOOK),
        ) else {
            continue;
        };
        // `ClientboundAddEntityPacket.getData`: the owner's id.
        let owner_id = EntityId(entity.data);
        let (x, y, z) = snapshot.position;
        let hook = (x, y, z);
        let hand = if owner_id == player.entity_id() {
            local_hand(player, hook_view)
        } else {
            let Some(owner) = world
                .entities()
                .all()
                .find(|owner| owner.id() == owner_id && owner.is_player())
            else {
                continue;
            };
            let Some(owner_snapshot) = world.entity_snapshot(owner_id, partial) else {
                continue;
            };
            let rod = is_rod(owner.equipment(EquipmentSlot::MainHand));
            let (old, current) = (owner.previous_body_y_rotation(), owner.body_y_rotation());
            let (ox, oy, oz) = owner_snapshot.position;
            let holder = Holder {
                eye: [ox, oy + f64::from(owner.eye_height()), oz],
                body_rot: mth_lerp_f32(partial, old, current),
                crouching: owner.is_crouching(),
                scale: owner.scale(),
                right_arm: holds_with_right_arm(owner.main_arm_is_left(), rod),
            };
            hand_position(&holder)
        };
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let light = packed_light(world, Vec3::new(x, y + f64::from(entity.eye_height()), z));
        let quad = fishing_hook_quad(&base, &orientation);
        // `entityCutoutCull`: the quad is turned to face the camera, so its front is what shows.
        sprites.extend(sprite_quad_triangles(
            &quad,
            rect,
            photon_render::block::light::light_uv(light),
        ));
        let offset = line_offset(hand, hook);
        lines.extend(string_strips(
            &string_vertices(&base, offset),
            view.camera,
            hook_view.line_width,
        ));
    }
    (sprites, lines)
}

/// Where the local player's hand is: by the camera's near plane in first person, else by the body.
fn local_hand(player: &LocalPlayer, hook_view: &HookView) -> [f64; 3] {
    let partial = hook_view.view.partial;
    let eye = eye_position(player, partial);
    let rod = is_rod(hook_view.main_hand);
    // The local player's main arm is the right (`ClientInformation`).
    let right_arm = holds_with_right_arm(false, rod);
    if hook_view.camera_type == CameraType::FirstPerson {
        let fov = hook_view.option_fov as f32;
        let plane = near_plane(
            crate::view::camera_vectors(hook_view.view.rotation),
            fov,
            f64::from(hook_view.aspect),
        );
        let swing = swing_sine(player.attack_anim(partial as f32));
        first_person_hand_position([eye.x, eye.y, eye.z], &plane, (fov, right_arm, swing))
    } else {
        let (old, current) = player.body_rotations();
        hand_position(&Holder {
            eye: [eye.x, eye.y, eye.z],
            body_rot: mth_lerp_f32(partial as f32, old, current),
            crouching: player.model_pose(partial as f32).crouching,
            scale: 1.0,
            right_arm,
        })
    }
}
