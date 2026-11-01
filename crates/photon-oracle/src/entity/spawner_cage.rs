//! `spawner_cage_pose`: the turn and scale a spawner reads off its mob, and the pose it gives it.

use super::*;
use photon_render::entity::mob_model::SpawnerCage;
use photon_world::SpawnerDisplayPose;

pub(super) fn spawner_cage_pose_result(request: &Value) -> Result<Value, String> {
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let single = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let double = |row: &Value, key: &str| -> Result<f64, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u64::from_str_radix(text, 16)
            .map(f64::from_bits)
            .map_err(|error| error.to_string())
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let display = SpawnerDisplayPose::of(
            (0, 0, 0),
            (double(row, "o_spin")?, double(row, "spin")?),
            single(row, "partial")?,
            (single(row, "width")?, single(row, "height")?),
        );
        let pose = SpawnerCage {
            origin: [0.0; 3],
            spin: display.spin,
            scale: display.scale,
        }
        .pose();
        // Column by column, as `Matrix4f.get` writes them.
        let described: Vec<String> = (0..4)
            .flat_map(|column| (0..4).map(move |row| (column, row)))
            .map(|(column, row)| hex(pose.pose.m[column][row]))
            .collect();
        out.push(json!({
            "label": row["label"],
            "spin": hex(display.spin),
            "scale": hex(display.scale),
            "pose": described,
        }));
    }
    Ok(json!({"rows": out}))
}
