//! `item_pickup`: an entity taken flies to a taker that moves each tick.

use super::*;
use voxel_world::particles::{ItemPickup, PickedUp, pickup_target};

/// Where the row's taker is at one of its ticks, halfway between its feet and its eyes.
fn target_of(at: &Value) -> Result<(f64, f64, f64), String> {
    let eye = u32::from_str_radix(string(at, "eye")?, 16)
        .map(f32::from_bits)
        .map_err(|error| format!("eye: {error}"))?;
    Ok(pickup_target(
        (parsed64(at, "x")?, parsed64(at, "y")?, parsed64(at, "z")?),
        eye,
    ))
}

pub(crate) fn item_pickup_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let targets = row["targets"]
            .as_array()
            .ok_or("targets must be an array")?;
        let from = (
            parsed64(row, "x")?,
            parsed64(row, "y")?,
            parsed64(row, "z")?,
        );
        let camera = (
            parsed64(row, "camera_x")?,
            parsed64(row, "camera_y")?,
            parsed64(row, "camera_z")?,
        );
        let first = targets.first().ok_or("targets must not be empty")?;
        let mut pickup = ItemPickup::new((PickedUp::Other, from), None, target_of(first)?);
        let partials = f32_bits(&row["partials"])?;
        let mut states = Vec::new();
        for (tick, at) in targets.iter().enumerate() {
            if tick > 0 {
                pickup.tick(target_of(at)?);
            }
            let drawn: Vec<Value> = partials
                .iter()
                .map(|partial| {
                    let (x, y, z) = pickup.position_at(*partial);
                    json!([
                        bits64(x - camera.0),
                        bits64(y - camera.1),
                        bits64(z - camera.2)
                    ])
                })
                .collect();
            states.push(json!({"removed": !pickup.is_alive(), "drawn": drawn}));
        }
        out.push(json!({"label": string(row, "label")?, "states": states}));
    }
    Ok(json!({"rows": out}))
}
