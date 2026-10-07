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
    pub(super) fn assemble(
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
    pub(super) fn off_hand_item(&self) -> i32 {
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

    pub(super) fn section_range(&self) -> std::ops::Range<i32> {
        let dimension = self.trace.world.dimension();
        let low = dimension.min_y.div_euclid(16);
        low..low + dimension.height / 16
    }

    /// Mark a chunk column and every column around it.
    pub(super) fn mark_columns_around(&mut self, chunk_x: i32, chunk_z: i32) {
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
    pub(super) fn mark_block(&mut self, x: i32, y: i32, z: i32) {
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
    pub(super) fn tick_clocks(&mut self, game_time: i64) {
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

    pub(super) fn warn(&mut self, message: String) {
        if self.warned.as_deref() != Some(message.as_str()) {
            log::warn!("{message}");
            self.warned = Some(message);
        }
    }

    /// `MultiPlayerGameMode.ensureHasSentCarriedItem`: the server hears of
    /// a new hotbar slot before anything the slot's item does.
    pub(super) fn ensure_has_sent_carried_item(&mut self) -> Result<(), ConnectionError> {
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

    /// Whether the server has opened a container this client has no menu for.
    pub fn open_container_unsupported(&self) -> bool {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .is_some_and(|container| {
                voxel_game::container_menu::MenuKind::from_menu_type(container.menu_type_id())
                    .is_none()
            })
    }

    /// The menu an open screen is over: the container the server opened, or
    /// the player's own.
    pub fn active_menu(&mut self) -> &voxel_game::container_menu::ContainerMenu {
        if self.open_container_menu().is_some() {
            return self.open_menu.as_ref().expect("synced above");
        }
        self.container_menu()
    }

    /// The menu of the container the server opened, as the client holds it
    /// now; `None` until its contents have come.
    pub fn open_container_menu(&mut self) -> Option<&voxel_game::container_menu::ContainerMenu> {
        let voxel_world::OpenContainerMenu {
            container_id,
            menu_type_id,
            slots,
            carried,
            state_id,
        } = self.trace.world.local_player_container_menu()?;
        let kind = voxel_game::container_menu::MenuKind::from_menu_type(menu_type_id)?;
        match self.open_menu.as_mut() {
            Some(menu) if menu.container_id == container_id && menu.kind == kind => {
                menu.load(slots, carried, state_id);
            }
            _ => {
                self.open_menu = Some(voxel_game::container_menu::ContainerMenu::for_menu(
                    kind,
                    container_id,
                    slots,
                    carried,
                    state_id,
                ));
            }
        }
        // What a furnace menu reads besides its slots.
        if let Some(menu) = self.open_menu.as_mut() {
            menu.accepted = menu
                .kind
                .input_sets()
                .iter()
                .map(|set| self.trace.world.recipe_property_set(set))
                .collect();
            if menu.kind == voxel_game::container_menu::MenuKind::Stonecutter {
                menu.accepted = vec![std::sync::Arc::new(self.trace.world.stonecutter_inputs())];
                menu.options = menu
                    .slots
                    .first()
                    .and_then(Option::as_ref)
                    .map(|input| {
                        self.trace
                            .world
                            .stonecutter_options(input.item_registry_id())
                    })
                    .unwrap_or_default();
            }
            if menu.kind == voxel_game::container_menu::MenuKind::Loom {
                let (patterns, layers) = Self::loom_patterns(&self.trace.world, menu);
                menu.patterns = patterns;
                menu.banner_layers = layers;
            }
            menu.item_tags = self.trace.world.block_state_registry().item_tags().clone();
            menu.xp_level = self.status.experience_level;
            if menu.kind == voxel_game::container_menu::MenuKind::Merchant {
                // The merchant's offers; the result is the client's own to make.
                if let Some((_, state)) = self
                    .merchant
                    .as_ref()
                    .filter(|(id, _)| *id == menu.container_id)
                {
                    menu.offers = std::sync::Arc::new(state.offers.clone());
                    menu.trader = state.trader;
                }
                menu.update_sell_item();
            }
            if let Some(container) = self
                .trace
                .world
                .local_player()
                .and_then(|player| player.open_container())
            {
                menu.data = (0..10).map(|index| container.data(index)).collect();
            }
        }
        self.open_menu.as_ref()
    }

    /// `MultiPlayerGameMode.handleInventoryButtonClick`: a button of the open
    /// menu (a stonecutter recipe, a loom pattern, an enchantment) pressed.
    pub fn container_button(&mut self, button: i32) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        // `StonecutterMenu.clickMenuButton`: the entry is chosen, and the
        // result, which only the server can make, is emptied.
        let stonecutter = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == voxel_game::container_menu::MenuKind::Stonecutter);
        let loom = self
            .open_menu
            .as_ref()
            .is_some_and(|menu| menu.kind == voxel_game::container_menu::MenuKind::Loom);
        if loom {
            // `LoomMenu.clickMenuButton`: the pattern is chosen.
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
        }
        if stonecutter {
            self.trace
                .world
                .apply_local_player_container_data(container_id, 0, button as i16);
            if let Some(voxel_world::OpenContainerMenu {
                mut slots, carried, ..
            }) = self.trace.world.local_player_container_menu()
            {
                slots[1] = None;
                self.trace
                    .world
                    .set_local_player_container_menu(&slots, carried);
            }
        }
        self.send(
            PLAY_SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(container_id, output);
                voxel_core::wire::write_varint(button, output);
                Ok(())
            },
        )
    }

    /// `ClientPacketListener.applyPlayerInfoUpdate`: an `Add` makes the
    /// player; the rest change one that is known and are dropped for one
    /// that is not.
    pub(super) fn apply_player_info(&mut self, id: (u64, u64), actions: Vec<InfoAction>) {
        for action in actions {
            if let InfoAction::Add { name } = action {
                self.player_infos.entry(id).or_insert_with(|| PlayerInfo {
                    name,
                    ..PlayerInfo::default()
                });
                continue;
            }
            let Some(info) = self.player_infos.get_mut(&id) else {
                continue;
            };
            match action {
                InfoAction::GameMode(mode) => info.game_mode = mode,
                InfoAction::Listed(listed) => info.listed = listed,
                InfoAction::Latency(latency) => info.latency = latency,
                InfoAction::DisplayName(name) => info.display_name = name,
                InfoAction::ListOrder(order) => info.list_order = order,
                InfoAction::Add { .. } => {}
            }
        }
    }

    /// `ServerboundCommandSuggestionPacket`: ask for what could follow `text`.
    pub fn request_suggestions(&mut self, id: i32, text: &str) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, |output| {
            voxel_core::wire::write_varint(id, output);
            let units: Vec<u16> = text.encode_utf16().collect();
            voxel_core::utf8::write_utf8(output, &units, 32500)
                .map_err(|_| ConnectionError::Codec("suggestion request"))
        })
    }

    /// The answer to the last request, if one came.
    pub fn take_suggestions(&mut self) -> Option<CommandSuggestions> {
        self.suggestions.take()
    }

    /// The title, action bar and tab list packets since the last call.
    pub fn drain_hud(&mut self) -> Vec<HudEvent> {
        std::mem::take(&mut self.hud_events)
    }

    /// A death the death screen is yet to show: the server's message.
    pub fn take_death(&mut self) -> Option<voxel_game::text::Component> {
        self.death.take()
    }

    /// `LocalPlayer.respawn`: ask the server to bring the player back.
    pub fn respawn(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_perform_respawn,
        )
    }

    /// Whether the player may use entity selectors
    /// (`Permissions.COMMANDS_ENTITY_SELECTORS`: game master and above).
    pub fn can_use_selectors(&self) -> bool {
        self.permission_level >= 2
    }

    /// The waypoints the server has the client track.
    pub fn waypoints(&self) -> &voxel_game::waypoint::WaypointManager {
        &self.waypoints
    }

    /// What the server's scoreboard packets built.
    pub fn scoreboard(&self) -> &voxel_game::scoreboard::Scoreboard {
        &self.scoreboard
    }

    /// The command tree the server sent, once it has.
    pub fn commands(&self) -> Option<&std::sync::Arc<voxel_game::commands::CommandTree>> {
        self.commands.as_ref()
    }

    /// The local player's name, which scoreboard teams and scores go by.
    pub fn local_name(&self) -> &str {
        &self.username
    }

    /// `ClientSuggestionProvider.getSelectedEntities`: the string UUID of the
    /// entity the crosshair is on.
    pub fn selected_entities(&self) -> Vec<String> {
        self.aimed_entity
            .and_then(|id| self.trace.world.entity_at(id))
            .map(|entity| entity.uuid.to_string_java())
            .into_iter()
            .collect()
    }

    /// `getOnlinePlayers`: the names of every player the server told of.
    pub fn online_player_names(&self) -> Vec<String> {
        self.player_infos
            .values()
            .map(|info| info.name.clone())
            .collect()
    }

    /// `ClientPacketListener.getPlayerInfo(uuid)`'s profile name: what a
    /// remote player is called, if the server told of them.
    pub fn player_name(&self, uuid: voxel_core::binary::Uuid) -> Option<&str> {
        self.player_infos
            .get(&(uuid.most, uuid.least))
            .map(|info| info.name.as_str())
    }

    /// The players the tab list names (`getOnlinePlayers`, listed ones).
    pub fn listed_players(&self) -> Vec<&PlayerInfo> {
        self.player_infos
            .values()
            .filter(|info| info.listed)
            .collect()
    }

    /// `ClientPacketListener.handlePlayerChat`: the count, the signatures
    /// looked up and pushed into the cache, and the sender looked up.
    pub(super) fn receive_player_chat(&mut self, packet: PlayerChatPacket) {
        // The reference drops the connection when the count skips; this
        // client counts on from what it saw.
        if packet.global_index != self.next_chat_index {
            self.warn(format!(
                "chat index {}, expected {}",
                packet.global_index, self.next_chat_index
            ));
        }
        self.next_chat_index = packet.global_index + 1;
        let mut queue = std::collections::VecDeque::new();
        for seen in &packet.last_seen {
            match self.signature_cache.unpack(seen) {
                Some(signature) => queue.push_back(signature),
                None => {
                    self.warn("player chat: unknown signature".to_owned());
                    return;
                }
            }
        }
        if let Some(signature) = &packet.signature {
            queue.push_back(signature.clone());
        }
        self.signature_cache.push(queue);
        let sender_known = self.player_infos.contains_key(&packet.sender);
        let sender_is_local = self
            .trace
            .profile_id
            .is_some_and(|id| (id.most, id.least) == packet.sender);
        self.chat_events.push(ChatEvent::Player {
            packet,
            sender_known,
            sender_is_local,
        });
    }

    /// The chat the server sent since the last call.
    pub fn drain_chat(&mut self) -> Vec<ChatEvent> {
        std::mem::take(&mut self.chat_events)
    }

    /// `ClientPacketListener.sendChat` and `sendCommand`: a line typed into
    /// the chat, sent as a command when `command` (without its slash).
    /// Without a chat key the message goes unsigned.
    pub fn send_chat(&mut self, text: &str, command: bool) -> Result<(), ConnectionError> {
        if command {
            return self.send(PLAY_SERVERBOUND_CHAT_COMMAND_PACKET_ID, |output| {
                let units: Vec<u16> = text.encode_utf16().collect();
                voxel_core::utf8::write_utf8(output, &units, 32767)
                    .map_err(|_| ConnectionError::Codec("chat command"))
            });
        }
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let salt = i64::from(rand_salt());
        self.send(PLAY_SERVERBOUND_CHAT_PACKET_ID, |output| {
            encode_chat(text, millis, salt, output);
            Ok(())
        })
    }

    /// `ServerboundPlaceRecipePacket`: the recipe book asks the server to
    /// move a recipe's ingredients into the open menu's grid.
    pub fn place_recipe(&mut self, recipe: i32, shift: bool) -> Result<(), ConnectionError> {
        let container_id = self.open_container_id();
        self.send(PLAY_SERVERBOUND_PLACE_RECIPE_PACKET_ID, |output| {
            voxel_core::wire::write_varint(container_id, output);
            voxel_core::wire::write_varint(recipe, output);
            output.push(u8::from(shift));
            Ok(())
        })
    }

    /// `ServerboundRecipeBookChangeSettingsPacket`: `book_type` is the
    /// `RecipeBookType` ordinal.
    pub fn recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(book_type, output);
                output.push(u8::from(open));
                output.push(u8::from(filtering));
                Ok(())
            },
        )
    }

    /// `RecipeBookComponent.setVisible` and the filter button: the book's
    /// settings change here at once and the server is told.
    pub fn set_recipe_book_settings(
        &mut self,
        book_type: i32,
        open: bool,
        filtering: bool,
    ) -> Result<(), ConnectionError> {
        let book = self.trace.world.recipe_book_mut();
        if let Some(settings) = book.settings.get_mut(book_type as usize) {
            settings.open = open;
            settings.filtering = filtering;
            book.revision += 1;
        }
        self.recipe_book_settings(book_type, open, filtering)
    }

    /// `GhostSlots.clear`.
    pub fn clear_recipe_ghost(&mut self) {
        self.trace.world.recipe_book_mut().ghost = None;
    }

    /// `LocalPlayer.removeRecipeHighlight`: a new recipe that showed on a page
    /// stops being new, and the server is told.
    pub fn mark_recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        if self.trace.world.recipe_book_mut().highlight.remove(&recipe) {
            self.recipe_seen(recipe)?;
        }
        Ok(())
    }

    /// `ServerboundRecipeBookSeenRecipePacket`: a highlighted recipe was seen.
    pub fn recipe_seen(&mut self, recipe: i32) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
            |output| {
                voxel_core::wire::write_varint(recipe, output);
                Ok(())
            },
        )
    }

    /// The open menu's container id; 0 is the player's own.
    pub(super) fn open_container_id(&self) -> i32 {
        self.trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map_or(0, |container| container.container_id())
    }

    /// `ServerboundRenameItemPacket`: the anvil's name box changed.
    pub fn rename_item(&mut self, name: &str) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_RENAME_ITEM_PACKET_ID, |output| {
            voxel_core::wire::write_varint(name.len() as i32, output);
            output.extend_from_slice(name.as_bytes());
            Ok(())
        })
    }

    /// `MerchantScreen.postButtonClick`: the chosen trade is tried first and its payments moved
    /// in, and the server is told.
    pub fn select_trade(&mut self, trade: i32) -> Result<(), ConnectionError> {
        let Some(menu) = self
            .open_menu
            .as_mut()
            .filter(|menu| menu.kind == voxel_game::container_menu::MenuKind::Merchant)
        else {
            return Ok(());
        };
        let index = usize::try_from(trade).unwrap_or(0);
        menu.selection_hint = index;
        menu.update_sell_item();
        menu.try_move_items(index);
        let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
        self.trace
            .world
            .set_local_player_container_menu(&slots, carried);
        self.send(PLAY_SERVERBOUND_SELECT_TRADE_PACKET_ID, |output| {
            output.extend_from_slice(&encode_select_trade(trade));
            Ok(())
        })
    }
}
