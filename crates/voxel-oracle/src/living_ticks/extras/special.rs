use super::super::*;

pub(crate) fn bat_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["resting"] = json!(entity.is_bat_resting());
    let (_, y, _) = entity.position();
    described["y"] = json!(format!("{:016x}", y.to_bits()));
    let (_, motion_y, _) = entity.motion();
    described["motion_y"] = json!(format!("{:016x}", motion_y.to_bits()));
    let state = entity.bat().copied().unwrap_or_default();
    let age = entity.tick_count() as f32;
    for (name, animation) in [("fly", state.fly), ("rest", state.rest)] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    Ok(())
}

pub(crate) fn breeze_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.breeze().copied().unwrap_or_default();
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("idle", state.idle),
        ("shoot", state.shoot),
        ("slide", state.slide),
        ("slide_back", state.slide_back),
        ("inhale", state.inhale),
        ("long_jump", state.long_jump),
    ] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    Ok(())
}

pub(crate) fn sniffer_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.sniffer().copied().unwrap_or_default();
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("digging", state.digging),
        ("sniffing", state.sniffing),
        ("rising", state.rising),
        ("happy", state.feeling_happy),
        ("scenting", state.scenting),
    ] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    described["searching"] = json!(state.is_searching());
    Ok(())
}

pub(crate) fn armadillo_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.armadillo().copied().unwrap_or_default();
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("roll_out", state.roll_out),
        ("roll_up", state.roll_up),
        ("peek", state.peek),
    ] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    described["hiding"] = json!(state.should_hide_in_shell());
    described["scared"] = json!(state.is_scared());
    described["head_yaw"] = json!(hex(entity.y_head_rot()));
    Ok(())
}

pub(crate) fn creaking_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity
        .creaking()
        .cloned()
        .unwrap_or_else(|| voxel_world::mobs::creaking::CreakingState::new(0));
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("attack", state.attack),
        ("invulnerable", state.invulnerable),
        ("death", state.death),
    ] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    described["can_move"] = json!(state.can_move());
    described["active"] = json!(state.is_active());
    described["tearing_down"] = json!(state.is_tearing_down());
    described["heart_bound"] = json!(state.is_heart_bound());
    Ok(())
}

pub(crate) fn warden_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.warden().copied().unwrap_or_default();
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("roar", state.roar),
        ("sniff", state.sniff),
        ("emerge", state.emerge),
        ("digging", state.digging),
        ("attack", state.attack),
        ("sonic_boom", state.sonic_boom),
    ] {
        described[format!("{name}_started")] = json!(animation.is_started());
        described[format!("{name}_millis")] = json!(if animation.is_started() {
            animation.time_in_millis(age)
        } else {
            0
        });
    }
    described["tendril_old"] = json!(hex(state.tendril_animation(0.0)));
    described["tendril"] = json!(hex(state.tendril_animation(1.0)));
    described["heart_old"] = json!(hex(state.heart_animation(0.0)));
    described["heart"] = json!(hex(state.heart_animation(1.0)));
    Ok(())
}

pub(crate) fn parrot_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.parrot().copied().unwrap_or_default();
    described["variant"] = json!(entity.parrot_variant());
    let (_, motion_y, _) = entity.motion();
    described["motion_y"] = json!(format!("{:016x}", motion_y.to_bits()));
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
        described[format!("flap_angle_{key}")] = json!(hex(state.flap_angle(partial)));
    }
    Ok(())
}

pub(crate) fn allay_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let state = entity.allay().copied().unwrap_or_default();
    described["dancing"] = json!(entity.is_allay_dancing());
    described["spinning"] = json!(state.is_spinning());
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
        described[format!("holding_{key}")] = json!(hex(state.holding_item_progress(partial)));
        described[format!("spin_{key}")] = json!(hex(state.spinning_progress(partial)));
    }
    Ok(())
}

pub(crate) fn shulker_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let shulker = entity.shulker();
    described["face"] = json!(entity.shulker_attach_face());
    described["color"] = json!(entity.shulker_color().map_or(16, i32::from));
    described["peek_half"] = json!(hex(shulker.peek(0.5)));
    described["peek_start"] = json!(hex(shulker.peek(0.0)));
    described["peek_end"] = json!(hex(shulker.peek(1.0)));
    described["y_rot"] = json!(hex(entity.rotation().0));
    described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
    described["y_body_rot_old"] = json!(hex(entity.previous_body_y_rotation()));
    described["y_head_rot"] = json!(hex(entity.rotation().2));
    let (x, y, z) = entity.position();
    let (old_x, old_y, old_z) = entity.previous_position();
    for (key, value) in [
        ("pos_x", x),
        ("pos_y", y),
        ("pos_z", z),
        ("old_x", old_x),
        ("old_y", old_y),
        ("old_z", old_z),
    ] {
        described[key] = json!(format!("{:016x}", value.to_bits()));
    }
    described["riding"] = json!(entity.vehicle_id().is_some());
    let area = entity.bounding_box();
    for (key, value) in [
        ("box_min_x", area.min_x),
        ("box_min_y", area.min_y),
        ("box_min_z", area.min_z),
        ("box_max_x", area.max_x),
        ("box_max_y", area.max_y),
        ("box_max_z", area.max_z),
    ] {
        described[key] = json!(format!("{:016x}", value.to_bits()));
    }
    for (key, partial) in [
        ("render_0.0", 0.0_f32),
        ("render_0.5", 0.5),
        ("render_1.0", 1.0),
    ] {
        described[key] = json!(match entity.shulker_render_offset(partial) {
            None => "none".to_owned(),
            Some(offset) => offset
                .map(|value| format!("{:016x}", value.to_bits()))
                .join(","),
        });
    }
    Ok(())
}

pub(crate) fn cube_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let cube = entity.cube_mob();
    described["size"] = json!(entity.cube_size());
    described["squish"] = json!(hex(cube.current_squish()));
    described["old_squish"] = json!(hex(cube.old_squish()));
    described["target_squish"] = json!(hex(cube.target_squish()));
    described["squish_half"] = json!(hex(cube.squish(0.5)));
    described["y_rot"] = json!(hex(entity.rotation().0));
    described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
    described["on_ground"] = json!(entity.on_ground());
    if voxel_data::entity_type::name(entity.type_id) == Some("minecraft:sulfur_cube") {
        described["fuse"] = json!(entity.sulfur_fuse());
    }
    let area = entity.bounding_box();
    for (key, value) in [
        ("min_x", area.min_x),
        ("min_y", area.min_y),
        ("min_z", area.min_z),
        ("max_x", area.max_x),
        ("max_y", area.max_y),
        ("max_z", area.max_z),
    ] {
        described[format!("box_{key}")] = json!(format!("{:016x}", value.to_bits()));
    }
    Ok(())
}

pub(crate) fn copper_golem_extra(
    entity: &voxel_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let tick = entity.tick_count() as f32;
    if let Some(golem) = entity.copper_golem() {
        let playing = golem.playing;
        for (key, state) in [
            ("idle", playing.idle),
            ("get_item", playing.get_item),
            ("get_no_item", playing.get_no_item),
            ("drop_item", playing.drop_item),
            ("drop_no_item", playing.drop_no_item),
        ] {
            described[format!("{key}_started")] = json!(state.is_started());
            described[format!("{key}_millis")] = json!(state.time_in_millis(tick));
        }
    }
    described["state"] = json!(entity.copper_golem_action() as i32);
    described["weather"] = json!(entity.copper_golem_weather());
    Ok(())
}
