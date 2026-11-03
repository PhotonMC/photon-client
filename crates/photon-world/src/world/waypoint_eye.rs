//! Where a tracked waypoint that names an entity points: at the entity's eyes while it is near the
//! position the server sent (`TrackedWaypoint.Vec3iWaypoint.position`).

use std::collections::HashMap;

use photon_core::ids::EntityId;

use crate::ClientWorld;

impl ClientWorld {
    /// The eyes, between ticks and at each entity's own partial tick, of the entities the
    /// `wanted` waypoints name (a UUID and the position sent) that stand within three blocks
    /// (Manhattan) of that position; a waypoint whose entity is away keeps its block.
    pub fn waypoint_eyes(
        &self,
        wanted: &[(u128, [i32; 3])],
        partial_of: impl Fn(EntityId) -> f32,
    ) -> HashMap<u128, [f64; 3]> {
        if wanted.is_empty() {
            return HashMap::new();
        }
        let vectors: HashMap<u128, [i32; 3]> = wanted.iter().copied().collect();
        self.entities
            .all()
            .filter_map(|entity| {
                let uuid = u128::from(entity.uuid.most) << 64 | u128::from(entity.uuid.least);
                let vector = vectors.get(&uuid)?;
                let (x, y, z) = entity.position();
                let block = [x.floor() as i32, y.floor() as i32, z.floor() as i32];
                let manhattan: i64 = (0..3)
                    .map(|axis| i64::from((block[axis] - vector[axis]).abs()))
                    .sum();
                if manhattan > 3 {
                    return None;
                }
                let (x, y, z) = entity.interpolated_position(partial_of(entity.id()));
                let eye = y + f64::from(entity.scaled_dimensions().eye_height());
                Some((uuid, [x, eye, z]))
            })
            .collect()
    }
}
