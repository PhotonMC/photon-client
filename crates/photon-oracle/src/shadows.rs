use super::*;
use photon_core::ids::EntityId;

/// `name_tags`: a name tag over a remote player or another entity, whether it shows and what is
/// submitted for it, with the viewer the scene names.
pub(super) fn name_tags_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::name_tag::{Viewer, name_tag};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = photon_render::gui::font::Font::load(&root).ok_or("font")?;
    let lang = photon_game::text::Lang::load(&root);
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        let ids = (EntityId(2), EntityId(3));
        let kind = row["type"].as_str();
        match kind {
            Some(kind) => add_entity(&mut world, row, kind, ids)?,
            None => add_player(&mut world, row, ids)?,
        }
        let entity = world.entities().at(ids.0).ok_or("entity vanished")?;
        let viewer = Viewer {
            camera_position: Vec3::new(
                f64_field(row, "camera_x")?,
                f64_field(row, "camera_y")?,
                f64_field(row, "camera_z")?,
            ),
            spectator: flag(row, "local_spectator"),
            hud_hidden: flag(row, "hud_hidden"),
        };
        let tag = match kind {
            Some(_) => entity_tag(entity, row, (viewer, &lang)),
            None => name_tag(entity, string(row, "name")?, viewer, 0.0),
        };
        out.push(described_tag(row, tag.as_ref(), &font)?);
    }
    Ok(json!({"rows": out}))
}

fn flag(row: &Value, key: &str) -> bool {
    row[key].as_bool().unwrap_or(false)
}

/// A request's remote player, in the world the way the server's packets would leave it.
fn add_player(
    world: &mut ClientWorld,
    row: &Value,
    (entity_id, rider_id): (EntityId, EntityId),
) -> Result<(), String> {
    for id in [entity_id, rider_id] {
        if !world.apply_entity(EntityInput {
            entity_id: id,
            uuid: Uuid::new(0, id.0 as u64),
            type_id: photon_data::entity_type::named("minecraft:player"),
            x: f64_field(row, "x")?,
            y: f64_field(row, "y")?,
            z: f64_field(row, "z")?,
            movement: (0.0, 0.0, 0.0),
            rotation: (f32_field(row, "y_rot")?, 0.0, 0.0),
            data: 0,
        }) {
            return Err("name tag player was rejected".into());
        }
    }
    // The shared flags (crouching key, invisible) and the pose, as the server's data packet
    // carries them.
    let flags =
        u8::from(flag(row, "shift_key_down")) * 0x02 + u8::from(flag(row, "invisible")) * 0x20;
    let pose = if flag(row, "crouching") { 5 } else { 0 };
    if !world.apply_entity_metadata(entity_id, &[0, 0, flags, 6, 20, pose, 255]) {
        return Err("name tag metadata was rejected".into());
    }
    if flag(row, "vehicle") && !world.apply_entity_passengers(entity_id, &[rider_id]) {
        return Err("name tag passengers were rejected".into());
    }
    Ok(())
}

/// What a row answers: whether the tag shows, and what is submitted for it.
fn described_tag(
    row: &Value,
    tag: Option<&photon_render::entity::name_tag::NameTag>,
    font: &photon_render::gui::font::Font,
) -> Result<Value, String> {
    use photon_render::entity::name_tag::{DisplayMode, camera_orientation, submits};
    let mut described = json!({"label": row["label"]});
    let Some(tag) = tag else {
        described["shown"] = json!(false);
        described["submits"] = json!([]);
        return Ok(described);
    };
    let hex32 = |value: f32| format!("{:08x}", value.to_bits());
    let orientation = camera_orientation(
        f32_field(row, "camera_y_rot")?,
        f32_field(row, "camera_x_rot")?,
    );
    let opacity = if flag(row, "background_for_chat_only") {
        0.25
    } else {
        f32_field(row, "background_opacity")?
    };
    let placement = Vec3::new(
        f64_field(row, "x")? - f64_field(row, "camera_x")?,
        f64_field(row, "y")? - f64_field(row, "camera_y")?,
        f64_field(row, "z")? - f64_field(row, "camera_z")?,
    );
    let placed = submits(
        tag,
        placement,
        font.width_of_text_runs(&tag.runs),
        i32_field(row, "light")?,
        &orientation,
        opacity,
    );
    described["shown"] = json!(true);
    described["text"] = json!(tag.text());
    described["attachment_y"] = json!(format!("{:016x}", tag.attachment.y.to_bits()));
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
                "text": tag.text(),
                "light": submit.light,
                "color": submit.color,
                "background": submit.background,
                "mode": if submit.mode == DisplayMode::SeeThrough { "SEE_THROUGH" } else { "NORMAL" },
            }))
            .collect::<Vec<_>>()
    );
    Ok(described)
}

/// A request's entity of a type other than the player, with the state the row names, in the
/// world the way the server's packets would leave it.
fn add_entity(
    world: &mut ClientWorld,
    row: &Value,
    kind: &str,
    (entity_id, rider_id): (EntityId, EntityId),
) -> Result<(), String> {
    for id in [entity_id, rider_id] {
        let input = EntityInput {
            entity_id: id,
            uuid: Uuid::new(0, id.0 as u64),
            type_id: photon_data::entity_type::named(if id == entity_id {
                kind
            } else {
                "minecraft:player"
            }),
            x: f64_field(row, "x")?,
            y: f64_field(row, "y")?,
            z: f64_field(row, "z")?,
            movement: (0.0, 0.0, 0.0),
            rotation: (f32_field(row, "y_rot")?, 0.0, 0.0),
            data: 0,
        };
        if !world.apply_entity(input) {
            return Err(format!("{kind} was rejected"));
        }
    }
    if !world.apply_entity_metadata(entity_id, &entity_metadata(row)) {
        return Err("entity metadata was rejected".into());
    }
    world
        .entities_mut()
        .apply_attributes(entity_id, &changed_attributes(row)?);
    if flag(row, "vehicle") && !world.apply_entity_passengers(entity_id, &[rider_id]) {
        return Err("entity passengers were rejected".into());
    }
    Ok(())
}

/// The data packet of a row's entity: the shared flags, the custom name and whether it shows,
/// and the baby flag.
fn entity_metadata(row: &Value) -> Vec<u8> {
    let flags =
        u8::from(flag(row, "shift_key_down")) * 0x02 + u8::from(flag(row, "invisible")) * 0x20;
    let mut metadata = vec![0, 0, flags];
    if let Some(name) = row["custom_name"].as_str() {
        metadata.extend([2, 6, 1]);
        photon_core::nbt::write_any_nbt(
            Some(&photon_core::nbt::FieldValue::Str(name.to_owned())),
            &mut metadata,
        );
    }
    metadata.extend([3, 8, u8::from(flag(row, "custom_name_visible"))]);
    if flag(row, "baby") {
        metadata.extend([16, 8, 1]);
    }
    metadata.push(255);
    metadata
}

/// The attributes a row changes from the type's own.
fn changed_attributes(row: &Value) -> Result<Vec<photon_world::ClientAttributeState>, String> {
    let mut changed = Vec::new();
    for (key, name) in [
        ("scale", "minecraft:scale"),
        ("name_tag_distance", "minecraft:name_tag_distance"),
    ] {
        if row.get(key).is_some() {
            let (id, _) = photon_data::entity_type::attribute(name).ok_or("attribute")?;
            changed.push(photon_world::ClientAttributeState::new(
                id as i32,
                f64_field(row, key)?,
                Vec::new(),
            ));
        }
    }
    Ok(changed)
}

/// The tag of an entity that is not a player, with the name the row's entity would show.
fn entity_tag(
    entity: &photon_world::ClientEntity,
    row: &Value,
    (viewer, lang): (
        photon_render::entity::name_tag::Viewer,
        &photon_game::text::Lang,
    ),
) -> Option<photon_render::entity::name_tag::NameTag> {
    use photon_render::entity::tag_rules::{NameRules, entity_name_tag, rules_of};
    // The renderer the reference stands in with, and the rules the client classes the type by.
    let rules = rules_of(entity);
    let stands_in = match row["renderer"].as_str() {
        Some("mob") => Some(NameRules::Mob),
        Some("armor_stand") => Some(NameRules::ArmorStand),
        _ => Some(NameRules::Entity),
    };
    if rules != stands_in {
        return None;
    }
    let rules = rules?;
    let picked = row["picked"].as_bool().unwrap_or(false);
    let runs = photon_gui::entity_name::entity_name_runs(entity, lang);
    entity_name_tag(entity, runs, (rules, picked), viewer)
}

/// `entity_shadows`: the pieces of an entity's shadow over a lit scene and
/// the corners drawn for them.
pub(super) fn entity_shadows_result(request: &Value) -> Result<Value, String> {
    use photon_world::ShadowInput;
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
            .flat_map(|piece| photon_render::entity::shadow::corners(piece, radius, translation))
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
