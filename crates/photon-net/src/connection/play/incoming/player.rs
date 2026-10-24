use super::*;
use photon_core::ids::EntityId;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (PLAY_CLIENTBOUND_LOGIN_PACKET_ID, typed::<Login>),
    (PLAY_CLIENTBOUND_RESPAWN_PACKET_ID, typed::<Respawn>),
    (PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID, typed::<SetHealth>),
    (
        PLAY_CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID,
        typed::<PlayerCombatKill>,
    ),
    (
        PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        typed::<SetExperience>,
    ),
    (
        PLAY_CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
        typed::<SetHeldSlot>,
    ),
    (PLAY_CLIENTBOUND_AWARD_STATS_PACKET_ID, typed::<AwardStats>),
    (
        PLAY_CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID,
        typed::<photon_game::advancements::UpdateAdvancements>,
    ),
    (
        PLAY_CLIENTBOUND_SELECT_ADVANCEMENTS_TAB_PACKET_ID,
        typed::<SelectAdvancementsTab>,
    ),
    (
        PLAY_CLIENTBOUND_TICKING_STATE_PACKET_ID,
        typed::<TickingState>,
    ),
    (
        PLAY_CLIENTBOUND_TICKING_STEP_PACKET_ID,
        typed::<TickingStep>,
    ),
    (PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID, typed::<GameEvent>),
    (
        PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
        typed::<PlayerAbilities>,
    ),
    (
        PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        typed::<PlayerPosition>,
    ),
    (
        PLAY_CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID,
        typed::<PlayerInfoUpdate>,
    ),
    (
        PLAY_CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID,
        typed::<PlayerInfoRemove>,
    ),
];

/// `handleLogin`.
impl ClientboundPacket for Login {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_login(self.0)
    }
}

/// `handleRespawn`.
impl ClientboundPacket for Respawn {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_respawn(self.0)
    }
}

/// `handleSetHealth`.
impl ClientboundPacket for SetHealth {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_health(self.0)
    }
}

/// `handlePlayerCombatKill`.
impl ClientboundPacket for PlayerCombatKill {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_player_combat_kill(self.0)
    }
}

/// `handleSetExperience`.
impl ClientboundPacket for SetExperience {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_experience(*self)
    }
}

/// `handleSetHeldSlot`.
impl ClientboundPacket for SetHeldSlot {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_held_slot(self.0)
    }
}

/// `handleAwardStats`: each statistic is set to the number the server says.
impl ClientboundPacket for AwardStats {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        for (key, amount) in self.0 {
            session.local.stats.insert(key, amount);
        }
        session.local.stats_updated = true;
        Ok(())
    }
}

/// `handleUpdateAdvancementsPacket`: the copy of the tree changes, and what was done shows a toast.
impl ClientboundPacket for photon_game::advancements::UpdateAdvancements {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let updated = session.local.advancements.update(&self);
        let toasts = updated
            .toasts
            .into_iter()
            .map(|(_, display)| photon_game::toast::PendingToast::Advancement(Box::new(display)));
        session.local.pending_toasts.extend(toasts);
        Ok(())
    }
}

/// `handleSelectAdvancementsTab`: the screen is told which tab to show; a tab that is not in the
/// tree is none.
impl ClientboundPacket for SelectAdvancementsTab {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let advancements = &mut session.local.advancements;
        let known = self
            .0
            .as_deref()
            .filter(|id| advancements.get(id).is_some());
        advancements.select_tab(known);
        Ok(())
    }
}

/// `handleTickingState`.
impl ClientboundPacket for TickingState {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.level_info.tick_rate.set_tick_rate(self.tick_rate);
        session.level_info.tick_rate.set_frozen(self.frozen);
        Ok(())
    }
}

/// `handleTickingStep`.
impl ClientboundPacket for TickingStep {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.level_info.tick_rate.set_frozen_ticks_to_run(self.0);
        Ok(())
    }
}

/// `handleGameEvent`.
impl ClientboundPacket for GameEvent {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_game_event(self.0)
    }
}

/// `handlePlayerAbilities`.
impl ClientboundPacket for PlayerAbilities {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_player_abilities(self.0)
    }
}

/// `handleMovePlayer`.
impl ClientboundPacket for PlayerPosition {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_player_position(self.0)
    }
}

impl ClientboundPacket for PlayerInfoUpdate {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        for (id, actions) in self.0 {
            session.apply_player_info(id, actions);
        }
        Ok(())
    }
}

impl ClientboundPacket for PlayerInfoRemove {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        for id in self.0 {
            session.social.player_infos.remove(&id);
        }
        Ok(())
    }
}

/// `CommonPlayerSpawnInfo.lastDeathLocation`.
fn last_death_of(info: &SpawnInfoObservation) -> Option<GlobalBlock> {
    info.last_death_location
        .as_ref()
        .map(|death| GlobalBlock::from_packed(death.dimension.clone(), death.position))
}

impl PlaySession {
    fn on_login(&mut self, login: LoginObservation) -> Result<(), ConnectionError> {
        self.social.next_chat_index = 0;
        self.world
            .set_server_simulation_distance(login.simulation_distance);
        let bounds = self.state.dimension_bounds(&login.spawn_info);
        let environment = self.state.dimension_environment(&login.spawn_info);
        self.world.replace_dimension_with_environment(
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
        self.world.configure_chunk_cache(login.chunk_radius);
        self.level_info.view_distance = login.chunk_radius;
        self.level_info.dimension = login.spawn_info.dimension.clone();
        self.level_info.respawn = None;
        self.local.last_death = last_death_of(&login.spawn_info);
        self.level_info.dimension_type_id = login.spawn_info.dimension_type_id;
        self.level_info.hashed_seed = login.spawn_info.seed;
        self.world.set_local_player_id(login.player_id);
        if !login.enforces_secure_chat && !self.effects.seen_unsecure_warning {
            self.effects.unsecure_warning = true;
            self.effects.seen_unsecure_warning = true;
        }
        self.local.hardcore = login.hardcore;
        self.local.reduced_debug_info = login.reduced_debug_info;
        // The advancements are the connection's: a login starts with none.
        self.local.advancements = photon_game::advancements::ClientAdvancements::default();
        self.local.pending_toasts.clear();
        // `handleLogin` gives the new player a new `StatsCounter`.
        self.local.stats.clear();
        self.local.show_death_screen = login.show_death_screen;
        self.local.player_id = Some(login.player_id);
        self.local.camera = None;
        self.local.permission_level = 0;
        self.local.game_mode = ClientPlayerGameMode::from_id(i32::from(login.spawn_info.game_type));
        self.local.previous_game_mode = login
            .spawn_info
            .previous_game_type
            .map(|id| ClientPlayerGameMode::from_id(i32::from(id)));
        self.local.loaded = false;
        self.start_waiting_for_level(LoadingReason::Other, bounds);
        Ok(())
    }

    fn on_respawn(
        &mut self,
        (spawn_info, data_to_keep): (SpawnInfoObservation, u8),
    ) -> Result<(), ConnectionError> {
        let reason = LoadingReason::of_respawn(
            self.local
                .player
                .as_ref()
                .is_some_and(|player| player.health() <= 0.0),
            &spawn_info.dimension,
            &self.level_info.dimension,
        );
        // A new level starts dry; the server repeats its weather.
        self.level_info.rain_level = 0.0;
        // The new level has a manager of its own.
        self.level_info.tick_rate = Default::default();
        self.level_info.thunder_level = 0.0;
        self.world.set_weather(0.0, 0.0);
        let bounds = self.state.dimension_bounds(&spawn_info);
        let environment = self.state.dimension_environment(&spawn_info);
        let swapped = self.world.replace_dimension_with_environment(
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
        self.local.game_mode = ClientPlayerGameMode::from_id(i32::from(spawn_info.game_type));
        self.local.previous_game_mode = spawn_info
            .previous_game_type
            .map(|id| ClientPlayerGameMode::from_id(i32::from(id)));
        // The new player starts with an empty inventory; the server
        // sends it again.
        self.world.recreate_local_player(data_to_keep);
        if swapped {
            // A new dimension is a new `ClientLevel` with no respawn data yet.
            self.level_info.respawn = None;
        }
        self.level_info.dimension = spawn_info.dimension.clone();
        self.local.last_death = last_death_of(&spawn_info);
        self.level_info.dimension_type_id = spawn_info.dimension_type_id;
        self.level_info.hashed_seed = spawn_info.seed;
        // A respawn makes a new player that waits for its position; the portal effect goes with it.
        self.local.respawn_portal = self
            .local
            .player
            .as_ref()
            .map(|player| (player.portal_carry(), spawn_info.portal_cooldown));
        self.local.player = None;
        self.local.camera = None;
        self.local.permission_level = 0;
        self.local.loaded = false;
        self.start_waiting_for_level(reason, bounds);
        self.level_info.level_reset = true;
        self.level_info.level_swapped |= swapped;
        self.local.health_received = false;
        Ok(())
    }

    fn on_set_health(&mut self, packet: HealthObservation) -> Result<(), ConnectionError> {
        self.local.health_received = true;
        if let Some(player) = self.local.player.as_mut() {
            player.hurt_to(packet.health);
        }
        self.local.status.health = packet.health;
        self.local.status.food = packet.food;
        self.local.status.saturation = packet.saturation;
        Ok(())
    }

    fn on_player_combat_kill(
        &mut self,
        (id, message): (EntityId, photon_game::text::Component),
    ) -> Result<(), ConnectionError> {
        if self.local.player_id == Some(id) {
            if self.local.show_death_screen {
                self.local.death = Some(message);
            } else {
                self.respawn()?;
            }
        }
        Ok(())
    }

    fn on_set_experience(&mut self, packet: SetExperience) -> Result<(), ConnectionError> {
        let progress = packet.progress;
        if progress != self.local.status.experience_progress {
            // `setExperienceDisplayStartTickToTickCount`: the first
            // change, the one a login brings, never shows.
            let tick_count = self
                .local
                .player
                .as_ref()
                .map_or(0, LocalPlayer::tick_count);
            self.local.status.experience_display_start_tick =
                if self.local.status.experience_display_start_tick == i32::MIN {
                    -i32::MAX
                } else {
                    tick_count
                };
        }
        self.local.status.experience_progress = progress;
        self.local.status.experience_level = packet.level;
        self.local.status.total_experience = packet.total;
        Ok(())
    }

    fn on_set_held_slot(&mut self, slot: i32) -> Result<(), ConnectionError> {
        // `Inventory.isHotbarSlot`: anything else is ignored.
        if (0..9).contains(&slot) {
            self.local.status.selected_slot = slot;
        }
        Ok(())
    }

    fn on_game_event(&mut self, event: GameEventPacket) -> Result<(), ConnectionError> {
        // The weather events set the level's rain and thunder, both
        // the current and the previous tick's value, clamped.
        match event.event_id {
            1 => self.level_info.rain_level = 0.0,
            2 => self.level_info.rain_level = 1.0,
            7 => self.level_info.rain_level = event.param.clamp(0.0, 1.0),
            8 => self.level_info.thunder_level = event.param.clamp(0.0, 1.0),
            _ => {}
        }
        self.world
            .set_weather(self.level_info.rain_level, self.level_info.thunder_level);
        self.note_loading_event(event.event_id);
        if event.event_id == GAME_EVENT_IMMEDIATE_RESPAWN_ID {
            self.local.show_death_screen = event.param == 0.0;
        }
        if event.event_id == GAME_EVENT_ELDER_GUARDIAN_ID {
            self.show_elder_guardian(event.param as i32);
        }
        if event.event_id == GAME_EVENT_WIN_GAME_ID {
            self.local.win_game = true;
        }
        if event.event_id == GAME_EVENT_CHANGE_GAME_MODE_ID {
            // `MultiPlayerGameMode.setLocalMode`: a change keeps the mode left as the previous.
            let mode = ClientPlayerGameMode::from_game_event_param(event.param);
            if mode != self.local.game_mode {
                self.local.previous_game_mode = Some(self.local.game_mode);
            }
            self.local.game_mode = mode;
            if let Some(player) = self.local.player.as_mut() {
                player.set_game_mode(self.local.game_mode);
            }
        }
        Ok(())
    }

    fn on_player_abilities(
        &mut self,
        packet: PlayerAbilitiesPacket,
    ) -> Result<(), ConnectionError> {
        let abilities = ClientPlayerAbilities::new(
            packet.invulnerable,
            packet.flying,
            packet.can_fly,
            packet.can_instabuild,
            packet.flying_speed,
            packet.walking_speed,
        );
        self.local.abilities = Some(abilities);
        if let Some(player) = self.local.player.as_mut() {
            player.set_abilities(abilities);
        }
        Ok(())
    }

    fn on_player_position(&mut self, packet: PlayerPositionPacket) -> Result<(), ConnectionError> {
        self.local.teleports += 1;
        self.handle_player_position(packet)?;
        self.world.prediction_teleport();
        Ok(())
    }

    /// `GUARDIAN_ELDER_EFFECT`: the elder guardian flashes over the screen, at the player, and
    /// with a `param` of 1 the curse is heard.
    fn show_elder_guardian(&mut self, param: i32) {
        let (Some(player), Some(type_id)) = (
            self.local.player.as_ref(),
            photon_data::particle_type::id("elder_guardian"),
        ) else {
            return;
        };
        let at = player.position();
        self.effects
            .level_particles
            .push(photon_world::particles::LevelParticles {
                type_id,
                data: photon_world::particles::ParticleData::Plain,
                // `ParticleTypes.ELDER_GUARDIAN` ignores the limiter.
                override_limiter: true,
                always_show: false,
                at: (at.x, at.y, at.z),
                spread: (0.0, 0.0, 0.0),
                max_speed: 0.0,
                count: 0,
            });
        if param == 1 {
            self.world
                .play_local_sound(photon_world::local_player::LocalSound {
                    relative: false,
                    delay_ticks: 0,
                    event: "minecraft:entity.elder_guardian.curse",
                    source: photon_world::local_player::LocalSoundSource::Hostile,
                    volume: 1.0,
                    pitch: 1.0,
                    position: (at.x, at.y, at.z),
                });
        }
    }
}
