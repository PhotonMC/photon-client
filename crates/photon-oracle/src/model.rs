use super::*;

type Operation = fn(&Value) -> Result<Value, String>;

/// The operations that have a function of their own.
const DELEGATED: [(&str, Operation); 11] = [
    ("nearest_direction", model_nearest_direction_result),
    ("uv_lock", model_uv_lock_result),
    ("blockstate_parse", model_blockstate_result),
    ("model_state", model_state_result),
    ("parse_model", model_parse_result),
    ("model_ao", crate::model_ao::model_ambient_occlusion_result),
    ("face_uv_rotation", model_uv_rotation_result),
    ("euler_rotation", model_euler_result),
    ("quad_winding", model_winding_result),
    ("element_rotation", model_rotation_result),
    ("face_geometry", model_geometry_result),
];

fn model_nearest_direction_result(request: &Value) -> Result<Value, String> {
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
            "direction": photon_render::approximate_nearest_direction(vector).name(),
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "cull_rotation" {
        use photon_core::direction::Direction;
        use photon_render::block::rotation;
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
    let operation = string(request, "operation")?;
    if let Some((_, run)) = DELEGATED.iter().find(|(name, _)| *name == operation) {
        return run(request);
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
        let mut resolver = photon_render::TextureSlotsResolver::new();
        let chain = row["chain"].as_array().ok_or("chain must be an array")?;
        for level in chain {
            let slots = level.as_array().ok_or("chain level must be an array")?;
            let mut entries = Vec::with_capacity(slots.len());
            for slot in slots {
                let name = string(slot, "slot")?;
                let contents = if slot.get("raw").is_some() {
                    photon_render::SlotContents::parse_string(string(slot, "raw")?)
                        .map_err(|error| format!("texture value: {error:?}"))?
                } else {
                    photon_render::SlotContents::Value(photon_render::Material {
                        sprite: Identifier::parse(string(slot, "sprite")?)
                            .map_err(|error| format!("sprite: {error:?}"))?,
                        force_translucent: slot["force_translucent"]
                            .as_bool()
                            .ok_or("force_translucent must be a bool")?,
                    })
                };
                entries.push((name.to_owned(), contents));
            }
            resolver.add_last(photon_render::TextureSlotsData::from_entries(entries));
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

pub(super) fn model_rotation_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::rotation;
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
        let mut origin = read("origin")?;
        // The deserializer scales the origin into block units before use.
        for axis in &mut origin {
            *axis *= 0.0625;
        }
        let angle = f32::from_bits(
            u32::from_str_radix(string(row, "angle")?, 16)
                .map_err(|error| format!("angle: {error}"))?,
        );
        let axis = match string(row, "axis")? {
            "x" => rotation::RotationAxis::X,
            "y" => rotation::RotationAxis::Y,
            "z" => rotation::RotationAxis::Z,
            _ => {
                out.push(json!({"label": label, "outcome": "unknown_input"}));
                continue;
            }
        };
        let Some(facing) = direction_by_name(string(row, "facing")?) else {
            out.push(json!({"label": label, "outcome": "unknown_input"}));
            continue;
        };
        let rescale = row["rescale"].as_bool().ok_or("rescale must be a bool")?;

        let transform = rotation::element_transform(axis, angle, rescale);
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        let corners: Vec<Value> = photon_render::face_vertices(from, to, facing)
            .iter()
            .map(|vertex| {
                let rotated = rotation::rotate_vertex(*vertex, origin, &transform);
                json!({
                    "x": format!("{:08x}", rotated[0].to_bits()),
                    "y": format!("{:08x}", rotated[1].to_bits()),
                    "z": format!("{:08x}", rotated[2].to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "rotated",
            "transform": matrix,
            "corners": corners,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_winding_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let given = row["corners"]
            .as_array()
            .ok_or("corners must be an array")?;
        let mut positions = [[0.0_f32; 3]; 4];
        for (slot, item) in positions.iter_mut().zip(given) {
            let array = item.as_array().ok_or("corner must be an array")?;
            for (axis, value) in slot.iter_mut().zip(array) {
                let hex = value.as_str().ok_or("corner element must be a string")?;
                *axis = f32::from_bits(
                    u32::from_str_radix(hex, 16).map_err(|error| format!("corner: {error}"))?,
                );
            }
        }

        let normal = photon_render::triangle_normal(positions[0], positions[1], positions[2]);
        let mut entry = serde_json::Map::new();
        entry.insert("label".into(), json!(label));
        entry.insert("outcome".into(), json!("wound"));
        entry.insert(
            "normal_x".into(),
            json!(format!("{:08x}", normal[0].to_bits())),
        );
        entry.insert(
            "normal_y".into(),
            json!(format!("{:08x}", normal[1].to_bits())),
        );
        entry.insert(
            "normal_z".into(),
            json!(format!("{:08x}", normal[2].to_bits())),
        );

        match photon_render::closest_direction(normal) {
            None => {
                entry.insert("direction".into(), json!("none"));
            }
            Some(direction) => {
                entry.insert("direction".into(), json!(direction_name(direction)));
                let mut payloads = [10_u64, 20, 30, 40];
                if photon_render::recalculate_winding(&mut positions, &mut payloads, direction)
                    .is_err()
                {
                    entry.insert("winding".into(), json!("no_matching_vertex"));
                } else {
                    let wound: Vec<Value> = positions
                        .iter()
                        .map(|vertex| {
                            json!({
                                "x": format!("{:08x}", vertex[0].to_bits()),
                                "y": format!("{:08x}", vertex[1].to_bits()),
                                "z": format!("{:08x}", vertex[2].to_bits()),
                            })
                        })
                        .collect();
                    entry.insert("wound".into(), json!(wound));
                    entry.insert("uv_order".into(), json!(payloads.to_vec()));
                }
            }
        }
        out.push(Value::Object(entry));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn direction_name(direction: photon_core::direction::Direction) -> &'static str {
    use photon_core::direction::Direction;
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

pub(super) fn model_euler_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let read = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(string(row, key)?, 16)
                    .map_err(|error| format!("{key}: {error}"))?,
            ))
        };
        let transform = rotation::euler_element_transform(
            read("x")?,
            read("y")?,
            read("z")?,
            row["rescale"].as_bool().ok_or("rescale must be a bool")?,
        );
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "rotated",
            "transform": matrix,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_uv_rotation_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let degrees = i32_field(row, "degrees")?;
        let Some(rotation) = photon_render::Quadrant::from_degrees(degrees) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let values = row["uvs"].as_array().ok_or("uvs must be an array")?;
        let mut parsed = [0.0_f32; 4];
        for (slot, item) in parsed.iter_mut().zip(values) {
            let hex = item.as_str().ok_or("uv element must be a string")?;
            *slot = f32::from_bits(
                u32::from_str_radix(hex, 16).map_err(|error| format!("uv: {error}"))?,
            );
        }
        let uvs = photon_render::FaceUvs {
            min_u: parsed[0],
            min_v: parsed[1],
            max_u: parsed[2],
            max_v: parsed[3],
        };
        let pairs: Vec<Value> = (0..4)
            .map(|vertex| {
                let (u, v) = photon_render::face_vertex_uv(uvs, rotation, vertex);
                json!({
                    "u": format!("{:08x}", u.to_bits()),
                    "v": format!("{:08x}", v.to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "rotated",
            "shift": rotation.shift(),
            "pairs": pairs,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn model_parse_result(request: &Value) -> Result<Value, String> {
    use photon_render::model;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "text")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("text hex: {error}"))?;
        let Ok(text) = core::str::from_utf8(&bytes) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let parsed = match model::parse_model(text) {
            Err(_) => {
                out.push(json!({"label": label, "outcome": "rejected"}));
                continue;
            }
            Ok(parsed) => parsed,
        };

        let slots: Vec<Value> = parsed
            .texture_slots
            .entries()
            .map(|(slot, contents)| match contents {
                model::SlotContents::Reference(target) => json!({
                    "slot": slot, "kind": "Reference", "target": target,
                }),
                model::SlotContents::Value(material) => json!({
                    "slot": slot,
                    "kind": "Value",
                    "sprite": material.sprite.as_string(),
                    "force_translucent": material.force_translucent,
                }),
            })
            .collect();

        let elements: Vec<Value> = parsed
            .elements
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|element| {
                let mut faces: Vec<&(photon_core::direction::Direction, model::ModelFace)> =
                    element.faces.iter().collect();
                // The oracle reads faces out of a map keyed by direction name.
                faces.sort_by_key(|(direction, _)| direction_name(*direction));
                let described_faces: Vec<Value> = faces
                    .iter()
                    .map(|(direction, face)| {
                        json!({
                            "direction": direction_name(*direction),
                            "cullface": face
                                .cull_for_direction
                                .map_or("none", direction_name),
                            "tint_index": face.tint_index,
                            "texture": face.texture,
                            "uv_rotation": face.rotation.shift(),
                            "uvs": match face.uvs {
                                None => json!("none"),
                                Some(uvs) => json!([
                                    format!("{:08x}", uvs.min_u.to_bits()),
                                    format!("{:08x}", uvs.min_v.to_bits()),
                                    format!("{:08x}", uvs.max_u.to_bits()),
                                    format!("{:08x}", uvs.max_v.to_bits()),
                                ]),
                            },
                        })
                    })
                    .collect();
                let bits = |vector: [f32; 3]| {
                    json!(
                        vector
                            .iter()
                            .map(|axis| format!("{:08x}", axis.to_bits()))
                            .collect::<Vec<_>>()
                    )
                };
                json!({
                    "from": bits(element.from),
                    "to": bits(element.to),
                    "shade": element.shade,
                    "light_emission": element.light_emission,
                    "rotation": match &element.rotation {
                        None => json!("none"),
                        Some(rotation) => {
                            let mut matrix = Vec::with_capacity(16);
                            for column in &rotation.transform {
                                for value in column {
                                    matrix.push(json!(format!("{:08x}", value.to_bits())));
                                }
                            }
                            json!({
                                "origin": bits(rotation.origin),
                                "rescale": rotation.rescale,
                                "transform": matrix,
                            })
                        }
                    },
                    "faces": described_faces,
                })
            })
            .collect();

        let mut display = serde_json::Map::new();
        if let Some(transforms) = &parsed.display {
            let bits = |vector: [f32; 3]| {
                json!(
                    vector
                        .iter()
                        .map(|axis| format!("{:08x}", axis.to_bits()))
                        .collect::<Vec<_>>()
                )
            };
            for context in model::DisplayContext::ALL {
                if let Some(transform) = transforms.get(context) {
                    display.insert(
                        context.name().to_owned(),
                        json!({
                            "rotation": bits(transform.rotation),
                            "translation": bits(transform.translation),
                            "scale": bits(transform.scale),
                        }),
                    );
                }
            }
        }
        out.push(json!({
            "label": label,
            "outcome": "parsed",
            "parent": parsed
                .parent
                .as_ref()
                .map_or("none".to_owned(), |name| name.as_string()),
            "ambient_occlusion": parsed
                .ambient_occlusion
                .map_or("unset".to_owned(), |value| value.to_string()),
            "gui_light": match parsed.gui_light {
                None => "unset",
                Some(model::GuiLight::Front) => "FRONT",
                Some(model::GuiLight::Side) => "SIDE",
            },
            "texture_slots": slots,
            "has_geometry": parsed.elements.is_some(),
            "elements": elements,
            "display": display,
        }));
    }
    Ok(json!({"rows": out}))
}
