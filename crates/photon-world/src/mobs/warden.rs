//! What a warden does of its own on the client: the animation states its pose
//! and its events start (`Warden.onSyncedDataUpdated`, `handleEntityEvent`)
//! and the tendril and heart pulses its ticks keep (`Warden.tick`).

use crate::animation_state::AnimationState;
use crate::entity::ClientEntityPose;

/// `Warden.CLIENT_ANGER_LEVEL`'s data id: `Mob` ends at 15, nothing between.
pub const CLIENT_ANGER_DATA_ID: u8 = 16;

/// `AngerLevel.ANGRY.getMinimumAnger()`.
const ANGRY_MINIMUM_ANGER: f32 = 80.0;

/// The events a warden answers itself: its attack, its tendrils clicking and its sonic boom.
pub const ATTACK_EVENT: i8 = 4;
pub const TENDRIL_EVENT: i8 = 61;
pub const SONIC_BOOM_EVENT: i8 = 62;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WardenState {
    pub roar: AnimationState,
    pub sniff: AnimationState,
    pub emerge: AnimationState,
    pub digging: AnimationState,
    pub attack: AnimationState,
    pub sonic_boom: AnimationState,
    /// `Warden.getClientAngerLevel`, as the server last synced it.
    pub(crate) anger: i32,
    tendril: i32,
    tendril_old: i32,
    heart: i32,
    heart_old: i32,
}

impl WardenState {
    /// `Warden.onSyncedDataUpdated(DATA_POSE)`: emerging, digging, roaring and
    /// sniffing each start their animation again; the other poses start none.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose, tick_count: i32) {
        match pose {
            ClientEntityPose::Emerging => self.emerge.start(tick_count),
            ClientEntityPose::Digging => self.digging.start(tick_count),
            ClientEntityPose::Roaring => self.roar.start(tick_count),
            ClientEntityPose::Sniffing => self.sniff.start(tick_count),
            _ => {}
        }
    }

    /// `Warden.handleEntityEvent`: whether the warden answers the event itself.
    pub(crate) fn event(&mut self, event: i8, tick_count: i32) -> bool {
        match event {
            ATTACK_EVENT => {
                self.roar.stop();
                self.attack.start(tick_count);
            }
            TENDRIL_EVENT => self.tendril = 10,
            SONIC_BOOM_EVENT => self.sonic_boom.start(tick_count),
            _ => return false,
        }
        true
    }

    /// `Warden.getHeartBeatDelay`: forty ticks calm, ten at full anger.
    fn heart_beat_delay(&self) -> i32 {
        let anger = (self.anger as f32 / ANGRY_MINIMUM_ANGER).clamp(0.0, 1.0);
        40 - (anger * 30.0).floor() as i32
    }

    /// `Warden.tick` on the client, after the rest of the tick (the beat's
    /// sound and the digging particles are left out).
    pub(crate) fn tick(&mut self, tick_count: i32) {
        if tick_count % self.heart_beat_delay() == 0 {
            self.heart = 10;
        }
        self.tendril_old = self.tendril;
        if self.tendril > 0 {
            self.tendril -= 1;
        }
        self.heart_old = self.heart;
        if self.heart > 0 {
            self.heart -= 1;
        }
    }

    /// `Warden.getTendrilAnimation`.
    pub fn tendril_animation(&self, partial: f32) -> f32 {
        photon_core::math::mth_lerp_f32(partial, self.tendril_old as f32, self.tendril as f32)
            / 10.0
    }

    /// `Warden.getHeartAnimation`.
    pub fn heart_animation(&self, partial: f32) -> f32 {
        photon_core::math::mth_lerp_f32(partial, self.heart_old as f32, self.heart as f32) / 10.0
    }
}
