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
        "entity_shadows" => Some(entity_shadows_result(request)),
        _ => None,
    }
}

/// `block_lighting`: the colour and light each corner of a quad takes from
/// the blocks around it (`BlockModelLighter`), over a level lit as the
/// client lights it.
pub(super) fn block_lighting_result(request: &Value) -> Result<Value, String> {
    use photon_render::block::light::{Lighter, cardinal, grey};
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
            let side = if quad["shade"].as_bool().ok_or("shade must be a bool")? {
                cardinal(face, photon_world::CardinalLightType::Default)
            } else {
                cardinal(Direction::Up, photon_world::CardinalLightType::Default)
            };
            let vertices: Vec<Value> = (0..4)
                .map(|corner| json!({"grey": grey(&lit, corner, side), "light": lit.light[corner]}))
                .collect();
            quads.push(json!({"label": quad["label"], "vertices": vertices}));
        }
        out.push(json!({"label": row["label"], "quads": quads}));
    }
    Ok(json!({"rows": out}))
}
