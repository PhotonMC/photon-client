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
                photon_data::entity_type::listed_in(crate::case_ids::oracle_version())
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

/// `sky_flash`: the overworld's colours and sky light factor where nothing is loaded (plains), at
/// a clock and in a weather a row names (the clocks at zero and no weather when it names none),
/// with the lightning flash on or off.
pub(super) fn sky_flash_result(request: &Value) -> Result<Value, String> {
    use photon_render::sky::{Attribute, Environment, Value as Attr};
    let data_root = crate::jar_root().join("data");
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
        let ticks = row["clock"].as_i64().unwrap_or(0);
        let attributes = environment.evaluate(
            [number("x")?, number("y")?, number("z")?],
            |_, _, _| None,
            |_| ticks,
            weather_of(row)?,
            flash,
        );
        let colour = |attribute: Attribute| -> Result<i32, String> {
            match attributes.get(attribute) {
                Attr::Color(colour) => Ok(packed(colour)),
                _ => Err(format!("{attribute:?} is not a colour")),
            }
        };
        let Attr::Float(factor) = attributes.get(Attribute::SkyLightFactor) else {
            return Err("the sky light factor is not a float".into());
        };
        rows.push(json!({
            "label": row["label"],
            "sky_color": colour(Attribute::SkyColor)?,
            "colours": {
                "fog": colour(Attribute::FogColor)?,
                "cloud": colour(Attribute::CloudColor)?,
                "sunrise": colour(Attribute::SunriseSunsetColor)?,
                "sky_light": colour(Attribute::SkyLightColor)?,
            },
            "sky_light_factor": format!("{:08x}", factor.to_bits()),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// The rain and thunder levels a row names as float bit patterns, or none.
fn weather_of(row: &Value) -> Result<photon_render::sky::Weather, String> {
    let level = |index: usize| -> Result<f32, String> {
        let Some(text) = row["weather"].get(index).and_then(Value::as_str) else {
            return Ok(0.0);
        };
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("weather: {error}"))
    };
    // The level reports the thunder scaled by the rain.
    let rain = level(0)?;
    Ok(photon_render::sky::Weather {
        rain,
        thunder: level(1)? * rain,
    })
}

/// The colour as the signed int the reference's rows hold: the version's own conversion of the
/// attribute's value (`ARGB.colorFromVector4f` floors each channel; 26.2 held the packed int, and
/// the channel is the nearest to the float that stands for it).
fn packed(colour: [f32; 4]) -> i32 {
    let before = crate::case_ids::before_26_3();
    let channel = |value: f32| {
        let scaled = value * 255.0;
        (if before {
            scaled.round()
        } else {
            scaled.floor()
        }) as u32
    };
    (channel(colour[3]) << 24
        | channel(colour[0]) << 16
        | channel(colour[1]) << 8
        | channel(colour[2])) as i32
}

/// `biome_noise`: the precipitation and the grass colour a biome gives at a block, from the
/// biome tables read off the data folder.
pub(super) fn biome_noise_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::biome_tint::{BiomeTints, Precipitation, TintKind};
    let data_root = crate::jar_root().join("data");
    let resources = photon_core::resources::FolderResources::new(crate::jar_root().join("assets"));
    let mut names: Vec<String> = std::fs::read_dir(data_root.join("minecraft/worldgen/biome"))
        .map_err(|error| error.to_string())?
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            Some(format!("minecraft:{}", name.strip_suffix(".json")?))
        })
        .collect();
    names.sort();
    let tints = BiomeTints::load(&data_root, &resources, &names, 0)
        .ok_or("the biome tables could not be read")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let whole = |key: &str| -> Result<i32, String> {
            row[key]
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| format!("{key} is not an int"))
        };
        let name = string(row, "biome")?;
        let index = names
            .iter()
            .position(|candidate| candidate == name)
            .ok_or_else(|| format!("{name} is not a biome"))?;
        let (x, y, z) = (whole("x")?, whole("y")?, whole("z")?);
        let precipitation = tints.precipitation_at(
            [x, y, z],
            &|_, _, _| i32::try_from(index).ok(),
            whole("sea_level")?,
        );
        let mut out = json!({
            "label": row["label"],
            "precipitation": match precipitation {
                Precipitation::None => "NONE",
                Precipitation::Rain => "RAIN",
                Precipitation::Snow => "SNOW",
            },
        });
        if row["grass"].as_bool() == Some(true) {
            out["grass"] = json!(tints.colour(TintKind::Grass, index, x, z) as i32);
        }
        rows.push(out);
    }
    Ok(json!({"rows": rows}))
}

/// The sky operations of the movement family, by name; none for another kind.
pub(super) fn sky_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    match operation {
        "biome_noise" => Some(biome_noise_result(request)),
        "sky_flash" => Some(sky_flash_result(request)),
        "fog_data" => Some(super::view_state::fog_data_result(request)),
        "lightmap_data" => Some(super::view_state::lightmap_data_result(request)),
        _ => None,
    }
}
