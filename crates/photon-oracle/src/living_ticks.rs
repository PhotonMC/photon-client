use super::*;

mod events;
use events::*;
mod extras;
use extras::*;

/// A double a scenario spells as the hex of its bits.
pub(crate) fn bits64(row: &Value, key: &str) -> Result<f64, String> {
    u64::from_str_radix(string(row, key)?, 16)
        .map(f64::from_bits)
        .map_err(|error| format!("{key}: {error}"))
}

/// A float as the hex of its bits, which the scenarios compare exactly.
fn hex(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

/// `living_entity_ticks`: a living entity through the packets that hurt or
/// kill it, as the net layer hands them to the world, ticked on.
pub(super) fn living_entity_ticks_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        let entity_id = i32_field(row, "entity_id")?;
        let name = string(row, "entity")?;
        // The client's own player, whose chunk decides whether a death ticks
        // (the scene's simulation distance is eight chunks).
        world.set_local_player_chunk(
            if row.get("player_x").is_some() {
                bits64(row, "player_x")?
            } else {
                6.5
            },
            0.5,
        );
        world.set_server_simulation_distance(8);
        world.set_minecart_improvements(row.get("minecart_improvements").is_some());
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id: photon_data::entity_type::id(name).ok_or("unknown entity type")?,
            x: bits64(row, "x")?,
            y: bits64(row, "y")?,
            z: bits64(row, "z")?,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("entity was rejected".into());
        }
        // Another entity that stays where it is (what a guardian's beam aims at).
        if let Some(second) = row.get("second") {
            let second_id = i32_field(second, "entity_id")?;
            if !world.apply_entity(EntityInput {
                entity_id: second_id,
                uuid: Uuid::new(0, u64::from(second_id as u32)),
                type_id: photon_data::entity_type::id(string(second, "entity")?)
                    .ok_or("unknown entity type")?,
                x: bits64(second, "x")?,
                y: bits64(second, "y")?,
                z: bits64(second, "z")?,
                movement: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                data: 0,
            }) {
                return Err("second entity was rejected".into());
            }
        }
        let mut ticks = Vec::new();
        for tick in 0..i32_field(row, "tick_count")? {
            for event in row["events"].as_array().ok_or("events must be an array")? {
                if i32_field(event, "tick")? != tick {
                    continue;
                }
                match string(event, "type")? {
                    "damage" => damage_event(&mut world, entity_id)?,
                    "hurt" => hurt_event(&mut world, entity_id)?,
                    "death" => death_event(&mut world, entity_id)?,
                    "move" => move_event(&mut world, entity_id, event)?,
                    "creeper_data" => creeper_data_event(&mut world, entity_id, event)?,
                    "animate" => animate_event(&mut world, entity_id, event)?,
                    "bee_data" => bee_data_event(&mut world, entity_id, event)?,
                    "sync" => sync_event(&mut world, entity_id, event)?,
                    "wolf_data" => wolf_data_event(&mut world, entity_id, event)?,
                    "weather" => weather_event(&mut world, event)?,
                    "equipment" => equipment_event(&mut world, entity_id, event)?,
                    "mob_flags" => mob_flags_event(&mut world, entity_id, event)?,
                    "strider_data" => strider_data_event(&mut world, entity_id, event)?,
                    "panda_data" => panda_data_event(&mut world, entity_id, event)?,
                    "fox_data" => fox_data_event(&mut world, entity_id, event)?,
                    "polar_bear_data" => polar_bear_data_event(&mut world, entity_id, event)?,
                    "cat_data" => cat_data_event(&mut world, entity_id, event)?,
                    "horse_data" => horse_data_event(&mut world, entity_id, event)?,
                    "game_time" => world.set_game_time(i64_field(event, "time")? as u64),
                    "camel_data" => camel_data_event(&mut world, entity_id, event)?,
                    "seed" => seed_event(&mut world, entity_id, event, name)?,
                    "copper_golem_data" => copper_golem_data_event(&mut world, entity_id, event)?,
                    "entity_motion" => entity_motion_event(&mut world, entity_id, event)?,
                    "passengers" => passengers_event(&mut world, event)?,
                    "shulker_data" => shulker_data_event(&mut world, entity_id, event)?,
                    "sulfur_fuse" => sulfur_fuse_event(&mut world, entity_id, event)?,
                    "head_rotation" => head_rotation_event(&mut world, entity_id, event)?,
                    "cube_size" => cube_size_event(&mut world, entity_id, event)?,
                    "glow_squid_data" => glow_squid_data_event(&mut world, entity_id, event)?,
                    "baby_data" => baby_data_event(&mut world, entity_id, event)?,
                    "axolotl_data" => axolotl_data_event(&mut world, entity_id, event)?,
                    "boat_data" => boat_data_event(&mut world, entity_id, event)?,
                    "flier_data" => flier_data_event(&mut world, entity_id, event)?,
                    "guardian_random" => guardian_random_event(&mut world, entity_id, event)?,
                    "guardian_data" => guardian_data_event(&mut world, entity_id, event)?,
                    "frog_data" => frog_data_event(&mut world, entity_id, event)?,
                    "arrow_data" => arrow_data_event(&mut world, entity_id, event)?,
                    "minecart_data" => minecart_data_event(&mut world, entity_id, event)?,
                    "crystal_data" => crystal_data_event(&mut world, entity_id, event)?,
                    "dragon_data" => dragon_data_event(&mut world, entity_id, event)?,
                    "display_data" => display_data_event(&mut world, entity_id, event)?,
                    "minecart_steps" => minecart_steps_event(&mut world, entity_id, event)?,
                    "move_rot" => move_rot_event(&mut world, entity_id, event)?,
                    "wither_data" => wither_data_event(&mut world, entity_id, event)?,
                    "sniffer_data" => sniffer_data_event(&mut world, entity_id, event)?,
                    "armadillo_data" => armadillo_data_event(&mut world, entity_id, event)?,
                    "creaking_data" => creaking_data_event(&mut world, entity_id, event)?,
                    "warden_data" => warden_data_event(&mut world, entity_id, event)?,
                    "turtle_data" => turtle_data_event(&mut world, entity_id, event)?,
                    "fish_data" => fish_data_event(&mut world, entity_id, event)?,
                    "squid_speed" => squid_speed_event(&mut world, entity_id, event)?,
                    "copper_golem_idle" => copper_golem_idle_event(&mut world, entity_id, event)?,
                    "rabbit_idle" => rabbit_idle_event(&mut world, entity_id, event)?,
                    "attributes" => attributes_event(&mut world, entity_id, event)?,
                    "entity_event" => entity_event_event(&mut world, entity_id, event)?,
                    other => return Err(format!("unknown event {other}")),
                }
            }
            world.tick_entities();
            let entity = world.entities().at(entity_id).ok_or("entity vanished")?;
            let mut described = json!({
                "hurt_time": entity.hurt_time(),
                "death_time": entity.death_time(),
                "dead": entity.is_dead_or_dying(),
                "walk_speed": hex(entity.walk_animation().speed()),
                "walk_position": hex(entity.walk_animation().position()),
            });
            for extra in row["extras"].as_array().into_iter().flatten() {
                match extra.as_str().ok_or("extras must be strings")? {
                    "swing" => swing_extra(entity, &mut described)?,
                    "sheep" => sheep_extra(entity, &mut described)?,
                    "camel" => camel_extra(entity, &mut described)?,
                    "horse" => horse_extra(entity, &mut described)?,
                    "bee" => bee_extra(entity, &mut described, &world)?,
                    "wolf" => wolf_extra(entity, &mut described, &world)?,
                    "strider" => strider_extra(entity, &mut described)?,
                    "ravager" => ravager_extra(entity, &mut described)?,
                    "panda" => panda_extra(entity, &mut described, &world)?,
                    "fox" => fox_extra(entity, &mut described)?,
                    "hoglin" => hoglin_extra(entity, &mut described)?,
                    "polar_bear" => polar_bear_extra(entity, &mut described)?,
                    "cat" => cat_extra(entity, &mut described)?,
                    "rabbit" => rabbit_extra(entity, &mut described)?,
                    "turtle" => turtle_extra(entity, &mut described)?,
                    "boat" => boat_extra(entity, &mut described)?,
                    "bat" => bat_extra(entity, &mut described)?,
                    "breeze" => breeze_extra(entity, &mut described)?,
                    "arrow" => arrow_extra(entity, &mut described)?,
                    "minecart" => minecart_extra(entity, &mut described, &world, row, tick)?,
                    "display" => display_extra(entity, &mut described)?,
                    "end_crystal" => end_crystal_extra(entity, &mut described)?,
                    "ender_dragon" => ender_dragon_extra(entity, &mut described, row, tick)?,
                    "wither" => wither_extra(entity, &mut described)?,
                    "sniffer" => sniffer_extra(entity, &mut described)?,
                    "armadillo" => armadillo_extra(entity, &mut described)?,
                    "creaking" => creaking_extra(entity, &mut described)?,
                    "warden" => warden_extra(entity, &mut described)?,
                    "parrot" => parrot_extra(entity, &mut described)?,
                    "vex" => described["charging"] = json!(entity.is_vex_charging()),
                    "allay" => allay_extra(entity, &mut described)?,
                    "guardian" => guardian_extra(entity, &mut described)?,
                    "frog" => frog_extra(entity, &mut described)?,
                    "axolotl" => axolotl_extra(entity, &mut described)?,
                    "dolphin" => dolphin_extra(entity, &mut described)?,
                    "fish" => fish_extra(entity, &mut described)?,
                    "shulker" => shulker_extra(entity, &mut described)?,
                    "cube" => cube_extra(entity, &mut described)?,
                    "squid" => squid_extra(entity, &mut described)?,
                    "copper_golem" => copper_golem_extra(entity, &mut described)?,
                    "attributes" => attributes_extra(entity, &mut described)?,
                    "goat" => goat_extra(entity, &mut described)?,
                    "iron_golem" => iron_golem_extra(entity, &mut described)?,
                    "creeper" => creeper_extra(entity, &mut described)?,
                    "chicken" => chicken_extra(entity, &mut described)?,
                    "cull" => cull_extra(entity, &mut described)?,
                    other => return Err(format!("unknown extra {other}")),
                }
            }
            ticks.push(described);
        }
        out.push(json!({"label": row["label"], "outcome": "ticked", "ticks": ticks}));
    }
    Ok(json!({"rows": out}))
}
