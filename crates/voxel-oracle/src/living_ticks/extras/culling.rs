use super::super::*;

/// The view scales and squared distances `MovementObjectExtras.extraCull` asks at.
const VIEW_SCALES: [(&str, f64); 2] = [("1.0", 1.0), ("2.5", 2.5)];
const DISTANCES: [f64; 20] = [
    0.0, 12.0, 12.5, 400.0, 1000.0, 1100.0, 2500.0, 4000.0, 4200.0, 8000.0, 16000.0, 16500.0,
    40000.0, 100000.0, 400000.0, 1.0e6, 4.0e6, 1.0e7, 1.0e8, 1.0e9,
];

/// Whether the entity is near enough to be drawn, from a camera along x at each distance, and
/// the box its renderer tests against the view.
pub(crate) fn cull_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let culling = entity.culling_box();
    described["culled"] = json!(culling.is_some());
    if let Some(aabb) = culling {
        described["culling_box"] = json!(
            [
                aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z
            ]
            .map(|value| format!("{:016x}", value.to_bits()))
            .join(",")
        );
    }
    let (x, y, z) = entity.position();
    for (key, scale) in VIEW_SCALES {
        let near: String = DISTANCES
            .iter()
            .map(|distance| {
                if entity.should_render((x + distance.sqrt(), y, z), scale) {
                    '1'
                } else {
                    '0'
                }
            })
            .collect();
        described[format!("render_at_{key}")] = json!(near);
    }
    Ok(())
}
