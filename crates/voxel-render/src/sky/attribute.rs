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

/// One attribute value.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    Color(u32),
    Float(f32),
    Phase(u8),
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

impl Operation {
    pub(super) fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "override" => Self::Override,
            "multiply" => Self::Multiply,
            "add" => Self::Add,
            "subtract" => Self::Subtract,
            "minimum" => Self::Minimum,
            "maximum" => Self::Maximum,
            _ => return None,
        })
    }
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

impl Easing {
    pub(super) fn apply(self, x: f32) -> f32 {
        match self {
            Self::Constant => 0.0,
            Self::Linear => x,
            Self::CubicBezier([x1, y1, x2, y2]) => {
                let curve =
                    |v1: f32, v2: f32| (3.0 * v1 - 3.0 * v2 + 1.0, -6.0 * v1 + 3.0 * v2, 3.0 * v1);
                let (xa, xb, xc) = curve(x1, x2);
                let (ya, yb, yc) = curve(y1, y2);
                let sample = |a: f32, b: f32, c: f32, t: f32| ((a * t + b) * t + c) * t;
                let gradient = |t: f32| (3.0 * xa * t + 2.0 * xb) * t + xc;
                let mut t = x;
                let mut solved = None;
                for _ in 0..4 {
                    let error = sample(xa, xb, xc, t) - x;
                    if error.abs() < 1.0e-5 {
                        solved = Some(t);
                        break;
                    }
                    let slope = gradient(t);
                    if slope < 1.0e-5 {
                        break;
                    }
                    t -= (error / slope).clamp(-0.25, 0.25);
                }
                let t = solved.unwrap_or_else(|| {
                    let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
                    let mut t = t;
                    while t0 < t1 {
                        let error = sample(xa, xb, xc, t) - x;
                        if error.abs() < 1.0e-5 {
                            return t;
                        }
                        if error < 0.0 {
                            t0 = t;
                        } else {
                            t1 = t;
                        }
                        t = (t1 + t0) / 2.0;
                    }
                    t
                });
                sample(ya, yb, yc, t)
            }
        }
    }
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

impl Track {
    pub(super) fn parse(
        attribute: Attribute,
        period: Option<i64>,
        object: &JsonObject,
    ) -> Option<Self> {
        let operation = match object.text("modifier") {
            Some(name) => Operation::parse(&name)?,
            None => Operation::Override,
        };
        let easing = match object.get_str("ease") {
            None => Easing::Linear,
            Some(JsonValue::String(name)) => match String::from_utf16(name).ok()?.as_str() {
                "linear" => Easing::Linear,
                "constant" => Easing::Constant,
                _ => return None,
            },
            Some(JsonValue::Object(bezier)) => {
                let values = bezier.array("cubic_bezier")?;
                let mut controls = [0.0_f32; 4];
                for (slot, value) in controls.iter_mut().zip(values) {
                    *slot = value.as_f32()?;
                }
                Easing::CubicBezier(controls)
            }
            Some(_) => return None,
        };
        let mut keyframes = Vec::new();
        for keyframe in object.array("keyframes")? {
            let JsonValue::Object(keyframe) = keyframe else {
                return None;
            };
            let ticks = i64::from(keyframe.i32("ticks")?);
            keyframes.push((
                ticks,
                parse_value(attribute.kind(), keyframe.get_str("value")?)?,
            ));
        }
        let segment = |from: (i64, Value), from_ticks, to: (i64, Value), to_ticks| Segment {
            easing,
            from: from.1,
            from_ticks,
            to: to.1,
            to_ticks,
        };
        let mut segments = Vec::new();
        if keyframes.len() == 1 {
            let value = keyframes[0].1;
            segments.push(Segment {
                easing: Easing::Constant,
                from: value,
                from_ticks: 0,
                to: value,
                to_ticks: 0,
            });
        } else {
            let first = keyframes[0];
            let last = *keyframes.last()?;
            if let Some(period) = period {
                segments.push(segment(last, last.0 - period, first, first.0));
            }
            for pair in keyframes.windows(2) {
                segments.push(segment(pair[0], pair[0].0, pair[1], pair[1].0));
            }
            if let Some(period) = period {
                segments.push(segment(last, last.0, first, first.0 + period));
            }
        }
        Some(Self {
            attribute,
            operation,
            period,
            segments,
        })
    }

    /// `KeyframeTrackSampler.sample`.
    pub(super) fn sample(&self, ticks: i64) -> Value {
        let ticks = match self.period {
            Some(period) => ticks.rem_euclid(period),
            None => ticks,
        };
        let segment = self
            .segments
            .iter()
            .find(|segment| ticks < segment.to_ticks)
            .unwrap_or_else(|| self.segments.last().expect("a track has a segment"));
        if ticks <= segment.from_ticks {
            return segment.from;
        }
        if ticks >= segment.to_ticks {
            return segment.to;
        }
        let alpha =
            (ticks - segment.from_ticks) as f32 / (segment.to_ticks - segment.from_ticks) as f32;
        lerp(
            self.attribute.kind(),
            segment.easing.apply(alpha),
            segment.from,
            segment.to,
        )
    }
}

/// A timeline the dimension runs, and the clock it reads.
#[derive(Clone, Debug)]
pub(super) struct Timeline {
    pub(super) clock: String,
    pub(super) tracks: Vec<Track>,
}

/// Which sky a dimension draws.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Skybox {
    Overworld,
    End,
    None,
}

/// Everything the sky's attributes are computed from.
#[derive(Clone, Debug)]
pub struct Environment {
    pub(super) skybox: Skybox,
    pub(super) dimension: AttributeMap,
    /// Each biome's map, by its network registry number.
    pub(super) biomes: Vec<AttributeMap>,
    pub(super) plains: usize,
    pub(super) biome_attributes: Vec<Attribute>,
    pub(super) timelines: Vec<Timeline>,
    /// `Level.canHaveWeather`: sky light, no ceiling, and not the End.
    pub(super) weather: bool,
    /// Whether each biome has precipitation, by its network number.
    pub(super) precipitation: Vec<bool>,
}

/// How hard it rains and thunders, each 0 to 1, thunder already scaled by
/// the rain as the level reports it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Weather {
    pub rain: f32,
    pub thunder: f32,
}

/// One tick's attribute values.
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    pub(super) values: BTreeMap<Attribute, Value>,
}
