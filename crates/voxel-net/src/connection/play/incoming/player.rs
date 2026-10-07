use super::*;

impl PlaySession {
    pub(super) fn on_login(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let login = parse_login_packet(payload)?;
        self.next_chat_index = 0;
        self.trace
            .world
            .set_server_simulation_distance(login.simulation_distance);
        let bounds = dimension_bounds_for_spawn_info(&self.trace, &login.spawn_info);
        let environment = dimension_environment_for_spawn_info(&self.trace, &login.spawn_info);
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
        self.game_mode = ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
        Ok(())
    }

    pub(super) fn on_respawn(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
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
        Ok(())
    }

    pub(super) fn on_set_health(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_set_health_packet(payload)?;
        self.health_received = true;
        if let Some(player) = self.player.as_mut() {
            player.hurt_to(packet.health);
        }
        self.status.health = packet.health;
        self.status.food = packet.food;
        self.status.saturation = packet.saturation;
        Ok(())
    }

    pub(super) fn on_player_combat_kill(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let (id, message) = parse_player_combat_kill_packet(payload)?;
        if self.player_id == Some(id) {
            if self.show_death_screen {
                self.death = Some(message);
            } else {
                self.respawn()?;
            }
        }
        Ok(())
    }

    pub(super) fn on_set_experience(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
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
        Ok(())
    }

    pub(super) fn on_set_held_slot(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let slot = read_varint(&mut input, "held slot")?;
        // `Inventory.isHotbarSlot`: anything else is ignored.
        if (0..9).contains(&slot) {
            self.status.selected_slot = slot;
        }
        Ok(())
    }

    pub(super) fn on_game_event(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
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
        Ok(())
    }

    pub(super) fn on_player_abilities(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
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
        Ok(())
    }

    pub(super) fn on_player_position(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_player_position_packet(payload)?;
        self.teleports += 1;
        self.handle_player_position(packet)?;
        self.trace.world.prediction_teleport();
        Ok(())
    }

    pub(super) fn on_player_info_update(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_player_info_update(payload) {
            Ok(entries) => {
                for (id, actions) in entries {
                    self.apply_player_info(id, actions);
                }
            }
            Err(error) => self.warn(format!("player info: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_player_info_remove(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_player_info_remove(payload) {
            Ok(ids) => {
                for id in ids {
                    self.player_infos.remove(&id);
                }
            }
            Err(error) => self.warn(format!("player info remove: {error:?}")),
        }
        Ok(())
    }
}
