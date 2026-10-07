use super::*;

/// The limb animation state, stepped and read between ticks.
pub(super) fn walk_animation_result(request: &Value) -> Result<Value, String> {
    use voxel_world::living::WalkAnimation;
    let bits = |value: &Value, key: &str| -> Result<f32, String> {
        let text = value[key]
            .as_str()
            .ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let hex = |value: f32| format!("{:08x}", value.to_bits());
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut state = WalkAnimation::default();
        let partials: Vec<f32> = row["partials"]
            .as_array()
            .ok_or("partials")?
            .iter()
            .map(|text| {
                u32::from_str_radix(text.as_str().unwrap_or_default(), 16)
                    .map(f32::from_bits)
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<_, _>>()?;
        let mut steps = Vec::new();
        for step in row["steps"].as_array().ok_or("steps")? {
            if step["stop"].as_bool().unwrap_or(false) {
                state.stop();
            } else {
                state.update(
                    bits(step, "target")?,
                    bits(step, "factor")?,
                    bits(step, "scale")?,
                );
            }
            steps.push(json!({
                "speed": hex(state.speed()),
                "position": hex(state.position()),
                "partials": partials
                    .iter()
                    .map(|partial| format!(
                        "{}{}",
                        hex(state.position_at(*partial)),
                        hex(state.speed_at(*partial)),
                    ))
                    .collect::<Vec<_>>(),
            }));
        }
        out.push(json!({"label": row["label"], "steps": steps}));
    }
    Ok(json!({"rows": out}))
}
