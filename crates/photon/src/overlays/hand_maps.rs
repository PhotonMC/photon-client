//! A map in the first-person view (`ItemInHandRenderer.submitArmWithItem` for a stack with a map
//! id): its arms, and where its picture is drawn.

use super::*;
use photon_render::entity::map_in_hand;

/// A held map: which one, and the pose its picture is drawn from.
pub struct HeldMap {
    pub id: i32,
    pub pose: photon_render::joml::Pose,
}

/// The map in a hand and the arms that hold it: both of the player's when it is in the main hand
/// and the off hand is empty, else the one arm of the hand it is in.
fn held_map(
    model: &mut PlayerModel,
    hand_view: &HandView,
    id: i32,
    (right_arm, attack, height): (bool, f32, f32),
    two_handed: bool,
    invisible: bool,
) -> (Vec<Vertex>, HeldMap) {
    let held = if two_handed {
        map_in_hand::two_handed(
            model,
            hand_view,
            (hand_view.view_x_rot, attack, height),
            invisible,
        )
    } else {
        map_in_hand::one_handed(model, hand_view, (right_arm, attack, height), invisible)
    };
    (
        held.arms,
        HeldMap {
            id,
            pose: held.pose,
        },
    )
}

/// A stack that is a map: its arms join the hand's, the map joins the maps; whether it was one.
pub(super) fn push_map(
    (model, hand_view): (&mut PlayerModel, &HandView),
    stack: &photon_world::ClientItemStackState,
    (arm, two_handed, invisible): ((bool, f32, f32), bool, bool),
    (hand, maps): (&mut Vec<Vertex>, &mut Vec<HeldMap>),
) -> bool {
    let Some(id) = crate::objects::item_frames::framed_map_id(stack) else {
        return false;
    };
    // The arm the hand is on: the main hand on the main arm's side.
    let arm = (
        arm.0 != crate::overlays::local_main_arm_left(),
        arm.1,
        arm.2,
    );
    let (arms, map) = held_map(model, hand_view, id, arm, two_handed, invisible);
    hand.extend(arms);
    maps.push(map);
    true
}

/// The bare main arm, when the hand is empty and shows.
pub(super) fn bare_arm(model: &mut PlayerModel, hand_view: &HandView, shown: bool) -> Vec<Vertex> {
    if shown {
        photon_render::entity::player_renderer::hand_vertices(model, hand_view)
    } else {
        Vec::new()
    }
}
