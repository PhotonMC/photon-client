use super::*;

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
