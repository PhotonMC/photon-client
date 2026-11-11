//! `bed_sleeping_position`: where a sleeper lies over each bed a row names.

use super::*;

pub(super) fn bed_sleeping_position_result(request: &Value) -> Result<Value, String> {
    let world = crate::ticks::movement_world(&json!({
        "blocks": request["blocks"],
        "chunk_radius": 1,
    }))?;
    let hex = |value: f64| format!("{:016x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let bed = photon_core::binary::BlockPos::new(
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        );
        let mut described = json!({"label": row["label"]});
        if let Some((x, y, z)) = photon_world::block::bed::sleeping_position(&world, bed) {
            described["position"] = json!([hex(x), hex(y), hex(z)]);
        }
        out.push(described);
    }
    Ok(json!({"rows": out}))
}
