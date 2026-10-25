use super::*;

/// The uuid the oracle gives the local player in rows that name an owner (the Java side sets the same).
pub(crate) const ORACLE_PLAYER_UUID: u64 = 0x7001;

fn oracle_player_uuid() -> photon_core::binary::Uuid {
    photon_core::binary::Uuid::new(0, ORACLE_PLAYER_UUID)
}

/// A click count of a row: a non-negative int.
fn count_field(value: &Value, key: &str) -> Result<u32, String> {
    u32::try_from(i32_field(value, key)?).map_err(|e| e.to_string())
}

pub(super) fn movement_result(request: &Value) -> Result<Value, String> {
    use photon_world::ClientPlayerGameMode;
    use photon_world::local_player::LocalPlayer;

    fn bits64(row: &Value, key: &str) -> Result<f64, String> {
        let text = string(row, key)?;
        u64::from_str_radix(text, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    }
    fn bits32(row: &Value, key: &str) -> Result<f32, String> {
        let text = string(row, key)?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    }
    fn flag(row: &Value, key: &str) -> Result<bool, String> {
        row[key]
            .as_bool()
            .ok_or_else(|| format!("{key} must be a bool"))
    }

    if string(request, "operation")? == "light_changes" {
        return light_changes_result(request);
    }
    if string(request, "operation")? == "container_clicks" {
        return container_clicks_result(request);
    }
    if string(request, "operation")? == "component_hashes" {
        return component_hashes_result(request);
    }
    if string(request, "operation")? == "particle_types" {
        return particle_types_result(request);
    }
    if string(request, "operation")? == "particle_terrain" {
        return particle_terrain_result(request);
    }
    if string(request, "operation")? == "particle_quads" {
        return particle_quads_result(request);
    }
    if string(request, "operation")? == "particle_engine" {
        return particle_engine_result(request);
    }
    if let Some(result) = crate::data_tables::table_result(string(request, "operation")?) {
        return Ok(result);
    }
    if string(request, "operation")? == "block_map_colors" {
        // The map colours and dust colours, as the data crate carries them.
        let (blocks, states, dust) = photon_data::map::map_color_tables();
        let rows: Vec<Value> = blocks
            .iter()
            .map(|(name, color)| {
                let others: Vec<Value> = states
                    .iter()
                    .filter(|(state, _)| {
                        photon_data::block::registry::block_for_state(BlockStateId(*state as i32))
                            .is_some_and(|(block, _)| block.name == *name)
                    })
                    .map(|(state, color)| json!([state, color]))
                    .collect();
                let mut entry = json!({"name": name, "color": color, "states": others});
                if let Some((_, color)) = dust.iter().find(|(block, _)| block == name) {
                    entry["dust"] = json!(color);
                }
                entry
            })
            .collect();
        return Ok(json!({"rows": [{"label": "all", "blocks": rows}]}));
    }
    if string(request, "operation")? == "leaf_particles" {
        let blocks: Vec<Value> = photon_data::leaves::all()
            .iter()
            .map(|(name, chance, particle, color)| {
                let mut entry = json!({
                    "name": name,
                    "chance": format!("{:08x}", chance.to_bits()),
                    "particle": particle,
                });
                if let Some(color) = color {
                    entry["color"] = json!(color);
                }
                entry
            })
            .collect();
        return Ok(json!({"rows": [{"label": "all", "blocks": blocks}]}));
    }
    if string(request, "operation")? == "item_pickup" {
        return item_pickup_result(request);
    }
    if string(request, "operation")? == "particle_ticks" {
        return particle_ticks_result(request);
    }
    if string(request, "operation")? == "projectile_ticks" {
        return projectile_ticks_result(request);
    }
    if string(request, "operation")? == "item_entity_ticks" {
        return item_entity_ticks_result(request);
    }
    if string(request, "operation")? == "living_entity_ticks" {
        return living_entity_ticks_result(request);
    }
    if let Some(result) = lighting_result(string(request, "operation")?, request) {
        return result;
    }
    if string(request, "operation")? == "entity_pick_table" {
        return Ok(entity_pick_table_result(request));
    }
    if let Some(result) = sky_result(string(request, "operation")?, request) {
        return result;
    }
    if string(request, "operation")? == "elytra_state" {
        return elytra_state_result(request);
    }
    if string(request, "operation")? != "local_player_ticks" {
        return Err("unknown movement operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("movement rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    let (instruments, spin_attack) = (vanilla_instruments(), vanilla_spin_attack());
    for row in rows {
        let label = string(row, "label")?;
        let mut world = movement_world(row)?;
        if let Some(border) = row.get("border") {
            world.apply_world_border_center(bits64(border, "x")?, bits64(border, "z")?);
            world.apply_world_border_size(bits64(border, "size")?);
        }
        let game_mode_of_row = match string(row, "game_mode")? {
            "survival" => ClientPlayerGameMode::Survival,
            "creative" => ClientPlayerGameMode::Creative,
            "adventure" => ClientPlayerGameMode::Adventure,
            "spectator" => ClientPlayerGameMode::Spectator,
            other => return Err(format!("unknown game mode {other}")),
        };
        let mut player = LocalPlayer::new(
            EntityId(i32_field(row, "entity_id")?),
            photon_core::math::Vec3::new(bits64(row, "x")?, bits64(row, "y")?, bits64(row, "z")?),
            bits32(row, "y_rot")?,
            bits32(row, "x_rot")?,
            game_mode_of_row,
            flag(row, "auto_jump")?,
        );
        movement_player::start_player(&mut player, row)?;
        let breaking = row.get("breaking");
        let mut game_mode = photon_game::game_mode::GameMode::default();
        let mut cooldowns = photon_game::item::cooldowns::ItemCooldowns::default();
        if let Some(breaking) = breaking {
            world.set_local_player_id(player.entity_id());
            told_entities(&mut world, &player, row)?;
            hold_main_hand(&mut world, &mut player, breaking)?;
            worn_items(&mut world, &mut player, breaking)?;
            apply_effects(&mut world, &player, breaking)?;
        }
        // A vehicle the player rides, put aboard as the server's passengers packet does.
        let mut ridden = None;
        let mut held_stick = "";
        if let Some(ride) = row.get("ride") {
            let kind = string(ride, "type")?;
            let id = entity_id_field(ride, "id")?;
            world.set_local_player_id(player.entity_id());
            let motion = |key: &str| bits64(ride, key).unwrap_or(0.0);
            if !world.apply_entity(EntityInput {
                entity_id: id,
                uuid: Uuid::new(0, id.0 as u64),
                type_id: photon_data::entity_type::id(kind)
                    .ok_or_else(|| format!("unknown entity type {kind}"))?,
                x: bits64(ride, "x")?,
                y: bits64(ride, "y")?,
                z: bits64(ride, "z")?,
                movement: (motion("motion_x"), motion("motion_y"), motion("motion_z")),
                rotation: (bits32(ride, "y_rot")?, 0.0, bits32(ride, "y_rot")?),
                data: 0,
            }) || !world.apply_entity_passengers(id, &[player.entity_id()])
            {
                return Err(format!("the {kind} could not be boarded"));
            }
            if kind.ends_with("_boat") || kind.ends_with("_raft") {
                player.face_boat(bits32(ride, "y_rot")?);
            }
            if kind.contains("camel") {
                // The game's clock, and a camel that has sat for so long.
                world.set_game_time(1000);
                if let Some(age) = ride["sit_age"].as_i64() {
                    let mut data = vec![photon_world::CAMEL_LAST_POSE_CHANGE_DATA_ID, 2];
                    photon_core::wire::write_varlong(-(1000 - age), &mut data);
                    data.extend([6, 20, 10, 255]);
                    world.apply_entity_metadata(id, &data);
                }
            }
            if (kind.ends_with("horse")
                || kind.ends_with("donkey")
                || kind.ends_with("mule")
                || kind.contains("camel"))
                && ride.get("unsaddled").is_none()
            {
                // `AbstractHorse.addPassenger`: the rider looks the way the horse does.
                player.set_rotation(bits32(ride, "y_rot")?, 0.0);
                // A tamed horse in a saddle, as the oracle makes it.
                world.entities_mut().apply_equipment(
                    id,
                    photon_world::EquipmentSlot::Saddle,
                    Some(photon_world::ClientItemStackState::new(
                        1,
                        photon_game::item::id_of("minecraft:saddle").ok_or("no saddle item")?,
                    )),
                );
            }
            // What the rider holds in the selected slot, and a saddle where the row has one.
            if let Some(name) = ride["holding"].as_str() {
                let item =
                    photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
                world.apply_local_player_container_slot(
                    0,
                    0,
                    36,
                    Some(&photon_world::ClientItemStackState::new(1, item)),
                );
                player.set_main_hand_item(item);
                held_stick = name;
            }
            if ride["saddled"].as_bool().unwrap_or(false) {
                world.entities_mut().apply_equipment(
                    id,
                    photon_world::EquipmentSlot::Saddle,
                    Some(photon_world::ClientItemStackState::new(
                        1,
                        photon_game::item::id_of("minecraft:saddle").ok_or("no saddle item")?,
                    )),
                );
            }
            ridden = Some(id);
        }
        let mut ticks = Vec::new();
        let mut failure = None;
        let block_events = row.get("block_events").and_then(Value::as_array);
        for (index, tick) in row["ticks"]
            .as_array()
            .ok_or("ticks must be an array")?
            .iter()
            .enumerate()
        {
            apply_block_events(&mut world, block_events, index)?;
            let keys = photon_world::input::Input::new(
                flag(tick, "forward")?,
                flag(tick, "backward")?,
                flag(tick, "left")?,
                flag(tick, "right")?,
                flag(tick, "jump")?,
                flag(tick, "sneak")?,
                flag(tick, "sprint")?,
            );
            let (mut y_rot, mut x_rot) = player.rotation();
            if tick.get("y_rot").is_some() {
                y_rot = bits32(tick, "y_rot")?;
            }
            if tick.get("x_rot").is_some() {
                x_rot = bits32(tick, "x_rot")?;
            }
            player.set_rotation(y_rot, x_rot);
            server_events(tick, (&mut world, &mut player, &mut cooldowns));
            let mut actions = Vec::new();
            if let Some(attack) = tick.get("attack") {
                for event in attack["server"].as_array().into_iter().flatten() {
                    if event.get("ack").is_some() {
                        for synced in world.handle_block_changed_ack(i32_field(event, "ack")?) {
                            let pos =
                                photon_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
                            if let Some(position) = synced.player_position
                                && player.is_colliding(&world, pos, synced.state)
                            {
                                player.abs_snap_to(position);
                            }
                        }
                    } else {
                        world.apply_server_block_state(
                            i32_field(event, "x")?,
                            i32_field(event, "y")?,
                            i32_field(event, "z")?,
                            named_state(event)?,
                        );
                    }
                }
                let aim = match attack.get("target") {
                    _ if attack.get("pick").is_some() => {
                        // `getEyePosition(1.0)`: the position the frame blends to.
                        let at = player.interpolated_position(1.0);
                        photon_game::input::pick::pick(
                            &world,
                            &player,
                            photon_core::math::Vec3::new(
                                at.x,
                                at.y + f64::from(player.eye_height()),
                                at.z,
                            ),
                        )
                    }
                    Some(target) => {
                        let (x, y, z) = (
                            i32_field(target, "x")?,
                            i32_field(target, "y")?,
                            i32_field(target, "z")?,
                        );
                        let location = match target.get("location") {
                            Some(at) => [bits64(at, "x")?, bits64(at, "y")?, bits64(at, "z")?],
                            None => [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5],
                        };
                        photon_game::game_mode::Aim::Block {
                            x,
                            y,
                            z,
                            face: match string(target, "face")? {
                                "down" => 0,
                                "up" => 1,
                                "north" => 2,
                                "south" => 3,
                                "west" => 4,
                                "east" => 5,
                                other => return Err(format!("unknown face {other}")),
                            },
                            location,
                            inside: target["inside"].as_bool().unwrap_or(false),
                        }
                    }
                    None => photon_game::game_mode::Aim::Miss,
                };
                let use_ = attack.get("use");
                let keys = photon_game::game_mode::Keys {
                    attack_clicks: count_field(attack, "clicks")?,
                    attack_held: flag(attack, "down")?,
                    use_clicks: use_.map_or(Ok(0), |use_| count_field(use_, "clicks"))?,
                    use_held: use_.map_or(Ok(false), |use_| flag(use_, "down"))?,
                    pick_clicks: attack
                        .get("pick_item")
                        .map_or(Ok(0), |pick| count_field(pick, "clicks"))?,
                    control_down: false,
                };
                let mut ctx = photon_game::game_mode::Context {
                    miner: &photon_game::game_mode::Miner::of(&world, 0),
                    world: &mut world,
                    player: &mut player,
                    mode: game_mode_of_row,
                    selected_slot: 0,
                    cooldowns: &mut cooldowns,
                    instruments: &instruments,
                    spin_attack: &spin_attack,
                    game_master: false,
                    profile_id: row.get("player_uuid").map(|_| oracle_player_uuid()),
                };
                actions = game_mode.keys_tick(&mut ctx, keys, aim);
            }
            // As the session does: the main hand the player ticks with is
            // the world's inventory now.
            before_tick(&world, &mut player, breaking);
            let sent = if let Some(vehicle_id) = ridden {
                // The level ticks the vehicle (which tells the server of its paddles), then its
                // passenger, which hands the boat the keys for the next tick.
                let mut controls = player.controller();
                controls.holds_carrot_on_a_stick = held_stick == "minecraft:carrot_on_a_stick";
                controls.holds_warped_fungus_on_a_stick =
                    held_stick == "minecraft:warped_fungus_on_a_stick";
                world.set_rider_controls(controls);
                world.tick_entities();
                let mut sent: Vec<photon_world::local_player::ServerboundMovement> = world
                    .take_paddle_packets()
                    .into_iter()
                    .map(|(left, right)| {
                        photon_world::local_player::ServerboundMovement::PaddleBoat { left, right }
                    })
                    .collect();
                let Some(view) = world.vehicle_view(vehicle_id) else {
                    return Err("the boat is gone".into());
                };
                match player.tick_riding(&world, keys, &view) {
                    Ok(rode) => {
                        if view.kind == photon_world::VehicleKind::Boat && view.controlled {
                            world.set_boat_input(vehicle_id, rode.boat_input);
                        }
                        if let Some(amount) = rode.jump {
                            world.on_player_jump(vehicle_id, amount);
                        }
                        sent.extend(rode.sent);
                        sent
                    }
                    Err(error) => {
                        failure = Some(error.to_string());
                        break;
                    }
                }
            } else {
                match player.tick(&world, keys) {
                    Ok(sent) => sent,
                    Err(error) => {
                        failure = Some(error.to_string());
                        break;
                    }
                }
            };
            after_tick(&mut world, &mut player, breaking);
            failure = failure.or(tick_block_entities(&mut world, &mut player, block_events).err());
            let position = player.position();
            let motion = player.delta_movement();
            let (y_rot, x_rot) = player.rotation();
            let mut packets: Vec<Value> = actions
                .iter()
                .map(|action| json!({"type": action.type_name(), "bytes": hex(&action.encode_body())}))
                .collect();
            packets.extend(sent.iter().map(
                |packet| json!({"type": packet.type_name(), "bytes": hex(&packet.encode_body())}),
            ));
            // The sounds the tick asked for (the player's own, and the world's: a block's), in a fixed order.
            let mut sounds: Vec<String> = player
                .take_sounds()
                .into_iter()
                .chain(world.take_sounds())
                // A block the player broke plays its sound as a level event.
                .chain(
                    world
                        .take_level_events()
                        .iter()
                        .filter_map(photon_world::LevelEvent::break_sound),
                )
                .map(|sound| {
                    let source = sound.source.name();
                    // The swim, splash, hurt and death sounds' pitch is random in Java and is compared by volume only.
                    let pitch = if crate::view_state::has_random_pitch(sound.event) {
                        0.0_f32
                    } else {
                        sound.pitch
                    };
                    format!(
                        "{},{source},{:08x},{:08x},{:016x},{:016x},{:016x}",
                        sound.event,
                        sound.volume.to_bits(),
                        pitch.to_bits(),
                        sound.position.0.to_bits(),
                        sound.position.1.to_bits(),
                        sound.position.2.to_bits()
                    )
                })
                .collect();
            sounds.sort();
            // The fall of a vehicle that lands is passed on to its passenger, which is not ported.
            sounds.retain(|_| ridden.is_none());
            let mut described = json!({
                "sounds": sounds,
                "x": format!("{:016x}", position.x.to_bits()),
                "y": format!("{:016x}", position.y.to_bits()),
                "z": format!("{:016x}", position.z.to_bits()),
                "motion_x": format!("{:016x}", motion.x.to_bits()),
                "motion_y": format!("{:016x}", motion.y.to_bits()),
                "motion_z": format!("{:016x}", motion.z.to_bits()),
                "y_rot": format!("{:08x}", y_rot.to_bits()),
                "x_rot": format!("{:08x}", x_rot.to_bits()),
                "on_ground": player.on_ground(),
                "horizontal_collision": player.horizontal_collision(),
                "vertical_collision": player.vertical_collision(),
                "minor_horizontal_collision": player.minor_horizontal_collision(),
                "sprinting": player.is_sprinting(),
                "shift_key_down": player.is_shift_key_down(),
                "pose": player.pose().java_name(),
                "fall_flying": player.is_fall_flying(),
                "fall_distance": format!("{:016x}", player.fall_distance().to_bits()),
                "water_vision": format!("{:08x}", player.water_vision().to_bits()),
                "flying": player.is_flying(),
                "portal_effect": format!("{:08x}", player.portal_effect_intensity_now().to_bits()),
                "portal_effect_old": format!("{:08x}", player.portal_effect_intensity(0.0).to_bits()),
                "bob_walk": format!("{:08x}", player.view_bob(0.5).0.to_bits()),
                "bob": format!("{:08x}", player.view_bob(0.5).1.to_bits()),
                "body_rot": format!("{:08x}", player.model_pose(0.5).body_rot.to_bits()),
                "head_rot": format!("{:08x}", player.model_pose(0.5).head_rot.to_bits()),
                "walk_position": format!("{:08x}", player.model_pose(0.5).walk_position.to_bits()),
                "walk_speed": format!("{:08x}", player.model_pose(0.5).walk_speed.to_bits()),
                "swim_amount": format!("{:08x}", player.model_pose(0.5).swim_amount.to_bits()),
                "hand_x_bob": format!("{:08x}", player.hand_bob(0.5).0.to_bits()),
                "hand_y_bob": format!("{:08x}", player.hand_bob(0.5).1.to_bits()),
                "swinging": player.is_swinging(),
                "swing_time": player.swing_time(),
                "swing_off_hand": player.swinging_off_hand(),
                "attack_anim": format!("{:08x}", player.attack_anim(0.5).to_bits()),
                "using_item": player.is_using_item(),
                "using_off_hand": player.used_item_is_off_hand(),
                "ticks_using_item": format!("{:08x}", player.ticks_using_item(0.5).to_bits()),
                "view_block": player.view_blocking_state(&world).map_or(-1, |state| state.0),
                "sent": packets,
            });
            if let Some(vehicle_id) = ridden {
                described["vehicle"] = vehicle_report(&world, vehicle_id)?;
            }
            // `Player.tick` counts the item cooldowns down.
            cooldowns.tick();
            if let Some(attack) = tick.get("attack") {
                described["breaking"] =
                    breaking_report(&world, &player, (&game_mode, &cooldowns), attack)?;
            }
            ticks.push(with_rockets(described, &world, breaking));
        }
        match failure {
            None => out.push(json!({"label": label, "outcome": "ticked", "ticks": ticks})),
            Some(message) => out.push(json!({
                "label": label,
                "outcome": "unsupported",
                "message": message,
                "ticks": ticks,
            })),
        }
    }
    Ok(json!({"rows": out}))
}
