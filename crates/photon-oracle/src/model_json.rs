use super::*;

/// An element's rotation as the oracle reports it: `none`, or origin, rescale and matrix.
pub(super) fn rotation_json(rotation: Option<&photon_render::model::ElementRotation>) -> Value {
    let bits = |vector: [f32; 3]| {
        json!(
            vector
                .iter()
                .map(|axis| format!("{:08x}", axis.to_bits()))
                .collect::<Vec<_>>()
        )
    };
    let Some(rotation) = rotation else {
        return json!("none");
    };
    let mut matrix = Vec::with_capacity(16);
    for column in &rotation.transform {
        for value in column {
            matrix.push(json!(format!("{:08x}", value.to_bits())));
        }
    }
    json!({
        "origin": bits(rotation.origin),
        "rescale": rotation.rescale,
        "transform": matrix,
    })
}
