use super::*;

/// What the pick table says of how a leash is tied to a type, in the oracle's fields.
pub(super) fn leash_fields(
    out: &mut Value,
    name: &str,
    pick: &photon_data::entity_type::EntityPick,
    (width, height, eye_height): (f32, f32, f32),
    baby: bool,
    prefix: &str,
) {
    use photon_data::leash::{Size, leash_offset, quad_holder_offsets, quad_leash_offsets};
    let size = Size {
        width,
        height,
        eye_height,
    };
    let bits = |vector: [f64; 3]| -> Value {
        json!(vector.map(|value| format!("{:016x}", value.to_bits())))
    };
    if let Some(leash) = pick.leash {
        out[format!("{prefix}leashable")] = json!(true);
        out[format!("{prefix}leash_offset")] =
            bits(leash_offset(name, size, baby, Default::default()));
        out[format!("{prefix}quad")] = json!(leash.quad);
        if leash.quad {
            out[format!("{prefix}quad_offsets")] = json!(quad_leash_offsets(name, size).map(bits));
        }
    }
    out[format!("{prefix}quad_holder")] = json!(pick.quad_holder);
    if pick.quad_holder {
        out[format!("{prefix}quad_holder_offsets")] =
            json!(quad_holder_offsets(name, size).map(bits));
    }
}

/// The point a name tag hangs at, as the reference's row writes it (absent for none).
fn name_tag_field(out: &mut Value, point: Option<(f64, f64, f64)>, prefix: &str) {
    if let Some((x, y, z)) = point {
        out[format!("{prefix}name_tag")] =
            json!([x, y, z].map(|value| format!("{:016x}", value.to_bits())));
    }
}

/// `entity_pick_table`: the generated table of what the crosshair needs of
/// every entity type, answered as the reference's rows are.
pub(super) fn entity_pick_table_result(request: &Value) -> Value {
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let rows: Vec<Value> = request["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            let types: Vec<Value> =
                photon_data::entity_type::listed_in(crate::case_ids::ORACLE_VERSION)
                    .map(|(id, canonical, name)| {
                        let mut out = json!({"label": name, "id": id});
                        let Some(pick) = photon_data::entity_type::pick(canonical) else {
                            out["created"] = json!(false);
                            return out;
                        };
                        out["created"] = json!(true);
                        out["pickable"] = json!(pick.pickable);
                        out["pick_radius"] = json!(hex(pick.pick_radius));
                        out["from_inside"] = json!(pick.from_inside);
                        out["monster"] = json!(pick.monster);
                        let (width, height, eye_height) = pick.size;
                        out["width"] = json!(hex(width));
                        out["height"] = json!(hex(height));
                        out["eye_height"] = json!(hex(eye_height));
                        out["passengers"] = json!(
                            pick.passengers
                                .iter()
                                .map(|&(x, y, z)| [x, y, z]
                                    .map(|value| format!("{:016x}", value.to_bits())))
                                .collect::<Vec<_>>()
                        );
                        name_tag_field(&mut out, pick.name_tag, "");
                        leash_fields(&mut out, name, pick, pick.size, false, "");
                        if let Some((width, height, eye_height)) = pick.baby {
                            out["baby_width"] = json!(hex(width));
                            out["baby_height"] = json!(hex(height));
                            out["baby_eye_height"] = json!(hex(eye_height));
                            name_tag_field(&mut out, pick.baby_name_tag, "baby_");
                            leash_fields(
                                &mut out,
                                name,
                                pick,
                                (width, height, eye_height),
                                true,
                                "baby_",
                            );
                        }
                        out
                    })
                    .collect();
            json!({"label": row["label"], "types": types})
        })
        .collect();
    json!({"rows": rows})
}

/// `sky_flash`: the overworld's sky colour and sky light factor where nothing is loaded
/// (plains, no weather, the clocks at zero), with the lightning flash on or off.
pub(super) fn sky_flash_result(request: &Value) -> Result<Value, String> {
    use photon_render::sky::{Attribute, Environment, Value as Attr, Weather};
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let mut biomes: Vec<String> = std::fs::read_dir(data_root.join("minecraft/worldgen/biome"))
        .map_err(|error| error.to_string())?
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            Some(format!("minecraft:{}", name.strip_suffix(".json")?))
        })
        .collect();
    biomes.sort();
    let environment = Environment::load(&data_root, "minecraft:overworld", &biomes)
        .ok_or("the overworld's sky data could not be read")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let number = |key: &str| -> Result<f64, String> {
            u64::from_str_radix(string(row, key)?, 16)
                .map(f64::from_bits)
                .map_err(|error| format!("{key}: {error}"))
        };
        let flash =
            row["flash"].as_i64().ok_or("flash")? > 0 && !row["hide"].as_bool().ok_or("hide")?;
        let attributes = environment.evaluate(
            [number("x")?, number("y")?, number("z")?],
            |_, _, _| None,
            |_| 0,
            Weather {
                rain: 0.0,
                thunder: 0.0,
            },
            flash,
        );
        let Attr::Color(colour) = attributes.get(Attribute::SkyColor) else {
            return Err("the sky colour is not a colour".into());
        };
        let Attr::Float(factor) = attributes.get(Attribute::SkyLightFactor) else {
            return Err("the sky light factor is not a float".into());
        };
        rows.push(json!({
            "label": row["label"],
            "sky_color": colour as i32,
            "sky_light_factor": format!("{:08x}", factor.to_bits()),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// The sky operations of the movement family, by name; none for another kind.
pub(super) fn sky_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    match operation {
        "sky_flash" => Some(sky_flash_result(request)),
        "fog_data" => Some(super::view_state::fog_data_result(request)),
        "lightmap_data" => Some(super::view_state::lightmap_data_result(request)),
        _ => None,
    }
}
