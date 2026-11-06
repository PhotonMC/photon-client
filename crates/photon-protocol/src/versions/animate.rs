//! `ClientboundAnimatePacket`'s actions. 26.3 kept three of them (wake up, the two crits) with new
//! numbers and moved the swings to their own packet; before it the one packet carried all.

use photon_core::version::Version;

/// What an animate packet asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimateAction {
    WakeUp,
    CriticalHit,
    MagicCriticalHit,
    /// A swing of a hand with the default animation (before 26.3).
    Swing {
        off_hand: bool,
    },
    Other,
}

pub fn animate_action(version: Version, action: u8) -> AnimateAction {
    if version >= Version::V26_3 {
        return match action {
            0 => AnimateAction::WakeUp,
            1 => AnimateAction::CriticalHit,
            2 => AnimateAction::MagicCriticalHit,
            _ => AnimateAction::Other,
        };
    }
    match action {
        0 => AnimateAction::Swing { off_hand: false },
        3 => AnimateAction::Swing { off_hand: true },
        2 => AnimateAction::WakeUp,
        4 => AnimateAction::CriticalHit,
        5 => AnimateAction::MagicCriticalHit,
        _ => AnimateAction::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_actions_are_numbered_by_version() {
        assert_eq!(animate_action(Version::V26_3, 0), AnimateAction::WakeUp);
        assert_eq!(
            animate_action(Version::V26_3, 2),
            AnimateAction::MagicCriticalHit
        );
        assert_eq!(animate_action(Version::V26_2, 2), AnimateAction::WakeUp);
        assert_eq!(
            animate_action(Version::V26_2, 3),
            AnimateAction::Swing { off_hand: true }
        );
        assert_eq!(
            animate_action(Version::V26_2, 4),
            AnimateAction::CriticalHit
        );
    }
}
