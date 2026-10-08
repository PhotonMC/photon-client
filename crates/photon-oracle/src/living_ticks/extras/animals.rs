use super::super::*;

pub(crate) fn swing_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let swing = entity.swing_state();
    described["swinging"] = json!(swing.is_swinging());
    described["swing_time"] = json!(swing.swing_time());
    described["off_hand"] = json!(swing.is_off_hand());
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
        described[format!("attack_anim_{key}")] = json!(hex(swing.attack_anim(partial)));
    }
    Ok(())
}

pub(crate) fn sheep_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        let (position, angle) = entity.sheep_head_eat_scales(partial, 0.0);
        described[format!("eat_position_{key}")] = json!(hex(position));
        described[format!("eat_angle_{key}")] = json!(hex(angle));
    }
    Ok(())
}

pub(crate) fn camel_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    if let Some(camel) = entity.horse().map(|horse| &horse.camel) {
        let tick = entity.tick_count() as f32;
        for (name, state) in [
            ("sit", camel.sit),
            ("sit_pose", camel.sit_pose),
            ("sit_up", camel.sit_up),
            ("idle", camel.idle),
            ("dash", camel.dash),
        ] {
            described[format!("{name}_started")] = json!(state.is_started());
            described[format!("{name}_millis")] = json!(if state.is_started() {
                state.time_in_millis(tick)
            } else {
                0
            });
        }
        described["dashing"] = json!(camel.is_dashing());
    }
    Ok(())
}

pub(crate) fn horse_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    if let Some(horse) = entity.horse() {
        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
            described[format!("eat_{key}")] = json!(hex(horse.eat_animation(partial)));
            described[format!("stand_{key}")] = json!(hex(horse.stand_animation(partial)));
            described[format!("mouth_{key}")] = json!(hex(horse.mouth_animation(partial)));
        }
        described["tail_counter"] = json!(horse.tail_counter());
    }
    Ok(())
}

pub(crate) fn bee_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
    world: &ClientWorld,
) -> Result<(), String> {
    described["roll_0.0"] = json!(hex(entity.bee_roll_amount(0.0)));
    described["roll_0.5"] = json!(hex(entity.bee_roll_amount(0.5)));
    described["stung"] = json!(entity.bee_has_stung());
    described["nectar"] = json!(entity.bee_has_nectar());
    described["angry"] = json!(entity.is_angry(world.game_time()));
    Ok(())
}

pub(crate) fn wolf_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
    world: &ClientWorld,
) -> Result<(), String> {
    let game_time = world.game_time();
    described["angry"] = json!(entity.is_angry(game_time));
    described["sitting"] = json!(entity.is_sitting());
    described["tame"] = json!(entity.is_tame());
    described["tail_angle"] = json!(hex(entity.wolf_tail_angle(game_time)));
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        let wolf = entity.wolf();
        described[format!("head_roll_{key}")] = json!(hex(wolf.head_roll_angle(partial)));
        described[format!("shake_{key}")] = json!(hex(wolf.shake_anim(partial)));
    }
    Ok(())
}

pub(crate) fn strider_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["suffocating"] = json!(entity.is_suffocating());
    Ok(())
}

pub(crate) fn ravager_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let ravager = entity.ravager();
    described["attack_tick"] = json!(ravager.attack_tick());
    described["stunned_tick"] = json!(ravager.stunned_tick());
    described["roar_tick"] = json!(ravager.roar_tick());
    Ok(())
}

pub(crate) fn panda_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
    world: &ClientWorld,
) -> Result<(), String> {
    let panda = entity.panda();
    let thundering = world.is_thundering();
    described["variant"] = json!(panda.variant());
    described["unhappy_counter"] = json!(panda.unhappy_counter());
    described["sneeze_counter"] = json!(panda.sneeze_counter());
    described["sneezing"] = json!(panda.is_sneezing());
    described["eating"] = json!(panda.is_eating());
    described["sitting"] = json!(panda.is_sitting());
    described["rolling"] = json!(panda.is_rolling());
    described["on_back"] = json!(panda.is_on_back());
    described["roll_counter"] = json!(panda.roll_counter());
    described["scared"] = json!(panda.is_scared(thundering));
    described["x_rot"] = json!(hex(entity.rotation().1));
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        described[format!("sit_{key}")] = json!(hex(panda.sit_amount(partial)));
        described[format!("back_{key}")] = json!(hex(panda.lie_on_back_amount(partial)));
        described[format!("roll_{key}")] = json!(hex(panda.roll_amount(partial)));
    }
    Ok(())
}

pub(crate) fn fox_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let fox = entity.fox();
    described["variant"] = json!(fox.variant());
    described["sitting"] = json!(fox.is_sitting());
    described["crouching"] = json!(fox.is_crouching());
    described["sleeping"] = json!(fox.is_sleeping());
    described["faceplanted"] = json!(fox.is_faceplanted());
    described["pouncing"] = json!(fox.is_pouncing());
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        described[format!("head_roll_{key}")] = json!(hex(fox.head_roll_angle(partial)));
        described[format!("crouch_{key}")] = json!(hex(fox.crouch_amount(partial)));
    }
    Ok(())
}

pub(crate) fn hoglin_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["attack_ticks"] = json!(entity.hoglin_attack_ticks());
    Ok(())
}

pub(crate) fn polar_bear_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let bear = entity.polar_bear();
    described["standing"] = json!(bear.is_standing());
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        described[format!("stand_{key}")] = json!(hex(bear.standing_animation_scale(partial)));
    }
    Ok(())
}

pub(crate) fn cat_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["sitting"] = json!(entity.is_sitting());
    described["tame"] = json!(entity.is_tame());
    described["crouching"] = json!(entity.is_crouching());
    let cat = entity.cat();
    described["on_player"] = json!(cat.is_lying_on_top_of_sleeping_player());
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
        described[format!("lie_{key}")] = json!(hex(cat.lie_down_amount(partial)));
        described[format!("lie_tail_{key}")] = json!(hex(cat.lie_down_amount_tail(partial)));
        described[format!("relax_{key}")] = json!(hex(cat.relax_state_one_amount(partial)));
    }
    Ok(())
}

pub(crate) fn rabbit_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    if let Some(rabbit) = entity.rabbit() {
        let tick = entity.tick_count() as f32;
        described["hop_started"] = json!(rabbit.hop.is_started());
        described["idle_started"] = json!(rabbit.idle_head_tilt.is_started());
        described["hop_millis"] = json!(rabbit.hop.time_in_millis(tick));
        described["idle_millis"] = json!(rabbit.idle_head_tilt.time_in_millis(tick));
        described["jump_completion"] = json!(hex(rabbit.jump_completion(0.5)));
    }
    Ok(())
}

pub(crate) fn attributes_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["max_health"] = json!(hex(entity.max_health().unwrap_or(0.0)));
    described["scale"] = json!(hex(entity.scale()));
    Ok(())
}

pub(crate) fn goat_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    described["ramming"] = json!(hex(entity.goat_ramming_x_head_rot()));
    Ok(())
}

pub(crate) fn iron_golem_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let (attack, offer) = entity.iron_golem_ticks();
    described["attack_tick"] = json!(attack);
    described["offer_flower_tick"] = json!(offer);
    Ok(())
}

pub(crate) fn creeper_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
        described[format!("swelling_{key}")] = json!(hex(entity.creeper_swelling(partial)));
    }
    Ok(())
}

pub(crate) fn chicken_extra(
    entity: &photon_world::ClientEntity,
    described: &mut Value,
) -> Result<(), String> {
    let (flap, flap_speed) = entity.chicken_flap(1.0);
    let (old_flap, old_flap_speed) = entity.chicken_flap(0.0);
    described["flap"] = json!(hex(flap));
    described["flap_speed"] = json!(hex(flap_speed));
    described["old_flap"] = json!(hex(old_flap));
    described["old_flap_speed"] = json!(hex(old_flap_speed));
    Ok(())
}
