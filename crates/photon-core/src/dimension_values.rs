//! Dimension values a packet carries, apart from the client's world.

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

    pub fn min_section_y(self) -> i32 {
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
