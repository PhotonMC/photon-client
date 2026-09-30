//! The overworld sky: what colour and where everything in it is, and the
//! geometry that draws it.
//!
//! The reference does not hard-code its sky. Every value the sky reads is an
//! environment attribute: a default, then the dimension's value, then the
//! biomes' values blended around the camera, then each timeline the dimension
//! runs, sampled at its clock. This module reads those same data files from
//! the local vanilla data pack and evaluates them in that order, once per
//! tick, and blends the last two ticks by the partial tick as the camera's
//! attribute probe does.
//!
//! The geometry is the sky renderer's: a disc above in the sky colour faded
//! into the fog colour with distance, the sunrise fan, the sun and the moon
//! phase on quads a hundred blocks out, the seeded star field, and the dark
//! disc below the horizon.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use voxel_core::json::{JsonObject, JsonValue};
use voxel_core::math::{mth_cos_f64, mth_sin_f64};
use voxel_core::random::RandomSource;

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

const ATTRIBUTES: [Attribute; 21] = [
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
enum Kind {
    Rgb,
    Argb,
    Float,
    Angle,
    Phase,
}

impl Attribute {
    fn name(self) -> &'static str {
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

    fn kind(self) -> Kind {
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

    fn default_value(self) -> Value {
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

    fn sanitize(self, value: Value) -> Value {
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
    fn color(self) -> u32 {
        match self {
            Self::Color(color) => color,
            _ => 0,
        }
    }

    fn float(self) -> f32 {
        match self {
            Self::Float(value) => value,
            _ => 0.0,
        }
    }

    fn phase(self) -> u8 {
        match self {
            Self::Phase(phase) => phase,
            _ => 0,
        }
    }
}

const MOON_PHASES: [&str; 8] = [
    "full_moon",
    "waning_gibbous",
    "third_quarter",
    "waning_crescent",
    "new_moon",
    "waxing_crescent",
    "first_quarter",
    "waxing_gibbous",
];

/// The moon phase sprite names, in the phases' order.
pub fn moon_sprites() -> impl Iterator<Item = String> {
    MOON_PHASES
        .iter()
        .map(|phase| format!("minecraft:environment/celestial/moon/{phase}"))
}

pub const SUN_SPRITE: &str = "minecraft:environment/celestial/sun";

fn parse_value(kind: Kind, value: &JsonValue) -> Option<Value> {
    match kind {
        Kind::Rgb | Kind::Argb => match value {
            JsonValue::String(text) => {
                let text = String::from_utf16(text).ok()?;
                let hex = text.strip_prefix('#')?;
                let parsed = u32::from_str_radix(hex, 16).ok()?;
                match hex.len() {
                    6 => Some(Value::Color(0xFF00_0000 | parsed)),
                    8 => Some(Value::Color(parsed)),
                    _ => None,
                }
            }
            other => Some(Value::Color(other.as_i64()? as u32)),
        },
        Kind::Float | Kind::Angle => Some(Value::Float(value.as_f32()?)),
        Kind::Phase => match value {
            JsonValue::String(text) => {
                let text = String::from_utf16(text).ok()?;
                MOON_PHASES
                    .iter()
                    .position(|phase| *phase == text)
                    .map(|index| Value::Phase(index as u8))
            }
            _ => None,
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Operation {
    Override,
    Multiply,
    Add,
    Subtract,
    Minimum,
    Maximum,
}

impl Operation {
    fn parse(name: &str) -> Option<Self> {
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

fn channel(color: u32, shift: u32) -> i32 {
    ((color >> shift) & 0xff) as i32
}

use crate::tint::{argb, multiply};

fn lerp_int(alpha: f32, from: i32, to: i32) -> i32 {
    from + (alpha * (to - from) as f32).floor() as i32
}

/// `ARGB.srgbLerp`: each channel on its own, truncated as integers are.
pub fn srgb_lerp(alpha: f32, from: u32, to: u32) -> u32 {
    argb(
        lerp_int(alpha, channel(from, 24), channel(to, 24)),
        lerp_int(alpha, channel(from, 16), channel(to, 16)),
        lerp_int(alpha, channel(from, 8), channel(to, 8)),
        lerp_int(alpha, channel(from, 0), channel(to, 0)),
    )
}

fn lerp_f32(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * (to - from)
}

fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

fn apply(kind: Kind, operation: Operation, subject: Value, argument: Value) -> Value {
    match (kind, operation) {
        (_, Operation::Override) => argument,
        (Kind::Rgb | Kind::Argb, Operation::Multiply) => {
            Value::Color(multiply(subject.color(), argument.color()))
        }
        (Kind::Rgb | Kind::Argb, Operation::Add) => {
            let (s, a) = (subject.color(), argument.color());
            Value::Color(argb(
                channel(s, 24),
                (channel(s, 16) + channel(a, 16)).min(255),
                (channel(s, 8) + channel(a, 8)).min(255),
                (channel(s, 0) + channel(a, 0)).min(255),
            ))
        }
        (Kind::Rgb | Kind::Argb, Operation::Subtract) => {
            let (s, a) = (subject.color(), argument.color());
            Value::Color(argb(
                channel(s, 24),
                (channel(s, 16) - channel(a, 16)).max(0),
                (channel(s, 8) - channel(a, 8)).max(0),
                (channel(s, 0) - channel(a, 0)).max(0),
            ))
        }
        (Kind::Float | Kind::Angle, operation) => {
            let (s, a) = (subject.float(), argument.float());
            Value::Float(match operation {
                Operation::Multiply => s * a,
                Operation::Add => s + a,
                Operation::Subtract => s - a,
                Operation::Minimum => s.min(a),
                Operation::Maximum => s.max(a),
                Operation::Override => a,
            })
        }
        _ => subject,
    }
}

/// The lerp a keyframe track and a biome blend use.
fn lerp(kind: Kind, alpha: f32, from: Value, to: Value) -> Value {
    match kind {
        Kind::Rgb | Kind::Argb => Value::Color(srgb_lerp(alpha, from.color(), to.color())),
        Kind::Float | Kind::Angle => Value::Float(lerp_f32(alpha, from.float(), to.float())),
        // Not interpolated: a keyframe switches when its segment ends.
        Kind::Phase => {
            if alpha >= 1.0 {
                to
            } else {
                from
            }
        }
    }
}

/// The lerp between the last two ticks' values.
fn partial_tick_lerp(kind: Kind, alpha: f32, from: Value, to: Value) -> Value {
    match kind {
        Kind::Angle => {
            let (from, to) = (from.float(), to.float());
            let delta = wrap_degrees(to - from);
            Value::Float(if delta.abs() >= 90.0 {
                to
            } else {
                from + alpha * delta
            })
        }
        Kind::Phase => to,
        _ => lerp(kind, alpha, from, to),
    }
}

/// One entry of an attribute map: a value, or a modifier and its argument.
#[derive(Clone, Copy, Debug)]
struct Entry {
    operation: Operation,
    argument: Value,
}

fn parse_entry(attribute: Attribute, value: &JsonValue) -> Option<Entry> {
    if let JsonValue::Object(object) = value
        && attribute.kind() != Kind::Phase
        && object.has("modifier")
    {
        let operation = Operation::parse(&object.text("modifier")?)?;
        let argument = parse_value(attribute.kind(), object.get_str("argument")?)?;
        return Some(Entry {
            operation,
            argument,
        });
    }
    Some(Entry {
        operation: Operation::Override,
        argument: parse_value(attribute.kind(), value)?,
    })
}

type AttributeMap = BTreeMap<Attribute, Entry>;

fn parse_attribute_map(object: Option<&JsonObject>) -> AttributeMap {
    let mut map = AttributeMap::new();
    let Some(object) = object else {
        return map;
    };
    for attribute in ATTRIBUTES {
        if let Some(value) = object.get_str(attribute.name())
            && let Some(entry) = parse_entry(attribute, value)
        {
            map.insert(attribute, entry);
        }
    }
    map
}

#[derive(Clone, Copy, Debug)]
enum Easing {
    Constant,
    Linear,
    CubicBezier([f32; 4]),
}

impl Easing {
    fn apply(self, x: f32) -> f32 {
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
struct Segment {
    easing: Easing,
    from: Value,
    from_ticks: i64,
    to: Value,
    to_ticks: i64,
}

/// A timeline's track for one attribute, baked into segments.
#[derive(Clone, Debug)]
struct Track {
    attribute: Attribute,
    operation: Operation,
    period: Option<i64>,
    segments: Vec<Segment>,
}

impl Track {
    fn parse(attribute: Attribute, period: Option<i64>, object: &JsonObject) -> Option<Self> {
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
    fn sample(&self, ticks: i64) -> Value {
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
struct Timeline {
    clock: String,
    tracks: Vec<Track>,
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
    skybox: Skybox,
    dimension: AttributeMap,
    /// Each biome's map, by its network registry number.
    biomes: Vec<AttributeMap>,
    plains: usize,
    biome_attributes: Vec<Attribute>,
    timelines: Vec<Timeline>,
    /// `Level.canHaveWeather`: sky light, no ceiling, and not the End.
    weather: bool,
    /// Whether each biome has precipitation, by its network number.
    precipitation: Vec<bool>,
}

/// How hard it rains and thunders, each 0 to 1, thunder already scaled by
/// the rain as the level reports it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Weather {
    pub rain: f32,
    pub thunder: f32,
}

/// `WeatherAttributes`: how rain and thunder change one attribute's value,
/// or nothing for an attribute they leave alone.
fn weather_modifier(attribute: Attribute, thunder: bool, value: Value) -> Option<Value> {
    // BlendToGray: the greyscale of the colour, scaled, blended in.
    let blend_to_grey = |brightness: f32, factor: f32| {
        let colour = value.color();
        let grey = (channel(colour, 16) as f32 * 0.3
            + channel(colour, 8) as f32 * 0.59
            + channel(colour, 0) as f32 * 0.11) as i32;
        let scaled = ((grey as f32 * brightness) as i32).clamp(0, 255);
        let grey = argb(channel(colour, 24), scaled, scaled, scaled);
        Value::Color(srgb_lerp(factor, colour, grey))
    };
    let float_blend =
        |target: f32, alpha: f32| Value::Float(lerp_f32(alpha, value.float(), target));
    // The night sky light colour, at the alpha the weather blends it in by.
    let night_sky_light = |alpha: f32| {
        let colour = argb(
            (alpha * 255.0).floor() as i32,
            (0.48_f32 * 255.0).floor() as i32,
            (0.48_f32 * 255.0).floor() as i32,
            255,
        );
        Value::Color(alpha_blend(value.color(), colour))
    };
    let multiply_by = |red: f32, green: f32, blue: f32| {
        let by = argb(
            255,
            (red * 255.0).floor() as i32,
            (green * 255.0).floor() as i32,
            (blue * 255.0).floor() as i32,
        );
        Value::Color(multiply(value.color(), by))
    };
    Some(match (attribute, thunder) {
        (Attribute::SkyColor, false) => blend_to_grey(0.6, 0.75),
        (Attribute::SkyColor, true) => blend_to_grey(0.24, 0.94),
        (Attribute::FogColor | Attribute::SunriseSunsetColor, false) => multiply_by(0.5, 0.5, 0.6),
        (Attribute::FogColor | Attribute::SunriseSunsetColor, true) => multiply_by(0.25, 0.25, 0.3),
        (Attribute::CloudColor, false) => blend_to_grey(0.24, 0.5),
        (Attribute::CloudColor, true) => blend_to_grey(0.095, 0.94),
        (Attribute::SkyLightColor, false) => night_sky_light(0.3125),
        (Attribute::SkyLightColor, true) => night_sky_light(0.527_343_75),
        (Attribute::SkyLightFactor, false) => float_blend(0.24, 0.3125),
        (Attribute::SkyLightFactor, true) => float_blend(0.24, 0.527_343_75),
        (Attribute::StarBrightness, _) => Value::Float(0.0),
        _ => return None,
    })
}

/// `ARGB.alphaBlend`: the source over the destination by the source's alpha.
fn alpha_blend(destination: u32, source: u32) -> u32 {
    let source_alpha = channel(source, 24);
    if source_alpha == 255 {
        return source;
    }
    if source_alpha == 0 {
        return destination;
    }
    let alpha = source_alpha + channel(destination, 24) * (255 - source_alpha) / 255;
    let blend = |shift: u32| {
        (channel(source, shift) * source_alpha
            + channel(destination, shift) * (alpha - source_alpha))
            / alpha
    };
    argb(alpha, blend(16), blend(8), blend(0))
}

fn data_path(root: &Path, kind: &str, name: &str, extension: &str) -> PathBuf {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    root.join(namespace)
        .join(kind)
        .join(format!("{path}{extension}"))
}

/// A tag's entries with nested tags expanded, in order.
fn expand_tag(root: &Path, kind: &str, name: &str, out: &mut Vec<String>) {
    let Some(tag) =
        voxel_core::json::read_object(&data_path(root, &format!("tags/{kind}"), name, ".json"))
    else {
        return;
    };
    for value in tag.array("values").unwrap_or(&[]) {
        let entry = match value {
            JsonValue::String(text) => String::from_utf16(text).unwrap_or_default(),
            JsonValue::Object(object) => object.text("id").unwrap_or_default(),
            _ => continue,
        };
        if let Some(nested) = entry.strip_prefix('#') {
            expand_tag(root, kind, nested, out);
        } else if !out.contains(&entry) {
            out.push(entry);
        }
    }
}

impl Environment {
    /// Read a dimension's sky data from a data pack folder (the one holding
    /// `minecraft/`), with the biomes numbered as the server numbered them.
    pub fn load(data_root: &Path, dimension_type: &str, biome_names: &[String]) -> Option<Self> {
        let dimension = voxel_core::json::read_object(&data_path(
            data_root,
            "dimension_type",
            dimension_type,
            ".json",
        ))?;
        let skybox = match dimension.text("skybox").as_deref() {
            Some("end") => Skybox::End,
            Some("none") => Skybox::None,
            _ => Skybox::Overworld,
        };
        let dimension_map = parse_attribute_map(dimension.object("attributes"));
        let weather = dimension.bool("has_skylight").unwrap_or(false)
            && !dimension.bool("has_ceiling").unwrap_or(false)
            && dimension_type != "minecraft:the_end";
        let biome_files: Vec<Option<JsonObject>> = biome_names
            .iter()
            .map(|name| {
                voxel_core::json::read_object(&data_path(
                    data_root,
                    "worldgen/biome",
                    name,
                    ".json",
                ))
            })
            .collect();
        let precipitation = biome_files
            .iter()
            .map(|biome| {
                biome
                    .as_ref()
                    .and_then(|biome| biome.bool("has_precipitation"))
                    .unwrap_or(false)
            })
            .collect();
        let biomes: Vec<AttributeMap> = biome_files
            .iter()
            .map(|biome| {
                parse_attribute_map(biome.as_ref().and_then(|biome| biome.object("attributes")))
            })
            .collect();
        let plains = biome_names
            .iter()
            .position(|name| name == "minecraft:plains")
            .unwrap_or(0);
        let mut biome_attributes = Vec::new();
        for map in &biomes {
            for attribute in map.keys() {
                if !biome_attributes.contains(attribute) {
                    biome_attributes.push(*attribute);
                }
            }
        }
        let mut timeline_names = Vec::new();
        match dimension.get_str("timelines") {
            Some(JsonValue::String(text)) => {
                let text = String::from_utf16(text).ok()?;
                match text.strip_prefix('#') {
                    Some(tag) => expand_tag(data_root, "timeline", tag, &mut timeline_names),
                    None => timeline_names.push(text),
                }
            }
            Some(JsonValue::Array(list)) => {
                for value in list {
                    if let JsonValue::String(text) = value {
                        timeline_names.push(String::from_utf16(text).ok()?);
                    }
                }
            }
            _ => {}
        }
        let mut timelines = Vec::new();
        for name in timeline_names {
            let Some(timeline) =
                voxel_core::json::read_object(&data_path(data_root, "timeline", &name, ".json"))
            else {
                continue;
            };
            let clock = timeline.text("clock").unwrap_or_default();
            let period = timeline.i32("period_ticks").map(i64::from);
            let mut tracks = Vec::new();
            if let Some(object) = timeline.object("tracks") {
                for attribute in ATTRIBUTES {
                    if let Some(JsonValue::Object(track)) = object.get_str(attribute.name())
                        && let Some(track) = Track::parse(attribute, period, track)
                    {
                        tracks.push(track);
                    }
                }
            }
            timelines.push(Timeline { clock, tracks });
        }
        Some(Self {
            skybox,
            dimension: dimension_map,
            biomes,
            plains,
            biome_attributes,
            timelines,
            weather,
            precipitation,
        })
    }

    /// Whether the biome with this network number has precipitation; an
    /// unknown number reads as plains.
    pub fn has_precipitation(&self, biome: Option<i32>) -> bool {
        let biome = biome
            .and_then(|id| usize::try_from(id).ok())
            .filter(|id| *id < self.precipitation.len())
            .unwrap_or(self.plains);
        self.precipitation.get(biome).copied().unwrap_or(false)
    }

    pub fn skybox(&self) -> Skybox {
        self.skybox
    }

    /// Every attribute at a camera position and the clocks' current ticks.
    ///
    /// `biome_at` answers the network biome number at a quart position, or
    /// nothing where no chunk is loaded, which the client reads as plains.
    pub fn evaluate(
        &self,
        eye: [f64; 3],
        biome_at: impl Fn(i32, i32, i32) -> Option<i32>,
        clock_ticks: impl Fn(&str) -> i64,
        weather: Weather,
    ) -> Attributes {
        // GaussianSampler over the biomes around the camera, a quarter scale.
        let mut weights: Vec<(usize, f64)> = Vec::new();
        const KERNEL: [f64; 7] = [0.0, 1.0, 4.0, 6.0, 4.0, 1.0, 0.0];
        let position = [
            eye[0] * 0.25 - 0.5,
            eye[1] * 0.25 - 0.5,
            eye[2] * 0.25 - 0.5,
        ];
        let integral = position.map(|v| v.floor() as i32);
        let relative = [
            position[0] - f64::from(integral[0]),
            position[1] - f64::from(integral[1]),
            position[2] - f64::from(integral[2]),
        ];
        let weight = |axis: usize, step: usize| {
            KERNEL[step + 1] + relative[axis] * (KERNEL[step] - KERNEL[step + 1])
        };
        for z in 0..6 {
            let weight_z = weight(2, z);
            for x in 0..6 {
                let weight_x = weight(0, x);
                for y in 0..6 {
                    let weight_y = weight(1, y);
                    let biome = biome_at(
                        integral[0] - 2 + x as i32,
                        integral[1] - 2 + y as i32,
                        integral[2] - 2 + z as i32,
                    )
                    .and_then(|id| usize::try_from(id).ok())
                    .filter(|id| *id < self.biomes.len())
                    .unwrap_or(self.plains);
                    let sample = weight_x * weight_y * weight_z;
                    match weights.iter_mut().find(|(id, _)| *id == biome) {
                        Some((_, total)) => *total += sample,
                        None => weights.push((biome, sample)),
                    }
                }
            }
        }

        let mut values = BTreeMap::new();
        for attribute in ATTRIBUTES {
            let kind = attribute.kind();
            let mut value = attribute.default_value();
            if let Some(entry) = self.dimension.get(&attribute) {
                value = apply(kind, entry.operation, value, entry.argument);
            }
            if self.biome_attributes.contains(&attribute) {
                let from_biome = |biome: usize, base: Value| match self
                    .biomes
                    .get(biome)
                    .and_then(|map| map.get(&attribute))
                {
                    Some(entry) => apply(kind, entry.operation, base, entry.argument),
                    None => base,
                };
                value = match weights.as_slice() {
                    [] => value,
                    [(biome, _)] => from_biome(*biome, value),
                    _ if kind == Kind::Phase => from_biome(weights[0].0, value),
                    _ => {
                        let mut result: Option<Value> = None;
                        let mut accumulated = 0.0_f64;
                        for (biome, weight) in &weights {
                            let source = from_biome(*biome, value);
                            accumulated += weight;
                            result = Some(match result {
                                None => source,
                                Some(previous) => {
                                    lerp(kind, (weight / accumulated) as f32, previous, source)
                                }
                            });
                        }
                        result.unwrap_or(value)
                    }
                };
            }
            for timeline in &self.timelines {
                for track in timeline
                    .tracks
                    .iter()
                    .filter(|track| track.attribute == attribute)
                {
                    let argument = track.sample(clock_ticks(&timeline.clock));
                    value = apply(kind, track.operation, value, argument);
                }
            }
            // The weather layer: the rain's change blended in by the rain
            // not already thunder, then the thunder's by the thunder.
            if self.weather {
                let rain = weather.rain - weather.thunder;
                if rain > 0.0
                    && let Some(rained) = weather_modifier(attribute, false, value)
                {
                    value = lerp(kind, rain, value, rained);
                }
                if weather.thunder > 0.0
                    && let Some(thundered) = weather_modifier(attribute, true, value)
                {
                    value = lerp(kind, weather.thunder, value, thundered);
                }
            }
            values.insert(attribute, attribute.sanitize(value));
        }
        Attributes { values }
    }
}

/// One tick's attribute values.
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    values: BTreeMap<Attribute, Value>,
}

impl Attributes {
    fn get(&self, attribute: Attribute) -> Value {
        self.values
            .get(&attribute)
            .copied()
            .unwrap_or_else(|| attribute.default_value())
    }

    /// Blend towards the next tick's values as the attribute probe does.
    pub fn lerp(&self, next: &Self, partial_tick: f32) -> Self {
        let values = ATTRIBUTES
            .into_iter()
            .map(|attribute| {
                (
                    attribute,
                    partial_tick_lerp(
                        attribute.kind(),
                        partial_tick,
                        self.get(attribute),
                        next.get(attribute),
                    ),
                )
            })
            .collect();
        Self { values }
    }
}

/// The lightmap's inputs, as the lightmap extractor reads them.
#[derive(Debug, Clone, Copy)]
pub struct LightmapInputs {
    pub sky_colour: [f32; 3],
    pub sky_factor: f32,
    pub block_tint: [f32; 3],
    pub ambient: [f32; 3],
}

impl Attributes {
    /// The sky light colour and factor, the block light tint and the ambient
    /// colour, each colour as the lightmap takes it (0 to 1 per channel).
    pub fn lightmap(&self) -> LightmapInputs {
        let rgb = |attribute: Attribute| {
            let colour = self.get(attribute).color();
            [
                channel(colour, 16) as f32 / 255.0,
                channel(colour, 8) as f32 / 255.0,
                channel(colour, 0) as f32 / 255.0,
            ]
        };
        LightmapInputs {
            sky_colour: rgb(Attribute::SkyLightColor),
            sky_factor: self.get(Attribute::SkyLightFactor).float(),
            block_tint: rgb(Attribute::BlockLightTint),
            ambient: rgb(Attribute::AmbientLightColor),
        }
    }
}

/// A sky vertex, in view-rotated space around the camera.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SkyVertex {
    pub position: [f32; 3],
    /// Atlas coordinate, or negative for an untextured vertex.
    pub uv: [f32; 2],
    pub colour: [f32; 4],
    /// The spherical and cylindrical distances fog is measured over, or
    /// negative for no fog.
    pub fog_distance: [f32; 2],
}

/// How a run of sky vertices blends onto what is below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyBlend {
    /// Source alpha over the destination.
    Translucent,
    /// Source times its alpha added to the destination.
    Overlay,
}

/// Everything one frame of sky needs.
#[derive(Debug, Clone)]
pub struct SkyFrame {
    /// The colour the frame is cleared to, which is the fog colour.
    pub clear_colour: [f32; 4],
    pub fog_colour: [f32; 4],
    /// Where fog on the sky is complete, in blocks.
    pub sky_fog_end: f32,
    /// The terrain's fog, in blocks: where the environmental fog starts and
    /// ends over spherical distance, and where the render distance fog
    /// starts and ends over cylindrical distance.
    pub terrain_fog: [f32; 4],
    /// The cloud colour (red, green, blue, alpha, 0 to 1), the height of the
    /// clouds' bottom, and where cloud fog is complete, in blocks.
    pub cloud_colour: [f32; 4],
    pub cloud_height: f32,
    pub cloud_fog_end: f32,
    /// Triangles, three vertices each.
    pub vertices: Vec<SkyVertex>,
    /// Runs of triangles and how each blends.
    pub batches: Vec<(SkyBlend, u32, u32)>,
}

fn unpack(colour: u32) -> [f32; 4] {
    [
        channel(colour, 16) as f32 / 255.0,
        channel(colour, 8) as f32 / 255.0,
        channel(colour, 0) as f32 / 255.0,
        channel(colour, 24) as f32 / 255.0,
    ]
}

type Matrix = [[f32; 4]; 4];

/// `left * right`, both row-major as `m[row][column]`.
fn mul(left: Matrix, right: Matrix) -> Matrix {
    let mut out = [[0.0; 4]; 4];
    for (row, out_row) in out.iter_mut().enumerate() {
        for (column, cell) in out_row.iter_mut().enumerate() {
            *cell = (0..4).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    out
}

fn rotate_x(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotate_y(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotate_z(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn translate(x: f32, y: f32, z: f32) -> Matrix {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn scale(x: f32, y: f32, z: f32) -> Matrix {
    [
        [x, 0.0, 0.0, 0.0],
        [0.0, y, 0.0, 0.0],
        [0.0, 0.0, z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn transform(matrix: Matrix, point: [f32; 3]) -> [f32; 3] {
    let row = |r: usize| {
        matrix[r][0] * point[0] + matrix[r][1] * point[1] + matrix[r][2] * point[2] + matrix[r][3]
    };
    [row(0), row(1), row(2)]
}

/// The seeded star field, as quads around the origin, built once.
pub fn build_stars() -> Vec<[[f32; 3]; 4]> {
    let mut random = RandomSource::create(10842);
    let mut stars = Vec::new();
    for _ in 0..1500 {
        let x = random.next_float() * 2.0 - 1.0;
        let y = random.next_float() * 2.0 - 1.0;
        let z = random.next_float() * 2.0 - 1.0;
        let size = 0.15 + random.next_float() * 0.1;
        let length_squared = x * x + y * y + z * z;
        if length_squared <= 0.010_000_001 || length_squared >= 1.0 {
            continue;
        }
        let length = length_squared.sqrt();
        let centre = [x / length * 100.0, y / length * 100.0, z / length * 100.0];
        let z_rotation = (random.next_double() * f64::from(std::f32::consts::PI) * 2.0) as f32;
        // A basis whose +Z faces the origin, then turned about that axis.
        let forward = {
            let f = [-centre[0], -centre[1], -centre[2]];
            let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            [f[0] / l, f[1] / l, f[2] / l]
        };
        let up = [0.0_f32, 1.0, 0.0];
        let left = {
            let c = [
                up[1] * forward[2] - up[2] * forward[1],
                up[2] * forward[0] - up[0] * forward[2],
                up[0] * forward[1] - up[1] * forward[0],
            ];
            let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            [c[0] / l, c[1] / l, c[2] / l]
        };
        let upn = [
            forward[1] * left[2] - forward[2] * left[1],
            forward[2] * left[0] - forward[0] * left[2],
            forward[0] * left[1] - forward[1] * left[0],
        ];
        let (s, c) = ((-z_rotation).sin(), (-z_rotation).cos());
        let corner = |a: f32, b: f32| {
            let (u, v) = (a * c - b * s, a * s + b * c);
            [
                left[0] * u + upn[0] * v + centre[0],
                left[1] * u + upn[1] * v + centre[1],
                left[2] * u + upn[2] * v + centre[2],
            ]
        };
        stars.push([
            corner(size, -size),
            corner(size, size),
            corner(-size, size),
            corner(-size, -size),
        ]);
    }
    stars
}

/// Where the sky's sprites sit on the atlas.
pub struct SkySprites {
    pub sun: [f32; 4],
    pub moon: [[f32; 4]; 8],
}

/// The inputs a frame of sky reads besides the attributes.
pub struct SkyView {
    /// The camera's forward direction.
    pub forward: [f32; 3],
    /// The camera's eye height above the horizon (sea level on a normal world).
    pub eye_above_horizon: f64,
    pub eye_in_water: bool,
    /// The effective render distance in chunks.
    pub render_distance: i32,
    /// The cloud range option, in chunks.
    pub cloud_range: i32,
    /// The level's rain and thunder at this frame.
    pub weather: Weather,
    /// How far rain has pulled the fog in, 0 to 1.
    pub rain_fog: f32,
}

/// Build one frame of sky from the blended attributes.
pub fn build_frame(
    attributes: &Attributes,
    view: &SkyView,
    stars: &[[[f32; 3]; 4]],
    sprites: &SkySprites,
    skybox: Skybox,
) -> SkyFrame {
    let sky_colour = attributes.get(Attribute::SkyColor).color();
    let sunrise = attributes.get(Attribute::SunriseSunsetColor).color();
    let sun_angle = attributes.get(Attribute::SunAngle).float() * std::f32::consts::PI / 180.0;
    let moon_angle = attributes.get(Attribute::MoonAngle).float() * std::f32::consts::PI / 180.0;
    let star_angle = attributes.get(Attribute::StarAngle).float() * std::f32::consts::PI / 180.0;
    let star_brightness = attributes.get(Attribute::StarBrightness).float();
    let moon_phase = usize::from(attributes.get(Attribute::MoonPhase).phase()).min(7);
    let render_blocks = (view.render_distance * 16) as f32;

    // AtmosphericFogEnvironment.getBaseColor, then the fog renderer's scaling;
    // under water the water fog colour takes its place.
    let fog = if view.eye_in_water {
        attributes.get(Attribute::WaterFogColor).color()
    } else {
        let mut fog = attributes.get(Attribute::FogColor).color();
        if view.render_distance >= 4 {
            let sun_x = if mth_sin_f64(f64::from(sun_angle)) > 0.0 {
                -1.0
            } else {
                1.0
            };
            let looking = view.forward[0] * sun_x;
            if looking > 0.0 {
                let alpha = channel(sunrise, 24) as f32 / 255.0;
                if alpha > 0.0 {
                    fog = srgb_lerp(looking * alpha, fog, sunrise | 0xFF00_0000);
                }
            }
        }
        let sky_fog_end = (attributes.get(Attribute::SkyFogEndDistance).float() / 16.0)
            .min(view.render_distance as f32);
        let delta = sky_fog_end / 32.0;
        let mix = if delta < 0.0 {
            0.25
        } else if delta > 1.0 {
            1.0
        } else {
            0.25 + delta * 0.75
        };
        let mix = 1.0 - (f64::from(mix)).powf(0.25) as f32;
        // The sky colour the fog blends towards is darkened by the weather.
        let scale = |colour: u32, red: f32, green: f32, blue: f32| {
            argb(
                channel(colour, 24),
                ((channel(colour, 16) as f32 * red) as i32).clamp(0, 255),
                ((channel(colour, 8) as f32 * green) as i32).clamp(0, 255),
                ((channel(colour, 0) as f32 * blue) as i32).clamp(0, 255),
            )
        };
        let mut darkened = sky_colour;
        if view.weather.rain > 0.0 {
            let modifier = 1.0 - view.weather.rain * 0.5;
            darkened = scale(darkened, modifier, modifier, 1.0 - view.weather.rain * 0.4);
        }
        if view.weather.thunder > 0.0 {
            let modifier = 1.0 - view.weather.thunder * 0.5;
            darkened = scale(darkened, modifier, modifier, modifier);
        }
        srgb_lerp(mix, fog, darkened)
    };
    let fog_colour = {
        let [r, g, b, _] = unpack(fog);
        [r, g, b, 1.0]
    };
    // FogRenderer.setupFog: the water or atmospheric environment's
    // distances, then the render distance's own band. The player's water
    // vision is taken as zero, so under water the end is its lowest quarter.
    let (environmental_start, environmental_end, sky_fog_end, cloud_fog_end) = if view.eye_in_water
    {
        let end = attributes.get(Attribute::WaterFogEndDistance).float() * 0.25;
        (
            attributes.get(Attribute::WaterFogStartDistance).float(),
            end,
            end,
            end,
        )
    } else {
        let end = attributes.get(Attribute::FogEndDistance).float();
        (
            attributes.get(Attribute::FogStartDistance).float() - 160.0 * view.rain_fog,
            end.min(96.0).max(end - 256.0 * view.rain_fog),
            render_blocks.min(attributes.get(Attribute::SkyFogEndDistance).float()),
            ((view.cloud_range * 16) as f32)
                .min(attributes.get(Attribute::CloudFogEndDistance).float()),
        )
    };
    let cloud_colour = unpack(attributes.get(Attribute::CloudColor).color());
    let render_fog_span = (render_blocks / 10.0).clamp(4.0, 64.0);
    let terrain_fog = [
        environmental_start,
        environmental_end,
        render_blocks - render_fog_span,
        render_blocks,
    ];

    let mut frame = SkyFrame {
        clear_colour: fog_colour,
        fog_colour,
        sky_fog_end,
        terrain_fog,
        cloud_colour,
        cloud_height: attributes.get(Attribute::CloudHeight).float(),
        cloud_fog_end,
        vertices: Vec::new(),
        batches: Vec::new(),
    };
    if skybox != Skybox::Overworld {
        return frame;
    }

    let untextured = [-1.0, -1.0];
    let push_batch = |frame: &mut SkyFrame, blend: SkyBlend, vertices: Vec<SkyVertex>| {
        if vertices.is_empty() {
            return;
        }
        let first = frame.vertices.len() as u32;
        let count = vertices.len() as u32;
        frame.vertices.extend(vertices);
        match frame.batches.last_mut() {
            Some((last_blend, last_first, last_count))
                if *last_blend == blend && *last_first + *last_count == first =>
            {
                *last_count += count;
            }
            _ => frame.batches.push((blend, first, count)),
        }
    };
    let fan = |centre: SkyVertex, ring: &[SkyVertex]| -> Vec<SkyVertex> {
        ring.windows(2)
            .flat_map(|pair| [centre, pair[0], pair[1]])
            .collect()
    };
    let quad = |corners: [SkyVertex; 4]| -> Vec<SkyVertex> {
        vec![
            corners[0], corners[1], corners[2], corners[0], corners[2], corners[3],
        ]
    };
    let disc = |height: f32, colour: [f32; 4], lift: f32| -> Vec<SkyVertex> {
        let x = height.signum() * 512.0;
        let point = |position: [f32; 3]| SkyVertex {
            position: [position[0], position[1] + lift, position[2]],
            uv: untextured,
            colour,
            fog_distance: [
                (position[0] * position[0] + position[1] * position[1] + position[2] * position[2])
                    .sqrt(),
                (position[0] * position[0] + position[2] * position[2])
                    .sqrt()
                    .max(position[1].abs()),
            ],
        };
        let centre = point([0.0, height, 0.0]);
        let ring: Vec<SkyVertex> = (-180..=180)
            .step_by(45)
            .map(|degrees| {
                let angle = degrees as f32 * (std::f32::consts::PI / 180.0);
                point([
                    x * mth_cos_f64(f64::from(angle)),
                    height,
                    512.0 * mth_sin_f64(f64::from(angle)),
                ])
            })
            .collect();
        fan(centre, &ring)
    };

    // The sky disc, in the sky colour, fogged with distance.
    push_batch(
        &mut frame,
        SkyBlend::Translucent,
        disc(16.0, unpack(sky_colour | 0xFF00_0000), 0.0),
    );

    // The sunrise fan, stretched by its colour's alpha.
    let sunrise_alpha = channel(sunrise, 24) as f32 / 255.0;
    if sunrise_alpha > 0.001 {
        let flip = if mth_sin_f64(f64::from(sun_angle)) < 0.0 {
            180.0_f32
        } else {
            0.0
        };
        let pose = mul(
            mul(
                rotate_x(90.0_f32.to_radians()),
                rotate_z((flip + 90.0).to_radians()),
            ),
            scale(1.0, 1.0, sunrise_alpha),
        );
        let [r, g, b, a] = unpack(sunrise);
        let point = |position: [f32; 3], alpha: f32| SkyVertex {
            position: transform(pose, position),
            uv: untextured,
            colour: [r, g, b, a * alpha],
            fog_distance: [-1.0, -1.0],
        };
        let centre = point([0.0, 100.0, 0.0], 1.0);
        let ring: Vec<SkyVertex> = (0..=16)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let (s, c) = (mth_sin_f64(f64::from(angle)), mth_cos_f64(f64::from(angle)));
                point([s * 120.0, c * 120.0, -c * 40.0], 0.0)
            })
            .collect();
        push_batch(&mut frame, SkyBlend::Translucent, fan(centre, &ring));
    }

    let base = rotate_y((-90.0_f32).to_radians());
    let celestial = |angle: f32, size: f32, uv: [[f32; 2]; 4]| -> Vec<SkyVertex> {
        let pose = mul(
            mul(mul(base, rotate_x(angle)), translate(0.0, 100.0, 0.0)),
            scale(size, 1.0, size),
        );
        let corners = [
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
        ];
        let vertex = |i: usize| SkyVertex {
            position: transform(pose, corners[i]),
            uv: uv[i],
            // Rain fades the sun and the moon.
            colour: [1.0, 1.0, 1.0, 1.0 - view.weather.rain],
            fog_distance: [-1.0, -1.0],
        };
        quad([vertex(0), vertex(1), vertex(2), vertex(3)])
    };
    let [u0, v0, u1, v1] = sprites.sun;
    push_batch(
        &mut frame,
        SkyBlend::Overlay,
        celestial(sun_angle, 30.0, [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]),
    );
    let [u0, v0, u1, v1] = sprites.moon[moon_phase];
    push_batch(
        &mut frame,
        SkyBlend::Overlay,
        celestial(moon_angle, 20.0, [[u1, v1], [u0, v1], [u0, v0], [u1, v0]]),
    );

    if star_brightness > 0.0 {
        let pose = mul(base, rotate_x(star_angle));
        let colour = [star_brightness; 4];
        let mut vertices = Vec::with_capacity(stars.len() * 6);
        for star in stars {
            let vertex = |i: usize| SkyVertex {
                position: transform(pose, star[i]),
                uv: untextured,
                colour,
                fog_distance: [-1.0, -1.0],
            };
            vertices.extend(quad([vertex(0), vertex(1), vertex(2), vertex(3)]));
        }
        push_batch(&mut frame, SkyBlend::Overlay, vertices);
    }

    if view.eye_above_horizon < 0.0 && !view.eye_in_water {
        // The dark disc sits lifted by twelve, but its fog is measured from
        // where it was built.
        push_batch(
            &mut frame,
            SkyBlend::Translucent,
            disc(-16.0, [0.0, 0.0, 0.0, 1.0], 12.0),
        );
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(attribute: Attribute, period: Option<i64>, text: &str) -> Track {
        let JsonValue::Object(object) = voxel_core::json::parse(text).expect("json") else {
            panic!("an object");
        };
        Track::parse(attribute, period, &object).expect("a track")
    }

    #[test]
    fn the_sun_track_wraps_its_period_and_is_overhead_at_noon() {
        let sun = track(
            Attribute::SunAngle,
            Some(24000),
            r#"{"ease": {"cubic_bezier": [0.362, 0.241, 0.638, 0.759]},
                "keyframes": [{"ticks": 6000, "value": 360.0}, {"ticks": 6000, "value": 0.0}]}"#,
        );
        assert_eq!(sun.sample(6000), Value::Float(0.0));
        assert_eq!(sun.sample(30000), Value::Float(0.0));
        // Halfway round the clock the eased curve is at its midpoint.
        assert_eq!(sun.sample(18000), Value::Float(180.0));
        let before_noon = sun.sample(5999).float();
        assert!(before_noon > 359.0 && before_noon < 360.0);
    }

    #[test]
    fn a_colour_multiply_keeps_white_as_identity() {
        assert_eq!(multiply(0xFFFF_FFFF, 0xFF78_A7FF), 0xFF78_A7FF);
        assert_eq!(multiply(0xFF78_A7FF, 0xFF00_0000), 0xFF00_0000);
        assert_eq!(srgb_lerp(0.5, 0xFF00_0000, 0xFFFF_FFFF), 0xFF7F_7F7F);
    }

    #[test]
    fn a_constant_track_holds_until_the_next_keyframe() {
        let phase = track(
            Attribute::MoonPhase,
            Some(192000),
            r#"{"keyframes": [{"ticks": 0, "value": "full_moon"}, {"ticks": 24000, "value": "waning_gibbous"}]}"#,
        );
        assert_eq!(phase.sample(23999), Value::Phase(0));
        assert_eq!(phase.sample(24000), Value::Phase(1));
    }
}
