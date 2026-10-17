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
        boss_fog: false,
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

/// One row of `lightmap_data`.
fn lightmap_row(row: &Value) -> Result<Value, String> {
    let bits = |values: &[f32]| -> Vec<String> {
        values
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect()
    };
    let colour = |key: &str| -> Result<[f32; 3], String> {
        let packed = row["attributes"][key]
            .as_i64()
            .ok_or_else(|| key.to_owned())? as u32;
        Ok([16, 8, 0].map(|shift| ((packed >> shift) & 255) as f32 / 255.0))
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
    let inputs = photon_render::sky::LightmapInputs {
        sky_colour: colour("sky_light_color")?,
        sky_factor: 0.0,
        block_tint: colour("block_light_tint")?,
        ambient: colour("ambient_light_color")?,
        night_vision_colour: colour("night_vision_color")?,
    };
    let effects = inputs.effects(
        &photon_render::sky::LightmapView {
            darkness_blend: view.darkness.unwrap_or(0.0),
            tick_count: i32_field(row, "tick_count")?,
            night_vision: intensity,
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
    }))
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
