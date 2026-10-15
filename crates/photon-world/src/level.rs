//! What the world knows of the level it is in and of the weather over it.

use crate::*;

/// The level's own data: which dimension it is and how it is shaped (`ClientLevel`, `Level`).
#[derive(Debug, Clone)]
pub(crate) struct Level {
    pub(crate) dimension: DimensionBounds,
    pub(crate) dimension_key: String,
    pub(crate) dimension_type_id: i32,
    pub(crate) seed: i64,
    pub(crate) is_debug: bool,
    pub(crate) is_flat: bool,
    pub(crate) sea_level: i32,
    pub(crate) dimension_environment: DimensionEnvironment,
    pub(crate) game_time: u64,
    /// `ClientLevel.serverSimulationDistance`, in chunks.
    pub(crate) server_simulation_distance: i32,
}

impl Level {
    /// The overworld, as a client starts in it.
    pub(crate) fn new(dimension: DimensionBounds) -> Self {
        Self {
            dimension,
            dimension_key: "minecraft:overworld".to_owned(),
            dimension_type_id: 0,
            seed: 0,
            is_debug: false,
            is_flat: false,
            sea_level: 63,
            dimension_environment: DimensionEnvironment::for_builtin_dimension(
                "minecraft:overworld",
            ),
            game_time: 0,
            server_simulation_distance: 3,
        }
    }
}

/// The weather over the level.
#[derive(Debug, Clone)]
pub(crate) struct Weather {
    /// `Level.rainLevel` and `thunderLevel`, as the bits of the floats.
    pub(crate) rain_level: JavaF32,
    pub(crate) thunder_level: JavaF32,
    /// `ClientLevel.skyFlashTime`: ticks the sky still flashes for.
    pub(crate) sky_flash_time: i32,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            rain_level: JavaF32::new(0.0),
            thunder_level: JavaF32::new(0.0),
            sky_flash_time: 0,
        }
    }
}
