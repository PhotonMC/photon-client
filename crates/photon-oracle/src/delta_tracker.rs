//! The game clock that counts client ticks (`DeltaTracker.Timer`).

use super::*;

/// `delta_tracker_script`: the game clock in milliseconds taken through rate changes, the whole
/// ticks it makes and the part of a tick it leaves.
pub(super) fn delta_tracker_script_result(request: &Value) -> Result<Value, String> {
    use photon_world::tick_rate::{TickRateManager, delta_ticks, take_whole_ticks};
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut manager = TickRateManager::default();
        let (mut last_ms, mut delta, mut residual) = (0_i64, 0.0_f32, 0.0_f32);
        let mut states = Vec::new();
        for step in row["steps"].as_array().ok_or("steps")? {
            let mut state = serde_json::Map::new();
            match step["call"].as_str().ok_or("call")? {
                "rate" => manager.set_tick_rate(f32::from_bits(
                    u32::from_str_radix(step["value"].as_str().ok_or("value")?, 16)
                        .map_err(|error| error.to_string())?,
                )),
                "frozen" => manager.set_frozen(step["value"].as_i64() != Some(0)),
                "tick" => manager.tick(),
                "advance" => {
                    let now = step["value"].as_i64().ok_or("value")?;
                    let target = if manager.runs_normally() {
                        50.0_f32.max(manager.milliseconds_per_tick())
                    } else {
                        50.0
                    };
                    delta = delta_ticks(now - last_ms, target);
                    last_ms = now;
                    residual += delta;
                    state.insert("ticks".into(), take_whole_ticks(&mut residual).into());
                }
                other => return Err(format!("unknown call {other}")),
            }
            state.insert("delta".into(), format!("{:08x}", delta.to_bits()).into());
            state.insert(
                "residual".into(),
                format!("{:08x}", residual.to_bits()).into(),
            );
            states.push(Value::Object(state));
        }
        rows.push(json!({"label": row["label"], "states": states}));
    }
    Ok(json!({"rows": rows}))
}
