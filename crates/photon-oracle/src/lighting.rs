use super::*;

fn direction(row: &Value, key: &str) -> Result<Direction, String> {
    Direction::from_name(string(row, key)?).ok_or_else(|| format!("{key}: unknown direction"))
}

fn bits32(value: &Value) -> Result<f32, String> {
    u32::from_str_radix(value.as_str().ok_or("float bits must be a string")?, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

/// The operations about how a scene is lit.
pub(super) fn lighting_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    match operation {
        "block_lighting" => Some(block_lighting_result(request)),
        "chunk_heightmaps" => Some(chunk_heightmaps_result(request)),
        "block_smooth_gate" => Some(block_smooth_gate_result(request)),
        "fluid_mesh" => Some(fluid_mesh_result(request)),
        "face_layer" => Some(face_layer_result(request)),
        "translucent_order" => Some(translucent_order_result(request)),
        "translucent_view" => Some(translucent_view_result(request)),
        "dimension_cardinal_light" => Some(dimension_cardinal_light_result(request)),
        "entity_shadows" => Some(entity_shadows_result(request)),
        _ => None,
    }
}

/// `block_lighting`: the colour and light each corner of a quad takes from
/// the blocks around it (`BlockModelLighter`), over a level lit as the
/// client lights it.
pub(super) fn block_lighting_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::light::{Lighter, face_shade, grey};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let mut world = movement_world(row)?;
        world.light_from_sources();
        world.run_light_updates();
        let lighter = Lighter {
            state: |p: [i32; 3]| world.block_state_at(p[0], p[1], p[2]),
            light: |p: [i32; 3]| {
                (
                    world.block_brightness(p[0], p[1], p[2]),
                    world.sky_brightness(p[0], p[1], p[2]),
                )
            },
        };
        let kind = match row["cardinal"].as_str() {
            Some("nether") => photon_world::CardinalLightType::Nether,
            _ => photon_world::CardinalLightType::Default,
        };
        let mut quads = Vec::new();
        for quad in row["quads"].as_array().ok_or("quads must be an array")? {
            let at = [
                i32_field(quad, "x")?,
                i32_field(quad, "y")?,
                i32_field(quad, "z")?,
            ];
            let state = world
                .block_state_at(at[0], at[1], at[2])
                .ok_or("the quad's block is not loaded")?;
            let face = direction(quad, "direction")?;
            let cull = match quad["cull"].as_str() {
                Some(_) => Some(direction(quad, "cull")?),
                None => None,
            };
            let mut corners = [[0.0_f32; 3]; 4];
            for (corner, values) in corners
                .iter_mut()
                .zip(quad["corners"].as_array().ok_or("corners")?)
            {
                for (axis, value) in corner.iter_mut().zip(values.as_array().ok_or("corner")?) {
                    *axis = bits32(value)?;
                }
            }
            let smooth = quad["smooth"].as_bool().ok_or("smooth must be a bool")?;
            let lit = lighter.light_quad(state, at, &corners, face, cull, smooth);
            let side = face_shade(
                face,
                quad["shade"].as_bool().ok_or("shade must be a bool")?,
                kind,
            );
            let vertices: Vec<Value> = (0..4)
                .map(|corner| json!({"grey": grey(&lit, corner, side), "light": lit.light[corner]}))
                .collect();
            quads.push(json!({"label": quad["label"], "vertices": vertices}));
        }
        out.push(json!({"label": row["label"], "quads": quads}));
    }
    Ok(json!({"rows": out}))
}

/// `block_smooth_gate`: whether a block with the given parts is lit smooth
/// (`ModelBlockRenderer.tesselateBlock`), for a block that may glow.
pub(super) fn block_smooth_gate_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let world = movement_world(row)?;
        let state = world
            .block_state_at(0, 64, 0)
            .ok_or("the block is not loaded")?;
        let parts = row["parts"].as_array().ok_or("parts must be an array")?;
        let smooth = photon_render::block::light::smooth_lighting(
            row["option"].as_bool().ok_or("option must be a bool")?,
            photon_world::block::state_or_air(state).light_emission,
            parts.iter().map(|part| part.as_bool().unwrap_or(false)),
        );
        out.push(json!({"label": row["label"], "smooth": smooth}));
    }
    Ok(json!({"rows": out}))
}

/// `dimension_cardinal_light`: the cardinal-light preset a serialized
/// dimension type decodes to, or that it does not decode. The row's
/// `cardinal` says what the field holds (`kind`, `value`) or that it is absent.
pub(super) fn dimension_cardinal_light_result(request: &Value) -> Result<Value, String> {
    let field = |key: &str, value: FieldValue| Field {
        key: key.to_owned(),
        value,
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let mut fields = vec![
            field("has_skylight", FieldValue::Byte(1)),
            field("has_ceiling", FieldValue::Byte(0)),
            field("has_ender_dragon_fight", FieldValue::Byte(0)),
            field("coordinate_scale", FieldValue::Double(1.0)),
            field("min_y", FieldValue::Int(-64)),
            field("height", FieldValue::Int(384)),
            field("logical_height", FieldValue::Int(384)),
            field("ambient_light", FieldValue::Float(0.0)),
        ];
        let cardinal = &row["cardinal"];
        match string(cardinal, "kind")? {
            "absent" => {}
            "string" => fields.push(field(
                "cardinal_light",
                FieldValue::Str(string(cardinal, "value")?.to_owned()),
            )),
            "int" => fields.push(field(
                "cardinal_light",
                FieldValue::Int(i32_field(cardinal, "value")?),
            )),
            "byte" => fields.push(field(
                "cardinal_light",
                FieldValue::Byte(
                    i8::try_from(i32_field(cardinal, "value")?).map_err(|e| e.to_string())?,
                ),
            )),
            "list" => fields.push(field("cardinal_light", FieldValue::List(Vec::new()))),
            "compound" => fields.push(field("cardinal_light", FieldValue::Compound(Vec::new()))),
            other => return Err(format!("unknown cardinal kind {other}")),
        }
        let entry = PackedRegistryEntry::new(
            Identifier::parse("photon:probe").map_err(|_| "identifier")?,
            Some(FieldValue::Compound(fields)),
        );
        let decoded = match entry.dimension_type_environment() {
            Ok(Some(environment)) => {
                json!({"decoded": true, "cardinal": environment.cardinal_light_type().as_str()})
            }
            _ => json!({"decoded": false}),
        };
        out.push(json!({"label": row["label"], "result": decoded}));
    }
    Ok(json!({"rows": out}))
}

/// The water colour of the scenes' biome, which `getAverageWaterColor` blends
/// to itself where the biome is the same all around.
const WATER_TINT: u32 = 0x3F_76E4;

/// `fluid_mesh`: the vertices of the fluid block at the row's position, as
/// `FluidRenderer` adds them, over a level lit as the client lights it.
pub(super) fn fluid_mesh_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::light::Lighter;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let mut world = movement_world(row)?;
        world.light_from_sources();
        world.run_light_updates();
        let lighter = Lighter {
            state: |p: [i32; 3]| world.block_state_at(p[0], p[1], p[2]),
            light: |p: [i32; 3]| {
                (
                    world.block_brightness(p[0], p[1], p[2]),
                    world.sky_brightness(p[0], p[1], p[2]),
                )
            },
        };
        let kind = match row["cardinal"].as_str() {
            Some("nether") => photon_world::CardinalLightType::Nether,
            _ => photon_world::CardinalLightType::Default,
        };
        let atlas = fluid_atlas(row);
        let at = [
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        ];
        let view = photon_render::block::world_mesh::FluidView {
            read: &|x, y, z| world.block_state_at(x, y, z),
            lighter: &lighter,
            atlas: &atlas,
            cardinal: kind,
            origin: [at[0] & !15, at[1] & !15, at[2] & !15],
            water_tint: WATER_TINT,
        };
        let mesh = photon_render::block::world_mesh::fluid_vertices(&view, at);
        let distinct = distinct_vertices(&mesh.vertices);
        out.push(json!({
            "label": row["label"],
            // Each face once, and once more where it is drawn again from behind.
            "quads": (mesh.indices.len() + mesh.translucent_indices.len()) / 6,
            "layer": if mesh.translucent_indices.is_empty() { "opaque" } else { "translucent" },
            "vertices": distinct.into_values().collect::<Vec<_>>(),
        }));
    }
    Ok(json!({"rows": out}))
}

/// An atlas that knows where the fluid sprites are, which the renderer asks
/// for by name; their coordinates are not compared. The row says which of
/// them hold `translucent` or `transparent` texels (the last part of the
/// sprite's name is the key); the rest are opaque.
fn fluid_atlas(row: &Value) -> photon_render::BuiltAtlas {
    let alpha =
        |name: &str| match row["sprites"][name.trim_start_matches("minecraft:block/")].as_str() {
            Some("translucent") => photon_render::Transparency {
                has_transparent: false,
                has_translucent: true,
            },
            Some("transparent") => photon_render::Transparency {
                has_transparent: true,
                has_translucent: false,
            },
            _ => photon_render::Transparency::default(),
        };
    let names = [
        "minecraft:block/water_still",
        "minecraft:block/water_flow",
        "minecraft:block/water_overlay",
        "minecraft:block/lava_still",
        "minecraft:block/lava_flow",
    ];
    photon_render::BuiltAtlas {
        alpha: names
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    photon_render::SpriteAlpha::uniform(alpha(name)),
                )
            })
            .collect(),
        width: 1,
        height: 1,
        levels: Vec::new(),
        sprites: [
            "minecraft:block/water_still",
            "minecraft:block/water_flow",
            "minecraft:block/water_overlay",
            "minecraft:block/lava_still",
            "minecraft:block/lava_flow",
        ]
        .into_iter()
        .map(|name| (name.to_owned(), [0.0; 4]))
        .collect(),
        animations: Vec::new(),
    }
}

/// The distinct vertices, each as place bits, colour and light, ordered by
/// those.
fn distinct_vertices(
    vertices: &[photon_render::scene::BlockVertex],
) -> std::collections::BTreeMap<String, Value> {
    vertices
        .iter()
        .map(|vertex| {
            let [r, g, b, a] = vertex.colour;
            let [block, sky] = vertex.light_levels();
            let described = json!({
                "position": vertex.position.map(|part| format!("{:x}", part.to_bits())),
                "color": (u32::from(a) << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
                "block_light": block as i64,
                "sky_light": sky as i64,
            });
            let key = format!(
                "{}:{}:{}:{}",
                described["position"], described["color"], described["block_light"], described["sky_light"],
            );
            (key, described)
        })
        .collect()
}

/// `face_layer`: the layer a face bakes to over a sprite whose pixels the row
/// gives as alphas (a strip of frames played in the row's order when it
/// names a `playback`), or that the face's rectangle falls outside them.
pub(super) fn face_layer_result(request: &Value) -> Result<Value, String> {
    use photon_render::{DecodedImage, Layer, SpriteAlpha};
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let (width, height) = (
            i32_field(row, "width")? as u32,
            i32_field(row, "height")? as u32,
        );
        let frame = (
            i32_field(row, "frame_width")? as u32,
            i32_field(row, "frame_height")? as u32,
        );
        let mut rgba = Vec::new();
        for alpha in row["alphas"].as_array().ok_or("alphas must be an array")? {
            rgba.extend_from_slice(&[0x33, 0x66, 0x99, alpha.as_u64().ok_or("alpha")? as u8]);
        }
        let image = DecodedImage {
            width,
            height,
            rgba,
        };
        let mut playback: Vec<u32> = Vec::new();
        for index in row["playback"].as_array().into_iter().flatten() {
            playback.push(index.as_u64().ok_or("frame index")? as u32);
        }
        let alpha = SpriteAlpha::measure(&image, frame, &playback);
        let mut uvs = [0.0_f32; 4];
        for (slot, uv) in row["uvs"]
            .as_array()
            .ok_or("uvs must be an array")?
            .iter()
            .enumerate()
            .take(4)
        {
            uvs[slot] = bits32(uv)?;
        }
        let force = row["force_translucent"]
            .as_bool()
            .ok_or("force_translucent")?;
        out.push(match alpha.face(force, uvs) {
            Some(transparency) => json!({
                "label": row["label"],
                "outcome": "layer",
                "layer": Layer::of(transparency).as_str(),
            }),
            None => json!({"label": row["label"], "outcome": "error"}),
        });
    }
    Ok(json!({"rows": out}))
}

fn bits_triple(value: &Value) -> Result<[f32; 3], String> {
    let parts = value.as_array().ok_or("a point must be an array")?;
    let mut out = [0.0_f32; 3];
    for (slot, part) in parts.iter().enumerate().take(3) {
        out[slot] = bits32(part)?;
    }
    Ok(out)
}

/// `translucent_order`: the order faces with the given middles are drawn in,
/// from a camera.
pub(super) fn translucent_order_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let centroids = row["centroids"]
            .as_array()
            .ok_or("centroids must be an array")?
            .iter()
            .map(bits_triple)
            .collect::<Result<Vec<_>, _>>()?;
        let order = photon_render::translucent::order(bits_triple(&row["camera"])?, &centroids);
        out.push(json!({"label": row["label"], "order": order}));
    }
    Ok(json!({"rows": out}))
}

/// `translucent_view`: which neighbour of a section the camera stands in.
pub(super) fn translucent_view_result(request: &Value) -> Result<Value, String> {
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows must be an array")? {
        let camera: Vec<f32> = row["camera"]
            .as_array()
            .ok_or("camera must be an array")?
            .iter()
            .map(|part| part.as_f64().unwrap_or_default() as f32)
            .collect();
        let section: Vec<f32> = row["section"]
            .as_array()
            .ok_or("section must be an array")?
            .iter()
            .map(|part| part.as_i64().unwrap_or_default() as f32 * 16.0 + 8.0)
            .collect();
        let view = photon_render::translucent::point_of_view(
            [camera[0], camera[1], camera[2]],
            [section[0], section[1], section[2]],
        );
        out.push(json!({"label": row["label"], "view": view, "aligned": view.contains(&0)}));
    }
    Ok(json!({"rows": out}))
}
