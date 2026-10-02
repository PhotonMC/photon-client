//! The waypoints the server has the client track (`TrackedWaypoint`,
//! `ClientWaypointManager`): where each is, how it looks, and the
//! operations that add, move and remove them.

use std::collections::HashMap;

/// `TrackedWaypoint.identifier`: a UUID, or a name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaypointId {
    Uuid(u128),
    Name(String),
}

impl WaypointId {
    /// Java's `hashCode` of the UUID or of the name, which the dot's
    /// colour is made from.
    pub fn java_hash(&self) -> i32 {
        match self {
            Self::Uuid(uuid) => {
                let (high, low) = ((*uuid >> 64) as u64, *uuid as u64);
                let both = high ^ low;
                ((both >> 32) as i32) ^ (both as i32)
            }
            Self::Name(name) => name.encode_utf16().fold(0_i32, |hash, unit| {
                hash.wrapping_mul(31).wrapping_add(i32::from(unit))
            }),
        }
    }
}

/// `Waypoint.Icon`: the style it is drawn in and its colour, if it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointIcon {
    /// The style's identifier, `minecraft:default`.
    pub style: String,
    /// `0xRRGGBB`.
    pub colour: Option<i32>,
}

impl WaypointIcon {
    /// `Waypoint.Icon.NULL`.
    pub fn null() -> Self {
        Self {
            style: "minecraft:default".to_owned(),
            colour: None,
        }
    }
}

/// Where a waypoint is.
#[derive(Debug, Clone, PartialEq)]
pub enum WaypointKind {
    Empty,
    /// A block position.
    Position([i32; 3]),
    /// A chunk: its x and z.
    Chunk([i32; 2]),
    /// A heading, in radians.
    Azimuth(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackedWaypoint {
    pub id: WaypointId,
    pub icon: WaypointIcon,
    pub kind: WaypointKind,
}

impl TrackedWaypoint {
    /// `update`: a waypoint of the same kind moves; another kind is
    /// refused (the reference logs it).
    fn update(&mut self, other: &Self) {
        let same_kind = std::mem::discriminant(&self.kind) == std::mem::discriminant(&other.kind);
        if same_kind {
            self.kind = other.kind.clone();
        }
    }
}

/// What a `ClientboundTrackedWaypointPacket` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointOperation {
    Track,
    Untrack,
    Update,
}

/// `ClientWaypointManager`.
#[derive(Debug, Clone, Default)]
pub struct WaypointManager {
    waypoints: HashMap<WaypointId, TrackedWaypoint>,
}

impl WaypointManager {
    /// `ClientboundTrackedWaypointPacket.apply`.
    pub fn apply(&mut self, operation: WaypointOperation, waypoint: TrackedWaypoint) {
        match operation {
            WaypointOperation::Track => {
                self.waypoints.insert(waypoint.id.clone(), waypoint);
            }
            WaypointOperation::Untrack => {
                self.waypoints.remove(&waypoint.id);
            }
            WaypointOperation::Update => {
                if let Some(known) = self.waypoints.get_mut(&waypoint.id) {
                    known.update(&waypoint);
                }
            }
        }
    }

    pub fn has_waypoints(&self) -> bool {
        !self.waypoints.is_empty()
    }

    /// The waypoints, the farthest first from `from` (`forEachWaypoint`),
    /// by the distance a waypoint has from it.
    pub fn farthest_first(
        &self,
        mut distance_squared: impl FnMut(&TrackedWaypoint) -> f64,
    ) -> Vec<&TrackedWaypoint> {
        let mut sorted: Vec<(f64, &TrackedWaypoint)> = self
            .waypoints
            .values()
            .map(|waypoint| (distance_squared(waypoint), waypoint))
            .collect();
        // Waypoints as far as each other (a heading has no distance) keep
        // a fixed order, which the reference leaves to its hash map.
        sorted.sort_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then_with(|| a.1.id.java_hash().cmp(&b.1.id.java_hash()))
        });
        sorted.into_iter().map(|(_, waypoint)| waypoint).collect()
    }
}
