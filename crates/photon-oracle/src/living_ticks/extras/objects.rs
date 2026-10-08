use super::super::*;

pub(crate) fn boat_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.boat().copied().unwrap_or_default();
    described["under_water"] = json!(state.is_under_water);
    described["hurt_time"] = json!(state.hurt_time);
    described["hurt_dir"] = json!(state.hurt_dir);
    described["damage"] = json!(hex(state.damage));
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
        described[format!("bubble_angle_{key}")] = json!(hex(state.bubble_angle(partial)));
        for side in 0..2 {
            described[format!("rowing_{side}_{key}")] = json!(hex(state.rowing_time(
                side,
                entity.boat_paddle_rows(side),
                partial
            )));
        }
    }
    Ok(())
}

pub(crate) fn arrow_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.arrow().copied().unwrap_or_default();
    described["shake_time"] = json!(state.shake_time());
    described["in_ground"] = json!(state.is_in_ground());
    described["tipped"] = json!(state.is_tipped());
    Ok(())
}

pub(crate) fn minecart_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
    world: &ClientWorld,
    row: &Value,
    tick: i32,
) -> Result<(), String> {
    let state = entity.minecart().copied().unwrap_or_default();
    described["hurt_time"] = json!(state.hurt_time);
    described["hurt_dir"] = json!(state.hurt_dir);
    described["damage"] = json!(hex(state.damage));
    described["display_block"] = json!(entity.minecart_display_block().map_or(0, i64::from));
    described["display_offset"] = json!(entity.minecart_display_offset());
    described["fuse"] = json!(state.fuse());
    let position = entity.position();
    let (y_rot, x_rot, _) = entity.rotation();
    for (key, value) in [("x", position.0), ("y", position.1), ("z", position.2)] {
        described[key] = json!(format!("{:016x}", value.to_bits()));
    }
    described["y_rot"] = json!(hex(y_rot));
    described["x_rot"] = json!(hex(x_rot));
    if let Some(track) = entity.minecart_track() {
        let motion = entity.motion();
        for (key, value) in [("x", motion.0), ("y", motion.1), ("z", motion.2)] {
            described[format!("motion_{key}")] = json!(format!("{:016x}", value.to_bits()));
        }
        described["playing"] = json!(track.is_playing());
        let mut samples = Vec::new();
        // Unseen ticks are not asked (a place asked for is kept for the next ask).
        if track.is_playing()
            && tick
                >= row
                    .get("sample_from")
                    .map_or(Ok(0), |_| i32_field(row, "sample_from"))?
        {
            for partial in [0.0_f32, 0.5] {
                let place = track.place(partial).ok_or("playing")?;
                samples.push(json!(format!(
                    "{:016x}{:016x}{:016x}{}{}",
                    place.position.0.to_bits(),
                    place.position.1.to_bits(),
                    place.position.2.to_bits(),
                    hex(place.x_rot),
                    hex(place.y_rot)
                )));
            }
        }
        described["lerp"] = json!(samples);
    }
    let mut probes = Vec::new();
    if tick + 1 == i32_field(row, "tick_count")? {
        for probe in row["rail_probes"].as_array().into_iter().flatten() {
            let parts = probe.as_array().ok_or("a probe is an array")?;
            let double = |index: usize| -> Result<f64, String> {
                u64::from_str_radix(parts[index].as_str().ok_or("a double")?, 16)
                    .map(f64::from_bits)
                    .map_err(|error| error.to_string())
            };
            let at = (double(0)?, double(1)?, double(2)?);
            let text = |place: Option<(f64, f64, f64)>| {
                place.map_or_else(
                    || "none".to_string(),
                    |place| {
                        format!(
                            "{:016x}{:016x}{:016x}",
                            place.0.to_bits(),
                            place.1.to_bits(),
                            place.2.to_bits()
                        )
                    },
                )
            };
            let half = f64::from(0.3_f32);
            probes.push(json!([
                text(world.minecart_rail_pos(at)),
                text(world.minecart_rail_pos_offset(at, half)),
                text(world.minecart_rail_pos_offset(at, -half)),
            ]));
        }
    }
    described["rail"] = json!(probes);
    Ok(())
}

pub(crate) fn display_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let (x, y, z) = entity.position();
    let (y_rot, x_rot, _) = entity.rotation();
    for (key, value) in [("x", x), ("y", y), ("z", z)] {
        described[key] = json!(format!("{:016x}", value.to_bits()));
    }
    described["y_rot"] = json!(hex(y_rot));
    described["x_rot"] = json!(hex(x_rot));
    let display = entity.display().ok_or("a display")?;
    if let Some(block) = display.block_state() {
        described["block_state"] = json!(block);
    }
    if let Some(context) = display.item_context() {
        described["item_context"] = json!(context);
        described["item_empty"] = json!(
            entity
                .item_stack()
                .is_none_or(|stack| stack.item_count() <= 0)
        );
    }
    let state = display.render_state();
    described["has_state"] = json!(state.is_some());
    let mut samples = Vec::new();
    if let Some(state) = state {
        for partial in [0.0_f32, 0.5] {
            let progress = display.interpolation_progress(entity.tick_count(), partial);
            let matrix = photon_render::objects::display::transformation_matrix(
                &state.transformation.get(progress),
            );
            let mut sample = hex(progress);
            for value in matrix {
                sample.push_str(&hex(value));
            }
            sample.push_str(&hex(state.shadow_radius.get(progress)));
            sample.push_str(&hex(state.shadow_strength.get(progress)));
            samples.push(json!(sample));
        }
        described["billboard"] = json!(state.billboard as i32);
        described["brightness"] = json!(state.brightness_override);
        described["glow"] = json!(state.glow_color_override);
    }
    described["samples"] = json!(samples);
    if let Some((text, _)) = display.text_render_state() {
        let mut data = text.text.as_slice();
        let tag = photon_core::nbt::read_any_nbt(&mut data)
            .ok()
            .flatten()
            .and_then(|tag| photon_game::text::Component::from_nbt(&tag))
            .ok_or("a component")?;
        let words: String = photon_game::text::Lang::default()
            .runs(&tag, &photon_game::text::Style::default())
            .into_iter()
            .map(|run| run.text)
            .collect();
        described["text"] = json!(words);
        described["line_width"] = json!(text.line_width);
        described["style_flags"] = json!(i32::from(text.flags));
        let mut text_samples = Vec::new();
        for partial in [0.0_f32, 0.5] {
            let progress = display.interpolation_progress(entity.tick_count(), partial);
            text_samples.push(json!(format!(
                "{},{}",
                text.text_opacity.get(progress),
                text.background.get(progress)
            )));
        }
        described["text_samples"] = json!(text_samples);
    }
    described["render_at"] = json!(
        [0.0, 4095.99, 4096.0, 16383.99, 16384.0, 1.0e9]
            .map(|distance| display.should_render_at_sqr_distance(distance, 1.0))
    );
    Ok(())
}

pub(crate) fn end_crystal_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.end_crystal().copied();
    described["beam_target"] = json!(state.and_then(|state| state.beam_target()).map_or_else(
        || "none".to_string(),
        |target| format!("{},{},{}", target.x, target.y, target.z)
    ));
    described["shows_bottom"] = json!(state.is_none_or(|state| state.shows_bottom()));
    Ok(())
}

pub(crate) fn ender_dragon_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
    row: &Value,
    tick: i32,
) -> Result<(), String> {
    let state = entity
        .ender_dragon()
        .cloned()
        .unwrap_or_else(|| photon_world::mobs::ender_dragon::EnderDragonState::new(0));
    // Which crystal and when it is found rests on the entity's own random
    // source (a tick in ten): only the last tick says, by then it is found.
    described["nearest_crystal"] = json!(if tick + 1 == i32_field(row, "tick_count")? {
        state.nearest_crystal().unwrap_or(0)
    } else {
        -1
    });
    described["flap_time"] = json!(hex(state.flap_time()));
    described["old_flap_time"] = json!(hex(state.old_flap_time()));
    let samples: Vec<Value> = (0..64)
        .map(|delay| {
            let sample = state.history.get(delay);
            json!({"y": format!("{:016x}", sample.y.to_bits()),
                    "y_rot": hex(sample.y_rot)})
        })
        .collect();
    described["history"] = json!(samples);
    described["y_rot"] = json!(hex(entity.rotation().0));
    described["dragon_death_time"] = json!(state.dragon_death_time());
    let parts: Vec<String> = state
        .parts()
        .iter()
        .map(|part| {
            format!(
                "{:016x}{:016x}{:016x}",
                part.position.0.to_bits(),
                part.position.1.to_bits(),
                part.position.2.to_bits()
            )
        })
        .collect();
    described["parts"] = json!(parts);
    Ok(())
}

pub(crate) fn wither_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.wither().copied().unwrap_or_default();
    described["invulnerable"] = json!(state.invulnerable_ticks());
    for i in 0..2 {
        described[format!("x_head_{i}")] = json!(hex(state.x_rot_heads()[i]));
        described[format!("y_head_{i}")] = json!(hex(state.y_rot_heads()[i]));
    }
    described["y_rot"] = json!(hex(entity.rotation().0));
    described["y_body_rot"] = json!(hex(entity.y_body_rot()));
    Ok(())
}
