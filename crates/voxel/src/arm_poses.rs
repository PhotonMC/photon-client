//! What a player's arms are doing with what they hold (`AvatarRenderer.getArmPose` and the
//! use-item fields of `HumanoidMobRenderer.extractHumanoidRenderState`): a raised bow, a blocking
//! shield, a charging or charged crossbow, a spyglass, a horn, a brush, a trident. Not carried: a
//! spear's poses, a stabbing swing, and the local player's own use of an item (the client does not
//! track it yet), so the local model keeps to holding.

use voxel_data::item::classes::ItemClass;
use voxel_data::item::component_types::{
    BLOCKS_ATTACKS, CHARGED_PROJECTILES, CONSUMABLE, ENCHANTMENTS,
};
use voxel_game::item_values::Values;
use voxel_net::connection::PlaySession;
use voxel_render::player_model::{Arm, ArmPose};
use voxel_world::{ClientEntity, ClientItemStackState, EquipmentSlot};

/// `ItemUseAnimation`, of the ones that pose an arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UseAnimation {
    Block,
    Bow,
    Trident,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
    Other,
}

/// `ItemStack.getUseAnimation`: the item's own, else a shield's by its component.
fn use_animation(stack: &ClientItemStackState) -> UseAnimation {
    let item = voxel_world::block_rules::item(stack.item_registry_id());
    match item.map(|item| item.class) {
        Some(ItemClass::BowItem) => UseAnimation::Bow,
        Some(ItemClass::CrossbowItem) => UseAnimation::Crossbow,
        Some(ItemClass::SpyglassItem) => UseAnimation::Spyglass,
        Some(ItemClass::TridentItem) => UseAnimation::Trident,
        Some(ItemClass::InstrumentItem) => UseAnimation::TootHorn,
        Some(ItemClass::BrushItem) => UseAnimation::Brush,
        _ => {
            // A consumable's own animation comes first; a shield blocks by its component (the
            // item table says which items carry it, a patch may add or remove it).
            let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
            let blocks = match values.raw(BLOCKS_ATTACKS) {
                Some(_) => true,
                None => item.is_some_and(|item| item.blocks_attacks),
            };
            if values.has(CONSUMABLE) {
                UseAnimation::Other
            } else if blocks {
                UseAnimation::Block
            } else {
                UseAnimation::Other
            }
        }
    }
}

/// `CrossbowItem.isCharged`: the stack holds a projectile.
pub(crate) fn is_charged_crossbow(stack: &ClientItemStackState) -> bool {
    let is_crossbow = voxel_world::block_rules::item(stack.item_registry_id())
        .is_some_and(|item| item.class == ItemClass::CrossbowItem);
    is_crossbow
        && Values::new(stack.item_registry_id(), stack.patch_bytes())
            .raw(CHARGED_PROJECTILES)
            .and_then(|bytes| bytes.first())
            .is_some_and(|count| *count > 0)
}

/// `CrossbowItem.getChargeDuration`: a second and a quarter, less a quarter per level of quick
/// charge, in ticks.
fn crossbow_charge_duration(session: &PlaySession, stack: Option<&ClientItemStackState>) -> f32 {
    let level = stack.map_or(0, |stack| {
        let quick_charge = session
            .registry_names("minecraft:enchantment")
            .iter()
            .position(|name| name == "minecraft:quick_charge")
            .and_then(|index| i32::try_from(index).ok());
        Values::new(stack.item_registry_id(), stack.patch_bytes())
            .enchantments(ENCHANTMENTS)
            .into_iter()
            .find(|(id, _)| Some(*id) == quick_charge)
            .map_or(0, |(_, level)| level)
    });
    let seconds = if level > 0 {
        1.25 - 0.25 * level as f32
    } else {
        1.25
    };
    (seconds * 20.0).floor()
}

/// The arm poses and use-item fields of a player's model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmState {
    pub right: ArmPose,
    pub left: ArmPose,
    pub using_item: bool,
    /// Whether the main hand is the one using the item.
    pub use_main_hand: bool,
    pub ticks_using_item: f32,
    pub max_crossbow_charge: f32,
}

/// `AvatarRenderer.getArmPose` of one hand's stack.
fn hand_pose(
    entity: &ClientEntity,
    stack: Option<&ClientItemStackState>,
    off_hand: bool,
) -> ArmPose {
    let Some(stack) = stack else {
        return ArmPose::Empty;
    };
    if !entity.swing_state().is_swinging() && is_charged_crossbow(stack) {
        return ArmPose::CrossbowHold;
    }
    if entity.is_using_item() && entity.used_item_is_off_hand() == off_hand {
        match use_animation(stack) {
            UseAnimation::Block => return ArmPose::Block,
            UseAnimation::Bow => return ArmPose::BowAndArrow,
            UseAnimation::Trident => return ArmPose::ThrowTrident,
            UseAnimation::Crossbow => return ArmPose::CrossbowCharge,
            UseAnimation::Spyglass => return ArmPose::Spyglass,
            UseAnimation::TootHorn => return ArmPose::TootHorn,
            UseAnimation::Brush => return ArmPose::Brush,
            UseAnimation::Other => {}
        }
    }
    ArmPose::Item
}

/// The poses of a remote player's arms and the fields the model reads of what it is using.
pub fn arm_state(session: &PlaySession, entity: &ClientEntity, partial: f32) -> ArmState {
    let main = entity.equipment(EquipmentSlot::MainHand);
    let off = entity.equipment(EquipmentSlot::OffHand);
    let main_pose = hand_pose(entity, main, false);
    let mut off_pose = hand_pose(entity, off, true);
    if main_pose.is_two_handed() {
        off_pose = if off.is_none() {
            ArmPose::Empty
        } else {
            ArmPose::Item
        };
    }
    let main_arm = if entity.main_arm_is_left() {
        Arm::Left
    } else {
        Arm::Right
    };
    let (right, left) = if main_arm == Arm::Right {
        (main_pose, off_pose)
    } else {
        (off_pose, main_pose)
    };
    let in_use = if entity.used_item_is_off_hand() {
        off
    } else {
        main
    };
    ArmState {
        right,
        left,
        using_item: entity.is_using_item(),
        use_main_hand: !entity.used_item_is_off_hand(),
        ticks_using_item: entity.ticks_using_item(partial),
        max_crossbow_charge: crossbow_charge_duration(
            session,
            in_use.filter(|_| entity.is_using_item()),
        ),
    }
}

/// `PlayerItemInHandLayer.submitArmWithItem`: the arm whose spyglass is held to the eye, if one is.
pub fn eye_arm(
    arms: &ArmState,
    (main_arm, attack_time): (Arm, f32),
    (right, left): (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
) -> Option<Arm> {
    let is_spyglass = |stack: Option<&ClientItemStackState>| {
        stack.is_some_and(|stack| {
            voxel_world::block_rules::item(stack.item_registry_id())
                .is_some_and(|item| item.class == ItemClass::SpyglassItem)
        })
    };
    [(Arm::Right, right), (Arm::Left, left)]
        .into_iter()
        .find(|(side, stack)| {
            voxel_render::held_item::eye_held_arm(
                (arms.using_item, arms.use_main_hand),
                (main_arm, attack_time),
                is_spyglass(*stack),
                *side,
            )
        })
        .map(|(side, _)| side)
}

/// [`eye_arm`] of a remote player.
pub fn eye_arm_of(
    session: &PlaySession,
    entity: &ClientEntity,
    attack_time: f32,
    partial: f32,
) -> Option<Arm> {
    let main_arm = if entity.main_arm_is_left() {
        Arm::Left
    } else {
        Arm::Right
    };
    eye_arm(
        &arm_state(session, entity, partial),
        (main_arm, attack_time),
        (
            entity.item_held_by_arm(false),
            entity.item_held_by_arm(true),
        ),
    )
}

/// `AvatarRenderer.extractFlightData`: how far a gliding player has tipped forward, and which way
/// the flight turns it (the movement against the look, when both have a heading).
pub(crate) fn fall_flying(
    entity: &ClientEntity,
    snapshot: &voxel_world::EntitySnapshot,
    partial: f32,
) -> Option<voxel_render::player_renderer::FallFlying> {
    if !entity.is_fall_flying() {
        return None;
    }
    let time = entity.fall_flying_ticks() as f32 + partial;
    let (px, _, pz) = entity.previous_motion();
    let (cx, _, cz) = entity.motion();
    let partial = f64::from(partial);
    let movement = (px + partial * (cx - px), pz + partial * (cz - pz));
    // `Entity.getViewVector`: the look's heading.
    let look = voxel_core::math::Vec3::direction_from_rotation_components(
        snapshot.pitch,
        snapshot.head_yaw,
    );
    let (movement_len, look_len) = (
        movement.0 * movement.0 + movement.1 * movement.1,
        look.x * look.x + look.z * look.z,
    );
    let y_rot =
        (movement_len > f64::from(1.0e-5_f32) && look_len > f64::from(1.0e-5_f32)).then(|| {
            let (mn, ln) = (movement_len.sqrt(), look_len.sqrt());
            let dot = (movement.0 / mn) * (look.x / ln) + (movement.1 / mn) * (look.z / ln);
            let sign = movement.0 * look.z - movement.1 * look.x;
            let heading = if sign == 0.0 { 0.0 } else { sign.signum() };
            (heading * dot.abs().min(1.0).acos()) as f32
        });
    Some(voxel_render::player_renderer::FallFlying {
        scale: (time * time / 100.0).clamp(0.0, 1.0),
        y_rot,
    })
}

/// `CrossbowItem.getChargeDuration` of the stack the entity is using: the plain duration when it
/// uses nothing.
pub(crate) fn charge_duration_of_use(session: &PlaySession, entity: &ClientEntity) -> f32 {
    let in_use = if entity.used_item_is_off_hand() {
        entity.equipment(EquipmentSlot::OffHand)
    } else {
        entity.equipment(EquipmentSlot::MainHand)
    };
    crossbow_charge_duration(session, in_use.filter(|_| entity.is_using_item()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(name: &str) -> ClientItemStackState {
        ClientItemStackState::new(1, voxel_game::item::id_of(name).expect(name))
    }

    #[test]
    fn what_an_item_does_when_used_picks_the_arm_pose() {
        assert_eq!(use_animation(&stack("minecraft:bow")), UseAnimation::Bow);
        assert_eq!(
            use_animation(&stack("minecraft:crossbow")),
            UseAnimation::Crossbow
        );
        assert_eq!(
            use_animation(&stack("minecraft:spyglass")),
            UseAnimation::Spyglass
        );
        assert_eq!(
            use_animation(&stack("minecraft:trident")),
            UseAnimation::Trident
        );
        assert_eq!(
            use_animation(&stack("minecraft:goat_horn")),
            UseAnimation::TootHorn
        );
        assert_eq!(
            use_animation(&stack("minecraft:brush")),
            UseAnimation::Brush
        );
        // A shield blocks by its component; food and a stick do nothing to the arm.
        assert_eq!(
            use_animation(&stack("minecraft:shield")),
            UseAnimation::Block
        );
        assert_eq!(
            use_animation(&stack("minecraft:apple")),
            UseAnimation::Other
        );
        assert_eq!(
            use_animation(&stack("minecraft:stick")),
            UseAnimation::Other
        );
    }

    #[test]
    fn a_crossbow_with_nothing_loaded_is_not_held_up() {
        assert!(!is_charged_crossbow(&stack("minecraft:crossbow")));
        assert!(!is_charged_crossbow(&stack("minecraft:bow")));
    }

    #[test]
    fn a_spyglass_in_use_is_held_to_the_eye_by_the_arm_that_uses_it() {
        let arms = |using, main| ArmState {
            right: ArmPose::Spyglass,
            left: ArmPose::Item,
            using_item: using,
            use_main_hand: main,
            ticks_using_item: 3.0,
            max_crossbow_charge: 25.0,
        };
        let spyglass = stack("minecraft:spyglass");
        let stick = stack("minecraft:stick");
        let stacks = (Some(&spyglass), Some(&stick));
        assert_eq!(
            eye_arm(&arms(true, true), (Arm::Right, 0.0), stacks),
            Some(Arm::Right)
        );
        // Not in use, swinging, or used by the other hand: held in the hand.
        assert_eq!(eye_arm(&arms(false, true), (Arm::Right, 0.0), stacks), None);
        assert_eq!(eye_arm(&arms(true, true), (Arm::Right, 0.3), stacks), None);
        assert_eq!(eye_arm(&arms(true, false), (Arm::Right, 0.0), stacks), None);
        // A left-handed player's off hand is the right one.
        assert_eq!(
            eye_arm(&arms(true, false), (Arm::Left, 0.0), stacks),
            Some(Arm::Right)
        );
    }
}
