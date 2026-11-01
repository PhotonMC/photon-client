//! What a breeze does of its own on the client: the animation states its pose
//! starts (`Breeze.onSyncedDataUpdated`) and the ones its ticks keep
//! (`Breeze.tick`).

use crate::animation_state::AnimationState;
use crate::entity::ClientEntityPose;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BreezeState {
    pub idle: AnimationState,
    pub shoot: AnimationState,
    pub slide: AnimationState,
    pub slide_back: AnimationState,
    pub inhale: AnimationState,
    pub long_jump: AnimationState,
    /// `Breeze.soundTick`: the ticks left until the next whirl.
    sound_tick: i32,
    random: crate::entity_random::EntityRandom,
}

impl BreezeState {
    /// `Breeze.onSyncedDataUpdated(DATA_POSE)`: the shooting, idle, inhaling and
    /// jumping animations stop, then the pose's own (shooting, inhaling,
    /// sliding) starts if it is stopped. The slide goes on until a tick ends it.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose, tick_count: i32) {
        self.shoot.stop();
        self.idle.stop();
        self.inhale.stop();
        self.long_jump.stop();
        match pose {
            ClientEntityPose::Shooting => self.shoot.start_if_stopped(tick_count),
            ClientEntityPose::Inhaling => self.inhale.start_if_stopped(tick_count),
            ClientEntityPose::Sliding => self.slide.start_if_stopped(tick_count),
            _ => {}
        }
    }

    /// `Breeze.tick` on the client, before the rest of the tick: whether a whirl sounds.
    pub(crate) fn tick(&mut self, pose: ClientEntityPose, tick_count: i32) -> bool {
        if pose == ClientEntityPose::LongJumping {
            self.long_jump.start_if_stopped(tick_count);
        }
        self.idle.start_if_stopped(tick_count);
        if pose != ClientEntityPose::Sliding && self.slide.is_started() {
            self.slide_back.start(tick_count);
            self.slide.stop();
        }
        self.sound_tick = if self.sound_tick == 0 {
            self.random
                .source()
                .next_int_between_inclusive(1, 80)
                .unwrap_or(1)
        } else {
            self.sound_tick - 1
        };
        self.sound_tick == 0
    }
}
