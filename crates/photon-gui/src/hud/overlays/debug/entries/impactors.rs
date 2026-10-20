//! `DebugEntrySimplePerformanceImpactors`: the options that cost the most frames.

use super::super::layout::Line;

/// `CloudStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clouds {
    Off,
    Fast,
    Fancy,
}

/// `TextureFilteringMethod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filtering {
    None,
    Rgss,
    Anisotropic,
}

impl Filtering {
    /// The caption in the game's own language (`options.textureFiltering.*`).
    fn caption(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Rgss => "RGSS",
            Self::Anisotropic => "Anisotropic",
        }
    }
}

/// What the entry reads of the options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Impactors {
    pub improved_transparency: bool,
    pub clouds: Clouds,
    pub biome_blend: i32,
    pub filtering: Filtering,
    /// `Options.maxAnisotropyValue`: the option's power of two, held to what the device supports.
    pub anisotropy: i32,
}

pub fn display(impactors: &Impactors) -> Vec<Line> {
    let transparency = if impactors.improved_transparency {
        "improved-transparency "
    } else {
        ""
    };
    let clouds = match impactors.clouds {
        Clouds::Off => "",
        Clouds::Fast => "fast-clouds ",
        Clouds::Fancy => "fancy-clouds ",
    };
    let filtering = impactors.filtering;
    let second = if filtering == Filtering::Anisotropic {
        format!(
            "Filtering: {} {}x",
            filtering.caption(),
            impactors.anisotropy
        )
    } else {
        format!("Filtering: {}", filtering.caption())
    };
    vec![
        Line::Regular(format!(
            "{transparency}{clouds}B: {}",
            impactors.biome_blend
        )),
        Line::Regular(second),
    ]
}
