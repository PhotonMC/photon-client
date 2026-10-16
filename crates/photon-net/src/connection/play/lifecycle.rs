use super::*;
use photon_data::item::registry::AIR_ITEM;

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
            let (packet, failed) = match incoming::decode_packet(&body) {
                Ok(packet) => (packet, false),
                Err(error) => (incoming::failed(error), true),
            };
            if sender.send(Ok(packet)).is_err() || failed || terminal {
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
        if self.local.loaded {
            self.local.drop_spam = (self.local.drop_spam - 1).max(0);
        }
        let main_hand = self
            .world
            .local_player()
            .and_then(|state| state.inventory_slot((36 + self.local.status.selected_slot) as i16))
            .map_or(AIR_ITEM, |stack| stack.item_registry_id());
        // The level ticks a vehicle before its passengers: a boat the player rides moves first
        // (and tells the server of its paddles), then the player sits in it.
        // What the rider steers with: its input, its look, and the stick in its hands.
        if let Some(player) = self.local.player.as_ref() {
            let mut controls = player.controller();
            let holds = |name: &str| {
                [main_hand, self.off_hand_item()].into_iter().any(|item| {
                    photon_world::block::rules::item(item).is_some_and(|item| item.name == name)
                })
            };
            controls.holds_carrot_on_a_stick = holds("minecraft:carrot_on_a_stick");
            controls.holds_warped_fungus_on_a_stick = holds("minecraft:warped_fungus_on_a_stick");
            self.world.set_rider_controls(controls);
        }
        let ridden = self.world.ridden_vehicle_of_local_player();
        if ridden.is_some() {
            if let Some(player) = self.local.player.as_ref() {
                let position = player.position();
                self.world.set_local_player_chunk(position.x, position.z);
            }
            self.world.tick_entities();
            for (left, right) in self.world.take_paddle_packets() {
                self.send_movement(ServerboundMovement::PaddleBoat { left, right })?;
            }
        }
        if self.local.loaded
            && let Some(player) = self.local.player.as_mut()
        {
            // `Player.tick` counts the item cooldowns down.
            self.local.item_cooldowns.tick();
            player.set_hand_state(self.world.local_hand_state(self.local.status.selected_slot));
            let ticked = match ridden.and_then(|id| self.world.vehicle_view(id)) {
                Some(view) => player.tick_riding(&self.world, keys, &view).map(|rode| {
                    if view.kind == photon_world::VehicleKind::Boat && view.controlled {
                        self.world.set_boat_input(view.vehicle_id, rode.boat_input);
                    }
                    if let Some(amount) = rode.jump {
                        self.world.on_player_jump(view.vehicle_id, amount);
                    }
                    rode.sent
                }),
                None => player.tick(&self.world, keys),
            };
            match ticked {
                Ok(sent) => {
                    for packet in sent {
                        self.send_movement(packet)?;
                    }
                }
                Err(unsupported) => {
                    let message = unsupported.to_string();
                    if self.link.warned.as_deref() != Some(message.as_str()) {
                        log::warn!("movement: {message}");
                        self.link.warned = Some(message);
                    }
                }
            }
        }
        // `ClientLevel.tickEntities`: the entities the server added move on.
        if let Some(player) = self.local.player.as_ref() {
            let position = player.position();
            self.world.set_local_player_chunk(position.x, position.z);
        }
        if ridden.is_none() {
            self.world.tick_entities();
        }
        self.world.tick_after_entities();
        // ClientLevel.tickTime: the game time and the clocks move on.
        self.tick_clocks(self.level_info.game_time + 1);
        self.send(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, |_| Ok(()))
    }

    pub(super) fn send<F>(&mut self, id: i32, body: F) -> Result<(), ConnectionError>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
    {
        // Nothing is sent over a connection that is over.
        if self.link.disconnect.is_some() {
            return Ok(());
        }
        self.link
            .writer
            .send(frame_packet(id, body, self.link.compression_threshold)?)
    }

    /// Log in, configure, and start reading play packets in the background.
    /// The server is told the client's `information` (`ServerboundClientInformationPacket`).
    pub fn connect(
        address: SocketAddr,
        username: &str,
        timeout: Duration,
        auto_jump: bool,
        information: photon_protocol::configuration::ClientInformation,
    ) -> Result<Self, ConnectionError> {
        Self::connect_through(
            (address, None),
            (username, auto_jump),
            timeout,
            information,
            ConnectionState::default(),
        )
    }

    /// Connect to `address` (and the host name a handshake gives, where it is not the address's
    /// own) with what `first` carries over (`cookies`, `transferring`), following the
    /// server where it sends the client on to another.
    pub(super) fn connect_through(
        (mut address, mut host): (SocketAddr, Option<String>),
        (username, auto_jump): (&str, bool),
        timeout: Duration,
        information: photon_protocol::configuration::ClientInformation,
        first: ConnectionState,
    ) -> Result<Self, ConnectionError> {
        let mut carried = first;
        for _ in 0..=MAX_TRANSFERS {
            let mut state = ConnectionState {
                cookies: carried.cookies.clone(),
                handshake_host: host.clone(),
                transferring: carried.transferring,
                ..ConnectionState::default()
            };
            let opened = open_play_connection(
                (address, username),
                timeout,
                true,
                &information,
                (&mut state, &mut NoRecorder),
            );
            match opened {
                Err(ConnectionError::Transfer {
                    host: next_host,
                    port,
                }) => {
                    address = resolve_host(&next_host, port)?;
                    host = Some(next_host);
                    carried = ConnectionState {
                        cookies: state.cookies,
                        transferring: true,
                        ..ConnectionState::default()
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
                        state,
                        (username, auto_jump),
                    );
                    session.link.client_information = information;
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
        information: photon_protocol::configuration::ClientInformation,
    ) -> Result<(), ConnectionError> {
        if information == self.link.client_information {
            return Ok(());
        }
        self.link.client_information = information.clone();
        self.send(PLAY_SERVERBOUND_CLIENT_INFORMATION_PACKET_ID, |output| {
            photon_protocol::configuration::ServerboundClientInformationPacket::new(information)
                .encode(output)
                .map_err(|_| ConnectionError::Codec("client information"))
        })
    }

    /// A session over a connection already made, with nothing yet known.
    pub(super) fn assemble(
        writer: TcpStream,
        compression_threshold: Option<usize>,
        incoming: Receiver<Result<Incoming, ConnectionError>>,
        state: ConnectionState,
        login: (&str, bool),
    ) -> Self {
        let writer = PacketWriter::start(writer).expect("a thread to write on");
        Self {
            link: Link::new(writer, compression_threshold, incoming, login),
            world: state.new_world(),
            state,
            local: LocalState::default(),
            menus: Menus::default(),
            social: Social::default(),
            effects: Effects::default(),
            level_info: LevelInfo::default(),
        }
    }

    /// A world clock's total ticks, by its registry name.
    pub fn clock_ticks(&self, clock: &str) -> Option<i64> {
        let id = self
            .registry_names("minecraft:world_clock")
            .iter()
            .position(|name| name == clock)?;
        self.level_info
            .clocks
            .get(&(id as i32))
            .map(|state| state.total_ticks)
    }

    /// `ClientClockManager.tick`: every clock runs at its own rate.
    pub(super) fn tick_clocks(&mut self, game_time: i64) {
        let delta = game_time - self.level_info.game_time;
        self.level_info.game_time = game_time;
        for state in self.level_info.clocks.values_mut() {
            let ticks = f64::from(state.partial_tick) + delta as f64 * f64::from(state.rate);
            let whole = ticks.floor();
            state.partial_tick = (ticks - whole) as f32;
            state.total_ticks += whole as i64;
        }
    }

    /// The first unported movement branch the player reached, if any.
    pub fn warning(&self) -> Option<&str> {
        self.link.warned.as_deref()
    }

    /// Handle every packet that has arrived, without waiting for more.
    pub fn drain(&mut self) -> Result<(), ConnectionError> {
        loop {
            if self.link.disconnect.is_some() {
                return Ok(());
            }
            match self.link.incoming.try_recv() {
                Ok(Ok(packet)) => self.handle_incoming(packet)?,
                // `ClientCommonPacketListenerImpl.onDisconnect`: a failure to
                // read ends the connection with what went wrong.
                Ok(Err(ConnectionError::Io(error))) => {
                    self.link.disconnect = Some(photon_game::text::Component::translatable_with(
                        "disconnect.genericReason",
                        vec![photon_game::text::Component::literal(format!(
                            "Internal Exception: {error}"
                        ))],
                    ));
                }
                Ok(Err(error)) => return Err(error),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    self.link.disconnect = Some(photon_game::text::Component::translatable(
                        "disconnect.endOfStream",
                    ));
                }
            }
        }
    }

    /// `Connection.disconnect`: the client ends the connection, with the reason it shows.
    pub fn disconnect(&mut self, reason: photon_game::text::Component) {
        if self.link.disconnect.is_none() {
            self.link.disconnect = Some(reason);
        }
    }

    /// Why the connection is over, once it is.
    pub fn disconnected(&self) -> Option<&photon_game::text::Component> {
        self.link.disconnect.as_ref()
    }

    /// Wait up to `limit` for the next packet and handle it.
    pub fn wait(&mut self, limit: Duration) -> Result<(), ConnectionError> {
        match self.link.incoming.recv_timeout(limit) {
            Ok(packet) => self.handle_incoming(packet?),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ConnectionError::ServerDisconnected),
        }
    }

    pub(super) fn warn(&mut self, message: String) {
        if self.link.warned.as_deref() != Some(message.as_str()) {
            log::warn!("{message}");
            self.link.warned = Some(message);
        }
    }
}
