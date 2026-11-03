//! Where a tracked waypoint that names an entity points: at the entity's eyes while it is near the
//! position the server sent (`TrackedWaypoint.Vec3iWaypoint.position`).

use photon_core::ids::EntityId;

use crate::ClientWorld;

impl ClientWorld {
    /// The eye position of the entity `uuid` between ticks, `partial_of` its own partial tick, when
    /// its block is at most three blocks (Manhattan) from `vector`; otherwise the waypoint keeps
    /// its block.
    pub fn waypoint_eye(
        &self,
        uuid: photon_core::binary::Uuid,
        vector: [i32; 3],
        partial_of: impl Fn(EntityId) -> f32,
    ) -> Option<[f64; 3]> {
        let entity = self.entities.all().find(|entity| entity.uuid == uuid)?;
        let (x, y, z) = entity.position();
        let block = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
        let manhattan: i64 = (0..3)
            .map(|axis| i64::from((block[axis] - vector[axis]).abs()))
            .sum();
        if manhattan > 3 {
            return None;
        }
        let (x, y, z) = entity.interpolated_position(partial_of(entity.id()));
        Some([x, y + f64::from(entity.scaled_dimensions().eye_height()), z])
    }
}
