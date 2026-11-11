use super::*;

/// `glint_layers`: the pipeline the armor's glint is drawn with, its texture, and the matrix that
/// moves the texture.
pub(super) fn glint_layers_result(request: &Value) -> Result<Value, String> {
    use photon_render::render_types::{
        ARMOR_GLINT_SCALE, ARMOR_GLINT_TEXTURE, ITEM_GLINT_SCALE, ITEM_GLINT_TEXTURE,
        armor_glint_pipeline, glint_texture_matrix, item_glint_pipeline,
    };
    let (pipeline, texture, scale) = if request["render_type"].as_str() == Some("item") {
        (item_glint_pipeline(), ITEM_GLINT_TEXTURE, ITEM_GLINT_SCALE)
    } else {
        (
            armor_glint_pipeline(),
            ARMOR_GLINT_TEXTURE,
            ARMOR_GLINT_SCALE,
        )
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
        "pipeline": pipeline.id(),
        "flags": pipeline.flags,
        "values": pipeline.values,
        "blend": pipeline.blend,
        "depth_test": pipeline.depth_test,
        "depth_write": pipeline.depth_write,
        "cull": pipeline.cull,
        "texture": format!("textures/{texture}.png"),
        "rows": rows,
    }))
}
