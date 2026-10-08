//! What the data of a salmon, a tropical fish and a pufferfish says of how
//! they are drawn and how big they are (`Salmon.Variant`, `TropicalFish.Pattern`,
//! `Pufferfish.getPuffState`).

/// The paths (under `textures/`) of the fish textures.
pub const COD_TEXTURE: &str = "entity/fish/cod";
pub const SALMON_TEXTURE: &str = "entity/fish/salmon";
pub const PUFFERFISH_TEXTURE: &str = "entity/fish/pufferfish";
pub const TROPICAL_SMALL_TEXTURE: &str = "entity/fish/tropical_a";
pub const TROPICAL_LARGE_TEXTURE: &str = "entity/fish/tropical_b";

/// How many patterns each tropical fish base has.
const PATTERNS_PER_BASE: u8 = 6;

/// `Salmon.Variant` by its id (an id out of range takes the nearest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalmonSize {
    Small,
    Medium,
    Large,
}

impl SalmonSize {
    pub fn from_id(id: i32) -> Self {
        match id.clamp(0, 2) {
            0 => Self::Small,
            1 => Self::Medium,
            _ => Self::Large,
        }
    }

    /// `Salmon.Variant.boundingBoxScale`.
    pub fn bounding_box_scale(self) -> f32 {
        match self {
            Self::Small => 0.5,
            Self::Medium => 1.0,
            Self::Large => 1.5,
        }
    }

    /// The model layer `SalmonRenderer` draws this size with.
    pub fn layer(self) -> &'static str {
        match self {
            Self::Small => "minecraft:salmon_small#main",
            Self::Medium => "minecraft:salmon#main",
            Self::Large => "minecraft:salmon_large#main",
        }
    }
}

/// `SalmonRenderer.setupRotations` and `SalmonModel.setupAnim` swing harder and
/// faster out of water: the factors of the swing's size and of its pace.
pub fn salmon_swing(in_water: bool) -> (f32, f32) {
    if in_water { (1.0, 1.0) } else { (1.3, 1.7) }
}

/// `Pufferfish.getScale` of a puff state: the factor its box is scaled by.
pub fn pufferfish_bounding_box_scale(puff_state: i32) -> f32 {
    match puff_state {
        0 => 0.5,
        1 => 0.7,
        _ => 1.0,
    }
}

/// The model layer `PufferfishRenderer` draws a puff state with.
pub fn pufferfish_layer(puff_state: i32) -> &'static str {
    match puff_state {
        0 => "minecraft:pufferfish_small#main",
        1 => "minecraft:pufferfish_medium#main",
        _ => "minecraft:pufferfish_big#main",
    }
}

/// `TropicalFish.Pattern` of a packed variant: which base it is and which of
/// the base's six patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalPattern {
    pub large: bool,
    pub index: u8,
}

impl TropicalPattern {
    /// `TropicalFish.getPattern`: an id no pattern has is the first (kob).
    pub fn from_packed(packed: i32) -> Self {
        let id = packed & 0xFFFF;
        let (base, index) = (id & 0xFF, (id >> 8) as u8);
        if base > 1 || index >= PATTERNS_PER_BASE {
            return Self {
                large: false,
                index: 0,
            };
        }
        Self {
            large: base == 1,
            index,
        }
    }

    /// The texture of the fish itself (`TropicalFishRenderer.getTextureLocation`).
    pub fn base_texture(self) -> &'static str {
        if self.large {
            TROPICAL_LARGE_TEXTURE
        } else {
            TROPICAL_SMALL_TEXTURE
        }
    }

    /// The texture of the pattern on it (`TropicalFishPatternLayer`).
    pub fn pattern_texture(self) -> &'static str {
        const SMALL: [&str; 6] = [
            "entity/fish/tropical_a_pattern_1",
            "entity/fish/tropical_a_pattern_2",
            "entity/fish/tropical_a_pattern_3",
            "entity/fish/tropical_a_pattern_4",
            "entity/fish/tropical_a_pattern_5",
            "entity/fish/tropical_a_pattern_6",
        ];
        const LARGE: [&str; 6] = [
            "entity/fish/tropical_b_pattern_1",
            "entity/fish/tropical_b_pattern_2",
            "entity/fish/tropical_b_pattern_3",
            "entity/fish/tropical_b_pattern_4",
            "entity/fish/tropical_b_pattern_5",
            "entity/fish/tropical_b_pattern_6",
        ];
        let table = if self.large { LARGE } else { SMALL };
        table[usize::from(self.index)]
    }

    /// The model layers (`#main`, `#pattern`) of this pattern's base.
    pub fn layers(self) -> (&'static str, &'static str) {
        if self.large {
            (
                "minecraft:tropical_fish_large#main",
                "minecraft:tropical_fish_large#pattern",
            )
        } else {
            (
                "minecraft:tropical_fish_small#main",
                "minecraft:tropical_fish_small#pattern",
            )
        }
    }
}

/// `TropicalFish.getBaseColor`'s dye id (an id no dye has is the first, white).
pub fn tropical_base_dye(packed: i32) -> usize {
    dye_of((packed >> 16) & 0xFF)
}

/// `TropicalFish.getPatternColor`'s dye id.
pub fn tropical_pattern_dye(packed: i32) -> usize {
    dye_of((packed >> 24) & 0xFF)
}

fn dye_of(id: i32) -> usize {
    if id < photon_data::entity_type::DYE_COUNT as i32 {
        id as usize
    } else {
        0
    }
}
