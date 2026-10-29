use super::*;

/// `item_frame_pose`: the poses an item frame's model and the item in it are drawn from.
pub(super) fn item_frame_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::objects::item_frame::item_frame_poses;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let direction = photon_core::direction::Direction::from_name(string(row, "direction")?)
            .ok_or("unknown direction")?;
        let invisible = row["invisible"].as_bool().ok_or("invisible")?;
        let mut base = photon_render::joml::Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let poses = item_frame_poses(&base, direction, invisible, i32_field(row, "rotation")?);
        // The frame's model first, if it has one, then the item (a map takes its place).
        let mut described = Vec::new();
        if let Some(frame) = poses.frame {
            described.push(matrix_bits(&frame));
        }
        let mut map_vertices = Vec::new();
        let holds_map = row["map"].as_bool() == Some(true);
        if holds_map {
            // `getLightCoords(isGlowFrame, 15728850, lightCoords)`.
            let light = if row["glow"].as_bool() == Some(true) {
                15_728_850
            } else {
                i32_field(row, "light")?
            };
            for quad in photon_render::objects::map::map_quads(&poses.map, &[], true) {
                for (corner, uv) in quad.corners.iter().zip(quad.uvs) {
                    map_vertices.push(json!(format!(
                        "{:08x}{:08x}{:08x},{:08x}{:08x},-1,{light}",
                        corner[0].to_bits(),
                        corner[1].to_bits(),
                        corner[2].to_bits(),
                        uv[0].to_bits(),
                        uv[1].to_bits()
                    )));
                }
            }
        } else if row["item"].as_bool() == Some(true) {
            // The item's layer, which has no transform of its own here, centres its model.
            let mut item = poses.item;
            item.translate(-0.5, -0.5, -0.5);
            described.push(matrix_bits(&item));
        }
        out.push(json!({"label": row["label"], "poses": described, "map_vertices": map_vertices}));
    }
    Ok(json!({"rows": out}))
}

/// `display_pose`: the pose (and its normal matrix) a display's block or item is drawn from.
pub(super) fn display_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::objects::display::{DisplayTurns, display_pose, item_display_pose};
    use photon_world::objects::display::{Billboard, Transformation};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |value: &Value| -> Result<f32, String> {
            u32::from_str_radix(value.as_str().ok_or("a float is a string")?, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
        };
        let bits = |key: &str| float(&row[key]);
        let parts = |key: &str| -> Result<Vec<f32>, String> {
            row[key]
                .as_array()
                .ok_or("a vector is an array")?
                .iter()
                .map(float)
                .collect()
        };
        let (translation, scale) = (parts("translation")?, parts("scale")?);
        let (left, right) = (parts("left_rotation")?, parts("right_rotation")?);
        let transformation = Transformation {
            translation: [translation[0], translation[1], translation[2]],
            left_rotation: [left[0], left[1], left[2], left[3]],
            scale: [scale[0], scale[1], scale[2]],
            right_rotation: [right[0], right[1], right[2], right[3]],
        };
        let billboard = match i32_field(row, "billboard")? {
            1 => Billboard::Vertical,
            2 => Billboard::Horizontal,
            3 => Billboard::Center,
            _ => Billboard::Fixed,
        };
        let turns = DisplayTurns {
            entity: (bits("entity_y_rot")?, bits("entity_x_rot")?),
            camera: (bits("camera_y_rot")?, bits("camera_x_rot")?),
        };
        let mut base = photon_render::joml::Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let mut pose = display_pose(&base, (billboard, &transformation), &turns);
        if string(row, "kind")? == "item" {
            pose = item_display_pose(&pose);
            // The item's layer, which has no transform of its own here, centres its model.
            pose.translate(-0.5, -0.5, -0.5);
        }
        let mut described = matrix_bits(&pose).concat();
        for column in 0..3 {
            for row in 0..3 {
                described.push_str(&format!("{:08x}", pose.normal.m[column][row].to_bits()));
            }
        }
        out.push(json!({"label": row["label"], "poses": [described]}));
    }
    Ok(json!({"rows": out}))
}

/// `display_light`: the light a display is drawn with when the server set its brightness.
pub(super) fn display_light_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let brightness = i32_field(row, "brightness")?;
        // Only an override is asked, so the world's light does not show.
        let light = photon_render::objects::display::displayed_light(0, brightness);
        out.push(json!({"label": row["label"], "light": light}));
    }
    Ok(json!({"rows": out}))
}

/// `text_display_submit`: what a text display submits for lines of given widths.
pub(super) fn text_display_submit_result(request: &Value) -> Result<Value, String> {
    use photon_render::objects::display::{DisplayTurns, display_pose, text_submit};
    use photon_world::objects::display::{Billboard, Transformation};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(string(row, key)?, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
        };
        let widths: Vec<i32> = row["widths"]
            .as_array()
            .ok_or("widths")?
            .iter()
            .map(|width| width.as_i64().map(|width| width as i32).ok_or("a width"))
            .collect::<Result<_, _>>()?;
        let light = i32_field(row, "light")?;
        let mut base = photon_render::joml::Pose::default();
        base.translate(bits("feet_x")?, bits("feet_y")?, bits("feet_z")?);
        let turns = DisplayTurns {
            entity: (0.0, 0.0),
            camera: (0.0, 0.0),
        };
        let pose = display_pose(&base, (Billboard::Fixed, &Transformation::IDENTITY), &turns);
        let submit = text_submit(
            &pose,
            i32_field(row, "flags")? as i8,
            (i32_field(row, "opacity")?, i32_field(row, "background")?),
            (&widths, 0.25),
        );
        let matrix = submit
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect::<String>();
        let order = i32::from(submit.background.is_some());
        let mut described = json!({"label": row["label"], "pose": matrix, "text_order": order});
        let mut background = Vec::new();
        if let Some((corners, color)) = submit.background {
            for [x, y, z] in corners {
                let [px, py, pz] = submit.pose.transform_position(x, y, z);
                background.push(json!(format!(
                    "{:08x}{:08x}{:08x},{color},{light}",
                    px.to_bits(),
                    py.to_bits(),
                    pz.to_bits()
                )));
            }
            described["see_through"] = json!(submit.see_through);
        }
        let mode = if submit.see_through {
            "SEE_THROUGH"
        } else {
            "POLYGON_OFFSET"
        };
        let texts: Vec<Value> = submit
            .lines
            .iter()
            .map(|(x, y)| {
                json!(format!(
                    "{:08x},{:08x},{},{mode},{light},{},0,0",
                    x.to_bits(),
                    y.to_bits(),
                    submit.shadow,
                    submit.color
                ))
            })
            .collect();
        if texts.is_empty() {
            described
                .as_object_mut()
                .ok_or("object")?
                .remove("text_order");
        }
        described["background"] = json!(background);
        described["texts"] = json!(texts);
        out.push(described);
    }
    Ok(json!({"rows": out}))
}

/// `map_render`: the quads a map and its decorations are drawn with.
pub(super) fn map_render_result(request: &Value) -> Result<Value, String> {
    use photon_render::objects::map::{DecorationDraw, Sheet, map_quads};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |value: &Value| -> Result<f32, String> {
            u32::from_str_radix(value.as_str().ok_or("a float is a string")?, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())
        };
        let decorations: Vec<DecorationDraw> = row["decorations"]
            .as_array()
            .ok_or("decorations")?
            .iter()
            .map(|decoration| {
                Ok(DecorationDraw {
                    x: i32_field(decoration, "x")? as i8,
                    y: i32_field(decoration, "y")? as i8,
                    rot: i32_field(decoration, "rot")? as i8,
                    render_on_frame: decoration["on_frame"].as_bool().ok_or("on_frame")?,
                    sprite: ["u0", "v0", "u1", "v1"]
                        .map(|key| float(&decoration["sprite"][key]).unwrap_or(0.0)),
                })
            })
            .collect::<Result<_, String>>()?;
        let mut base = photon_render::joml::Pose::default();
        base.translate(
            float(&row["feet_x"])?,
            float(&row["feet_y"])?,
            float(&row["feet_z"])?,
        );
        let light = i32_field(row, "light")?;
        let mut vertices = Vec::new();
        for quad in map_quads(
            &base,
            &decorations,
            row["only_frame"].as_bool().ok_or("only_frame")?,
        ) {
            let sheet = match quad.sheet {
                Sheet::Map => "map",
                Sheet::Decorations => "decorations",
            };
            for (corner, uv) in quad.corners.iter().zip(quad.uvs) {
                vertices.push(json!(format!(
                    "{sheet},{:08x}{:08x}{:08x},{:08x}{:08x},-1,{light}",
                    corner[0].to_bits(),
                    corner[1].to_bits(),
                    corner[2].to_bits(),
                    uv[0].to_bits(),
                    uv[1].to_bits()
                )));
            }
        }
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}

/// The operations about the enchantment glint.
pub(super) fn foil_result(request: &Value) -> Result<Value, String> {
    match string(request, "operation")? {
        "foil_decal_uv" => foil_decal_uv_result(request),
        _ => item_foil_result(request),
    }
}

/// `foil_decal_uv`: the special foil's texture coordinate of a corner, rounded to a 4096th.
fn foil_decal_uv_result(request: &Value) -> Result<Value, String> {
    let float = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or_else(|| key.to_owned())?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let facing = photon_core::direction::Direction::from_name(string(row, "direction")?)
            .ok_or("direction")?;
        let corner = [float(row, "x")?, float(row, "y")?, float(row, "z")?];
        let uv =
            photon_render::block::breaking::foil_decal_uv(corner, facing, float(row, "factor")?);
        // `Math.round`: half up.
        let round = |value: f32| {
            (value * 4096.0).floor() as i64
                + i64::from(value * 4096.0 - (value * 4096.0).floor() >= 0.5)
        };
        out.push(json!({"label": row["label"], "u": round(uv[0]), "v": round(uv[1])}));
    }
    Ok(json!({"rows": out}))
}

/// `item_foil`: whether a stack has the enchantment glint, from the component patch it carries.
fn item_foil_result(request: &Value) -> Result<Value, String> {
    use photon_game::item::values::Values;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        if row.get("all").is_some() {
            let foil: Vec<bool> = (0..photon_data::item::registry::ITEMS.len())
                .map(|index| {
                    Values::new(photon_core::ids::ItemId(index as i32), &[0, 0]).has_foil()
                })
                .collect();
            out.push(json!({"label": row["label"], "foil": foil}));
            continue;
        }
        let item =
            photon_data::item::registry::index(string(row, "item")?).ok_or("unknown item")?;
        if let Some(tag) = row.get("tag") {
            out.push(tag_stack_result(row, tag, item)?);
            continue;
        }
        let patch = hex_bytes(string(row, "patch")?)?;
        out.push(json!({
            "label": row["label"],
            "foil": Values::new(crate::case_ids::item(item as i32), &patch).has_foil(),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `item_equippable`: the slot and the equipment asset a stack is worn as, from its patch.
pub(super) fn item_equippable_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let item =
            photon_data::item::registry::index(string(row, "item")?).ok_or("unknown item")?;
        let patch = hex_bytes(string(row, "patch")?)?;
        let found =
            photon_game::item::values::equippable_of(crate::case_ids::item(item as i32), &patch);
        out.push(json!({
            "label": row["label"],
            "slot": found.as_ref().map_or("none", |found| found.slot),
            "asset": found
                .and_then(|found| found.asset)
                .unwrap_or_else(|| "none".to_owned()),
        }));
    }
    Ok(json!({"rows": out}))
}

/// `glint_layers`: the pipeline the armor's glint is drawn with, its texture, and the matrix that
/// moves the texture.
pub(super) fn glint_layers_result(request: &Value) -> Result<Value, String> {
    use photon_render::render_types::{
        ARMOR_GLINT_SCALE, ARMOR_GLINT_TEXTURE, GLINT, ITEM_GLINT_SCALE, ITEM_GLINT_TEXTURE,
        glint_texture_matrix,
    };
    let (texture, scale) = if request["render_type"].as_str() == Some("item") {
        (ITEM_GLINT_TEXTURE, ITEM_GLINT_SCALE)
    } else {
        (ARMOR_GLINT_TEXTURE, ARMOR_GLINT_SCALE)
    };
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let millis = row["millis"].as_i64().ok_or("millis")?;
        let speed = row["speed"].as_f64().ok_or("speed")?;
        let matrix: Vec<String> = glint_texture_matrix(millis, speed, scale)
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({"label": row["label"], "matrix": matrix}));
    }
    Ok(json!({
        "pipeline": GLINT.location,
        "flags": GLINT.flags,
        "values": GLINT.values,
        "blend": GLINT.blend,
        "depth_test": GLINT.depth_test,
        "depth_write": GLINT.depth_write,
        "cull": GLINT.cull,
        "texture": format!("textures/{texture}.png"),
        "rows": rows,
    }))
}

/// `chest_model`: a chest's model with its lid at an eased opening.
pub(super) fn chest_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::MobModel;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let open = f32::from_bits(
            u32::from_str_radix(row["open"].as_str().ok_or("open")?, 16)
                .map_err(|error| error.to_string())?,
        );
        let mut model = MobModel::bake(layer);
        model.setup_chest(open);
        rows.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": rows}))
}

/// `chest_pose`: where, with what lid, light and texture a chest is drawn.
pub(super) fn chest_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::chest::{ChestKind, ChestMaterial, chest_pose, eased_open};
    use photon_render::joml::Pose;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(row[key].as_str().ok_or(key.to_owned())?, 16)
                    .map_err(|error| error.to_string())?,
            ))
        };
        let kind = ChestKind::from_name(row["type"].as_str().ok_or("type")?).ok_or("type")?;
        let material = ChestMaterial::from_name(row["material"].as_str().ok_or("material")?)
            .ok_or("material")?;
        let facing =
            photon_core::direction::Direction::from_name(row["facing"].as_str().ok_or("facing")?)
                .ok_or("facing")?;
        let mut base = Pose::default();
        base.translate(float("x")?, float("y")?, float("z")?);
        let pose = chest_pose(&base, facing);
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({
            "label": row["label"],
            "matrix": matrix,
            "open": format!("{:08x}", eased_open(float("open")?).to_bits()),
            "light": row["light"],
            "texture": material.texture(kind),
            "layer": row["type"],
        }));
    }
    Ok(json!({"rows": rows}))
}

/// `shulker_box_pose`: where, with what lid, light and texture a shulker box is drawn.
pub(super) fn shulker_box_pose_result(request: &Value) -> Result<Value, String> {
    use photon_render::block_entity::shulker_box::{LAYER, box_pose, texture};
    use photon_render::entity::mob_model::MobModel;
    use photon_render::joml::Pose;
    let layer = photon_data::entity_model::layer(LAYER).ok_or("no such layer")?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(row[key].as_str().ok_or(key.to_owned())?, 16)
                    .map_err(|error| error.to_string())?,
            ))
        };
        let facing =
            photon_core::direction::Direction::from_name(row["facing"].as_str().ok_or("facing")?)
                .ok_or("facing")?;
        let mut base = Pose::default();
        base.translate(float("x")?, float("y")?, float("z")?);
        let pose = box_pose(&base, facing);
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        let mut model = MobModel::bake(layer);
        model.setup_shulker_box(float("progress")?);
        rows.push(json!({
            "label": row["label"],
            "matrix": matrix,
            "progress": format!("{:08x}", float("progress")?.to_bits()),
            "light": row["light"],
            "texture": texture(row["color"].as_str()),
            "quads": posed_quads(&model),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// A stack as a block entity's tag writes it: the glint and the potion tint it is drawn with.
fn tag_stack_result(row: &Value, tag: &Value, item: usize) -> Result<Value, String> {
    use photon_game::item::values::Values;
    let registry: Vec<String> = row["registry"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(|name| name.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let components = nbt_fields(tag)?;
    let patch = photon_world::item_components::tag::patch_from_tag(&components, &registry);
    let bytes = if patch.added.is_empty() {
        Vec::new()
    } else {
        patch.trusted()
    };
    let values = Values::new(crate::case_ids::item(item as i32), &bytes);
    // `ItemTintSource.Potion` with its default colour, made opaque.
    let tint =
        (0xFF00_0000_u32 | (values.potion_color_or(-13_083_194) as u32 & 0x00FF_FFFF)) as i32;
    Ok(json!({"label": row["label"], "foil": values.has_foil(), "tint": tint}))
}

/// A JSON object as the fields of a compound tag: numbers are ints, strings are strings.
fn nbt_fields(value: &Value) -> Result<Vec<photon_core::nbt::Field>, String> {
    use photon_core::nbt::{Field, FieldValue};
    let mut fields = Vec::new();
    for (key, member) in value.as_object().ok_or("tag")? {
        let value = match member {
            Value::String(text) => FieldValue::Str(text.clone()),
            Value::Number(number) => FieldValue::Int(number.as_i64().ok_or("number")? as i32),
            Value::Object(_) => FieldValue::Compound(nbt_fields(member)?),
            _ => return Err(format!("unsupported tag member {key}")),
        };
        fields.push(Field {
            key: key.clone(),
            value,
        });
    }
    Ok(fields)
}
