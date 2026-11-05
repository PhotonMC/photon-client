use super::*;
use photon_render::render_types::PipelineState;

/// `RenderPipelines.ARMOR_CUTOUT_NO_CULL_GLINT`: since 26.3 the glint is a shader define of the
/// pipeline that draws the armor itself, not a pass over it: one draw, with no blend of its own,
/// cut out below a tenth of alpha, on both faces.
const ARMOR_GLINT: PipelineState = PipelineState {
    location: "minecraft:pipeline/armor_cutout_no_cull_glint",
    flags: "GLINT,NO_OVERLAY,PER_FACE_LIGHTING",
    values: "ALPHA_CUTOUT=0.1,",
    blend: "NONE",
    depth_test: "GREATER_THAN_OR_EQUAL",
    depth_write: true,
    cull: false,
};

/// `RenderPipelines.ITEM_CUTOUT_GLINT`: the same for an item, which keeps its faces' culling.
const ITEM_GLINT: PipelineState = PipelineState {
    location: "minecraft:pipeline/item_cutout",
    flags: "GLINT",
    values: "ALPHA_CUTOUT=0.1,",
    blend: "NONE",
    depth_test: "GREATER_THAN_OR_EQUAL",
    depth_write: true,
    cull: true,
};

/// `glint_layers`: the pipeline the armor's glint is drawn with, its texture, and the matrix that
/// moves the texture.
pub(super) fn glint_layers_result(request: &Value) -> Result<Value, String> {
    use photon_render::render_types::{
        ARMOR_GLINT_SCALE, ARMOR_GLINT_TEXTURE, ITEM_GLINT_SCALE, ITEM_GLINT_TEXTURE,
        glint_texture_matrix,
    };
    let (pipeline, texture, scale) = if request["render_type"].as_str() == Some("item") {
        (ITEM_GLINT, ITEM_GLINT_TEXTURE, ITEM_GLINT_SCALE)
    } else {
        (ARMOR_GLINT, ARMOR_GLINT_TEXTURE, ARMOR_GLINT_SCALE)
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
        "pipeline": pipeline.location,
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
