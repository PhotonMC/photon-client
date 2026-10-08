use super::*;

/// The attributes the sky reads.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Attribute {
    SkyColor,
    FogColor,
    SunriseSunsetColor,
    SunAngle,
    MoonAngle,
    StarAngle,
    StarBrightness,
    MoonPhase,
    SkyFogEndDistance,
    FogStartDistance,
    FogEndDistance,
    WaterFogColor,
    WaterFogStartDistance,
    WaterFogEndDistance,
    CloudColor,
    CloudHeight,
    CloudFogEndDistance,
    SkyLightColor,
    SkyLightFactor,
    BlockLightTint,
    AmbientLightColor,
}

pub(super) const ATTRIBUTES: [Attribute; 21] = [
    Attribute::SkyColor,
    Attribute::FogColor,
    Attribute::SunriseSunsetColor,
    Attribute::SunAngle,
    Attribute::MoonAngle,
    Attribute::StarAngle,
    Attribute::StarBrightness,
    Attribute::MoonPhase,
    Attribute::SkyFogEndDistance,
    Attribute::FogStartDistance,
    Attribute::FogEndDistance,
    Attribute::WaterFogStartDistance,
    Attribute::WaterFogEndDistance,
    Attribute::CloudColor,
    Attribute::CloudHeight,
    Attribute::CloudFogEndDistance,
    Attribute::WaterFogColor,
    Attribute::SkyLightColor,
    Attribute::SkyLightFactor,
    Attribute::BlockLightTint,
    Attribute::AmbientLightColor,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Kind {
    Rgb,
    Argb,
    Float,
    Angle,
    Phase,
}

/// One attribute value.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    Color(u32),
    Float(f32),
    Phase(u8),
}

pub(super) const MOON_PHASES: [&str; 8] = [
    "full_moon",
    "waning_gibbous",
    "third_quarter",
    "waning_crescent",
    "new_moon",
    "waxing_crescent",
    "first_quarter",
    "waxing_gibbous",
];

pub const SUN_SPRITE: &str = "minecraft:environment/celestial/sun";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Operation {
    Override,
    Multiply,
    Add,
    Subtract,
    Minimum,
    Maximum,
}

/// `ARGB.color(204, 204, 255)`, what a lightning flash lerps the sky colour towards.
pub(super) const FLASH_COLOUR: u32 = 0xFFCC_CCFF;

/// One entry of an attribute map: a value, or a modifier and its argument.
#[derive(Clone, Copy, Debug)]
pub(super) struct Entry {
    pub(super) operation: Operation,
    pub(super) argument: Value,
}

pub(super) type AttributeMap = BTreeMap<Attribute, Entry>;

#[derive(Clone, Copy, Debug)]
pub(super) enum Easing {
    Constant,
    Linear,
    CubicBezier([f32; 4]),
}

/// One baked segment of a keyframe track.
#[derive(Clone, Copy, Debug)]
pub(super) struct Segment {
    pub(super) easing: Easing,
    pub(super) from: Value,
    pub(super) from_ticks: i64,
    pub(super) to: Value,
    pub(super) to_ticks: i64,
}

/// A timeline's track for one attribute, baked into segments.
#[derive(Clone, Debug)]
pub(super) struct Track {
    pub(super) attribute: Attribute,
    pub(super) operation: Operation,
    pub(super) period: Option<i64>,
    pub(super) segments: Vec<Segment>,
}

/// A timeline the dimension runs, and the clock it reads.
#[derive(Clone, Debug)]
pub(super) struct Timeline {
    pub(super) clock: String,
    pub(super) tracks: Vec<Track>,
}

/// One tick's attribute values.
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    pub(super) values: BTreeMap<Attribute, Value>,
}

impl Attribute {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::SkyColor => "minecraft:visual/sky_color",
            Self::FogColor => "minecraft:visual/fog_color",
            Self::SunriseSunsetColor => "minecraft:visual/sunrise_sunset_color",
            Self::SunAngle => "minecraft:visual/sun_angle",
            Self::MoonAngle => "minecraft:visual/moon_angle",
            Self::StarAngle => "minecraft:visual/star_angle",
            Self::StarBrightness => "minecraft:visual/star_brightness",
            Self::MoonPhase => "minecraft:visual/moon_phase",
            Self::SkyFogEndDistance => "minecraft:visual/sky_fog_end_distance",
            Self::WaterFogColor => "minecraft:visual/water_fog_color",
            Self::FogStartDistance => "minecraft:visual/fog_start_distance",
            Self::FogEndDistance => "minecraft:visual/fog_end_distance",
            Self::WaterFogStartDistance => "minecraft:visual/water_fog_start_distance",
            Self::WaterFogEndDistance => "minecraft:visual/water_fog_end_distance",
            Self::CloudColor => "minecraft:visual/cloud_color",
            Self::CloudHeight => "minecraft:visual/cloud_height",
            Self::CloudFogEndDistance => "minecraft:visual/cloud_fog_end_distance",
            Self::SkyLightColor => "minecraft:visual/sky_light_color",
            Self::SkyLightFactor => "minecraft:visual/sky_light_factor",
            Self::BlockLightTint => "minecraft:visual/block_light_tint",
            Self::AmbientLightColor => "minecraft:visual/ambient_light_color",
        }
    }

    pub(super) fn kind(self) -> Kind {
        match self {
            Self::SkyColor
            | Self::FogColor
            | Self::WaterFogColor
            | Self::SkyLightColor
            | Self::BlockLightTint
            | Self::AmbientLightColor => Kind::Rgb,
            Self::SkyLightFactor => Kind::Float,
            Self::SunriseSunsetColor | Self::CloudColor => Kind::Argb,
            Self::SunAngle | Self::MoonAngle | Self::StarAngle => Kind::Angle,
            Self::StarBrightness
            | Self::SkyFogEndDistance
            | Self::FogStartDistance
            | Self::FogEndDistance
            | Self::WaterFogStartDistance
            | Self::WaterFogEndDistance
            | Self::CloudHeight
            | Self::CloudFogEndDistance => Kind::Float,
            Self::MoonPhase => Kind::Phase,
        }
    }

    pub(super) fn default_value(self) -> Value {
        match self {
            Self::SkyFogEndDistance => Value::Float(512.0),
            Self::FogEndDistance => Value::Float(1024.0),
            Self::WaterFogStartDistance => Value::Float(-8.0),
            Self::WaterFogEndDistance => Value::Float(96.0),
            Self::CloudHeight => Value::Float(192.33),
            Self::CloudFogEndDistance => Value::Float(2048.0),
            Self::WaterFogColor => Value::Color(-16_448_205_i32 as u32),
            Self::MoonPhase => Value::Phase(0),
            Self::SkyLightColor => Value::Color(0xFFFF_FFFF),
            Self::SkyLightFactor => Value::Float(1.0),
            Self::BlockLightTint => Value::Color(-10_100_i32 as u32),
            Self::AmbientLightColor => Value::Color(0xFF00_0000),
            _ => match self.kind() {
                Kind::Rgb | Kind::Argb => Value::Color(0),
                _ => Value::Float(0.0),
            },
        }
    }

    pub(super) fn sanitize(self, value: Value) -> Value {
        match (self, value) {
            (Self::StarBrightness | Self::SkyLightFactor, Value::Float(v)) => {
                Value::Float(v.clamp(0.0, 1.0))
            }
            (
                Self::SkyFogEndDistance
                | Self::FogEndDistance
                | Self::WaterFogEndDistance
                | Self::CloudFogEndDistance,
                Value::Float(v),
            ) => Value::Float(v.max(0.0)),
            _ => value,
        }
    }
}

impl Value {
    pub(super) fn color(self) -> u32 {
        match self {
            Self::Color(color) => color,
            _ => 0,
        }
    }

    pub(super) fn float(self) -> f32 {
        match self {
            Self::Float(value) => value,
            _ => 0.0,
        }
    }

    pub(super) fn phase(self) -> u8 {
        match self {
            Self::Phase(phase) => phase,
            _ => 0,
        }
    }
}
