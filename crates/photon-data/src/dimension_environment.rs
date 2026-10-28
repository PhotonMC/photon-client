//! The dimension-type fields the client world and renderer consume.

use photon_core::dimension_values::{CardinalLightType, DimensionSkybox};
use photon_core::java_float::{JavaF32, JavaF64};

/// The dimension-type fields consumed by the client world and renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionEnvironment {
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub has_ender_dragon_fight: bool,
    pub coordinate_scale: JavaF64,
    pub logical_height: i32,
    pub ambient_light: JavaF32,
    pub skybox: DimensionSkybox,
    pub cardinal_light_type: CardinalLightType,
    pub piglins_zombify: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DimensionEnvironmentInput {
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub has_ender_dragon_fight: bool,
    pub coordinate_scale: f64,
    pub logical_height: i32,
    pub ambient_light: f32,
    pub skybox: DimensionSkybox,
    pub cardinal_light_type: CardinalLightType,
    /// `EnvironmentAttributes.PIGLINS_ZOMBIFY` of the dimension type.
    pub piglins_zombify: bool,
}

impl DimensionEnvironment {
    pub fn new(input: DimensionEnvironmentInput) -> Self {
        Self {
            has_fixed_time: input.has_fixed_time,
            has_skylight: input.has_skylight,
            has_ceiling: input.has_ceiling,
            has_ender_dragon_fight: input.has_ender_dragon_fight,
            coordinate_scale: JavaF64::new(input.coordinate_scale),
            logical_height: input.logical_height,
            ambient_light: JavaF32::new(input.ambient_light),
            skybox: input.skybox,
            cardinal_light_type: input.cardinal_light_type,
            piglins_zombify: input.piglins_zombify,
        }
    }

    pub fn from_wire(input: DimensionEnvironmentInput, height: i32) -> Option<Self> {
        if !input.coordinate_scale.is_finite()
            || !(1.0e-5..=3.0e7).contains(&input.coordinate_scale)
            || !(0..=height).contains(&input.logical_height)
        {
            return None;
        }
        Some(Self::new(input))
    }

    /// Return the default environment for a built-in dimension.
    pub fn for_builtin_dimension(dimension_key: &str) -> Self {
        match dimension_key {
            "minecraft:the_nether" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: true,
                has_skylight: false,
                has_ceiling: true,
                has_ender_dragon_fight: false,
                coordinate_scale: 8.0,
                logical_height: 128,
                ambient_light: 0.1,
                skybox: DimensionSkybox::None,
                cardinal_light_type: CardinalLightType::Nether,
                piglins_zombify: false,
            }),
            "minecraft:the_end" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: true,
                has_skylight: true,
                has_ceiling: false,
                has_ender_dragon_fight: true,
                coordinate_scale: 1.0,
                logical_height: 256,
                ambient_light: 0.25,
                skybox: DimensionSkybox::End,
                cardinal_light_type: CardinalLightType::Default,
                piglins_zombify: true,
            }),
            "minecraft:overworld_caves" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: false,
                has_skylight: true,
                has_ceiling: true,
                has_ender_dragon_fight: false,
                coordinate_scale: 1.0,
                logical_height: 384,
                ambient_light: 0.0,
                skybox: DimensionSkybox::Overworld,
                cardinal_light_type: CardinalLightType::Default,
                piglins_zombify: true,
            }),
            _ => Self::new(DimensionEnvironmentInput {
                has_fixed_time: false,
                has_skylight: true,
                has_ceiling: false,
                has_ender_dragon_fight: false,
                coordinate_scale: 1.0,
                logical_height: 384,
                ambient_light: 0.0,
                skybox: DimensionSkybox::Overworld,
                cardinal_light_type: CardinalLightType::Default,
                piglins_zombify: true,
            }),
        }
    }

    /// Whether piglins and hoglins turn into their zombified kin here.
    pub fn piglins_zombify(&self) -> bool {
        self.piglins_zombify
    }

    pub fn has_fixed_time(&self) -> bool {
        self.has_fixed_time
    }

    pub fn has_skylight(&self) -> bool {
        self.has_skylight
    }

    pub fn has_ceiling(&self) -> bool {
        self.has_ceiling
    }

    pub fn has_ender_dragon_fight(&self) -> bool {
        self.has_ender_dragon_fight
    }

    pub fn coordinate_scale(&self) -> f64 {
        self.coordinate_scale.get()
    }

    pub fn logical_height(&self) -> i32 {
        self.logical_height
    }

    pub fn ambient_light(&self) -> f32 {
        self.ambient_light.get()
    }

    pub fn skybox(&self) -> DimensionSkybox {
        self.skybox
    }

    pub fn cardinal_light_type(&self) -> CardinalLightType {
        self.cardinal_light_type
    }
}
