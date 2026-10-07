use super::*;

impl PlaySession {
    /// Log in, configure, and start reading play packets in the background.
    pub fn connect(
        address: SocketAddr,
        username: &str,
        timeout: Duration,
        auto_jump: bool,
    ) -> Result<Self, ConnectionError> {
        let mut trace = PlayConnectionTrace::default();
        let (stream, compression_threshold) =
            open_play_connection(address, username, timeout, true, &mut trace)?;
        stream.set_read_timeout(None)?;
        let mut reader = stream.try_clone()?;
        let (sender, incoming) = mpsc::channel();
        std::thread::spawn(move || {
            loop {
                let packet = read_packet(&mut reader, compression_threshold).and_then(classify);
                let failed = packet.is_err();
                if sender.send(packet).is_err() || failed {
                    return;
                }
            }
        });
        Ok(Self::assemble(
            stream,
            compression_threshold,
            incoming,
            trace,
            (username, auto_jump),
        ))
    }

    /// A session over a connection already made, with nothing yet known.
    pub(in super::super) fn assemble(
        writer: TcpStream,
        compression_threshold: Option<usize>,
        incoming: Receiver<Result<Incoming, ConnectionError>>,
        trace: PlayConnectionTrace,
        (username, auto_jump): (&str, bool),
    ) -> Self {
        Self {
            writer,
            compression_threshold,
            incoming,
            trace,
            player_id: None,
            game_mode: ClientPlayerGameMode::Survival,
            abilities: None,
            player: None,
            loaded: false,
            auto_jump,
            warned: None,
            teleports: 0,
            view_distance: 0,
            dimension_type_id: 0,
            hashed_seed: 0,
            game_time: 0,
            rain_level: 0.0,
            thunder_level: 0.0,
            status: PlayerStatus::default(),
            show_death_screen: true,
            disconnect: None,
            death: None,
            swings: 0,
            sounds: Vec::new(),
            aimed_entity: None,
            hardcore: false,
            health_received: false,
            level_reset: false,
            sent_slot: 0,
            game_mode_state: voxel_game::game_mode::GameMode::default(),
            container_menu: voxel_game::container_menu::ContainerMenu::default(),
            open_menu: None,
            merchant: None,
            chat_events: Vec::new(),
            signature_cache: SignatureCache::default(),
            hud_events: Vec::new(),
            removed_player_vehicle: None,
            scoreboard: voxel_game::scoreboard::Scoreboard::default(),
            waypoints: voxel_game::waypoint::WaypointManager::default(),
            permission_level: 0,
            commands: None,
            username: username.to_owned(),
            suggestions: None,
            player_infos: std::collections::BTreeMap::new(),
            next_chat_index: 0,
            drop_spam: 0,
            dirty_sections: std::collections::BTreeSet::new(),
            forgotten_chunks: Vec::new(),
            clocks: std::collections::BTreeMap::new(),
        }
    }

    pub fn world(&self) -> &ClientWorld {
        &self.trace.world
    }

    /// The sounds the world's events and the player's own blocks made since the last call.
    pub fn take_world_sounds(&mut self) -> Vec<voxel_world::local_player::LocalSound> {
        self.trace.world.take_sounds()
    }

    /// Where an entity is drawn between two ticks.
    pub fn entity_snapshot(
        &self,
        entity_id: i32,
        partial_tick: f32,
    ) -> Option<voxel_world::EntitySnapshot> {
        self.trace.world.entity_snapshot(entity_id, partial_tick)
    }

    /// How many of the entities tick (those in a loaded chunk).
    pub fn ticking_entity_count(&self) -> usize {
        self.trace
            .world
            .entities()
            .filter(|entity| entity.is_ticking())
            .count()
    }

    /// How many entities the world holds besides the player.
    pub fn entity_count(&self) -> usize {
        self.trace.world.entities().count()
    }

    pub fn player(&self) -> Option<&LocalPlayer> {
        self.player.as_ref()
    }

    pub fn player_mut(&mut self) -> Option<&mut LocalPlayer> {
        self.player.as_mut()
    }

    /// The registry id of what the off hand holds (air when empty).
    pub(in super::super) fn off_hand_item(&self) -> i32 {
        self.trace
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(45))
            .map_or(0, |stack| stack.item_registry_id())
    }

    /// The mouse turns the player (`Entity.turn`), who in a boat is then held within a quarter
    /// turn of where the boat faces (`onPassengerTurned`).
    pub fn turn_player(&mut self, y_rot_delta: f64, x_rot_delta: f64) {
        let boat_y_rot = self
            .trace
            .world
            .ridden_vehicle_of_local_player()
            .and_then(|id| self.trace.world.vehicle_view(id))
            .filter(|view| view.kind == voxel_world::VehicleKind::Boat)
            .map(|view| view.y_rot);
        if let Some(player) = self.player.as_mut() {
            player.turn(y_rot_delta, x_rot_delta);
            if let Some(y_rot) = boat_y_rot {
                player.clamp_in_boat(y_rot);
            }
        }
    }

    /// Whether the level is ready and the player is being ticked.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// How many times the server has placed the player: once on joining,
    /// and once more for every movement it refused.
    pub fn teleports(&self) -> u32 {
        self.teleports
    }

    /// The server's view distance in chunks, from the login packet.
    pub fn view_distance(&self) -> i32 {
        self.view_distance
    }

    /// Sections whose drawing may have changed since the last call: their
    /// blocks, their light, or a neighbour they cull or shade against.
    pub fn take_dirty_sections(&mut self) -> Vec<(i32, i32, i32)> {
        std::mem::take(&mut self.dirty_sections)
            .into_iter()
            .collect()
    }

    /// Chunks the server unloaded since the last call.
    pub fn take_forgotten_chunks(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.forgotten_chunks)
    }

    pub(in super::super) fn section_range(&self) -> std::ops::Range<i32> {
        let dimension = self.trace.world.dimension();
        let low = dimension.min_y.div_euclid(16);
        low..low + dimension.height / 16
    }

    /// Mark a chunk column and every column around it.
    pub(in super::super) fn mark_columns_around(&mut self, chunk_x: i32, chunk_z: i32) {
        for section_y in self.section_range() {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    self.dirty_sections
                        .insert((chunk_x + dx, section_y, chunk_z + dz));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: queued light tasks
    /// and the engine run, and the sections whose light changed redraw.
    pub fn update_light(&mut self) {
        for section in self.trace.world.update_light() {
            self.dirty_sections.insert(section);
        }
    }

    /// Mark the section holding a block and the sections touching it.
    pub(in super::super) fn mark_block(&mut self, x: i32, y: i32, z: i32) {
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                for dz in [-1, 0, 1] {
                    self.dirty_sections.insert((
                        (x + dx).div_euclid(16),
                        (y + dy).div_euclid(16),
                        (z + dz).div_euclid(16),
                    ));
                }
            }
        }
    }

    /// The hashed world seed the biome zoom is jittered with.
    pub fn hashed_seed(&self) -> i64 {
        self.hashed_seed
    }

    /// The current dimension type's registry name.
    pub fn dimension_type(&self) -> Option<&str> {
        self.registry_names("minecraft:dimension_type")
            .get(usize::try_from(self.dimension_type_id).ok()?)
            .map(String::as_str)
    }

    /// The entry names of one synchronised registry, in network order.
    pub fn registry_names(&self, registry: &str) -> &[String] {
        self.trace
            .registry_entry_names
            .get(registry)
            .map_or(&[], Vec::as_slice)
    }

    /// The level's game time, as the server last set it and the ticks since.
    pub fn game_time(&self) -> i64 {
        self.game_time
    }

    /// `LocalPlayer.swing` with the main hand: the arm swings, and the
    /// server hears of it.
    pub fn swing(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.player.as_mut() else {
            return Ok(());
        };
        player.swing();
        self.swings += 1;
        self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
            wire::write_varint(0, output);
            Ok(())
        })
    }

    /// Whether the level is hardcore.
    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    /// The player's game mode, as the server last set it.
    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.game_mode
    }

    /// The sound packets that arrived since the last call, in order (`handleSoundEvent`,
    /// `handleSoundEntityEvent`, `handleStopSoundEvent`): the sound engine's to carry out.
    pub fn take_sounds(&mut self) -> Vec<voxel_game::sounds::SoundCommand> {
        std::mem::take(&mut self.sounds)
    }

    /// How many swings the session has sent.
    pub fn swings(&self) -> u32 {
        self.swings
    }

    /// The player's profile id, as the server logged it in.
    pub fn profile_id(&self) -> Option<voxel_core::binary::Uuid> {
        self.trace.profile_id
    }

    /// Whether the server replaced the level since the last call, so
    /// everything drawn from the old one should go.
    pub fn take_level_reset(&mut self) -> bool {
        std::mem::take(&mut self.level_reset)
    }

    /// What the HUD shows of the player.
    pub fn status(&self) -> &PlayerStatus {
        &self.status
    }

    /// Select a hotbar slot, 0 to 8; the server hears of it next tick.
    pub fn select_slot(&mut self, slot: i32) {
        if (0..9).contains(&slot) {
            self.status.selected_slot = slot;
        }
    }

    /// `Level.getRainLevel`: how hard it rains, 0 to 1.
    pub fn rain_level(&self) -> f32 {
        self.rain_level
    }

    /// `Level.getThunderLevel`: the thunder level scaled by the rain.
    pub fn thunder_level(&self) -> f32 {
        self.thunder_level * self.rain_level
    }

    /// A world clock's total ticks, by its registry name.
    pub fn clock_ticks(&self, clock: &str) -> Option<i64> {
        let id = self
            .registry_names("minecraft:world_clock")
            .iter()
            .position(|name| name == clock)?;
        self.clocks.get(&(id as i32)).map(|state| state.total_ticks)
    }

    /// `ClientClockManager.tick`: every clock runs at its own rate.
    pub(in super::super) fn tick_clocks(&mut self, game_time: i64) {
        let delta = game_time - self.game_time;
        self.game_time = game_time;
        for state in self.clocks.values_mut() {
            let ticks = f64::from(state.partial_tick) + delta as f64 * f64::from(state.rate);
            let whole = ticks.floor();
            state.partial_tick = (ticks - whole) as f32;
            state.total_ticks += whole as i64;
        }
    }

    /// The first unported movement branch the player reached, if any.
    pub fn warning(&self) -> Option<&str> {
        self.warned.as_deref()
    }

    /// Handle every packet that has arrived, without waiting for more.
    pub fn drain(&mut self) -> Result<(), ConnectionError> {
        loop {
            if self.disconnect.is_some() {
                return Ok(());
            }
            match self.incoming.try_recv() {
                Ok(Ok(packet)) => self.handle_incoming(packet)?,
                // `ClientCommonPacketListenerImpl.onDisconnect`: a failure to
                // read ends the connection with what went wrong.
                Ok(Err(ConnectionError::Io(error))) => {
                    self.disconnect = Some(voxel_game::text::Component::translatable_with(
                        "disconnect.genericReason",
                        vec![voxel_game::text::Component::literal(format!(
                            "Internal Exception: {error}"
                        ))],
                    ));
                }
                Ok(Err(error)) => return Err(error),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    self.disconnect = Some(voxel_game::text::Component::translatable(
                        "disconnect.endOfStream",
                    ));
                }
            }
        }
    }

    /// Why the connection is over, once it is.
    pub fn disconnected(&self) -> Option<&voxel_game::text::Component> {
        self.disconnect.as_ref()
    }

    /// Wait up to `limit` for the next packet and handle it.
    pub fn wait(&mut self, limit: Duration) -> Result<(), ConnectionError> {
        match self.incoming.recv_timeout(limit) {
            Ok(packet) => self.handle_incoming(packet?),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ConnectionError::ServerDisconnected),
        }
    }

    pub(in super::super) fn warn(&mut self, message: String) {
        if self.warned.as_deref() != Some(message.as_str()) {
            log::warn!("{message}");
            self.warned = Some(message);
        }
    }

    /// `MultiPlayerGameMode.ensureHasSentCarriedItem`: the server hears of
    /// a new hotbar slot before anything the slot's item does.
    pub(in super::super) fn ensure_has_sent_carried_item(&mut self) -> Result<(), ConnectionError> {
        if self.status.selected_slot != self.sent_slot {
            self.sent_slot = self.status.selected_slot;
            let slot = self.sent_slot as i16;
            self.send(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, |output| {
                output.extend_from_slice(&slot.to_be_bytes());
                Ok(())
            })?;
        }
        Ok(())
    }

    /// The breaking state, for drawing the crack and reporting.
    pub fn game_mode_state(&self) -> &voxel_game::game_mode::GameMode {
        &self.game_mode_state
    }

    /// The player's inventory menu as the client holds it now.
    pub fn container_menu(&mut self) -> &voxel_game::container_menu::ContainerMenu {
        if let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() {
            self.container_menu.load(slots, carried, state_id);
        }
        &self.container_menu
    }

    /// The container the server opened, as a screen needs to know it: its id,
    /// the menu it is and its title as NBT. `None` with none open, or one of
    /// a kind not modelled yet.
    pub fn open_container_info(
        &self,
    ) -> Option<(i32, voxel_game::container_menu::MenuKind, Vec<u8>)> {
        let container = self.trace.world.local_player()?.open_container()?;
        let kind = voxel_game::container_menu::MenuKind::from_menu_type(container.menu_type_id())?;
        // A screen opens once the server has sent what is in the container.
        self.trace.world.local_player_container_menu()?;
        Some((
            container.container_id(),
            kind,
            container.title_nbt().to_vec(),
        ))
    }
}
