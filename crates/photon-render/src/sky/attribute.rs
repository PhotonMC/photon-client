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
    NightVisionColor,
    AmbientLightColor,
    FireflyBushSounds,
    CreakingActive,
    SkyLightLevel,
}

pub(super) const ATTRIBUTES: [Attribute; 25] = [
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
    Attribute::NightVisionColor,
    Attribute::AmbientLightColor,
    Attribute::FireflyBushSounds,
    Attribute::CreakingActive,
    Attribute::SkyLightLevel,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Kind {
    Rgb,
    Argb,
    Float,
    Angle,
    Phase,
    Bool,
}

/// One attribute value.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    Color(u32),
    Float(f32),
    Phase(u8),
    Bool(bool),
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
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
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

/// What one attribute is: its name, the type of its values and what it is without any layer.
struct Definition {
    name: &'static str,
    kind: Kind,
    default: Value,
}

const fn definition(name: &'static str, kind: Kind, default: Value) -> Definition {
    Definition {
        name,
        kind,
        default,
    }
}

/// `EnvironmentAttributes`, for the attributes carried, in the order of `Attribute`.
const DEFINITIONS: [Definition; 25] = [
    definition("minecraft:visual/sky_color", Kind::Rgb, Value::Color(0)),
    definition("minecraft:visual/fog_color", Kind::Rgb, Value::Color(0)),
    definition(
        "minecraft:visual/sunrise_sunset_color",
        Kind::Argb,
        Value::Color(0),
    ),
    definition("minecraft:visual/sun_angle", Kind::Angle, Value::Float(0.0)),
    definition(
        "minecraft:visual/moon_angle",
        Kind::Angle,
        Value::Float(0.0),
    ),
    definition(
        "minecraft:visual/star_angle",
        Kind::Angle,
        Value::Float(0.0),
    ),
    definition(
        "minecraft:visual/star_brightness",
        Kind::Float,
        Value::Float(0.0),
    ),
    definition("minecraft:visual/moon_phase", Kind::Phase, Value::Phase(0)),
    definition(
        "minecraft:visual/sky_fog_end_distance",
        Kind::Float,
        Value::Float(512.0),
    ),
    definition(
        "minecraft:visual/fog_start_distance",
        Kind::Float,
        Value::Float(0.0),
    ),
    definition(
        "minecraft:visual/fog_end_distance",
        Kind::Float,
        Value::Float(1024.0),
    ),
    definition(
        "minecraft:visual/water_fog_color",
        Kind::Rgb,
        Value::Color(-16_448_205_i32 as u32),
    ),
    definition(
        "minecraft:visual/water_fog_start_distance",
        Kind::Float,
        Value::Float(-8.0),
    ),
    definition(
        "minecraft:visual/water_fog_end_distance",
        Kind::Float,
        Value::Float(96.0),
    ),
    definition("minecraft:visual/cloud_color", Kind::Argb, Value::Color(0)),
    definition(
        "minecraft:visual/cloud_height",
        Kind::Float,
        Value::Float(192.33),
    ),
    definition(
        "minecraft:visual/cloud_fog_end_distance",
        Kind::Float,
        Value::Float(2048.0),
    ),
    definition(
        "minecraft:visual/sky_light_color",
        Kind::Rgb,
        Value::Color(0xFFFF_FFFF),
    ),
    definition(
        "minecraft:visual/sky_light_factor",
        Kind::Float,
        Value::Float(1.0),
    ),
    definition(
        "minecraft:visual/block_light_tint",
        Kind::Rgb,
        Value::Color(-10_100_i32 as u32),
    ),
    definition(
        "minecraft:visual/night_vision_color",
        Kind::Rgb,
        Value::Color(0xFF99_9999),
    ),
    definition(
        "minecraft:visual/ambient_light_color",
        Kind::Rgb,
        Value::Color(0xFF00_0000),
    ),
    definition(
        "minecraft:audio/firefly_bush_sounds",
        Kind::Bool,
        Value::Bool(false),
    ),
    definition(
        "minecraft:gameplay/creaking_active",
        Kind::Bool,
        Value::Bool(false),
    ),
    definition(
        "minecraft:gameplay/sky_light_level",
        Kind::Float,
        Value::Float(15.0),
    ),
];

impl Attribute {
    fn definition(self) -> &'static Definition {
        &DEFINITIONS[self as usize]
    }

    pub(super) fn name(self) -> &'static str {
        self.definition().name
    }

    pub(super) fn kind(self) -> Kind {
        self.definition().kind
    }

    pub(super) fn default_value(self) -> Value {
        self.definition().default
    }

    pub(super) fn sanitize(self, value: Value) -> Value {
        match (self, value) {
            (Self::StarBrightness | Self::SkyLightFactor, Value::Float(v)) => {
                Value::Float(v.clamp(0.0, 1.0))
            }
            (Self::SkyLightLevel, Value::Float(v)) => Value::Float(v.clamp(0.0, 15.0)),
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

    pub fn float(self) -> f32 {
        match self {
            Self::Float(value) => value,
            _ => 0.0,
        }
    }

    pub fn bool(self) -> bool {
        matches!(self, Self::Bool(true))
    }

    pub(super) fn phase(self) -> u8 {
        match self {
            Self::Phase(phase) => phase,
            _ => 0,
        }
    }
}
