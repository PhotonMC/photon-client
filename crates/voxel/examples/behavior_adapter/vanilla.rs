use super::*;

/// Every state's collision or outline shape as the reference lists them: the
/// distinct shapes by their boxes in the order they are first met, then for
/// each block the shape of its one state or of each.
///
/// The collision shape of a block that shifts by position is the one at the
/// origin; the outline is listed before the shift and a block is position
/// dependent when its shape is not empty and the shift is not always zero.
pub(super) fn shape_table(label: &str, outline: bool) -> Value {
    const ELSEWHERE: [[i32; 3]; 4] = [[13, 70, -7], [-200, 5, 999], [1, 1, 1], [-3, 64, 12]];
    let mut ids: std::collections::HashMap<Vec<u64>, usize> = std::collections::HashMap::new();
    let mut shapes: Vec<Value> = Vec::new();
    let states = voxel_world::block::states();
    let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
        .iter()
        .map(|block| {
            let mut per_state = Vec::new();
            let mut moves = false;
            for offset in 0..block.state_count() {
                let info = voxel_world::block::state(block.first_state_id() + offset)
                    .expect("a registry state");
                let origin_collision;
                let shape = if outline {
                    states.outline(info)
                } else {
                    origin_collision = voxel_world::block_state::collision_shape_at(
                        block.first_state_id() + offset,
                        [0; 3],
                    );
                    &origin_collision
                };
                let boxes: Vec<[u64; 6]> = shape
                    .to_aabbs()
                    .iter()
                    .map(|aabb| {
                        [
                            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
                        ]
                        .map(f64::to_bits)
                    })
                    .collect();
                let origin = voxel_world::block::offset(block, [0, 0, 0]);
                moves |= !boxes.is_empty()
                    && voxel_world::block::moves_with_position(block)
                    && ELSEWHERE.iter().any(|at| {
                        let shift = voxel_world::block::offset(block, *at);
                        if outline {
                            shift != [0.0; 3]
                        } else {
                            shift != origin
                        }
                    });
                let key: Vec<u64> = boxes.iter().flatten().copied().collect();
                let id = *ids.entry(key).or_insert_with(|| {
                    shapes.push(Value::Array(
                        boxes
                            .iter()
                            .map(|bits| {
                                Value::Array(
                                    bits.iter()
                                        .map(|bit| json!(format!("{bit:016x}")))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ));
                    shapes.len() - 1
                });
                per_state.push(id);
            }
            let listed = if per_state.iter().all(|id| *id == per_state[0]) {
                vec![per_state[0]]
            } else {
                per_state
            };
            let mut entry = json!({
                "name": block.name,
                "shapes": listed,
                "position_dependent": moves,
                "dynamic": block.dynamic_shape,
            });
            if outline {
                // The shift is explained by the same offset at any position.
                entry["offset_explained"] = json!(true);
            }
            entry
        })
        .collect();
    json!({"label": label, "outcome": "listed", "shapes": shapes, "blocks": blocks})
}

/// `palette_permutation`: a picture recoloured by a palette, as the atlas's paletted permutations do.
pub(super) fn palette_permutation_result(request: &Value) -> Result<Value, String> {
    use voxel_render::trim::permute;
    let image = |row: &Value, key: &str| -> Result<voxel_render::DecodedImage, String> {
        let hex = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| error.to_string())?;
        voxel_render::png::decode_rgba(&bytes).map_err(|error| format!("{error:?}"))
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mapped = permute(
            &image(row, "png")?,
            &image(row, "keys")?,
            &image(row, "values")?,
        )
        .ok_or("palettes differ in size")?;
        // The reference prints each pixel as `0xAARRGGBB`.
        let pixels: String = mapped
            .rgba
            .chunks_exact(4)
            .map(|p| format!("{:02x}{:02x}{:02x}{:02x}", p[3], p[0], p[1], p[2]))
            .collect();
        out.push(json!({"label": row["label"], "outcome": "mapped", "pixels": pixels}));
    }
    Ok(json!({"rows": out}))
}

/// `trim_sprite_id`: the picture a pattern in a material takes on an armor's layer.
pub(super) fn trim_sprite_id_result(request: &Value) -> Result<Value, String> {
    use voxel_render::trim::{TrimData, sprite_path};
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let data = TrimData::load(&data_root);
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let pattern = data
            .pattern(string(row, "pattern")?)
            .ok_or("no such trim pattern")?;
        let material = data
            .material(string(row, "material")?)
            .ok_or("no such trim material")?;
        let prefix = format!("trims/entity/{}", string(row, "layer")?);
        let path = sprite_path(&prefix, pattern, material, string(row, "equipment")?);
        out.push(json!({
            "label": row["label"],
            "outcome": "named",
            "sprite": format!("minecraft:{path}"),
            "decal": pattern.decal,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "srgb_tables" {
        return image_tables_result(request);
    }
    if string(request, "operation")? == "palette_permutation" {
        return palette_permutation_result(request);
    }
    if string(request, "operation")? == "trim_sprite_id" {
        return trim_sprite_id_result(request);
    }
    if string(request, "operation")? == "mip_levels" {
        return image_mip_result(request);
    }
    if string(request, "operation")? == "compute_transparency" {
        return image_transparency_result(request);
    }
    if string(request, "operation")? == "texture_metadata" {
        return image_texture_metadata_result(request);
    }
    if string(request, "operation")? == "sprite_uv" {
        return image_sprite_uv_result(request);
    }
    if string(request, "operation")? == "colour_map" {
        return image_colour_map_result(request);
    }
    if string(request, "operation")? == "argb_multiply" {
        return image_argb_multiply_result(request);
    }
    if string(request, "operation")? == "item_side_faces" {
        let rows = request["rows"].as_array().ok_or("rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            // Test glue: the texture from the local copy of the pinned jar.
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.data/minecraft/assets/jar")
                .join(string(row, "texture")?);
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let image = voxel_render::png::decode_rgba(&bytes).map_err(|e| format!("{e:?}"))?;
            let alpha: Vec<u8> = image.rgba.chunks(4).map(|pixel| pixel[3]).collect();
            let faces: Vec<String> = voxel_render::item_model::side_faces(
                &alpha,
                image.width as i32,
                image.height as i32,
            )
            .into_iter()
            .map(|(facing, x, y)| format!("{},{x:02},{y:02}", facing.name()))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "scanned",
                "width": image.width,
                "height": image.height,
                "faces": faces,
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "tint_constants" {
        return image_tint_constants_result(request);
    }
    if string(request, "operation")? != "png_decode" {
        return Err("unknown image operation".into());
    }
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
        match voxel_render::decode_rgba(&bytes) {
            Err(_) => out.push(json!({"label": label, "outcome": "decode_failed"})),
            Ok(image) => {
                let mut encoded = String::with_capacity(image.rgba.len() * 2);
                for byte in &image.rgba {
                    encoded.push_str(&format!("{byte:02x}"));
                }
                out.push(json!({
                    "label": label,
                    "outcome": "decoded",
                    "width": image.width,
                    "height": image.height,
                    "rgba": encoded,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn atlas_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? != "stitch" {
        return Err("unknown atlas operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("atlas rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut stitcher = voxel_render::Stitcher::new(
            i32_field(row, "max_width")?,
            i32_field(row, "max_height")?,
            i32_field(row, "mip_level")?,
            i32_field(row, "anisotropy_bit")?,
        );
        let sprites = row["sprites"]
            .as_array()
            .ok_or("sprites must be an array")?;
        for sprite in sprites {
            let name = Identifier::parse(string(sprite, "name")?)
                .map_err(|error| format!("sprite name: {error:?}"))?;
            stitcher.register_sprite(voxel_render::SpriteEntry {
                width: i32_field(sprite, "width")?,
                height: i32_field(sprite, "height")?,
                name,
            });
        }
        let label = string(row, "label")?;
        match stitcher.stitch() {
            Err(_) => out.push(json!({"label": label, "outcome": "stitch_failed"})),
            Ok(()) => {
                let placements: Vec<Value> = stitcher
                    .placements()
                    .into_iter()
                    .map(|placement| {
                        json!({
                            "name": placement.name.as_string(),
                            "x": placement.x,
                            "y": placement.y,
                            "padding": placement.padding,
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "stitched",
                    "width": stitcher.width(),
                    "height": stitcher.height(),
                    "placements": placements,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn window_result(request: &Value) -> Result<Value, String> {
    match string(request, "operation")? {
        "base_size" => Ok(json!({
            "base_width": window::BASE_WIDTH,
            "base_height": window::BASE_HEIGHT,
        })),
        "min_size" => {
            let rows = request["rows"]
                .as_array()
                .ok_or("window rows must be an array")?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                out.push(json!({
                    "label": string(row, "label")?,
                    "allowed": window::allowed_window_min_size(i32_field(row, "requested")?),
                }));
            }
            Ok(json!({"rows": out}))
        }
        "gui_scale" => {
            let rows = request["rows"]
                .as_array()
                .ok_or("window rows must be an array")?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let mut scale = window::WindowScale::new(
                    i32_field(row, "framebuffer_width")?,
                    i32_field(row, "framebuffer_height")?,
                );
                let calculated = scale.calculate_scale(
                    i32_field(row, "max_scale")?,
                    row["enforce_unicode"]
                        .as_bool()
                        .ok_or("enforce_unicode must be a bool")?,
                );
                scale.set_gui_scale(calculated);
                out.push(json!({
                    "label": string(row, "label")?,
                    "calculated_scale": calculated,
                    "gui_scale": scale.gui_scale(),
                    "gui_scaled_width": scale.gui_scaled_width(),
                    "gui_scaled_height": scale.gui_scaled_height(),
                }));
            }
            Ok(json!({"rows": out}))
        }
        _ => Err("unknown window operation".into()),
    }
}

/// A block state named as the scenes name it: a block and its properties.
pub(super) fn named_state(block: &Value) -> Result<i32, String> {
    let name = string(block, "block")?;
    let definition =
        voxel_data::block::registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
    let mut chosen = std::collections::BTreeMap::new();
    if let Some(properties) = block["properties"].as_object() {
        for (key, value) in properties {
            chosen.insert(
                key.clone(),
                value
                    .as_str()
                    .ok_or("property values are strings")?
                    .to_owned(),
            );
        }
    }
    definition
        .state_id(&chosen)
        .map(|state| state as i32)
        .ok_or_else(|| format!("bad properties for {name}"))
}

/// The vanilla block tags, resolved from the pinned jar's data pack as the
/// server would send them. Test glue only: the client takes its tags from
/// the server. The files stay local under `.data`.
pub(super) fn vanilla_block_tags() -> Result<ClientboundUpdateTagsPacket, String> {
    vanilla_tags_of("block", "minecraft:block", &|name| {
        voxel_data::block::registry::BLOCKS
            .iter()
            .position(|block| block.name == name)
            .map(|index| index as i32)
    })
}

/// The item tags of the data pack, as a tag packet would carry them.
pub(super) fn vanilla_item_tags() -> Result<voxel_world::block::tags::ItemTags, String> {
    let packet = vanilla_tags_of("item", "minecraft:item", &voxel_game::item::id_of)?;
    voxel_world::block::tags::ItemTags::from_registries(&packet.registries)
        .ok_or_else(|| "no item tags".to_owned())
}
