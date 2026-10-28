//! The initialize-border packet's fields.

/// Fields carried by Java's initialize-border packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientWorldBorderInitialization {
    pub center_x: f64,
    pub center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
    pub absolute_max_size: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}
