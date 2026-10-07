//! What a player's arms are doing with what they hold (`AvatarRenderer.getArmPose` and the
//! use-item fields of `HumanoidMobRenderer.extractHumanoidRenderState`): a raised bow, a blocking
//! shield, a charging or charged crossbow, a spyglass, a horn, a brush, a trident, a spear held up
//! or thrust. The local player's use of an item is the server's flag (the client does not predict
//! it yet). The `spears` item tag is read as the seven spears.

use voxel_data::item::classes::ItemClass;
use voxel_data::item::component_types::{
    BLOCKS_ATTACKS, CHARGED_PROJECTILES, CONSUMABLE, ENCHANTMENTS, KINETIC_WEAPON,
};
use voxel_data::spear::{Spear, spear_of};
use voxel_game::item::values::Values;
use voxel_net::connection::PlaySession;
use voxel_render::entity::held_item::{SpearInput, SpearView, spear_view};
use voxel_render::entity::player_model::{Arm, ArmPose, SwingAnimation};
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
    Spear,
    Other,
}

/// `ItemStack.getUseAnimation`: the item's own, else a shield's by its component.
fn use_animation(stack: &ClientItemStackState) -> UseAnimation {
    let item = voxel_world::block::rules::item(stack.item_registry_id());
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
            let kinetic = match values.raw(KINETIC_WEAPON) {
                Some(_) => true,
                None => item.is_some_and(|item| item.kinetic_weapon),
            };
            if values.has(CONSUMABLE) {
                UseAnimation::Other
            } else if blocks {
                UseAnimation::Block
            } else if kinetic {
                UseAnimation::Spear
            } else {
                UseAnimation::Other
            }
        }
    }
}

/// `CrossbowItem.isCharged`: the stack holds a projectile.
pub(crate) fn is_charged_crossbow(stack: &ClientItemStackState) -> bool {
    let is_crossbow = voxel_world::block::rules::item(stack.item_registry_id())
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

/// What a player's arms are doing, which the arm poses read: a swing, and the use of an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmSource {
    pub main_arm_left: bool,
    pub swinging: bool,
    pub using_item: bool,
    pub used_off_hand: bool,
    pub ticks_using_item: f32,
}

/// `AvatarRenderer.getArmPose` of one hand's stack.
fn hand_pose(source: &ArmSource, stack: Option<&ClientItemStackState>, off_hand: bool) -> ArmPose {
    let Some(stack) = stack else {
        return ArmPose::Empty;
    };
    if !source.swinging && is_charged_crossbow(stack) {
        return ArmPose::CrossbowHold;
    }
    if source.using_item && source.used_off_hand == off_hand {
        match use_animation(stack) {
            UseAnimation::Block => return ArmPose::Block,
            UseAnimation::Bow => return ArmPose::BowAndArrow,
            UseAnimation::Trident => return ArmPose::ThrowTrident,
            UseAnimation::Crossbow => return ArmPose::CrossbowCharge,
            UseAnimation::Spyglass => return ArmPose::Spyglass,
            UseAnimation::TootHorn => return ArmPose::TootHorn,
            UseAnimation::Brush => return ArmPose::Brush,
            UseAnimation::Spear => return ArmPose::Spear,
            UseAnimation::Other => {}
        }
    }
    // A spear thrust in a swing, or held about to be, is posed as one.
    if spear_of_stack(stack).is_some() {
        ArmPose::Spear
    } else {
        ArmPose::Item
    }
}

/// The spear a stack is, by its item.
pub(crate) fn spear_of_stack(stack: &ClientItemStackState) -> Option<&'static Spear> {
    voxel_world::block::rules::item(stack.item_registry_id()).and_then(|item| spear_of(item.name))
}

/// [`spear_view`] of what two arms hold.
pub fn stacks_spear_view(
    stacks: (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    rest: SpearRest,
) -> SpearView {
    let spears = (
        stacks.0.and_then(spear_of_stack),
        stacks.1.and_then(spear_of_stack),
    );
    // `ArmedEntityRenderState`: the swing is the type of the stack in the arm that swings.
    let swinging_spear = if rest.attack.1 == Arm::Left {
        spears.1
    } else {
        spears.0
    };
    spear_view(&SpearInput {
        spears,
        main_arm: rest.main_arm,
        attack: rest.attack,
        swing_animation: if swinging_spear.is_some() {
            SwingAnimation::Stab
        } else {
            SwingAnimation::Whack
        },
        arm_poses: rest.arm_poses,
        using: rest.using,
        hit_feedback: rest.hit_feedback,
    })
}

/// What a spear view reads besides the stacks.
pub struct SpearRest {
    pub main_arm: Arm,
    /// `attackTime` and `attackArm`.
    pub attack: (f32, Arm),
    pub arm_poses: (ArmPose, ArmPose),
    /// Whether an item is in use, whether the main hand uses it, and for how long.
    pub using: (bool, bool, f32),
    /// `ticksSinceKineticHitFeedback`.
    pub hit_feedback: f32,
}

/// [`spear_view`] of an entity.
pub fn entity_spear_view(
    entity: &ClientEntity,
    attack: (f32, Arm),
    arm_poses: (ArmPose, ArmPose),
    (game_time, partial): (u64, f32),
) -> SpearView {
    stacks_spear_view(
        (
            entity.item_held_by_arm(false),
            entity.item_held_by_arm(true),
        ),
        SpearRest {
            main_arm: if entity.main_arm_is_left() {
                Arm::Left
            } else {
                Arm::Right
            },
            attack,
            arm_poses,
            using: (
                entity.is_using_item(),
                !entity.used_item_is_off_hand(),
                entity.ticks_using_item(partial),
            ),
            hit_feedback: entity.ticks_since_kinetic_hit(game_time, partial),
        },
    )
}

/// `HumanoidMobRenderer.getArmPose`: a mob's arm holding a spear is posed as one, and so is any
/// arm the other hand's stab swings (`AbstractZombieRenderer.getArmPose` for a zombie).
pub(crate) fn mob_spear_pose(entity: &ClientEntity, left: bool, zombie: bool) -> ArmPose {
    let holds = |left: bool| entity.item_held_by_arm(left).and_then(spear_of_stack);
    if (zombie && holds(!left).is_some()) || holds(left).is_some() {
        ArmPose::Spear
    } else {
        ArmPose::Empty
    }
}

/// The poses of a remote player's arms and the fields the model reads of what it is using.
pub fn arm_state(session: &PlaySession, entity: &ClientEntity, partial: f32) -> ArmState {
    arm_state_of(
        session,
        (
            entity.equipment(EquipmentSlot::MainHand),
            entity.equipment(EquipmentSlot::OffHand),
        ),
        &ArmSource {
            main_arm_left: entity.main_arm_is_left(),
            swinging: entity.swing_state().is_swinging(),
            using_item: entity.is_using_item(),
            used_off_hand: entity.used_item_is_off_hand(),
            ticks_using_item: entity.ticks_using_item(partial),
        },
    )
}

/// The poses of a player's arms from what its hands hold and what they are doing.
pub fn arm_state_of(
    session: &PlaySession,
    (main, off): (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
    source: &ArmSource,
) -> ArmState {
    let main_pose = hand_pose(source, main, false);
    let mut off_pose = hand_pose(source, off, true);
    if main_pose.is_two_handed() {
        off_pose = if off.is_none() {
            ArmPose::Empty
        } else {
            ArmPose::Item
        };
    }
    let main_arm = if source.main_arm_left {
        Arm::Left
    } else {
        Arm::Right
    };
    let (right, left) = if main_arm == Arm::Right {
        (main_pose, off_pose)
    } else {
        (off_pose, main_pose)
    };
    let in_use = if source.used_off_hand { off } else { main };
    ArmState {
        right,
        left,
        using_item: source.using_item,
        use_main_hand: !source.used_off_hand,
        ticks_using_item: source.ticks_using_item,
        max_crossbow_charge: crossbow_charge_duration(
            session,
            in_use.filter(|_| source.using_item),
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
            voxel_world::block::rules::item(stack.item_registry_id())
                .is_some_and(|item| item.class == ItemClass::SpyglassItem)
        })
    };
    [(Arm::Right, right), (Arm::Left, left)]
        .into_iter()
        .find(|(side, stack)| {
            voxel_render::entity::held_item::eye_held_arm(
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
) -> Option<voxel_render::entity::player_renderer::FallFlying> {
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
    Some(voxel_render::entity::player_renderer::FallFlying {
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
        // Every spear is a kinetic weapon, which raises the arm as it is used.
        assert_eq!(
            use_animation(&stack("minecraft:iron_spear")),
            UseAnimation::Spear
        );
        assert!(spear_of_stack(&stack("minecraft:wooden_spear")).is_some());
        assert!(spear_of_stack(&stack("minecraft:iron_sword")).is_none());
    }

    #[test]
    fn a_spear_thrusts_the_item_of_the_arm_that_swings_and_raises_the_one_in_use() {
        let iron = stack("minecraft:iron_spear");
        let view = |stacks, attack_arm, using, attack_time| {
            stacks_spear_view(
                stacks,
                SpearRest {
                    main_arm: Arm::Right,
                    attack: (attack_time, attack_arm),
                    arm_poses: (ArmPose::Spear, ArmPose::Item),
                    using,
                    hit_feedback: 2.5,
                },
            )
        };
        // Swinging a spear in the main hand stabs, and thrusts only the main arm's item.
        let stab = view((Some(&iron), None), Arm::Right, (false, true, 0.0), 0.3);
        assert_eq!(stab.swing_animation, SwingAnimation::Stab);
        assert_eq!(
            stab.motions[0].stab,
            Some((0.3, voxel_data::spear::FORWARD_MOVEMENT))
        );
        assert_eq!(stab.motions[1].stab, None);
        // The same swing with no spear in the swinging hand is a whack.
        let whack = view((Some(&iron), None), Arm::Left, (false, true, 0.0), 0.3);
        assert_eq!(whack.swing_animation, SwingAnimation::Whack);
        assert_eq!(whack.motions[1].stab, None);
        // A spear used by the main hand is held up with the ticks it has been used for.
        let used = view((Some(&iron), None), Arm::Right, (true, true, 12.5), 0.0);
        let held = used.motions[0]
            .spear_use
            .expect("the right arm holds it up");
        assert_eq!(held.ticks_using_item, 12.5);
        assert_eq!(held.hit_feedback, 2.5);
        // The off hand using something leaves the main hand's spear as it is.
        let other = view((Some(&iron), None), Arm::Right, (true, false, 12.5), 0.0);
        assert!(other.motions[0].spear_use.is_none());
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
