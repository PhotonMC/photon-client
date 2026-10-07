use super::*;

pub(super) fn model_state_result(request: &Value) -> Result<Value, String> {
    use voxel_render::rotation;
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
    use voxel_render::rotation;
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

pub(super) fn model_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "nearest_direction" {
        let rows = request["rows"]
            .as_array()
            .ok_or("model rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let read = |name: &str| -> Result<f32, String> {
                Ok(f32::from_bits(
                    u32::from_str_radix(string(row, name)?, 16)
                        .map_err(|error| format!("{name}: {error}"))?,
                ))
            };
            let vector = [read("x")?, read("y")?, read("z")?];
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "resolved",
                "direction": voxel_render::approximate_nearest_direction(vector).name(),
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "cull_rotation" {
        use voxel_core::direction::Direction;
        use voxel_render::rotation;
        let rows = request["rows"]
            .as_array()
            .ok_or("model rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let facing = Direction::from_name(string(row, "facing")?).ok_or("unknown facing")?;
            let rotated = rotation::rotate_cull_direction(
                i32_field(row, "x")?,
                i32_field(row, "y")?,
                i32_field(row, "z")?,
                facing,
            )
            .ok_or("rotation must be a multiple of ninety")?;
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "resolved",
                "direction": rotated.name(),
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "uv_lock" {
        return model_uv_lock_result(request);
    }
    if string(request, "operation")? == "blockstate_parse" {
        return model_blockstate_result(request);
    }
    if string(request, "operation")? == "model_state" {
        return model_state_result(request);
    }
    if string(request, "operation")? == "parse_model" {
        return model_parse_result(request);
    }
    if string(request, "operation")? == "face_uv_rotation" {
        return model_uv_rotation_result(request);
    }
    if string(request, "operation")? == "euler_rotation" {
        return model_euler_result(request);
    }
    if string(request, "operation")? == "quad_winding" {
        return model_winding_result(request);
    }
    if string(request, "operation")? == "element_rotation" {
        return model_rotation_result(request);
    }
    if string(request, "operation")? == "face_geometry" {
        return model_geometry_result(request);
    }
    if string(request, "operation")? != "texture_slots" {
        return Err("unknown model operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let probes = [
        "all", "top", "bottom", "side", "north", "particle", "missing", "#all", "#top",
    ];
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut resolver = voxel_render::TextureSlotsResolver::new();
        let chain = row["chain"].as_array().ok_or("chain must be an array")?;
        for level in chain {
            let slots = level.as_array().ok_or("chain level must be an array")?;
            let mut entries = Vec::with_capacity(slots.len());
            for slot in slots {
                let name = string(slot, "slot")?;
                let contents = if slot.get("raw").is_some() {
                    voxel_render::SlotContents::parse_string(string(slot, "raw")?)
                        .map_err(|error| format!("texture value: {error:?}"))?
                } else {
                    voxel_render::SlotContents::Value(voxel_render::Material {
                        sprite: Identifier::parse(string(slot, "sprite")?)
                            .map_err(|error| format!("sprite: {error:?}"))?,
                        force_translucent: slot["force_translucent"]
                            .as_bool()
                            .ok_or("force_translucent must be a bool")?,
                    })
                };
                entries.push((name.to_owned(), contents));
            }
            resolver.add_last(voxel_render::TextureSlotsData::from_entries(entries));
        }
        let resolved = resolver.resolve();
        let mut lookups = Vec::new();
        for probe in probes {
            if let Some(material) = resolved.material(probe) {
                lookups.push(json!({
                    "probe": probe,
                    "sprite": material.sprite.as_string(),
                    "force_translucent": material.force_translucent,
                }));
            }
        }
        lookups.sort_by(|left, right| left["probe"].as_str().cmp(&right["probe"].as_str()));
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "resolved",
            "lookups": lookups,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_tables_result(request: &Value) -> Result<Value, String> {
    let tables = voxel_render::SrgbTables::new();
    let mut forward = String::with_capacity(256 * 4);
    for channel in 0..=255_u8 {
        forward.push_str(&format!("{:04x}", tables.to_linear(channel)));
    }
    let mut backward = String::with_capacity(1024 * 2);
    for step in 0..1024_u16 {
        backward.push_str(&format!("{:02x}", tables.to_srgb(step)));
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let label = string(&rows[0], "label")?;
    Ok(json!({"rows": [{
        "label": label,
        "outcome": "listed",
        "srgb_to_linear": forward,
        "linear_to_srgb": backward,
    }]}))
}

pub(super) fn image_mip_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "png")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("png hex: {error}"))?;
        let levels = i32_field(row, "levels")?;
        let Ok(image) = voxel_render::decode_rgba(&bytes) else {
            out.push(json!({"label": label, "outcome": "failed"}));
            continue;
        };
        let name = string(row, "name")?;
        let path = name.split_once(':').map_or(name, |(_, path)| path);
        let strategy =
            voxel_render::MipmapStrategy::parse(&string(row, "strategy")?.to_lowercase())
                .ok_or("unknown mipmap strategy")?;
        let bias = f32::from_bits(
            u32::from_str_radix(string(row, "bias")?, 16)
                .map_err(|error| format!("mipmap bias: {error}"))?,
        );
        let transparency = voxel_render::Transparency::of(
            row["transparent"]
                .as_bool()
                .ok_or("transparent must be a boolean")?,
            row["translucent"]
                .as_bool()
                .ok_or("translucent must be a boolean")?,
        );
        let generated = voxel_render::generate_mip_levels_with(
            path,
            &image,
            levels.max(0) as u32,
            strategy,
            bias,
            transparency,
        );
        let described: Vec<Value> = generated
            .iter()
            .map(|level| {
                let mut encoded = String::with_capacity(level.rgba.len() * 2);
                for byte in &level.rgba {
                    encoded.push_str(&format!("{byte:02x}"));
                }
                json!({"width": level.width, "height": level.height, "rgba": encoded})
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "generated",
            "levels": described,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_transparency_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let bytes = hex_bytes(string(row, "png")?)?;
        match voxel_render::decode_rgba(&bytes) {
            Err(_) => out.push(json!({"label": label, "outcome": "failed"})),
            Ok(image) => {
                let measured = voxel_render::compute_transparency(&image);
                out.push(json!({
                    "label": label,
                    "outcome": "measured",
                    "transparent": measured.has_transparent,
                    "translucent": measured.has_translucent,
                    "opaque": measured.is_opaque(),
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}
