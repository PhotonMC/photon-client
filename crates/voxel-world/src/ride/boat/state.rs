/// `AbstractBoat.getDefaultGravity`.
pub(super) const GRAVITY: f64 = 0.04;

/// `AbstractBoat.getAirDrag`.
pub(super) const AIR_DRAG: f64 = 1.0;

/// `(float) (Math.PI / 180.0)`.
pub(super) const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;

/// `AbstractBoat.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

/// What a boat keeps from one tick to the next beyond its place and motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RideState {
    /// The status of the last tick and the one before (none before the first tick).
    pub status: Option<Status>,
    pub old_status: Option<Status>,
    pub water_level: f64,
    pub land_friction: f32,
    pub last_yd: f64,
    pub fall_distance: f64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    /// `Entity.stuckSpeedMultiplier`: set by a cobweb or powder snow, spent by the next move.
    pub stuck_speed_multiplier: (f64, f64, f64),
}

/// The values are finite, so the comparison is a total one.
impl Eq for RideState {}

impl Default for RideState {
    fn default() -> Self {
        Self {
            status: None,
            old_status: None,
            water_level: 0.0,
            land_friction: 0.0,
            last_yd: 0.0,
            fall_distance: 0.0,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            stuck_speed_multiplier: (0.0, 0.0, 0.0),
        }
    }
}

impl Status {
    /// `AbstractBoat.isUnderWater`.
    pub fn is_under_water(self) -> bool {
        matches!(self, Self::UnderWater | Self::UnderFlowingWater)
    }
}
