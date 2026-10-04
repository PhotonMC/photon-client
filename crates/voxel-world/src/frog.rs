//! What a frog does of its own on the client: the animation states its pose
//! starts (`Frog.onSyncedDataUpdated`) and the one that runs while it floats
//! (`Frog.tick`).

use crate::animation_state::AnimationState;
use crate::entity::ClientEntityPose;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrogState {
    pub jump: AnimationState,
    pub croak: AnimationState,
    pub tongue: AnimationState,
    pub swim_idle: AnimationState,
}

impl FrogState {
    /// `Frog.onSyncedDataUpdated(DATA_POSE)`: each state starts (again, if it
    /// was running) while its pose is the one and stops otherwise.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose, tick_count: i32) {
        for (state, own) in [
            (&mut self.jump, ClientEntityPose::LongJumping),
            (&mut self.croak, ClientEntityPose::Croaking),
            (&mut self.tongue, ClientEntityPose::UsingTongue),
        ] {
            if pose == own {
                state.start(tick_count);
            } else {
                state.stop();
            }
        }
    }

    /// `Frog.tick` on the client, before the rest of the tick: floating
    /// without walking runs the idle animation.
    pub(crate) fn tick(&mut self, in_water: bool, moving: bool, tick_count: i32) {
        self.swim_idle.animate_when(in_water && !moving, tick_count);
    }

    /// `Frog.updateWalkAnimation`: the target speed of a frog's stride, which
    /// is none while it jumps.
    pub(crate) fn walk_target(&self, moved: f64) -> f32 {
        if self.jump.is_started() {
            0.0
        } else {
            (moved as f32 * 25.0).min(1.0)
        }
    }
}
