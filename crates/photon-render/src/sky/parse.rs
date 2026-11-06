use super::*;

/// The names modifiers go by in the data files.
const OPERATIONS: [(&str, Operation); 12] = [
    ("override", Operation::Override),
    ("multiply", Operation::Multiply),
    ("add", Operation::Add),
    ("subtract", Operation::Subtract),
    ("minimum", Operation::Minimum),
    ("maximum", Operation::Maximum),
    ("and", Operation::And),
    ("nand", Operation::Nand),
    ("or", Operation::Or),
    ("nor", Operation::Nor),
    ("xor", Operation::Xor),
    ("xnor", Operation::Xnor),
];

impl Operation {
    pub(super) fn parse(name: &str) -> Option<Self> {
        OPERATIONS
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, operation)| *operation)
    }
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
}

/// A colour attribute's value: a `#RRGGBB` or `#AARRGGBB` text, the channels as an array, or a
/// packed number.
fn parse_colour(kind: Kind, value: &JsonValue) -> Option<Value> {
    let packed = |color: u32| match kind {
        Kind::Rgb => rgb24(color),
        _ => argb32(color),
    };
    match value {
        JsonValue::String(text) => {
            let text = String::from_utf16(text).ok()?;
            let hex = text.strip_prefix('#')?;
            let parsed = u32::from_str_radix(hex, 16).ok()?;
            match hex.len() {
                6 => Some(packed(0xFF00_0000 | parsed)),
                8 => Some(packed(parsed)),
                _ => None,
            }
        }
        // `ExtraCodecs.VECTOR3F` / `VECTOR4F`: the channels as they are.
        JsonValue::Array(channels) => {
            let mut color = [1.0_f32; 4];
            if channels.len() != if kind == Kind::Rgb { 3 } else { 4 } {
                return None;
            }
            for (slot, channel) in color.iter_mut().zip(channels) {
                *slot = channel.as_f32()?;
            }
            Some(Value::Color(color))
        }
        other => Some(packed(other.as_i64()? as u32)),
    }
}

/// A moon phase attribute's value: the phase's name.
fn parse_phase(value: &JsonValue) -> Option<Value> {
    let JsonValue::String(text) = value else {
        return None;
    };
    let text = String::from_utf16(text).ok()?;
    MOON_PHASES
        .iter()
        .position(|phase| *phase == text)
        .map(|index| Value::Phase(index as u8))
}

pub(super) fn parse_value(kind: Kind, value: &JsonValue) -> Option<Value> {
    match kind {
        Kind::Rgb | Kind::Argb => parse_colour(kind, value),
        Kind::Float | Kind::Angle => Some(Value::Float(value.as_f32()?)),
        Kind::Bool => match value {
            JsonValue::Bool(flag) => Some(Value::Bool(*flag)),
            _ => None,
        },
        Kind::Phase => parse_phase(value),
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

pub(super) fn data_path(root: &Path, kind: &str, name: &str, extension: &str) -> PathBuf {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    root.join(namespace)
        .join(kind)
        .join(format!("{path}{extension}"))
}

/// A tag's entries with nested tags expanded, in order.
pub(super) fn expand_tag(root: &Path, kind: &str, name: &str, out: &mut Vec<String>) {
    let Some(tag) =
        photon_core::json::read_object(&data_path(root, &format!("tags/{kind}"), name, ".json"))
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

/// A biome's `AMBIENT_PARTICLES`: each particle (its type, as a name or an object) and the chance
/// each pick makes one.
pub(super) fn parse_ambient_particles(biome: Option<&JsonObject>) -> Vec<AmbientParticle> {
    let list = biome
        .and_then(|biome| biome.object("attributes"))
        .and_then(|attributes| attributes.array("minecraft:visual/ambient_particles"));
    list.unwrap_or(&[])
        .iter()
        .filter_map(|value| {
            let JsonValue::Object(entry) = value else {
                return None;
            };
            let particle = match entry.get_str("particle")? {
                JsonValue::String(name) => String::from_utf16(name).ok()?,
                JsonValue::Object(options) => options.text("type")?,
                _ => return None,
            };
            Some(AmbientParticle {
                particle,
                probability: entry.get_str("probability")?.as_f32()?,
            })
        })
        .collect()
}
