use super::*;

impl PlaySession {
    /// A packet from the reader thread: the heavy ones arrive parsed.
    pub(super) fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        match packet {
            Incoming::Raw(body) => self.handle(&body),
            Incoming::Chunk(chunk) => {
                self.apply_chunk(&chunk);
                Ok(())
            }
            Incoming::Light(update) => {
                self.apply_light_update(&update);
                Ok(())
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per packet the play state reads"
    )]
    #[expect(
        clippy::cognitive_complexity,
        reason = "one arm per packet id, read like the reference's handler list"
    )]
    pub(super) fn handle(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        let (packet_id, payload) = split_packet(body)?;
        match packet_id {
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => {
                let login = parse_login_packet(payload)?;
                self.next_chat_index = 0;
                self.trace
                    .world
                    .set_server_simulation_distance(login.simulation_distance);
                let bounds = dimension_bounds_for_spawn_info(&self.trace, &login.spawn_info);
                let environment =
                    dimension_environment_for_spawn_info(&self.trace, &login.spawn_info);
                self.trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &login.spawn_info.dimension,
                        dimension_type_id: login.spawn_info.dimension_type_id,
                        seed: login.spawn_info.seed,
                        is_debug: login.spawn_info.is_debug,
                        is_flat: login.spawn_info.is_flat,
                        sea_level: login.spawn_info.sea_level,
                    },
                    bounds,
                    environment,
                );
                self.trace.world.configure_chunk_cache(login.chunk_radius);
                self.view_distance = login.chunk_radius;
                self.dimension_type_id = login.spawn_info.dimension_type_id;
                self.hashed_seed = login.spawn_info.seed;
                self.trace.world.set_local_player_id(login.player_id);
                self.hardcore = login.hardcore;
                self.show_death_screen = login.show_death_screen;
                self.player_id = Some(login.player_id);
                self.permission_level = 0;
                self.game_mode =
                    ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
            }
            // The entity packets change the world's entities; one that does
            // not parse is left alone, as the reference ignores what it
            // cannot place.
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => {
                if let Some(packet) = parsed(parse_add_entity_packet(payload), "add_entity") {
                    if self.removed_player_vehicle == Some(packet.entity_id) {
                        self.removed_player_vehicle = None;
                    }
                    entities::add(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_set_entity_motion_packet(payload), "set_entity_motion")
                {
                    entities::motion(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID => {
                if let Some(packet) = parsed(parse_map_item_data_packet(payload), "map_item_data") {
                    self.trace.world.apply_map_item_data(&packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID => {
                if let Some((entity_id, steps)) =
                    parsed(parse_move_minecart_packet(payload), "move_minecart")
                {
                    entities::minecart_steps(&mut self.trace.world, entity_id, &steps);
                }
            }
            PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID => {
                if let Some((source, holder)) =
                    parsed(parse_set_entity_link_packet(payload), "set_entity_link")
                {
                    self.trace
                        .world
                        .entities_mut()
                        .set_leash_holder(source, holder);
                }
            }
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => {
                if let Some(packet) = parsed(parse_rotate_head_packet(payload), "rotate_head") {
                    entities::rotate_head(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_remove_entities_packet(payload), "remove_entities")
                {
                    // `handleRemoveEntities`: a vehicle that goes with its rider aboard is
                    // remembered, for a teleport the server may still address to it.
                    for &id in &packet.entity_ids {
                        if self.trace.world.has_entity(id)
                            && self.trace.world.entity_carries_local_player(id)
                        {
                            self.removed_player_vehicle = Some(id);
                        }
                    }
                    entities::remove(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => {
                if let Some(packet) = parsed(parse_set_passengers_packet(payload), "set_passengers")
                {
                    let was_aboard = self.trace.world.ridden_vehicle_of_local_player();
                    entities::passengers(&mut self.trace.world, &packet);
                    if self
                        .player_id
                        .is_some_and(|id| packet.passenger_ids.contains(&id))
                    {
                        self.removed_player_vehicle = None;
                    }
                    // `handleSetEntityPassengersPacket`: boarding tells the rider how to get off.
                    if was_aboard != Some(packet.vehicle_id)
                        && self.trace.world.ridden_vehicle_of_local_player()
                            == Some(packet.vehicle_id)
                    {
                        self.hud_events.push(HudEvent::ActionBar(
                            voxel_game::text::Component::translatable_with(
                                "mount.onboard",
                                vec![voxel_game::text::Component::translatable(
                                    "key.keyboard.left.shift",
                                )],
                            ),
                        ));
                    }
                    // A player who has just boarded looks the way the boat or the horse faces.
                    if was_aboard.is_none()
                        && let Some(vehicle) = self.trace.world.ridden_vehicle_of_local_player()
                        && let Some(view) = self.trace.world.vehicle_view(vehicle)
                        && let Some(player) = self.player.as_mut()
                    {
                        match view.kind {
                            voxel_world::VehicleKind::Boat => player.face_boat(view.y_rot),
                            voxel_world::VehicleKind::Horse => {
                                player.set_rotation(view.y_rot, view.x_rot);
                            }
                            // Any other vehicle leaves its rider looking where it did.
                            voxel_world::VehicleKind::Pig | voxel_world::VehicleKind::Other => {}
                        }
                    }
                }
            }
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                if let Some(packet) = parsed(
                    parse_entity_position_sync_packet(payload),
                    "entity_position_sync",
                ) {
                    entities::position_sync(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_entity_teleport_packet(payload), "entity_teleport")
                {
                    if !self.trace.world.has_entity(packet.entity_id)
                        && self.removed_player_vehicle == Some(packet.entity_id)
                    {
                        // The vehicle is gone: the teleport meant for it moves the rider.
                        self.move_former_rider(&packet)?;
                    }
                    entities::teleport(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_move_entity_pos_packet(payload), "move_entity_pos")
                {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                if let Some(packet) = parsed(
                    parse_move_entity_pos_rot_packet(payload),
                    "move_entity_pos_rot",
                ) {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_move_entity_rot_packet(payload), "move_entity_rot")
                {
                    entities::step(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_ANIMATE_PACKET_ID => {
                if let Some((id, action)) = parsed(parse_animate_packet(payload), "animate") {
                    self.trace.world.entities_mut().apply_animate(id, action);
                }
            }
            PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID => {
                let (id, event) = parse_entity_event_packet(payload)?;
                // `LocalPlayer.handleEntityEvent`: 24 to 28 set the level.
                if self.player_id == Some(id) && (24..=28).contains(&event) {
                    self.permission_level = i32::from(event) - 24;
                }
                // `LivingEntity.handleEntityEvent`: 3 is a death.
                if event == 3 {
                    if self.player_id == Some(id)
                        && let Some(player) = self.player.as_mut()
                    {
                        player.handle_death_event();
                    }
                    self.trace.world.entities_mut().apply_death_event(id);
                } else {
                    self.trace.world.apply_entity_event(id, event);
                }
            }
            PLAY_CLIENTBOUND_SET_SIMULATION_DISTANCE_PACKET_ID => {
                if let Some(chunks) = parsed(parse_entity_id_prefix(payload), "entity_id_prefix") {
                    self.trace.world.set_server_simulation_distance(chunks);
                }
            }
            PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID => {
                if let Some((id, damage_type)) =
                    parsed(parse_damage_event_packet(payload), "damage_event")
                {
                    self.trace.world.entities_mut().apply_damage_event(id);
                    if self.player_id == Some(id) {
                        let name = damage_type
                            .and_then(|index| {
                                self.registry_names("minecraft:damage_type").get(index)
                            })
                            .cloned()
                            .unwrap_or_default();
                        if let Some(player) = self.player.as_mut() {
                            player.handle_damage_event(&name);
                        }
                    }
                }
            }
            PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID => {
                if let Some((id, yaw)) =
                    parsed(parse_hurt_animation_packet(payload), "hurt_animation")
                {
                    self.trace.world.entities_mut().apply_hurt_animation(id);
                    if self.player_id == Some(id)
                        && let Some(player) = self.player.as_mut()
                    {
                        player.animate_hurt(yaw);
                    }
                }
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, data_to_keep) = parse_respawn_packet(payload)?;
                // A new level starts dry; the server repeats its weather.
                self.rain_level = 0.0;
                self.thunder_level = 0.0;
                self.trace.world.set_weather(0.0, 0.0);
                let bounds = dimension_bounds_for_spawn_info(&self.trace, &spawn_info);
                let environment = dimension_environment_for_spawn_info(&self.trace, &spawn_info);
                self.trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &spawn_info.dimension,
                        dimension_type_id: spawn_info.dimension_type_id,
                        seed: spawn_info.seed,
                        is_debug: spawn_info.is_debug,
                        is_flat: spawn_info.is_flat,
                        sea_level: spawn_info.sea_level,
                    },
                    bounds,
                    environment,
                );
                self.game_mode = ClientPlayerGameMode::from_id(i32::from(spawn_info.game_type));
                // The new player starts with an empty inventory; the server
                // sends it again.
                self.trace.world.recreate_local_player(data_to_keep);
                self.dimension_type_id = spawn_info.dimension_type_id;
                self.hashed_seed = spawn_info.seed;
                // A respawn makes a new player that waits for its position.
                self.player = None;
                self.permission_level = 0;
                self.loaded = false;
                self.level_reset = true;
                self.health_received = false;
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                self.apply_chunk(&parse_chunk_packet(payload)?);
            }
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                self.apply_light_update(&parse_light_update_packet(payload)?);
            }
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => {
                let (x, z) = parse_forget_level_chunk_packet(payload)?;
                self.trace.world.unload_chunk(x, z);
                self.forgotten_chunks.push((x, z));
                self.mark_columns_around(x, z);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let update = parse_block_update_packet(payload)?;
                self.mark_block(update.x, update.y, update.z);
                self.trace.world.apply_server_block_state(
                    update.x,
                    update.y,
                    update.z,
                    update.state_id,
                );
            }
            PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID => {
                let mut input = payload;
                let section = read_i64(&mut input)?;
                let section_x = (section >> 42) as i32;
                let section_y = ((section << 44) >> 44) as i32;
                let section_z = ((section << 22) >> 42) as i32;
                let count = wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("section blocks update"))?;
                for _ in 0..count {
                    let packed = wire::read_varlong(&mut input)
                        .map_err(|_| ConnectionError::Codec("section blocks update"))?;
                    let state = (packed as u64 >> 12) as i32;
                    let local = (packed & 0xfff) as i32;
                    let (bx, by, bz) = (
                        section_x * 16 + ((local >> 8) & 15),
                        section_y * 16 + (local & 15),
                        section_z * 16 + ((local >> 4) & 15),
                    );
                    self.mark_block(bx, by, bz);
                    self.trace.world.apply_server_block_state(bx, by, bz, state);
                }
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                let radius = parse_set_chunk_cache_radius_packet(payload)?;
                self.trace.world.update_chunk_cache_radius(radius);
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                let (x, z) = parse_set_chunk_cache_center_packet(payload)?;
                self.trace.world.update_chunk_cache_center(x, z);
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                self.send(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, |output| {
                    // The chunk batch calculator's starting estimate.
                    output.extend_from_slice(&3.5_f32.to_be_bytes());
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                self.send(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, |output| {
                    crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_PING_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundPingPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play ping"))?;
                self.send(PLAY_SERVERBOUND_PONG_PACKET_ID, |output| {
                    crate::common::ServerboundPongPacket::new(packet.id).encode(output);
                    Ok(())
                })?;
            }
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => {
                // Data of a kind this client cannot read leaves it as it was.
                if let Some(packet) =
                    parsed(parse_entity_metadata_packet(payload), "entity_metadata")
                {
                    entities::metadata(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID => {
                if let Some(packet) =
                    parsed(parse_entity_equipment_packet(payload), "entity_equipment")
                {
                    entities::equipment(&mut self.trace.world, &packet);
                }
            }
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => {
                let packet = parse_update_attributes_packet(payload)?;
                let world = &mut self.trace.world;
                if !world.apply_local_player_attributes(packet.entity_id, &packet.attributes) {
                    world
                        .entities_mut()
                        .apply_attributes(packet.entity_id, &packet.attributes);
                }
            }
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => {
                let packet = parse_update_mob_effect_packet(payload)?;
                self.trace.world.apply_local_player_mob_effect(
                    packet.entity_id,
                    packet.effect_registry_id,
                    packet.amplifier,
                    packet.duration_ticks,
                    packet.flags,
                );
            }
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => {
                let packet = parse_remove_mob_effect_packet(payload)?;
                self.trace
                    .world
                    .remove_local_player_mob_effect(packet.entity_id, packet.effect_registry_id);
            }
            PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID => {
                let packet = parse_set_health_packet(payload)?;
                self.health_received = true;
                if let Some(player) = self.player.as_mut() {
                    player.hurt_to(packet.health);
                }
                self.status.health = packet.health;
                self.status.food = packet.food;
                self.status.saturation = packet.saturation;
            }
            PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID => {
                let mut input = payload;
                let event = read_level_event_packet(&mut input)?;
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("level event trailing payload"));
                }
                self.trace
                    .world
                    .apply_level_event(event.event, event.pos, event.data);
            }
            PLAY_CLIENTBOUND_SOUND_PACKET_ID
            | PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID
            | PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID => {
                let mut input = payload;
                let command = match packet_id {
                    PLAY_CLIENTBOUND_SOUND_PACKET_ID => {
                        voxel_game::sounds::SoundCommand::At(read_sound_packet(&mut input)?)
                    }
                    PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID => {
                        voxel_game::sounds::SoundCommand::FollowingEntity(read_sound_entity_packet(
                            &mut input,
                        )?)
                    }
                    _ => {
                        voxel_game::sounds::SoundCommand::Stop(read_stop_sound_packet(&mut input)?)
                    }
                };
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("sound packet trailing payload"));
                }
                self.sounds.push(command);
            }
            PLAY_CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID => {
                let (id, message) = parse_player_combat_kill_packet(payload)?;
                if self.player_id == Some(id) {
                    if self.show_death_screen {
                        self.death = Some(message);
                    } else {
                        self.respawn()?;
                    }
                }
            }
            PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID => {
                let mut input = payload;
                let progress = read_f32(&mut input)?;
                if progress != self.status.experience_progress {
                    // `setExperienceDisplayStartTickToTickCount`: the first
                    // change, the one a login brings, never shows.
                    let tick_count = self.player.as_ref().map_or(0, LocalPlayer::tick_count);
                    self.status.experience_display_start_tick =
                        if self.status.experience_display_start_tick == i32::MIN {
                            -i32::MAX
                        } else {
                            tick_count
                        };
                }
                self.status.experience_progress = progress;
                self.status.experience_level = read_varint(&mut input, "experience level")?;
                self.status.total_experience = read_varint(&mut input, "total experience")?;
            }
            PLAY_CLIENTBOUND_SET_HELD_SLOT_PACKET_ID => {
                let mut input = payload;
                let slot = read_varint(&mut input, "held slot")?;
                // `Inventory.isHotbarSlot`: anything else is ignored.
                if (0..9).contains(&slot) {
                    self.status.selected_slot = slot;
                }
            }
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => {
                let event = parse_game_event_packet(payload)?;
                // The weather events set the level's rain and thunder, both
                // the current and the previous tick's value, clamped.
                match event.event_id {
                    1 => self.rain_level = 0.0,
                    2 => self.rain_level = 1.0,
                    7 => self.rain_level = event.param.clamp(0.0, 1.0),
                    8 => self.thunder_level = event.param.clamp(0.0, 1.0),
                    _ => {}
                }
                self.trace
                    .world
                    .set_weather(self.rain_level, self.thunder_level);
                if event.event_id == GAME_EVENT_IMMEDIATE_RESPAWN_ID {
                    self.show_death_screen = event.param == 0.0;
                }
                if event.event_id == GAME_EVENT_CHANGE_GAME_MODE_ID {
                    self.game_mode = ClientPlayerGameMode::from_game_event_param(event.param);
                    if let Some(player) = self.player.as_mut() {
                        player.set_game_mode(self.game_mode);
                    }
                }
            }
            PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID => {
                let packet = parse_player_abilities_packet(payload)?;
                let abilities = ClientPlayerAbilities::new(
                    packet.invulnerable,
                    packet.flying,
                    packet.can_fly,
                    packet.can_instabuild,
                    packet.flying_speed,
                    packet.walking_speed,
                );
                self.abilities = Some(abilities);
                if let Some(player) = self.player.as_mut() {
                    player.set_abilities(abilities);
                }
            }
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => {
                let packet = parse_player_position_packet(payload)?;
                self.teleports += 1;
                self.handle_player_position(packet)?;
                self.trace.world.prediction_teleport();
            }
            PLAY_CLIENTBOUND_SET_TIME_PACKET_ID => {
                let mut input = payload;
                let game_time = read_i64(&mut input)?;
                let count = wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("set time"))?;
                self.trace.world.set_game_time(game_time as u64);
                // handleUpdates: the clocks catch up first, then take the
                // server's values.
                self.tick_clocks(game_time);
                for _ in 0..count {
                    let clock = wire::read_varint(&mut input)
                        .map_err(|_| ConnectionError::Codec("set time clock"))?;
                    let total_ticks = wire::read_varlong(&mut input)
                        .map_err(|_| ConnectionError::Codec("set time ticks"))?;
                    let partial_tick = read_f32(&mut input)?;
                    let rate = read_f32(&mut input)?;
                    self.clocks.insert(
                        clock,
                        ClockState {
                            total_ticks,
                            partial_tick,
                            rate,
                        },
                    );
                }
            }
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => {
                let sequence = read_varint(&mut &payload[..], "block changed ack sequence")?;
                for synced in self.trace.world.handle_block_changed_ack(sequence) {
                    self.mark_block(synced.x, synced.y, synced.z);
                    // `syncBlockState`: a block put back into the player
                    // moves it back to where it stood when it predicted.
                    let pos = voxel_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
                    if let Some(position) = synced.player_position
                        && let Some(player) = self.player.as_mut()
                        && player.is_colliding(&self.trace.world, pos, synced.state)
                    {
                        player.abs_snap_to(position);
                    }
                }
            }
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => {
                let packet = parse_block_destruction_packet(payload)?;
                self.trace.world.destroy_block_progress(
                    packet.entity_id,
                    packet.x,
                    packet.y,
                    packet.z,
                    i32::from(packet.progress),
                );
            }
            PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID => {
                let packet = parse_open_screen_packet(payload)?;
                self.trace.world.apply_local_player_open_menu(
                    packet.container_id,
                    packet.menu_type_id,
                    &packet.title_nbt,
                );
            }
            PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let packet = parse_container_close_packet(payload)?;
                self.trace
                    .world
                    .apply_local_player_container_close(packet.container_id);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                let packet = match parse_container_set_content_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace.world.apply_local_player_container_content(
                    packet.container_id,
                    packet.state_id,
                    &packet.items,
                    packet.carried_item.as_ref(),
                );
            }
            PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID => {
                let entity = parse_block_entity_data_packet(payload)?;
                self.trace
                    .world
                    .apply_block_entity_update(BlockEntityInput {
                        x: entity.x,
                        y: entity.y,
                        z: entity.z,
                        type_id: entity.type_id,
                        tag: entity.tag.as_deref(),
                    });
            }
            PLAY_CLIENTBOUND_BLOCK_EVENT_PACKET_ID => {
                let (pos, event, data) = parse_block_event_packet(payload)?;
                self.trace.world.apply_chest_block_event(pos, event, data);
            }
            PLAY_CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID => {
                let packet = parse_initialize_border_packet(payload)?;
                self.trace.world.apply_world_border_initialize(packet);
            }
            PLAY_CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID => {
                let mut input = payload;
                let x = read_f64(&mut input, "set border center")?;
                let z = read_f64(&mut input, "set border center")?;
                self.trace.world.apply_world_border_center(x, z);
            }
            PLAY_CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID => {
                let (old_size, new_size, time) = parse_set_border_lerp_size_packet(payload)?;
                self.trace
                    .world
                    .apply_world_border_lerp_size(old_size, new_size, time);
            }
            PLAY_CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID => {
                let mut input = payload;
                let size = read_f64(&mut input, "set border size")?;
                self.trace.world.apply_world_border_size(size);
            }
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID => {
                let mut input = payload;
                let time = read_varint(&mut input, "set border warning delay")?;
                self.trace.world.apply_world_border_warning_time(time);
            }
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID => {
                let mut input = payload;
                let blocks = read_varint(&mut input, "set border warning distance")?;
                self.trace.world.apply_world_border_warning_blocks(blocks);
            }
            PLAY_CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID => {
                let (container_id, state) = parse_merchant_offers_packet(payload)?;
                self.merchant = Some((container_id, state));
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID => {
                let packet = parse_container_set_data_packet(payload)?;
                self.trace.world.apply_local_player_container_data(
                    packet.container_id,
                    packet.id,
                    packet.value,
                );
            }
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => match parse_system_chat_event(payload) {
                Ok(event) => self.chat_events.push(event),
                Err(error) => self.warn(format!("system chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_DISGUISED_CHAT_PACKET_ID => {
                match parse_disguised_chat_event(payload) {
                    Ok(event) => self.chat_events.push(event),
                    Err(error) => self.warn(format!("disguised chat: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_CHAT_PACKET_ID => match parse_player_chat_packet(payload) {
                Ok(packet) => self.receive_player_chat(packet),
                Err(error) => self.warn(format!("player chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_DELETE_CHAT_PACKET_ID => match parse_delete_chat_packet(payload) {
                Ok(packed) => match self.signature_cache.unpack(&packed) {
                    Some(signature) => self.chat_events.push(ChatEvent::Delete(signature)),
                    // The reference drops the connection over a number it
                    // does not know.
                    None => self.warn("delete chat: unknown signature".to_owned()),
                },
                Err(error) => self.warn(format!("delete chat: {error:?}")),
            },
            PLAY_CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID => {
                match parse_command_suggestions(payload) {
                    Ok(suggestions) => self.suggestions = Some(suggestions),
                    Err(error) => self.warn(format!("suggestions: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID
            | PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID
            | PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID
            | PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => match parse_hud_packet(packet_id, payload) {
                Ok(Some(event)) => self.hud_events.push(event),
                Ok(None) => {}
                Err(error) => self.warn(format!("hud packet {packet_id}: {error:?}")),
            },
            PLAY_CLIENTBOUND_WAYPOINT_PACKET_ID => match parse_waypoint_packet(payload) {
                Ok((operation, waypoint)) => self.waypoints.apply(operation, waypoint),
                Err(error) => self.warn(format!("waypoint: {error:?}")),
            },
            PLAY_CLIENTBOUND_COMMANDS_PACKET_ID => match parse_commands_packet(payload) {
                Ok(tree) => self.commands = Some(std::sync::Arc::new(tree)),
                Err(error) => self.warn(format!("commands: {error:?}")),
            },
            PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
                match parse_scoreboard_packet(packet_id, payload) {
                    Ok(Some(event)) => self.scoreboard.apply(event),
                    Ok(None) => {}
                    Err(error) => self.warn(format!("scoreboard packet {packet_id}: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID => {
                match parse_player_info_update(payload) {
                    Ok(entries) => {
                        for (id, actions) in entries {
                            self.apply_player_info(id, actions);
                        }
                    }
                    Err(error) => self.warn(format!("player info: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID => {
                match parse_player_info_remove(payload) {
                    Ok(ids) => {
                        for id in ids {
                            self.player_infos.remove(&id);
                        }
                    }
                    Err(error) => self.warn(format!("player info remove: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID => {
                match parse_recipe_book_add_packet(payload) {
                    Ok((entries, replace)) => {
                        self.trace.world.recipe_book_mut().add(entries, replace);
                    }
                    Err(error) => self.warn(format!("recipe book add: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID => {
                match parse_recipe_book_remove_packet(payload) {
                    Ok(ids) => self.trace.world.recipe_book_mut().remove(&ids),
                    Err(error) => self.warn(format!("recipe book remove: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID => {
                match parse_recipe_book_settings_packet(payload) {
                    Ok(settings) => self.trace.world.recipe_book_mut().set_settings(settings),
                    Err(error) => self.warn(format!("recipe book settings: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID => {
                match parse_place_ghost_recipe_packet(payload) {
                    Ok(ghost) => self.trace.world.recipe_book_mut().ghost = Some(ghost),
                    Err(error) => self.warn(format!("place ghost recipe: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_UPDATE_RECIPES_PACKET_ID => {
                match parse_update_recipes_packet(payload) {
                    Ok(update) => {
                        self.trace.world.apply_recipe_property_sets(update.sets);
                        self.trace
                            .world
                            .apply_stonecutter_recipes(update.stonecutter);
                    }
                    Err(error) => self.warn(format!("recipes: {error:?}")),
                }
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => {
                let packet = match parse_container_set_slot_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace.world.apply_local_player_container_slot(
                    packet.container_id,
                    packet.state_id,
                    packet.slot,
                    packet.item.as_ref(),
                );
            }
            PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID => {
                let packet = match parse_cursor_item_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace
                    .world
                    .apply_local_player_carried_item(packet.item.as_ref());
            }
            PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID => {
                let packet = match parse_set_player_inventory_packet(payload) {
                    Ok(packet) => packet,
                    // An item with a component the parser does not know
                    // leaves that stack unknown rather than the session.
                    Err(error) => {
                        self.warn(format!("inventory: {error:?}"));
                        return self.maybe_finish_loading();
                    }
                };
                self.trace
                    .world
                    .apply_local_player_inventory_slot(packet.slot, packet.item.as_ref());
            }
            PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID => {
                let mut input = payload;
                let packet = voxel_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("update tags"))?;
                self.trace.world.apply_update_tags_packet(&packet);
            }
            PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID => {
                let mut input = payload;
                self.disconnect = Some(read_disconnect_reason(&mut input)?);
            }
            _ => {}
        }
        self.maybe_finish_loading()
    }
}
