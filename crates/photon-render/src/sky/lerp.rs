use super::*;

impl Attributes {
    pub fn get(&self, attribute: Attribute) -> Value {
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

impl Track {
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

pub(super) fn channel(color: u32, shift: u32) -> i32 {
    ((color >> shift) & 0xff) as i32
}

pub(super) fn lerp_int(alpha: f32, from: i32, to: i32) -> i32 {
    from + (alpha * (to - from) as f32).floor() as i32
}

/// `ARGB.srgbLerp` on packed colours: each channel on its own, truncated as integers are.
pub fn srgb_lerp(alpha: f32, from: u32, to: u32) -> u32 {
    argb(
        lerp_int(alpha, channel(from, 24), channel(to, 24)),
        lerp_int(alpha, channel(from, 16), channel(to, 16)),
        lerp_int(alpha, channel(from, 8), channel(to, 8)),
        lerp_int(alpha, channel(from, 0), channel(to, 0)),
    )
}

/// `ARGB.srgbLerp` on the float vectors: each component by a fused multiply-add (`Vector4f.lerp`).
pub fn srgb_lerp_vec(alpha: f32, from: [f32; 4], to: [f32; 4]) -> [f32; 4] {
    [0, 1, 2, 3].map(|i| (to[i] - from[i]).mul_add(alpha, from[i]))
}

/// The red, green and blue of an RGB attribute's value.
pub(super) fn rgb_of(value: Value) -> [f32; 3] {
    let colour = value.color();
    [colour[0], colour[1], colour[2]]
}

pub(super) fn lerp_f32(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * (to - from)
}

pub(super) fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

pub(super) fn apply(kind: Kind, operation: Operation, subject: Value, argument: Value) -> Value {
    if operation == Operation::Override {
        return argument;
    }
    match kind {
        Kind::Rgb | Kind::Argb => apply_colour(operation, subject.color(), argument.color()),
        Kind::Float | Kind::Angle => apply_float(operation, subject.float(), argument.float()),
        Kind::Bool => Value::Bool(apply_bool(operation, subject.bool(), argument.bool())),
        Kind::Phase => subject,
    }
}

/// The modifiers of a colour (`ColorModifier`, `ARGB.multiply`, `addRgb`, `subtractRgb`): each
/// channel on its own, the alpha kept unless an ARGB colour multiplies it.
fn apply_colour(operation: Operation, subject: [f32; 4], argument: [f32; 4]) -> Value {
    let ops = colour_ops();
    Value::Color(match operation {
        Operation::Multiply => (ops.multiply)(subject, argument),
        Operation::Add => (ops.add)(subject, argument),
        Operation::Subtract => (ops.subtract)(subject, argument),
        _ => subject,
    })
}

/// The modifiers of a number (`FloatModifier`).
fn apply_float(operation: Operation, s: f32, a: f32) -> Value {
    Value::Float(match operation {
        Operation::Multiply => s * a,
        Operation::Add => s + a,
        Operation::Subtract => s - a,
        Operation::Minimum => s.min(a),
        Operation::Maximum => s.max(a),
        _ => s,
    })
}

/// `BooleanModifier`, as what each makes of the subject and the argument: the answers for
/// (false, false), (false, true), (true, false) and (true, true).
const BOOL_TABLES: [(Operation, [bool; 4]); 6] = [
    (Operation::And, [false, false, false, true]),
    (Operation::Nand, [true, true, true, false]),
    (Operation::Or, [false, true, true, true]),
    (Operation::Nor, [true, false, false, false]),
    (Operation::Xor, [false, true, true, false]),
    (Operation::Xnor, [true, false, false, true]),
];

/// The modifiers of a flag (`BooleanModifier`).
fn apply_bool(operation: Operation, subject: bool, argument: bool) -> bool {
    BOOL_TABLES
        .iter()
        .find(|(candidate, _)| *candidate == operation)
        .map_or(subject, |(_, table)| {
            table[usize::from(subject) * 2 + usize::from(argument)]
        })
}

/// The lerp a keyframe track and a biome blend use.
pub(super) fn lerp(kind: Kind, alpha: f32, from: Value, to: Value) -> Value {
    match kind {
        Kind::Rgb | Kind::Argb => {
            Value::Color((colour_ops().lerp)(alpha, from.color(), to.color()))
        }
        Kind::Float | Kind::Angle => Value::Float(lerp_f32(alpha, from.float(), to.float())),
        // Not interpolated: a keyframe switches when its segment ends.
        Kind::Phase | Kind::Bool => {
            if alpha >= 1.0 {
                to
            } else {
                from
            }
        }
    }
}

/// The lerp between the last two ticks' values.
pub(super) fn partial_tick_lerp(kind: Kind, alpha: f32, from: Value, to: Value) -> Value {
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
        Kind::Phase | Kind::Bool => to,
        _ => lerp(kind, alpha, from, to),
    }
}

/// How the weather changes one attribute.
#[derive(Clone, Copy)]
enum WeatherEffect {
    /// Blend to the colour's grey at this brightness, by this factor.
    Grey(f32, f32),
    /// Multiply the colour by these channels.
    Multiply(f32, f32, f32),
    /// Blend the night sky light colour in at this alpha.
    NightLight(f32),
    /// Blend the float towards the target by the alpha.
    Blend(f32, f32),
    /// Take the value to nothing.
    Dark,
}

/// What rain (or thunder) does to an attribute, in `WeatherAttributes`.
fn weather_effect(attribute: Attribute, thunder: bool) -> Option<WeatherEffect> {
    use WeatherEffect::{Blend, Dark, Grey, Multiply, NightLight};
    let [rain, storm] = match attribute {
        Attribute::SkyColor => [Grey(0.6, 0.75), Grey(0.24, 0.94)],
        Attribute::FogColor | Attribute::SunriseSunsetColor => {
            [Multiply(0.5, 0.5, 0.6), Multiply(0.25, 0.25, 0.3)]
        }
        Attribute::CloudColor => [Grey(0.24, 0.5), Grey(0.095, 0.94)],
        Attribute::SkyLightColor => [NightLight(0.3125), NightLight(0.527_343_75)],
        Attribute::SkyLightFactor => [Blend(0.24, 0.3125), Blend(0.24, 0.527_343_75)],
        Attribute::StarBrightness => [Dark, Dark],
        _ => return None,
    };
    Some(if thunder { storm } else { rain })
}

/// `WeatherAttributes`: how rain and thunder change one attribute's value,
/// or nothing for an attribute they leave alone.
pub(super) fn weather_modifier(attribute: Attribute, thunder: bool, value: Value) -> Option<Value> {
    let ops = colour_ops();
    let colour = value.color();
    let float_blend =
        |target: f32, alpha: f32| Value::Float(lerp_f32(alpha, value.float(), target));
    // `ColorModifier.ALPHA_BLEND_RGB` with the night sky light colour at the alpha the weather
    // blends it in by.
    let night_sky_light = |alpha: f32| {
        let source = [
            (0.48_f32 * 255.0).floor() / 255.0,
            (0.48_f32 * 255.0).floor() / 255.0,
            1.0,
            (alpha * 255.0).floor() / 255.0,
        ];
        Value::Color((ops.alpha_blend)(colour, source))
    };
    // `ColorModifier.MULTIPLY_RGB` and `MULTIPLY_ARGB`, by a colour made of floats.
    let multiply_by = |red: f32, green: f32, blue: f32| {
        let by = [red, green, blue, 1.0].map(|channel| (channel * 255.0).floor() / 255.0);
        Value::Color((ops.multiply)(colour, by))
    };
    Some(match weather_effect(attribute, thunder)? {
        WeatherEffect::Grey(brightness, factor) => {
            Value::Color((ops.blend_to_grey)(colour, brightness, factor))
        }
        WeatherEffect::Multiply(red, green, blue) => multiply_by(red, green, blue),
        WeatherEffect::NightLight(alpha) => night_sky_light(alpha),
        WeatherEffect::Blend(target, alpha) => float_blend(target, alpha),
        WeatherEffect::Dark => Value::Float(0.0),
    })
}
