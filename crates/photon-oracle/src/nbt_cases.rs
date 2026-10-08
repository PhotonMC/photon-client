use super::*;

pub(super) fn nbt_value_json(value: &FieldValue) -> Value {
    match value {
        FieldValue::Byte(value) => json!(value.to_string()),
        FieldValue::Short(value) => json!(value.to_string()),
        FieldValue::Int(value) => json!(value.to_string()),
        FieldValue::Long(value) => json!(value.to_string()),
        FieldValue::Float(value) => json!(format!("{:08x}", value.to_bits())),
        FieldValue::Double(value) => json!(format!("{:016x}", value.to_bits())),
        FieldValue::ByteArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::Str(value) => json!(value),
        FieldValue::List(values) => {
            Value::Array(values.iter().map(nbt_description_json).collect::<Vec<_>>())
        }
        FieldValue::Compound(fields) => {
            let mut object = serde_json::Map::new();
            let mut sorted: Vec<&Field> = fields.iter().collect();
            sorted.sort_by(|a, b| a.key.cmp(&b.key));
            for field in sorted {
                object.insert(field.key.clone(), nbt_description_json(&field.value));
            }
            Value::Object(object)
        }
        FieldValue::IntArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::LongArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
    }
}

pub(super) fn nbt_description_json(value: &FieldValue) -> Value {
    json!({
        "type": nbt_type_name(value),
        "value": nbt_value_json(value),
    })
}

pub(super) fn nbt_field_json(field: &Field) -> Value {
    json!({
        "key": field.key,
        "type": nbt_type_name(&field.value),
        "value": nbt_value_json(&field.value),
    })
}

pub(super) fn nbt_tree_values<T>(node: &Value, value_name: &str) -> Result<Vec<T>, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    node["value"]
        .as_array()
        .ok_or_else(|| format!("{value_name} value must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("{value_name} values must be strings"))?
                .parse::<T>()
                .map_err(|error| error.to_string())
        })
        .collect()
}

pub(super) fn nbt_tree_value(node: &Value) -> Result<FieldValue, String> {
    match string(node, "type")? {
        "byte" => Ok(FieldValue::Byte(
            string(node, "value")?
                .parse::<i8>()
                .map_err(|error| error.to_string())?,
        )),
        "short" => Ok(FieldValue::Short(
            string(node, "value")?
                .parse::<i16>()
                .map_err(|error| error.to_string())?,
        )),
        "int" => Ok(FieldValue::Int(
            string(node, "value")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        )),
        "long" => Ok(FieldValue::Long(
            string(node, "value")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?,
        )),
        "float" => Ok(FieldValue::Float(f32::from_bits(
            u32::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "double" => Ok(FieldValue::Double(f64::from_bits(
            u64::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "byte-array" => Ok(FieldValue::ByteArray(nbt_tree_values(node, "byte-array")?)),
        "string" => Ok(FieldValue::Str(string(node, "value")?.to_owned())),
        "list" => Ok(FieldValue::List(
            node["value"]
                .as_array()
                .ok_or("list value must be an array")?
                .iter()
                .map(nbt_tree_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "compound" => {
            let object = node["value"]
                .as_object()
                .ok_or("compound value must be an object")?;
            let fields = object
                .iter()
                .map(|(key, value)| {
                    Ok(Field {
                        key: key.clone(),
                        value: nbt_tree_value(value)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(FieldValue::Compound(fields))
        }
        "int-array" => Ok(FieldValue::IntArray(nbt_tree_values(node, "int-array")?)),
        "long-array" => Ok(FieldValue::LongArray(nbt_tree_values(node, "long-array")?)),
        _ => Err("unknown NBT field type".into()),
    }
}

pub(super) fn nbt_result(request: &Value) -> Result<Value, String> {
    fn nbt_error(error: nbt::ReadError) -> &'static str {
        match error {
            nbt::ReadError::BufferUnderflow => "buffer_underflow",
            nbt::ReadError::DecoderError => "decoder_error",
            nbt::ReadError::EncoderError => "encoder_error",
            nbt::ReadError::TooBig => "too_big",
        }
    }

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match nbt::read_nbt(&mut remaining) {
                Ok(None) => Ok(json!({
                    "null": true,
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Ok(Some(fields)) => {
                    let mut out: Vec<Value> = fields.iter().map(nbt_field_json).collect();
                    out.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
                    Ok(json!({
                        "fields": out,
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    }))
                }
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": nbt_error(error),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        "write" => {
            let mut buffer = prefix.clone();
            if request.get("null_root").and_then(Value::as_bool) == Some(true) {
                nbt::write_nbt(None, &mut buffer);
            } else if let Some(tree) = request.get("tree") {
                let value = nbt_tree_value(tree)?;
                let FieldValue::Compound(fields) = value else {
                    return Err("NBT tree root must be a compound".into());
                };
                nbt::write_nbt(Some(&fields), &mut buffer);
            } else {
                let fields = request["fields"]
                    .as_array()
                    .ok_or("fields must be an array")?;
                let mut root = Vec::with_capacity(fields.len());
                for entry in fields {
                    let key = string(entry, "key")?.to_owned();
                    let value = match string(entry, "type")? {
                        "byte" => FieldValue::Byte(
                            string(entry, "value")?
                                .parse::<i8>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "short" => FieldValue::Short(
                            string(entry, "value")?
                                .parse::<i16>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "int" => FieldValue::Int(
                            string(entry, "value")?
                                .parse::<i32>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "long" => FieldValue::Long(
                            string(entry, "value")?
                                .parse::<i64>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "float" => FieldValue::Float(f32::from_bits(
                            u32::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "double" => FieldValue::Double(f64::from_bits(
                            u64::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "string" => FieldValue::Str(string(entry, "value")?.to_owned()),
                        _ => return Err("unknown NBT field type".into()),
                    };
                    root.push(Field { key, value });
                }
                nbt::write_nbt(Some(&root), &mut buffer);
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        _ => Err("unknown NBT operation".into()),
    }
}

/// The same deterministic rendering the oracle writes, so the comparison sees
/// shape, order, number literals and string code units.
pub(super) fn json_canonical(value: &photon_core::json::JsonValue, out: &mut String) {
    use photon_core::json::JsonValue;
    let write_string = |units: &[u16], out: &mut String| {
        out.push('s');
        for unit in units {
            out.push_str(&format!("{unit:04x}"));
        }
        out.push(';');
    };
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(literal) => {
            out.push('n');
            out.push_str(literal);
        }
        JsonValue::String(units) => write_string(units, out),
        JsonValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                json_canonical(item, out);
            }
            out.push(']');
        }
        JsonValue::Object(object) => {
            out.push('{');
            for (index, (key, item)) in object.entries().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                json_canonical(item, out);
            }
            out.push('}');
        }
    }
}

pub(super) fn json_number_result(request: &Value) -> Result<Value, String> {
    use photon_core::json::JsonValue;
    let rows = request["rows"]
        .as_array()
        .ok_or("json rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let literal = string(row, "literal")?;
        let Ok(value) = photon_core::json::parse(literal) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let JsonValue::Number(kept) = &value else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        out.push(json!({
            "label": label,
            "outcome": "converted",
            "literal": kept,
            "as_int": value.as_i32().map_or("error".to_owned(), |v| v.to_string()),
            "as_long": value.as_i64().map_or("error".to_owned(), |v| v.to_string()),
            "as_float": value
                .as_f32()
                .map_or("error".to_owned(), |v| format!("{:08x}", v.to_bits())),
            "as_double": value
                .as_f64()
                .map_or("error".to_owned(), |v| format!("{:016x}", v.to_bits())),
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn json_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "number" {
        return json_number_result(request);
    }
    if string(request, "operation")? != "parse" {
        return Err("unknown json operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("json rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "text")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("text hex: {error}"))?;
        let text = match core::str::from_utf8(&bytes) {
            Ok(text) => text,
            Err(_) => {
                out.push(json!({"label": label, "outcome": "rejected"}));
                continue;
            }
        };
        match photon_core::json::parse(text) {
            Err(_) => out.push(json!({"label": label, "outcome": "rejected"})),
            Ok(value) => {
                let mut canonical = String::new();
                json_canonical(&value, &mut canonical);
                out.push(json!({
                    "label": label,
                    "outcome": "parsed",
                    "canonical": canonical,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_geometry_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let read = |key: &str| -> Result<[f32; 3], String> {
            let array = row[key].as_array().ok_or("vector must be an array")?;
            let mut value = [0.0_f32; 3];
            for (slot, item) in value.iter_mut().zip(array) {
                let hex = item.as_str().ok_or("vector element must be a string")?;
                *slot = f32::from_bits(
                    u32::from_str_radix(hex, 16).map_err(|error| format!("vector: {error}"))?,
                );
            }
            Ok(value)
        };
        let from = read("from")?;
        let to = read("to")?;
        let Some(facing) = direction_by_name(string(row, "facing")?) else {
            out.push(json!({"label": label, "outcome": "unknown_facing"}));
            continue;
        };
        let uvs = photon_render::default_face_uv(from, to, facing);
        let corners: Vec<Value> = photon_render::face_vertices(from, to, facing)
            .iter()
            .map(|vertex| {
                json!({
                    "x": format!("{:08x}", vertex[0].to_bits()),
                    "y": format!("{:08x}", vertex[1].to_bits()),
                    "z": format!("{:08x}", vertex[2].to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "baked",
            "min_u": format!("{:08x}", uvs.min_u.to_bits()),
            "min_v": format!("{:08x}", uvs.min_v.to_bits()),
            "max_u": format!("{:08x}", uvs.max_u.to_bits()),
            "max_v": format!("{:08x}", uvs.max_v.to_bits()),
            "corners": corners,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn direction_by_name(name: &str) -> Option<photon_core::direction::Direction> {
    use photon_core::direction::Direction;
    Some(match name {
        "down" => Direction::Down,
        "up" => Direction::Up,
        "north" => Direction::North,
        "south" => Direction::South,
        "west" => Direction::West,
        "east" => Direction::East,
        _ => return None,
    })
}
