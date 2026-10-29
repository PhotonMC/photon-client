use super::*;

/// Where an entity is now and where it was one tick ago.
#[derive(Debug, Clone, Copy, PartialEq, Eq, bevy_ecs::component::Component)]
pub struct Position {
    pub(crate) position: DVec3,
    pub(crate) old_position: DVec3,
}

impl Position {
    /// An entity that was just added: at the point, and not moved since the tick before.
    pub(crate) fn spawned(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: DVec3::new(x, y, z),
            old_position: DVec3::new(x, y, z),
        }
    }
}

/// How an entity moves: its velocity and the one the server last sent, which it eases to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, bevy_ecs::component::Component)]
pub struct Motion {
    pub(crate) motion: DVec3,
    pub(crate) motion_lerp_target: DVec3,
    pub(crate) motion_lerp_steps: u8,
}

impl Position {
    pub fn xyz(&self) -> (f64, f64, f64) {
        (
            self.position.x.get(),
            self.position.y.get(),
            self.position.z.get(),
        )
    }

    pub fn previous_xyz(&self) -> (f64, f64, f64) {
        (
            self.old_position.x.get(),
            self.old_position.y.get(),
            self.old_position.z.get(),
        )
    }
}

impl Motion {
    pub fn xyz(&self) -> (f64, f64, f64) {
        (
            self.motion.x.get(),
            self.motion.y.get(),
            self.motion.z.get(),
        )
    }

    /// An entity that was just added, moving at the velocity the server sent.
    pub(crate) fn spawned(x: f64, y: f64, z: f64) -> Self {
        Self {
            motion: DVec3::new(x, y, z),
            motion_lerp_target: DVec3::new(x, y, z),
            motion_lerp_steps: 0,
        }
    }
}
