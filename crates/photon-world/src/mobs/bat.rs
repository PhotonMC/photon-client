//! What a bat does of its own on the client (`Bat.tick`): the animation that
//! runs while it hangs, the one while it flies, and where it hangs.

use crate::animation_state::AnimationState;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatState {
    pub fly: AnimationState,
    pub rest: AnimationState,
}

impl BatState {
    /// `Bat.setupAnimationStates`.
    pub(crate) fn tick(&mut self, resting: bool, tick_count: i32) {
        if resting {
            self.fly.stop();
            self.rest.start_if_stopped(tick_count);
        } else {
            self.rest.stop();
            self.fly.start_if_stopped(tick_count);
        }
    }
}
