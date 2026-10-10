use super::*;

/// How many times in a row the server may send the client on before it is given up on.
const MAX_TRANSFERS: usize = 5;

/// A host name and port as an address (`ServerAddress` resolved).
pub(super) fn resolve_host(host: &str, port: i32) -> Result<SocketAddr, ConnectionError> {
    use std::net::ToSocketAddrs;
    let port =
        u16::try_from(port).map_err(|_| ConnectionError::InvalidArgument("transfer port"))?;
    (host, port)
        .to_socket_addrs()?
        .next()
        .ok_or(ConnectionError::InvalidArgument("transfer host"))
}

/// Read play packets off `reader` in the background, one at a time into the channel, until the
/// connection ends or the server starts the configuration phase (`Packet.isTerminal`): nothing
/// after that packet is a play packet.
pub(super) fn spawn_reader(
    mut reader: TcpStream,
    compression_threshold: Option<usize>,
) -> Receiver<Result<Incoming, ConnectionError>> {
    let (sender, incoming) = mpsc::channel();
    std::thread::spawn(move || {
        loop {
            let body = match read_packet(&mut reader, compression_threshold) {
                Ok(body) => body,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    return;
                }
            };
            let terminal = matches!(
                split_packet(&body),
                Ok((PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID, _))
            );
            let packet = classify(body);
            let failed = packet.is_err();
            if sender.send(packet).is_err() || failed || terminal {
                return;
            }
        }
    });
    incoming
}

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
                    photon_world::block::rules::item(item).is_some_and(|item| item.name == name)
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
            // `Player.tick` counts the item cooldowns down.
            self.item_cooldowns.tick();
            player.set_hand_state(self.trace.world.local_hand_state(self.status.selected_slot));
            let ticked = match ridden.and_then(|id| self.trace.world.vehicle_view(id)) {
                Some(view) => player
                    .tick_riding(&self.trace.world, keys, &view)
                    .map(|rode| {
                        if view.kind == photon_world::VehicleKind::Boat && view.controlled {
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
        Self::connect_through(
            (address, None),
            (username, auto_jump),
            timeout,
            information,
            PlayConnectionTrace::default(),
        )
    }

    /// Connect to `address` (and the host name a handshake gives, where it is not the address's
    /// own) with what `first` carries over (`trace.cookies`, `trace.transferring`), following the
    /// server where it sends the client on to another.
    pub(super) fn connect_through(
        (mut address, mut host): (SocketAddr, Option<String>),
        (username, auto_jump): (&str, bool),
        timeout: Duration,
        information: crate::configuration::ClientInformation,
        first: PlayConnectionTrace,
    ) -> Result<Self, ConnectionError> {
        let mut carried = first;
        for _ in 0..=MAX_TRANSFERS {
            let mut trace = PlayConnectionTrace {
                cookies: carried.cookies.clone(),
                handshake_host: host.clone(),
                transferring: carried.transferring,
                ..PlayConnectionTrace::default()
            };
            match open_play_connection(address, username, timeout, true, &information, &mut trace) {
                Err(ConnectionError::Transfer {
                    host: next_host,
                    port,
                }) => {
                    address = resolve_host(&next_host, port)?;
                    host = Some(next_host);
                    carried = PlayConnectionTrace {
                        cookies: trace.cookies,
                        transferring: true,
                        ..PlayConnectionTrace::default()
                    };
                }
                opened => {
                    let (stream, compression_threshold) = opened?;
                    stream.set_read_timeout(None)?;
                    let incoming = spawn_reader(stream.try_clone()?, compression_threshold);
                    let mut session = Self::assemble(
                        stream,
                        compression_threshold,
                        incoming,
                        trace,
                        (username, auto_jump),
                    );
                    session.client_information = information;
                    return Ok(session);
                }
            }
        }
        Err(ConnectionError::InvalidArgument("too many transfers"))
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
            previous_game_mode: None,
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
            enchantment_traits: std::sync::Arc::default(),
            lang: std::sync::Arc::default(),
            item_cooldowns: photon_game::item::cooldowns::ItemCooldowns::default(),
            game_time: 0,
            rain_level: 0.0,
            thunder_level: 0.0,
            status: PlayerStatus::default(),
            show_death_screen: true,
            disconnect: None,
            death: None,
            win_game: false,
            swings: 0,
            sounds: Vec::new(),
            level_particles: Vec::new(),
            item_pickups: Vec::new(),
            listener_random: photon_core::random::RandomSource::create(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |elapsed| elapsed.as_nanos() as i64),
            ),
            level_random: photon_core::random::RandomSource::create(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |elapsed| elapsed.as_nanos() as i64 ^ 0x5DEE_CE66),
            ),
            aimed_entity: None,
            hardcore: false,
            health_received: false,
            level_reset: false,
            reconfigured: false,
            pack_requests: Vec::new(),
            sent_slot: 0,
            game_mode_state: photon_game::game_mode::GameMode::default(),
            container_menu: photon_game::menus::container::ContainerMenu::default(),
            open_menu: None,
            sign_editor: None,
            opened_book: None,
            merchant: None,
            mount: None,
            chat_events: Vec::new(),
            signature_cache: SignatureCache::default(),
            hud_events: Vec::new(),
            removed_player_vehicle: None,
            scoreboard: photon_game::scoreboard::Scoreboard::default(),
            waypoints: photon_game::waypoint::WaypointManager::default(),
            permission_level: 0,
            commands: None,
            username: username.to_owned(),
            suggestions: None,
            player_infos: std::collections::BTreeMap::new(),
            seen_players: std::collections::BTreeMap::new(),
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
                    self.disconnect = Some(photon_game::text::Component::translatable_with(
                        "disconnect.genericReason",
                        vec![photon_game::text::Component::literal(format!(
                            "Internal Exception: {error}"
                        ))],
                    ));
                }
                Ok(Err(error)) => return Err(error),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    self.disconnect = Some(photon_game::text::Component::translatable(
                        "disconnect.endOfStream",
                    ));
                }
            }
        }
    }

    /// `Connection.disconnect`: the client ends the connection, with the reason it shows.
    pub fn disconnect(&mut self, reason: photon_game::text::Component) {
        if self.disconnect.is_none() {
            self.disconnect = Some(reason);
        }
    }

    /// Why the connection is over, once it is.
    pub fn disconnected(&self) -> Option<&photon_game::text::Component> {
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
