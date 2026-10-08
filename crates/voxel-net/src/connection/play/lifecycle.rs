use super::*;

impl PlaySession {
    /// One client tick: the local player, its packets, then the tick end.
    pub fn tick(&mut self, keys: Input) -> Result<(), ConnectionError> {
        // The game mode's tick runs before the player moves.
        self.ensure_has_sent_carried_item()?;
        if self.loaded {
            self.drop_spam = (self.drop_spam - 1).max(0);
        }
        let main_hand = self
            .trace
            .world
            .local_player()
            .and_then(|state| state.inventory_slot((36 + self.status.selected_slot) as i16))
            .map_or(0, |stack| stack.item_registry_id());
        // The level ticks a vehicle before its passengers: a boat the player rides moves first
        // (and tells the server of its paddles), then the player sits in it.
        // What the rider steers with: its input, its look, and the stick in its hands.
        if let Some(player) = self.player.as_ref() {
            let mut controls = player.controller();
            let holds = |name: &str| {
                [main_hand, self.off_hand_item()].into_iter().any(|item| {
                    voxel_world::block::rules::item(item).is_some_and(|item| item.name == name)
                })
            };
            controls.holds_carrot_on_a_stick = holds("minecraft:carrot_on_a_stick");
            controls.holds_warped_fungus_on_a_stick = holds("minecraft:warped_fungus_on_a_stick");
            self.trace.world.set_rider_controls(controls);
        }
        let ridden = self.trace.world.ridden_vehicle_of_local_player();
        if ridden.is_some() {
            if let Some(player) = self.player.as_ref() {
                let position = player.position();
                self.trace
                    .world
                    .set_local_player_chunk(position.x, position.z);
            }
            self.trace.world.tick_entities();
            for (left, right) in self.trace.world.take_paddle_packets() {
                self.send_movement(ServerboundMovement::PaddleBoat { left, right })?;
            }
        }
        if self.loaded
            && let Some(player) = self.player.as_mut()
        {
            player.set_hand_state(self.trace.world.local_hand_state(self.status.selected_slot));
            let ticked = match ridden.and_then(|id| self.trace.world.vehicle_view(id)) {
                Some(view) => player
                    .tick_riding(&self.trace.world, keys, &view)
                    .map(|rode| {
                        if view.kind == voxel_world::VehicleKind::Boat && view.controlled {
                            self.trace
                                .world
                                .set_boat_input(view.vehicle_id, rode.boat_input);
                        }
                        if let Some(amount) = rode.jump {
                            self.trace.world.on_player_jump(view.vehicle_id, amount);
                        }
                        rode.sent
                    }),
                None => player.tick(&self.trace.world, keys),
            };
            match ticked {
                Ok(sent) => {
                    for packet in sent {
                        self.send_movement(packet)?;
                    }
                }
                Err(unsupported) => {
                    let message = unsupported.to_string();
                    if self.warned.as_deref() != Some(message.as_str()) {
                        log::warn!("movement: {message}");
                        self.warned = Some(message);
                    }
                }
            }
        }
        // `ClientLevel.tickEntities`: the entities the server added move on.
        if let Some(player) = self.player.as_ref() {
            let position = player.position();
            self.trace
                .world
                .set_local_player_chunk(position.x, position.z);
        }
        if ridden.is_none() {
            self.trace.world.tick_entities();
        }
        self.trace.world.tick_sky_flash();
        // ClientLevel.tickTime: the game time and the clocks move on.
        self.tick_clocks(self.game_time + 1);
        self.send(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, |_| Ok(()))
    }

    pub(super) fn send<F>(&mut self, id: i32, body: F) -> Result<(), ConnectionError>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
    {
        // Nothing is sent over a connection that is over.
        if self.disconnect.is_some() {
            return Ok(());
        }
        send_packet(
            &mut self.writer,
            ConnectionProtocol::Play,
            id,
            body,
            self.compression_threshold,
        )
    }

    /// Log in, configure, and start reading play packets in the background.
    /// The server is told the client's `information` (`ServerboundClientInformationPacket`).
    pub fn connect(
        address: SocketAddr,
        username: &str,
        timeout: Duration,
        auto_jump: bool,
        information: crate::configuration::ClientInformation,
    ) -> Result<Self, ConnectionError> {
        let mut trace = PlayConnectionTrace::default();
        let (stream, compression_threshold) =
            open_play_connection(address, username, timeout, true, &information, &mut trace)?;
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
        let mut session = Self::assemble(
            stream,
            compression_threshold,
            incoming,
            trace,
            (username, auto_jump),
        );
        session.client_information = information;
        Ok(session)
    }

    /// `ClientPacketListener.broadcastClientInformation`: the client's information, sent again
    /// only when it is not what the server was last told.
    pub fn send_client_information(
        &mut self,
        information: crate::configuration::ClientInformation,
    ) -> Result<(), ConnectionError> {
        if information == self.client_information {
            return Ok(());
        }
        self.client_information = information.clone();
        self.send(PLAY_SERVERBOUND_CLIENT_INFORMATION_PACKET_ID, |output| {
            crate::configuration::ServerboundClientInformationPacket::new(information)
                .encode(output)
                .map_err(|_| ConnectionError::Codec("client information"))
        })
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
            client_information: crate::configuration::ClientInformation::default(),
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
            level_particles: Vec::new(),
            item_pickups: Vec::new(),
            listener_random: voxel_core::random::RandomSource::create(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |elapsed| elapsed.as_nanos() as i64),
            ),
            aimed_entity: None,
            hardcore: false,
            health_received: false,
            level_reset: false,
            sent_slot: 0,
            game_mode_state: voxel_game::game_mode::GameMode::default(),
            container_menu: voxel_game::menus::container::ContainerMenu::default(),
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
}
