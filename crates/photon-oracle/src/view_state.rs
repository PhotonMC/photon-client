//! What the fog and the lightmap read of the local player: the blend of an effect as it is ticked,
//! the night vision's flicker and the water vision.

use super::*;

/// One of the view state operations; none for an operation of another kind.
pub(super) fn view_state_result(operation: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match operation {
        "mob_effect_blend" => mob_effect_blend_result(request),
        "view_scales" => view_scales_result(request),
        "camera_spin" => camera_spin_result(request),
        "item_activation_pose" => item_activation_pose_result(request),
        "end_sky_buffer" => Ok(end_sky_buffer_result()),
        "tick_rate_script" => tick_rate_script_result(request),
        "end_flash_script" => end_flash_script_result(request),
        "end_flash_pose" => end_flash_pose_result(request),
        _ => return None,
    })
}

/// `end_flash_pose`: the matrix the End flash's quad is drawn through, for angles.
fn end_flash_pose_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let pose = photon_render::sky::end_flash_pose(float_of(row, "x")?, float_of(row, "y")?);
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({"label": row["label"], "matrix": matrix}));
    }
    Ok(json!({"rows": rows}))
}

/// `end_flash_script`: an `EndFlashState` ticked through clock times, described after each tick.
fn end_flash_script_result(request: &Value) -> Result<Value, String> {
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let partial = float_of(row, "partial")?;
        let mut state = photon_world::end_flash::EndFlashState::default();
        let mut states = Vec::new();
        for clock in row["clocks"].as_array().ok_or("clocks")? {
            state.tick(clock.as_i64().ok_or("clock")?);
            states.push(json!({
                "intensity": bits(state.intensity(partial)),
                "intensity_now": bits(state.intensity(1.0)),
                "intensity_old": bits(state.intensity(0.0)),
                "x": bits(state.x_angle()), "y": bits(state.y_angle()),
                "started": state.flash_started_this_tick(),
            }));
        }
        rows.push(json!({"label": row["label"], "states": states}));
    }
    Ok(json!({"rows": rows}))
}

/// `tick_rate_script`: a `TickRateManager` taken through a script of calls, its state after each.
fn tick_rate_script_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let mut manager = photon_world::tick_rate::TickRateManager::default();
        let mut states = Vec::new();
        for step in row["steps"].as_array().ok_or("steps")? {
            let value = || step["value"].as_str().ok_or("value");
            match step["call"].as_str().ok_or("call")? {
                "rate" => manager.set_tick_rate(f32::from_bits(
                    u32::from_str_radix(value()?, 16).map_err(|error| error.to_string())?,
                )),
                "frozen" => manager.set_frozen(step["value"].as_i64() != Some(0)),
                "steps" => {
                    manager.set_frozen_ticks_to_run(step["value"].as_i64().ok_or("value")? as i32);
                }
                "tick" => manager.tick(),
                other => return Err(format!("unknown call {other}")),
            }
            states.push(json!({
                "runs": manager.runs_normally(), "frozen": manager.is_frozen(),
                "steps": manager.frozen_ticks_to_run(),
                "rate": format!("{:08x}", manager.tick_rate().to_bits()),
            }));
        }
        rows.push(json!({"label": row["label"], "states": states}));
    }
    Ok(json!({"rows": rows}))
}

/// `end_sky_buffer`: the End sky's twenty-four corners, quad by quad, as the vertex buffer holds
/// them (the first, second, third and last corner of each quad, in the triangles' order).
fn end_sky_buffer_result() -> Value {
    let vertices = photon_render::sky::end_sky_vertices();
    // Each quad's triangles are 0 1 2 and 0 2 3: its corners are the first three vertices and the
    // last of the six.
    let rows: Vec<Value> = vertices
        .chunks(6)
        .flat_map(|quad| [quad[0], quad[1], quad[2], quad[5]])
        .map(|vertex| {
            json!({
                "position": vertex.position.map(|value| format!("{:08x}", value.to_bits())),
                "uv": vertex.uv.map(|value| format!("{:08x}", value.to_bits())),
                "colour": vertex.colour.map(|channel| (channel * 255.0).round() as i64),
            })
        })
        .collect();
    json!({"rows": rows})
}

/// `item_activation_pose`: the pose the totem's animation draws the item at.
fn item_activation_pose_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(row[key].as_str().ok_or(key.to_owned())?, 16)
                    .map_err(|error| error.to_string())?,
            ))
        };
        let pose = photon_render::item_activation::activation_pose(
            i32_field(row, "ticks")?,
            float("partial")?,
            float("aspect")?,
            (float("off_x")?, float("off_y")?),
        );
        let matrix: Vec<String> = pose
            .pose
            .m
            .iter()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({"label": row["label"], "matrix": matrix}));
    }
    Ok(json!({"rows": rows}))
}

/// `camera_spin`: the matrix the spinning effect multiplies onto the projection, after the ticks
/// that sum its time and speed.
fn camera_spin_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let float = |value: &Value, key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(value[key].as_str().ok_or(key.to_owned())?, 16)
                    .map_err(|error| error.to_string())?,
            ))
        };
        let mut spinning = photon_render::camera::Spinning::default();
        for tick in row["ticks"].as_array().ok_or("ticks")? {
            spinning.tick(float(tick, "portal")?, float(tick, "nausea")?);
        }
        let matrix: Vec<String> = spinning
            .matrix(
                float(row, "portal")?,
                float(row, "nausea")?,
                float(row, "scale")?,
                float(row, "partial")?,
            )
            .iter()
            .flatten()
            .flatten()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect();
        rows.push(json!({"label": row["label"], "matrix": matrix}));
    }
    Ok(json!({"rows": rows}))
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

/// A float of a row, written as its bits.
fn float_of(value: &Value, key: &str) -> Result<f32, String> {
    let text = value[key].as_str().ok_or_else(|| key.to_owned())?;
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

/// What the fog reads of the camera and its entity, for a row.
fn fog_view(row: &Value, partial: f32) -> Result<photon_render::sky::FogView, String> {
    use photon_render::sky::{FogType, FogView};
    let fog_type = match string(row, "fog_type")? {
        "water" => FogType::Water,
        "lava" => FogType::Lava,
        "powder_snow" => FogType::PowderSnow,
        _ => FogType::Atmospheric,
    };
    let height = u64::from_str_radix(string(row, "y")?, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())?;
    let mut view = FogView {
        fog_type,
        spectator: row["spectator"].as_bool().ok_or("spectator")?,
        fire_resistance: false,
        blindness: None,
        darkness: None,
        night_vision: None,
        water_vision: None,
        render_distance: i32_field(row, "render_distance")?,
        darken_world: float_of(row, "darken_world")?,
        // The overworld's, in a level that is not flat.
        void_darkness: (32.0, -64, height),
        boss_fog: row["boss_fog"].as_bool().ok_or("boss_fog")?,
    };
    add_effects(&mut view, row, partial)?;
    if row["local"].as_bool().ok_or("local")? {
        view.water_vision = Some(photon_world::local_player::water_vision(
            i32_field(row, "water_vision_time")?,
            fog_type == FogType::Water,
        ));
    }
    Ok(view)
}

/// What the atmospheric and the water environments make of a row's attributes.
fn fog_environments(
    row: &Value,
) -> Result<
    (
        photon_render::sky::AtmosphericFog,
        photon_render::sky::WaterFog,
    ),
    String,
> {
    use photon_render::sky::{AtmosphereAttributes, AtmosphericFog, WaterFog, atmospheric_colour};
    let attributes = &row["attributes"];
    let colour_of = |key: &str| i32_field(attributes, key).map(|colour| colour as u32);
    let render_distance = i32_field(row, "render_distance")?;
    let colour = atmospheric_colour(
        &AtmosphereAttributes {
            fog_colour: colour_of("fog_color")?,
            sunrise: colour_of("sunrise")?,
            sky_colour: colour_of("sky_color")?,
            sun_angle: float_of(attributes, "sun_angle")?,
            sky_fog_end: float_of(attributes, "sky_fog_end")?,
        },
        (float_of(row, "forward_x")?, render_distance),
        (0.0, 0.0),
    );
    // The options' cloud range is 128 chunks.
    let atmosphere = AtmosphericFog::of(
        colour,
        (
            float_of(attributes, "fog_start")?,
            float_of(attributes, "fog_end")?,
            float_of(attributes, "sky_fog_end")?,
            float_of(attributes, "cloud_fog_end")?,
        ),
        (0.0, render_distance, 128),
    );
    let water = WaterFog {
        colour: colour_of("water_color")?,
        start: float_of(attributes, "water_start")?,
        end: float_of(attributes, "water_end")?,
    };
    Ok((atmosphere, water))
}

/// `fog_data`: the colour and distances the fog is drawn with, for a camera in a fluid or the air
/// with some effects, in the overworld with no weather and the options' defaults.
pub(super) fn fog_data_result(request: &Value) -> Result<Value, String> {
    let bits = |values: &[f32]| -> Vec<String> {
        values
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect()
    };
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let view = fog_view(row, float_of(row, "partial")?)?;
        let (atmosphere, water) = fog_environments(row)?;
        let fog = photon_render::sky::fog_data(&view, &atmosphere, &water);
        rows.push(json!({
            "label": row["label"],
            "colour": bits(&fog.colour),
            "distances": bits(&[
                fog.environmental_start,
                fog.environmental_end,
                fog.render_distance_start,
                fog.render_distance_end,
                fog.sky_end,
                fog.cloud_end,
            ]),
        }));
    }
    Ok(json!({"rows": rows}))
}

/// The effects of a fog row, ticked as a client does, as what the fog reads of them.
fn add_effects(
    view: &mut photon_render::sky::FogView,
    row: &Value,
    partial: f32,
) -> Result<(), String> {
    for described in row["effects"].as_array().ok_or("effects")? {
        let name = string(described, "name")?;
        let id = photon_data::item::mob_effects::MOB_EFFECTS
            .iter()
            .position(|effect| effect.name == name)
            .ok_or("no such effect")?;
        let mut effect = photon_world::ClientMobEffectState::new(
            id as i32,
            0,
            i32_field(described, "duration")?,
            described["blend"].as_bool().ok_or("blend")?,
        );
        for _ in 0..i32_field(described, "ticks")? {
            effect.tick_client();
        }
        match name {
            "minecraft:fire_resistance" => view.fire_resistance = true,
            "minecraft:blindness" => view.blindness = Some(effect.duration_ticks()),
            "minecraft:darkness" => view.darkness = Some(effect.blend_factor(partial)),
            "minecraft:night_vision" => {
                view.night_vision = Some(effect.night_vision_scale(partial));
            }
            _ => {}
        }
    }
    Ok(())
}

/// `lightmap_data`: the lightmap extractor's night vision, darkness and brightness for the local
/// player with some effects, in the options' defaults.
pub(super) fn lightmap_data_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("rows")?
        .iter()
        .map(lightmap_row)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({"rows": rows}))
}

/// The lightmap attributes a row carries.
fn lightmap_attributes(row: &Value) -> Result<photon_render::sky::LightmapInputs, String> {
    let colour = |key: &str| -> Result<[f32; 3], String> {
        let packed = row["attributes"][key]
            .as_i64()
            .ok_or_else(|| key.to_owned())? as u32;
        Ok([16, 8, 0].map(|shift| ((packed >> shift) & 255) as f32 / 255.0))
    };
    Ok(photon_render::sky::LightmapInputs {
        sky_colour: colour("sky_light_color")?,
        sky_factor: float_of(&row["attributes"], "sky_light_factor")?,
        block_tint: colour("block_light_tint")?,
        ambient: colour("ambient_light_color")?,
        night_vision_colour: colour("night_vision_color")?,
    })
}

/// One row of `lightmap_data`.
fn lightmap_row(row: &Value) -> Result<Value, String> {
    let bits = |values: &[f32]| -> Vec<String> {
        values
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect()
    };
    let partial = float_of(row, "partial")?;
    let mut view = effects_only_fog_view();
    add_effects(&mut view, row, partial)?;
    let water_vision = photon_world::local_player::water_vision(
        i32_field(row, "water_vision_time")?,
        row["eyes_in_water"].as_bool().ok_or("eyes")?,
    );
    let conduit = row["effects"]
        .as_array()
        .ok_or("effects")?
        .iter()
        .any(|effect| effect["name"] == "minecraft:conduit_power");
    let intensity =
        photon_render::sky::night_vision_intensity(view.night_vision, conduit, water_vision);
    let inputs = lightmap_attributes(row)?;
    let effects = inputs.effects(
        &photon_render::sky::LightmapView {
            darkness_blend: view.darkness.unwrap_or(0.0),
            darkness_scale: row["darkness_scale"].as_f64().unwrap_or(1.0) as f32,
            brightness_option: row["gamma"].as_f64().unwrap_or(0.5) as f32,
            tick_count: i32_field(row, "tick_count")?,
            night_vision: intensity,
            boss_darkening: photon_render::camera::BossDarkening {
                old: float_of(row, "boss_darkening_old")?,
                now: float_of(row, "boss_darkening")?,
            }
            .at(partial),
            end_flash_boost: end_flash_boost_of(row, partial)?,
        },
        partial,
    );
    Ok(json!({
        "label": row["label"],
        "night_vision_intensity": bits(&[intensity]),
        "night_vision_colour": bits(&inputs.night_vision_colour),
        "ambient": bits(&effects.base),
        "brightness": bits(&[effects.brightness]),
        "darkness": bits(&[effects.darkness]),
        "boss_darkening": bits(&[effects.boss_darkening]),
        "sky_factor": bits(&[effects.sky_factor]),
    }))
}

/// What the End's flash adds to a lightmap row's sky factor: the state ticked through the row's
/// clock times, if it has any, and the options the row sets.
fn end_flash_boost_of(row: &Value, partial: f32) -> Result<f32, String> {
    let intensity = match row.get("flash_clocks").and_then(Value::as_array) {
        Some(clocks) => {
            let mut state = photon_world::end_flash::EndFlashState::default();
            for clock in clocks {
                state.tick(clock.as_i64().ok_or("clock")?);
            }
            Some(state.intensity(partial))
        }
        None => None,
    };
    Ok(photon_render::sky::end_flash_boost(
        intensity,
        row["hide_flash"].as_bool().ok_or("hide_flash")?,
        row["boss_fog"].as_bool().ok_or("boss_fog")?,
    ))
}

/// A fog view that holds only the effects a row adds, for a camera in the air.
fn effects_only_fog_view() -> photon_render::sky::FogView {
    photon_render::sky::FogView {
        fog_type: photon_render::sky::FogType::Atmospheric,
        spectator: false,
        fire_resistance: false,
        blindness: None,
        darkness: None,
        night_vision: None,
        water_vision: None,
        render_distance: 12,
        darken_world: 0.0,
        void_darkness: (32.0, -64, 80.0),
        boss_fog: false,
    }
}

/// The sounds whose pitch comes from a random source the scene does not seed in Java, so that
/// only their volume is compared: a player's swim, splash, hurt and death, and the portal's
/// trigger.
pub(crate) fn has_random_pitch(event: &str) -> bool {
    event == "minecraft:entity.player.swim"
        || event.starts_with("minecraft:entity.player.splash")
        || event.starts_with("minecraft:entity.player.hurt")
        || event == "minecraft:entity.player.death"
        || event == "minecraft:block.portal.trigger"
}
