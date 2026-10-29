use super::super::*;
use photon_world::EntityView;

pub(crate) fn turtle_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    described["in_water"] = json!(entity.is_in_water());
    described["on_ground"] = json!(entity.on_ground());
    described["has_egg"] = json!(entity.turtle_has_egg());
    described["laying_egg"] = json!(entity.turtle_laying_egg());
    described["is_on_land"] = json!(!entity.is_in_water() && entity.on_ground());
    let area = entity.bounding_box();
    described["max_y"] = json!(format!("{:016x}", area.max_y.to_bits()));
    described["max_x"] = json!(format!("{:016x}", area.max_x.to_bits()));
    Ok(())
}

pub(crate) fn guardian_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    described["in_water"] = json!(entity.is_in_water());
    let (y_rot, x_rot, head_y) = entity.rotation();
    described["y_rot"] = json!(hex(y_rot));
    described["x_rot"] = json!(hex(x_rot));
    described["head_y"] = json!(hex(head_y));
    described["body_y"] = json!(hex(entity.body_y_rotation()));
    described["on_ground"] = json!(entity.on_ground());
    let (pos_x, pos_y, pos_z) = entity.position();
    for (key, value) in [("x", pos_x), ("y", pos_y), ("z", pos_z)] {
        described[format!("pos_{key}")] = json!(format!("{:016x}", value.to_bits()));
    }
    let (motion_x, motion_y, motion_z) = entity.motion();
    for (key, value) in [("x", motion_x), ("y", motion_y), ("z", motion_z)] {
        described[format!("motion_{key}")] = json!(format!("{:016x}", value.to_bits()));
    }
    if let Some(guardian) = entity.guardian() {
        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
            described[format!("tail_{key}")] = json!(hex(guardian.tail_animation(partial)));
            described[format!("spikes_{key}")] = json!(hex(guardian.spikes_animation(partial)));
        }
        described["attack_time"] = json!(hex(guardian.attack_time() as f32));
        described["random_state"] = json!(guardian.random_state());
    }
    Ok(())
}

pub(crate) fn frog_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    described["in_water"] = json!(entity.is_in_water());
    if let Some(frog) = entity.frog() {
        let age = entity.tick_count() as f32;
        for (name, animation) in [
            ("jump", frog.jump),
            ("croak", frog.croak),
            ("tongue", frog.tongue),
            ("swim_idle", frog.swim_idle),
        ] {
            described[format!("{name}_started")] = json!(animation.is_started());
            described[format!("{name}_millis")] = json!(if animation.is_started() {
                animation.time_in_millis(age)
            } else {
                0
            });
        }
    }
    Ok(())
}

pub(crate) fn axolotl_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    let state = entity.axolotl().copied().unwrap_or_default();
    described["in_water"] = json!(entity.is_in_water());
    described["variant"] = json!(entity.axolotl_variant());
    for partial in [0.0_f32, 0.5, 1.0] {
        for (name, animator) in [
            ("playing_dead", state.playing_dead),
            ("in_water", state.in_water),
            ("on_ground", state.on_ground),
            ("moving", state.moving),
        ] {
            described[format!("{name}_{partial:?}")] = json!(hex(animator.factor(partial)));
        }
    }
    let age = entity.tick_count() as f32;
    for (name, animation) in [
        ("swim", state.swim),
        ("walk", state.walk),
        ("walk_under_water", state.walk_under_water),
        ("idle_under_water", state.idle_under_water),
        (
            "idle_under_water_on_ground",
            state.idle_under_water_on_ground,
        ),
        ("idle_on_ground", state.idle_on_ground),
        ("play_dead", state.play_dead),
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

pub(crate) fn dolphin_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    described["max_x"] = json!(format!("{:016x}", entity.bounding_box().max_x.to_bits()));
    let (x, _, z) = entity.motion();
    described["is_moving"] = json!(entity.is_moving_horizontally());
    described["motion_x"] = json!(format!("{:016x}", x.to_bits()));
    described["motion_z"] = json!(format!("{:016x}", z.to_bits()));
    Ok(())
}

pub(crate) fn fish_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    described["in_water"] = json!(entity.is_in_water());
    let area = entity.bounding_box();
    for (key, value) in [
        ("min_x", area.min_x),
        ("min_y", area.min_y),
        ("min_z", area.min_z),
        ("max_x", area.max_x),
        ("max_y", area.max_y),
        ("max_z", area.max_z),
    ] {
        described[key] = json!(format!("{:016x}", value.to_bits()));
    }
    Ok(())
}

pub(crate) fn squid_extra(entity: EntityView<'_>, described: &mut Value) -> Result<(), String> {
    if let Some(squid) = entity.squid() {
        let pose = squid.pose(0.5);
        described["in_water"] = json!(entity.is_in_water());
        described["y_rot"] = json!(hex(entity.rotation().0));
        described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
        described["tentacle_angle"] = json!(hex(squid.tentacle_angle));
        described["x_body_rot"] = json!(hex(squid.x_body_rot));
        described["z_body_rot"] = json!(hex(squid.z_body_rot));
        described["tentacle_angle_half"] = json!(hex(pose.tentacle_angle));
        described["x_body_rot_half"] = json!(hex(pose.x_body_rot));
        described["z_body_rot_half"] = json!(hex(pose.z_body_rot));
        if photon_data::entity_type::name(entity.type_id) == Some("minecraft:glow_squid") {
            described["dark_ticks"] = json!(entity.glow_dark_ticks());
        }
        let (x, y, z) = entity.motion();
        for (key, value) in [("motion_x", x), ("motion_y", y), ("motion_z", z)] {
            described[key] = json!(format!("{:016x}", value.to_bits()));
        }
    }
    Ok(())
}
