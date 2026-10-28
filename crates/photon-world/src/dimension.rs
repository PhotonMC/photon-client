//! `ClientWorld` dimension layer split out of the former single-file module.

pub use photon_core::dimension_values::{CardinalLightType, DimensionBounds, DimensionSkybox};
pub use photon_data::dimension_environment::{DimensionEnvironment, DimensionEnvironmentInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionInput<'a> {
    pub key: &'a str,
    pub dimension_type_id: i32,
    pub seed: i64,
    pub is_debug: bool,
    pub is_flat: bool,
    pub sea_level: i32,
}
