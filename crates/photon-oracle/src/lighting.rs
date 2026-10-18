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
        "block_smooth_gate" => Some(block_smooth_gate_result(request)),
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
