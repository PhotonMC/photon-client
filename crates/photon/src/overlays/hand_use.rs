//! What the first-person hands do with their stacks while a bow or a crossbow is in play
//! (`ItemInHandRenderer.submitArmWithItem` and `evaluateWhichHandsToRender`).

use crate::equipment::arm_poses::{UseAnimation, is_charged_crossbow, use_animation, use_duration};
use photon_data::item::classes::ItemClass;
use photon_render::entity::hand_motion::HandMotion;
use photon_world::ClientItemStackState;
use photon_world::local_player::LocalPlayer;

/// `Item.getUseDuration` of a bow or a crossbow: they run until released.
const DRAW_DURATION: i32 = 72000;

fn class_of(stack: Option<&ClientItemStackState>) -> Option<ItemClass> {
    stack
        .and_then(|stack| photon_world::block::rules::item(stack.item_registry_id()))
        .map(|item| item.class)
}

/// `getUseDuration - (getUseItemRemainingTicks - partialTick + 1)`, evaluated in single
/// precision as the reference does: the ticks the use has run plus the frame's part, less one.
fn held_time(ticks: i32, partial: f32) -> f32 {
    DRAW_DURATION as f32 - ((DRAW_DURATION - ticks) as f32 - partial + 1.0)
}

/// The motion of a hand's stack: a drawn bow, a charging crossbow, a loaded one in the main
/// hand, what an item in use does with the hand, else the swing. `ticks` is how long the player
/// has used this hand's stack, if it does, and `charge_duration` the crossbow's.
pub(super) fn motion(
    stack: &ClientItemStackState,
    (ticks, partial): (Option<i32>, f32),
    (main_hand, charge_duration): (bool, i32),
) -> HandMotion {
    let held = ticks.map(|ticks| held_time(ticks, partial));
    match (class_of(Some(stack)), held) {
        (Some(ItemClass::BowItem), Some(held)) => HandMotion::Bow { held },
        (Some(ItemClass::CrossbowItem), held) => match held {
            Some(held) if !is_charged_crossbow(stack) => HandMotion::CrossbowCharge {
                held,
                charge_duration: charge_duration as f32,
            },
            _ if main_hand && is_charged_crossbow(stack) => HandMotion::LoadedCrossbow,
            _ => HandMotion::Swing,
        },
        _ => in_use(stack, (ticks, partial)).unwrap_or_else(|| swing_of(stack)),
    }
}

/// The spear a stack is, by its item.
fn spear_of(stack: &ClientItemStackState) -> Option<&'static photon_data::spear::Spear> {
    let item = photon_world::block::rules::item(stack.item_registry_id())?;
    photon_data::spear::spear_of(item.name)
}

/// What a swing does with the stack: a spear stabs, anything else whacks
/// (`ItemStack.getSwingAnimation().type()`).
fn swing_of(stack: &ClientItemStackState) -> HandMotion {
    if spear_of(stack).is_some() {
        HandMotion::Stab
    } else {
        HandMotion::Swing
    }
}

/// What the hand does with a stack it is using, when it has ticks of use left
/// (`getUseItemRemainingTicks() > 0`).
fn in_use(
    stack: &ClientItemStackState,
    (ticks, partial): (Option<i32>, f32),
) -> Option<HandMotion> {
    let ticks = ticks?;
    let duration = use_duration(stack);
    let remaining = duration - ticks;
    if remaining <= 0 {
        return None;
    }
    let held = held_time(ticks, partial);
    Some(match use_animation(stack) {
        UseAnimation::Eat | UseAnimation::Drink => HandMotion::Eat {
            current: remaining as f32 - partial + 1.0,
            duration,
        },
        UseAnimation::Block => HandMotion::Block {
            shield: class_of(Some(stack)) == Some(ItemClass::ShieldItem),
        },
        UseAnimation::Trident => HandMotion::Trident { held },
        UseAnimation::Brush => HandMotion::Brush {
            phase: (remaining % 10) as f32 - partial + 1.0,
        },
        UseAnimation::Spear => HandMotion::SpearUse {
            spear: spear_of(stack)?,
            held,
            // The kick of a hit on the local player is not tracked yet.
            hit_feedback: 0.0,
        },
        _ => HandMotion::Lifted,
    })
}

/// `evaluateWhichHandsToRender`: whether the main and the off hand are drawn. A bow or a
/// crossbow in play draws only the hand that uses it, and a loaded crossbow in the main hand
/// hides the off hand.
pub(super) fn selection(
    player: &LocalPlayer,
    (main, off): (Option<&ClientItemStackState>, Option<&ClientItemStackState>),
) -> (bool, bool) {
    let bow_like = |stack| {
        matches!(
            class_of(stack),
            Some(ItemClass::BowItem | ItemClass::CrossbowItem)
        )
    };
    if !(bow_like(main) || bow_like(off)) {
        return (true, true);
    }
    let charged = |stack: Option<&ClientItemStackState>| stack.is_some_and(is_charged_crossbow);
    if !player.is_using_item() {
        return (true, !charged(main));
    }
    let off_hand = player.used_item_is_off_hand();
    let used = if off_hand { off } else { main };
    if bow_like(used) {
        (!off_hand, off_hand)
    } else {
        (true, off_hand || !charged(off))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_held_time_counts_the_ticks_used_less_the_tick_before_the_frame() {
        assert_eq!(held_time(10, 0.5), 9.5);
        assert_eq!(held_time(0, 0.0), -1.0);
    }
}
