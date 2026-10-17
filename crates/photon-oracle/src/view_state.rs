//! What the fog and the lightmap read of the local player: the blend of an effect as it is ticked,
//! the night vision's flicker and the water vision.

use super::*;

/// One of the view state operations; none for an operation of another kind.
pub(super) fn view_state_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match operation {
        "mob_effect_blend" => mob_effect_blend_result(request),
        "view_scales" => view_scales_result(request),
        _ => return None,
    })
}

/// `mob_effect_blend`: the duration and the blend's two factors after each tick of an effect.
fn mob_effect_blend_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let name = string(row, "effect")?;
        let id = photon_data::item::mob_effects::MOB_EFFECTS
            .iter()
            .position(|effect| effect.name == name)
            .ok_or("no such effect")?;
        let mut effect = photon_world::ClientMobEffectState::new(
            id as i32,
            i32_field(row, "amplifier")?,
            i32_field(row, "duration")?,
            row["blend"].as_bool().ok_or("blend")?,
        );
        let mut steps = Vec::new();
        for _ in 0..i32_field(row, "ticks")? {
            effect.tick_client();
            steps.push(json!({
                "duration": effect.duration_ticks(),
                "factor": format!("{:08x}", effect.blend_state().0.to_bits()),
                "previous": format!("{:08x}", effect.blend_state().1.to_bits()),
                "ends_within_200": effect.ends_within(200),
            }));
        }
        rows.push(json!({"label": row["label"], "steps": steps}));
    }
    Ok(json!({"rows": rows}))
}

/// `view_scales`: the night vision's flicker and the water vision, as bits.
fn view_scales_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let scale = if string(row, "kind")? == "night_vision" {
            let night_vision = photon_data::item::mob_effects::MOB_EFFECTS
                .iter()
                .position(|effect| effect.name == "minecraft:night_vision")
                .ok_or("no night vision")?;
            let effect = photon_world::ClientMobEffectState::new(
                night_vision as i32,
                0,
                i32_field(row, "duration")?,
                false,
            );
            let partial = u32::from_str_radix(string(row, "partial")?, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())?;
            effect.night_vision_scale(partial)
        } else {
            photon_world::local_player::water_vision(
                i32_field(row, "time")?,
                row["eyes_in_water"].as_bool().ok_or("eyes")?,
            )
        };
        rows.push(json!({"label": row["label"], "scale": format!("{:08x}", scale.to_bits())}));
    }
    Ok(json!({"rows": rows}))
}
