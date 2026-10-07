use super::*;

/// The moon phase sprite names, in the phases' order.
pub fn moon_sprites() -> impl Iterator<Item = String> {
    MOON_PHASES
        .iter()
        .map(|phase| format!("minecraft:environment/celestial/moon/{phase}"))
}

pub(super) fn parse_value(kind: Kind, value: &JsonValue) -> Option<Value> {
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

pub(super) fn channel(color: u32, shift: u32) -> i32 {
    ((color >> shift) & 0xff) as i32
}

pub(super) fn lerp_int(alpha: f32, from: i32, to: i32) -> i32 {
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
pub(super) fn lerp(kind: Kind, alpha: f32, from: Value, to: Value) -> Value {
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
        Kind::Phase => to,
        _ => lerp(kind, alpha, from, to),
    }
}

pub(super) fn parse_entry(attribute: Attribute, value: &JsonValue) -> Option<Entry> {
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

pub(super) fn parse_attribute_map(object: Option<&JsonObject>) -> AttributeMap {
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

/// `WeatherAttributes`: how rain and thunder change one attribute's value,
/// or nothing for an attribute they leave alone.
pub(super) fn weather_modifier(attribute: Attribute, thunder: bool, value: Value) -> Option<Value> {
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
pub(super) fn alpha_blend(destination: u32, source: u32) -> u32 {
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

pub(super) fn data_path(root: &Path, kind: &str, name: &str, extension: &str) -> PathBuf {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    root.join(namespace)
        .join(kind)
        .join(format!("{path}{extension}"))
}

/// A tag's entries with nested tags expanded, in order.
pub(super) fn expand_tag(root: &Path, kind: &str, name: &str, out: &mut Vec<String>) {
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

pub(super) fn unpack(colour: u32) -> [f32; 4] {
    [
        channel(colour, 16) as f32 / 255.0,
        channel(colour, 8) as f32 / 255.0,
        channel(colour, 0) as f32 / 255.0,
        channel(colour, 24) as f32 / 255.0,
    ]
}

/// `left * right`, both row-major as `m[row][column]`.
pub(super) fn mul(left: Matrix, right: Matrix) -> Matrix {
    let mut out = [[0.0; 4]; 4];
    for (row, out_row) in out.iter_mut().enumerate() {
        for (column, cell) in out_row.iter_mut().enumerate() {
            *cell = (0..4).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    out
}

pub(super) fn rotate_x(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn rotate_y(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn rotate_z(angle: f32) -> Matrix {
    let (s, c) = (angle.sin(), angle.cos());
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn translate(x: f32, y: f32, z: f32) -> Matrix {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn scale(x: f32, y: f32, z: f32) -> Matrix {
    [
        [x, 0.0, 0.0, 0.0],
        [0.0, y, 0.0, 0.0],
        [0.0, 0.0, z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(super) fn transform(matrix: Matrix, point: [f32; 3]) -> [f32; 3] {
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
