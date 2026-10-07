use super::*;

pub(super) fn model_state_result(request: &Value) -> Result<Value, String> {
    use voxel_render::block::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let Some(transform) = rotation::model_state_transform(
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        ) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        let uv_lock = row["uvlock"].as_bool().ok_or("uvlock must be a boolean")?;
        let (x, y, z) = (
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        );
        let flat = |matrix: rotation::FlatMatrix| -> Vec<Value> {
            matrix
                .iter()
                .map(|value| json!(format!("{:08x}", value.to_bits())))
                .collect()
        };
        let mut faces = Vec::with_capacity(6);
        for facing in [
            voxel_core::direction::Direction::Down,
            voxel_core::direction::Direction::Up,
            voxel_core::direction::Direction::North,
            voxel_core::direction::Direction::South,
            voxel_core::direction::Direction::West,
            voxel_core::direction::Direction::East,
        ] {
            let forward = rotation::face_transform(x, y, z, facing, uv_lock)
                .ok_or("no face transform for that rotation")?;
            let inverse = rotation::inverse_face_transform(x, y, z, facing, uv_lock)
                .ok_or("no inverse face transform for that rotation")?;
            faces.push(json!({
                "face": facing.name(),
                "forward": flat(forward),
                "inverse": flat(inverse),
            }));
        }
        out.push(json!({
            "label": label,
            "outcome": "resolved",
            "transformation": matrix,
            "faces": faces,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_uv_lock_result(request: &Value) -> Result<Value, String> {
    use voxel_core::direction::Direction;
    use voxel_render::block::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let facing = Direction::from_name(string(row, "facing")?).ok_or("unknown facing")?;
        let matrix = rotation::inverse_face_transform(
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
            facing,
            row["uvlock"].as_bool().ok_or("uvlock must be a boolean")?,
        )
        .ok_or("no face transform for that rotation")?;
        let read = |name: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(string(row, name)?, 16)
                    .map_err(|error| format!("{name}: {error}"))?,
            ))
        };
        let (u, v) = rotation::transform_face_uv(&matrix, read("u")?, read("v")?);
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "transformed",
            "identity": rotation::is_flat_identity(&matrix),
            "u": format!("{:08x}", u.to_bits()),
            "v": format!("{:08x}", v.to_bits()),
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn describe_variants(variants: &[voxel_render::Variant]) -> Value {
    Value::Array(
        variants
            .iter()
            .map(|variant| {
                json!({
                    "model": variant.model.as_string(),
                    "x": variant.x.name(),
                    "y": variant.y.name(),
                    "z": variant.z.name(),
                    "uvlock": variant.uvlock,
                    "weight": variant.weight,
                })
            })
            .collect::<Vec<_>>(),
    )
}

pub(super) fn describe_condition(condition: &voxel_render::Condition) -> Value {
    match condition {
        voxel_render::Condition::KeyValue(tests) => {
            let mut described = serde_json::Map::new();
            for (property, terms) in tests {
                described.insert(
                    property.clone(),
                    Value::Array(
                        terms
                            .iter()
                            .map(|term| json!({"value": term.value, "negated": term.negated}))
                            .collect(),
                    ),
                );
            }
            json!({"kind": "key_value", "tests": Value::Object(described)})
        }
        voxel_render::Condition::Combined { operation, terms } => json!({
            "kind": "combined",
            "operation": operation.name(),
            "terms": terms.iter().map(describe_condition).collect::<Vec<_>>(),
        }),
    }
}

pub(super) fn model_blockstate_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        match voxel_render::parse_block_state(string(row, "json")?) {
            None => out.push(json!({"label": label, "outcome": "rejected"})),
            Some(file) => {
                let mut described = serde_json::Map::new();
                described.insert("label".into(), json!(label));
                described.insert("outcome".into(), json!("parsed"));
                if let Some(variants) = &file.variants {
                    let mut listed = serde_json::Map::new();
                    for (key, value) in variants {
                        listed.insert(key.clone(), describe_variants(value));
                    }
                    described.insert("variants".into(), Value::Object(listed));
                }
                if let Some(multipart) = &file.multipart {
                    described.insert(
                        "multipart".into(),
                        Value::Array(
                            multipart
                                .iter()
                                .map(|selector| {
                                    let mut piece = serde_json::Map::new();
                                    if let Some(condition) = &selector.condition {
                                        piece.insert("when".into(), describe_condition(condition));
                                    }
                                    piece
                                        .insert("apply".into(), describe_variants(&selector.apply));
                                    Value::Object(piece)
                                })
                                .collect(),
                        ),
                    );
                }
                out.push(Value::Object(described));
            }
        }
    }
    Ok(json!({"rows": out}))
}
