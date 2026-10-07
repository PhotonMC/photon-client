use super::*;

impl ClientPlayerState {
    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    /// Return player/Avatar dimensions derived from the current pose.
    pub fn player_dimensions(&self) -> ClientEntityDimensions {
        self.pose.player_dimensions()
    }
}

/// Inputs used by Java Player's desired-pose and fit fallback decision.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientPlayerPoseInput {
    pub sleeping: bool,
    pub swimming: bool,
    pub fall_flying: bool,
    pub auto_spin_attack: bool,
    pub shift_key_down: bool,
    pub flying: bool,
    pub spectator: bool,
    pub passenger: bool,
    pub swimming_fits: bool,
    pub desired_fits: bool,
    pub crouching_fits: bool,
}

impl ClientPlayerPoseInput {
    /// Choose the desired pose using Java Player's branch priority.
    pub fn desired_pose(self) -> ClientEntityPose {
        if self.sleeping {
            ClientEntityPose::Sleeping
        } else if self.swimming {
            ClientEntityPose::Swimming
        } else if self.fall_flying {
            ClientEntityPose::FallFlying
        } else if self.auto_spin_attack {
            ClientEntityPose::SpinAttack
        } else if self.shift_key_down && !self.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        }
    }

    /// Apply Java Player's swimming gate, fit fallback, and bypass branches.
    pub fn updated_pose(self) -> Option<ClientEntityPose> {
        if !self.swimming_fits {
            return None;
        }
        let desired_pose = self.desired_pose();
        Some(if self.spectator || self.passenger || self.desired_fits {
            desired_pose
        } else if self.crouching_fits {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Swimming
        })
    }
}
