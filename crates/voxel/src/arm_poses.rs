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
fn is_charged_crossbow(stack: &ClientItemStackState) -> bool {
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
}
