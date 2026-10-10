use super::*;

/// `screen_effect_quads`: the quads the screen effects lay over the view.
pub(super) fn screen_effect_quads_result(request: &Value) -> Result<Value, String> {
    use photon_render::screen_effect::{block_quad, fire_quads, water_quad};
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
        let kind = row["kind"].as_str().ok_or("kind")?;
        let vertices = match kind {
            "fire" | "block" => {
                let sprite = [bits("u0")?, bits("u1")?, bits("v0")?, bits("v1")?];
                if kind == "fire" {
                    fire_quads(sprite).to_vec()
                } else {
                    block_quad(sprite, i32_field(row, "color")?).to_vec()
                }
            }
            "water" => water_quad(bits("y_rot")?, bits("x_rot")?, bits("brightness")?).to_vec(),
            other => return Err(format!("no screen effect {other}")),
        };
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let vertices: Vec<String> = vertices
            .iter()
            .map(|vertex| {
                format!(
                    "{}{}{}{}{}{:08x}{:08x}{}",
                    hex(vertex.position[0]),
                    hex(vertex.position[1]),
                    hex(vertex.position[2]),
                    hex(vertex.uv[0]),
                    hex(vertex.uv[1]),
                    vertex.color as u32,
                    0,
                    "000000000000000000000000"
                )
            })
            .collect();
        out.push(json!({"label": row["label"], "vertices": vertices}));
    }
    Ok(json!({"rows": out}))
}
