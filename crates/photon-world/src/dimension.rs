//! `ClientWorld` dimension layer split out of the former single-file module.

use photon_core::java_float::{JavaF32, JavaF64};
/// The vertical bounds used when mapping a chunk packet's section order to Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionBounds {
    pub min_y: i32,
    pub height: i32,
}

impl DimensionBounds {
    pub const fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    pub fn from_dimension_type(min_y: i32, height: i32) -> Option<Self> {
        // These limits and alignment checks mirror DimensionType's network codec
        // and record constructor in the pinned Java reference.
        const Y_SIZE: i32 = (1 << 12) - 32;
        const MAX_Y: i32 = (Y_SIZE >> 1) - 1;
        const MIN_Y: i32 = MAX_Y - Y_SIZE + 1;

        if !(MIN_Y..=MAX_Y).contains(&min_y)
            || !(16..=Y_SIZE).contains(&height)
            || height % 16 != 0
            || min_y % 16 != 0
        {
            return None;
        }
        let top = min_y.checked_add(height)?;
        if top > MAX_Y + 1 {
            return None;
        }
        Some(Self::new(min_y, height))
    }

    pub fn section_count(self) -> usize {
        if self.height <= 0 || self.height % 16 != 0 {
            return 0;
        }
        (self.height / 16) as usize
    }

    pub(crate) fn min_section_y(self) -> i32 {
        self.min_y.div_euclid(16)
    }

    /// The lowest and highest quart rows (`QuartPos`, four blocks) of the
    /// dimension.
    pub fn quart_rows(self) -> (i32, i32) {
        let min = self.min_y.div_euclid(4);
        (min, min + self.height / 4 - 1)
    }

    /// A quart row clamped into the dimension, as `LevelReader.getNoiseBiome`
    /// reads the biome storage.
    pub fn clamp_quart_y(self, y: i32) -> i32 {
        let (min, max) = self.quart_rows();
        y.clamp(min, max)
    }
}

impl Default for DimensionBounds {
    fn default() -> Self {
        Self::new(-64, 384)
    }
}

impl DimensionBounds {
    /// Return the bounds of one of Java's built-in dimensions.
    pub fn for_builtin_dimension(dimension_key: &str) -> Self {
        match dimension_key {
            "minecraft:overworld" | "minecraft:overworld_caves" => Self::new(-64, 384),
            "minecraft:the_nether" | "minecraft:the_end" => Self::new(0, 256),
            _ => Self::default(),
        }
    }
}

/// The dimension-type fields consumed by the client world and renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionEnvironment {
    pub(crate) has_fixed_time: bool,
    pub(crate) has_skylight: bool,
    pub(crate) has_ceiling: bool,
    pub(crate) has_ender_dragon_fight: bool,
    pub(crate) coordinate_scale: JavaF64,
    pub(crate) logical_height: i32,
    pub(crate) ambient_light: JavaF32,
    pub(crate) skybox: DimensionSkybox,
    pub(crate) cardinal_light_type: CardinalLightType,
    pub(crate) piglins_zombify: bool,
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
    pub(crate) fn new(input: DimensionEnvironmentInput) -> Self {
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

/// The skybox selected by Java's dimension type codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionSkybox {
    None,
    Overworld,
    End,
}

impl DimensionSkybox {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "overworld" => Some(Self::Overworld),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Overworld => "overworld",
            Self::End => "end",
        }
    }
}

/// The cardinal-light preset selected by Java's dimension type codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalLightType {
    Default,
    Nether,
}

impl CardinalLightType {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "nether" => Some(Self::Nether),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Nether => "nether",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionInput<'a> {
    pub key: &'a str,
    pub dimension_type_id: i32,
    pub seed: i64,
    pub is_debug: bool,
    pub is_flat: bool,
    pub sea_level: i32,
}
