use super::*;

/// `name_tags`: a remote player's name tag, whether it shows and what is
/// submitted for it, with the viewer the scene names.
pub(super) fn name_tags_result(request: &Value) -> Result<Value, String> {
    use voxel_render::entity::name_tag::{
        DisplayMode, Viewer, camera_orientation, name_tag, submits,
    };
    use voxel_render::gui::font::Font;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let bits64 = |row: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(row, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let bits32 = |row: &Value, key: &str| -> Result<f32, String> {
        u32::from_str_radix(string(row, key)?, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let flag = |row: &Value, key: &str| -> Result<bool, String> {
        row[key]
            .as_bool()
            .ok_or_else(|| format!("{key} must be a bool"))
    };
    let hex64 = |value: f64| format!("{:016x}", value.to_bits());
    let hex32 = |value: f32| format!("{:08x}", value.to_bits());
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        let (entity_id, rider_id) = (2, 3);
        let (x, y, z) = (bits64(row, "x")?, bits64(row, "y")?, bits64(row, "z")?);
        for (id, at) in [(entity_id, (x, y, z)), (rider_id, (x, y, z))] {
            if !world.apply_entity(EntityInput {
                entity_id: id,
                uuid: Uuid::new(0, id as u64),
                type_id: 156,
                x: at.0,
                y: at.1,
                z: at.2,
                movement: (0.0, 0.0, 0.0),
                rotation: (bits32(row, "y_rot")?, 0.0, 0.0),
                data: 0,
            }) {
                return Err("name tag player was rejected".into());
            }
        }
        // The shared flags (crouching key, invisible) and the pose, as the
        // server's data packet carries them.
        let flags = u8::from(flag(row, "shift_key_down")?) * 0x02
            + u8::from(flag(row, "invisible")?) * 0x20;
        let pose = if flag(row, "crouching")? { 5 } else { 0 };
        if !world.apply_entity_metadata(entity_id, &[0, 0, flags, 6, 20, pose, 255]) {
            return Err("name tag metadata was rejected".into());
        }
        if flag(row, "vehicle")? && !world.apply_entity_passengers(entity_id, &[rider_id]) {
            return Err("name tag passengers were rejected".into());
        }
        let entity = world.entities().at(entity_id).ok_or("player vanished")?;
        let camera = Vec3::new(
            bits64(row, "camera_x")?,
            bits64(row, "camera_y")?,
            bits64(row, "camera_z")?,
        );
        let viewer = Viewer {
            camera_position: camera,
            spectator: flag(row, "local_spectator")?,
            hud_hidden: flag(row, "hud_hidden")?,
        };
        let mut described = json!({"label": row["label"]});
        if let Some(tag) = name_tag(entity, string(row, "name")?, viewer, 0.0) {
            let orientation =
                camera_orientation(bits32(row, "camera_y_rot")?, bits32(row, "camera_x_rot")?);
            let opacity = if flag(row, "background_for_chat_only")? {
                0.25
            } else {
                bits32(row, "background_opacity")?
            };
            let placed = submits(
                &tag,
                Vec3::new(x - camera.x, y - camera.y, z - camera.z),
                font.width(&tag.text),
                i32_field(row, "light")?,
                &orientation,
                opacity,
            );
            described["shown"] = json!(true);
            described["text"] = json!(tag.text);
            described["attachment_y"] = json!(hex64(tag.attachment.y));
            described["submits"] = json!(
                placed
                    .iter()
                    .map(|submit| json!({
                        "phase": if submit.mode == DisplayMode::SeeThrough { "see_through" } else { "normal" },
                        "pose": (0..4)
                            .flat_map(|column| (0..4).map(move |row| (column, row)))
                            .map(|(column, row)| hex32(submit.pose.m[column][row]))
                            .collect::<Vec<_>>(),
                        "x": hex32(submit.x),
                        "y": hex32(submit.y),
                        "text": tag.text,
                        "light": submit.light,
                        "color": submit.color,
                        "background": submit.background,
                        "mode": if submit.mode == DisplayMode::SeeThrough { "SEE_THROUGH" } else { "NORMAL" },
                    }))
                    .collect::<Vec<_>>()
            );
        } else {
            described["shown"] = json!(false);
            described["submits"] = json!([]);
        }
        out.push(described);
    }
    Ok(json!({"rows": out}))
}

/// `entity_shadows`: the pieces of an entity's shadow over a lit scene and
/// the corners drawn for them.
pub(super) fn entity_shadows_result(request: &Value) -> Result<Value, String> {
    use voxel_world::ShadowInput;
    let bits64 = |row: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(row, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let bits32 = |row: &Value, key: &str| -> Result<f32, String> {
        u32::from_str_radix(string(row, key)?, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        world.light_from_sources();
        world.run_light_updates();
        let shown = row["shadows_option"].as_bool().ok_or("shadows_option")?
            && !row["invisible"].as_bool().ok_or("invisible")?;
        let radius = if shown {
            bits32(row, "radius")?.min(32.0)
        } else {
            0.0
        };
        let pieces = if shown {
            world.entity_shadow_pieces(ShadowInput {
                position: (bits64(row, "x")?, bits64(row, "y")?, bits64(row, "z")?),
                radius: bits32(row, "radius")?,
                strength: bits32(row, "strength")?,
                distance_sq: bits64(row, "distance_sq")?,
                sky_darken: 0,
            })
        } else {
            Vec::new()
        };
        let translation = [bits32(row, "tx")?, bits32(row, "ty")?, bits32(row, "tz")?];
        let described: Vec<Value> = pieces
            .iter()
            .map(|piece| {
                json!({
                    "rx": hex(piece.relative[0]),
                    "ry": hex(piece.relative[1]),
                    "rz": hex(piece.relative[2]),
                    "min": piece.min.map(hex).join(""),
                    "max": piece.max.map(hex).join(""),
                    "alpha": hex(piece.alpha),
                })
            })
            .collect();
        let vertices: Vec<Value> = pieces
            .iter()
            .flat_map(|piece| voxel_render::entity::shadow::corners(piece, radius, translation))
            .map(|corner| {
                json!({
                    "x": hex(corner.position[0]),
                    "y": hex(corner.position[1]),
                    "z": hex(corner.position[2]),
                    "color": (corner.alpha << 24) | 0x00FF_FFFF,
                    "u": hex(corner.uv[0]),
                    "v": hex(corner.uv[1]),
                })
            })
            .collect();
        out.push(json!({
            "label": row["label"],
            "pieces": described,
            "radius": hex(radius),
            "vertices": vertices,
        }));
    }
    Ok(json!({"rows": out}))
}
