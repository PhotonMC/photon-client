//! The drop key: `Minecraft.handleKeybinds` and `LocalPlayer.drop`.

use super::{ClientPlayerGameMode, Context, MAIN_HAND, Outgoing, PlayerAction};

/// A press of the drop key (not for a spectator): the selected stack loses one item, or all with
/// control down, at once; the server is told even when the hand was empty, and the hand swings
/// when something went.
pub(super) fn drop_selected(ctx: &mut Context<'_>, all: bool, out: &mut Vec<Outgoing>) {
    if ctx.mode == ClientPlayerGameMode::Spectator {
        return;
    }
    let dropped = ctx
        .held(MAIN_HAND)
        .map(|(_, count)| if all { count } else { 1 });
    if let Some(amount) = dropped {
        ctx.world
            .shrink_local_player_slot(ctx.hand_slot(MAIN_HAND), amount);
    }
    out.push(Outgoing::PlayerAction {
        action: if all {
            PlayerAction::DropAllItems
        } else {
            PlayerAction::DropItem
        },
        x: 0,
        y: 0,
        z: 0,
        face: 0,
        sequence: 0,
    });
    if dropped.is_some() {
        ctx.swing_hand(MAIN_HAND, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The drop actions are the reference's ordinals 3 and 4, at the zero position facing down.
    #[test]
    fn drop_actions_go_out_at_the_zero_position() {
        let packet = |action| Outgoing::PlayerAction {
            action,
            x: 0,
            y: 0,
            z: 0,
            face: 0,
            sequence: 0,
        };
        let mut expected = vec![4];
        expected.extend_from_slice(&[0; 8]);
        expected.extend_from_slice(&[0, 0]);
        assert_eq!(packet(PlayerAction::DropItem).encode_body(), expected);
        expected[0] = 3;
        assert_eq!(packet(PlayerAction::DropAllItems).encode_body(), expected);
    }
}
