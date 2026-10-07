use super::*;

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

pub(super) fn image_texture_metadata_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let parsed =
            voxel_core::json::parse(string(row, "json")?)
                .ok()
                .and_then(|value| match value {
                    voxel_core::json::JsonValue::Object(object) => {
                        voxel_render::TextureMetadata::parse(&object)
                    }
                    _ => None,
                });
        match parsed {
            None => out.push(json!({"label": label, "outcome": "rejected"})),
            Some(metadata) => out.push(json!({
                "label": label,
                "outcome": "parsed",
                "blur": metadata.blur,
                "clamp": metadata.clamp,
                "strategy": metadata.mipmap_strategy.name(),
                "bias": format!("{:08x}", metadata.alpha_cutoff_bias.to_bits()),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_sprite_uv_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let width = i32_field(row, "width")?;
        let height = i32_field(row, "height")?;
        let placement = voxel_render::SpritePlacement {
            name: voxel_core::identifier::Identifier::parse("minecraft:block/parity")
                .map_err(|_| "sprite name")?,
            x: i32_field(row, "x")?,
            y: i32_field(row, "y")?,
            padding: i32_field(row, "padding")?,
        };
        let sprite = voxel_render::AtlasSprite::new(
            &placement,
            width,
            height,
            i32_field(row, "atlas_width")?,
            i32_field(row, "atlas_height")?,
        );
        let mut interpolated = Vec::new();
        for offset in row["offsets"]
            .as_array()
            .ok_or("offsets must be an array")?
        {
            let bits = u32::from_str_radix(offset.as_str().ok_or("offset must be a string")?, 16)
                .map_err(|error| format!("offset: {error}"))?;
            let offset = f32::from_bits(bits);
            interpolated.push(json!({
                "u": format!("{:08x}", sprite.u(offset).to_bits()),
                "v": format!("{:08x}", sprite.v(offset).to_bits()),
            }));
        }
        out.push(json!({
            "label": label,
            "outcome": "placed",
            "x": sprite.x,
            "y": sprite.y,
            "u0": format!("{:08x}", sprite.u0.to_bits()),
            "u1": format!("{:08x}", sprite.u1.to_bits()),
            "v0": format!("{:08x}", sprite.v0.to_bits()),
            "v1": format!("{:08x}", sprite.v1.to_bits()),
            "interpolated": interpolated,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_colour_map_result(request: &Value) -> Result<Value, String> {
    let bytes = hex_bytes(string(request, "colour_map")?)?;
    let map = voxel_render::decode_rgba(&bytes).map_err(|_| "colour map did not decode")?;
    // The reference reads the map as packed colours, not as bytes.
    let pixels: Vec<u32> = map
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|texel| {
            (u32::from(texel[3]) << 24)
                | (u32::from(texel[0]) << 16)
                | (u32::from(texel[1]) << 8)
                | u32::from(texel[2])
        })
        .collect();
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let kind = match string(row, "kind")? {
            "grass" => voxel_render::ColourMap::Grass,
            "foliage" => voxel_render::ColourMap::Foliage,
            "dry_foliage" => voxel_render::ColourMap::DryFoliage,
            other => return Err(format!("unknown colour map kind: {other}")),
        };
        let read = |name: &str| -> Result<f64, String> {
            let bits = u64::from_str_radix(string(row, name)?, 16)
                .map_err(|error| format!("{name}: {error}"))?;
            Ok(f64::from_bits(bits))
        };
        match voxel_render::colour_map_lookup(kind, &pixels, read("temperature")?, read("rain")?) {
            Err(_) => out.push(json!({"label": label, "outcome": "out_of_bounds"})),
            Ok(colour) => out.push(json!({
                "label": label,
                "outcome": "resolved",
                "colour": format!("{colour:08x}"),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_argb_multiply_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let read = |name: &str| -> Result<u32, String> {
            u32::from_str_radix(string(row, name)?, 16).map_err(|error| format!("{name}: {error}"))
        };
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "multiplied",
            "colour": format!("{:08x}", voxel_render::multiply(read("lhs")?, read("rhs")?)),
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_tint_constants_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "listed",
            "values": {
                "foliage_default": format!("{:08x}", voxel_render::ColourMap::Foliage.default_colour()),
                "foliage_evergreen": format!("{:08x}", voxel_render::constant_tints("minecraft:spruce_leaves")[0]),
                "foliage_birch": format!("{:08x}", voxel_render::constant_tints("minecraft:birch_leaves")[0]),
                "foliage_mangrove": format!("{:08x}", voxel_render::FOLIAGE_MANGROVE),
                "dry_foliage_default": format!("{:08x}", voxel_render::ColourMap::DryFoliage.default_colour()),
                "lily_pad_in_hand": format!("{:08x}", voxel_render::constant_tints("minecraft:lily_pad")[0]),
                "lily_pad_in_world": format!("{:08x}", voxel_render::LILY_PAD_IN_WORLD),
            },
        }));
    }
    Ok(json!({"rows": out}))
}
