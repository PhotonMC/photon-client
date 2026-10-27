//! A geyser sending the player up (`PotentSulfurBlockEntity.LAUNCH_ENTITY_TICKER`).

use super::*;

/// `PotentSulfurBlockEntity.GEYSER_BASE_LAUNCH_SPEED`.
const GEYSER_BASE_LAUNCH_SPEED: f32 = 0.3;
/// `PotentSulfurBlockEntity.GEYSER_LAUNCH_FORCE`.
const GEYSER_LAUNCH_FORCE: f32 = 0.2;

impl LocalPlayer {
    /// What the launch ticker does to the one entity the client simulates: when the player is in
    /// the geyser's box, the fall distance is capped, and a player slower than the geyser's speed
    /// (and not flying or riding) gets a push up.
    pub(crate) fn launched_by_geyser(&mut self, column: Aabb, water_blocks: i32) {
        // `NO_SPECTATORS` and `ENTITY_STILL_ALIVE`, then the box.
        if self.is_spectator() || self.health <= 0.0 || !column.intersects(self.bounding_box) {
            return;
        }
        // `Entity.checkFallDistanceAccumulation`, before the conditions that refuse the launch.
        if self.delta.y > -0.5 && self.fall_distance > 1.0 {
            self.fall_distance = 1.0;
        }
        // `Player.canSimulateMovement` is true for the local player; the geysers tag holds no
        // player.
        if self.abilities.flying
            || self.is_passenger()
            || self.delta.y >= f64::from(GEYSER_BASE_LAUNCH_SPEED) + f64::from(water_blocks) * 0.1
        {
            return;
        }
        self.delta.y += f64::from(GEYSER_LAUNCH_FORCE);
    }
}
