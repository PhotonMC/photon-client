use super::*;
use photon_data::item::registry::AIR_ITEM;

/// How many times in a row the server may send the client on before it is given up on.
const MAX_TRANSFERS: usize = 5;

/// A host name and port as an address (`ServerAddress` resolved).
pub(super) fn resolve_host(host: &str, port: i32) -> Result<SocketAddr, ConnectionError> {
    let port =
        u16::try_from(port).map_err(|_| ConnectionError::InvalidArgument("transfer port"))?;
    crate::resolver::resolve_server(host, port)
        .ok_or(ConnectionError::InvalidArgument("transfer host"))
}

/// What a transfer carries to the next server: the account, the cookies, the packs and the stage.
fn transfer_state(state: ConnectionState) -> ConnectionState {
    ConnectionState {
        account: state.account,
        cookies: state.cookies,
        packs: state.packs,
        login_stage: state.login_stage,
        transferring: true,
        ..ConnectionState::default()
    }
}

/// Read play packets off `reader` in the background, one at a time into the channel, until the
/// connection ends or the server starts the configuration phase (`Packet.isTerminal`): nothing
/// after that packet is a play packet.
pub(super) fn spawn_reader(
    mut reader: Transport,
    compression_threshold: Option<usize>,
    registries: std::sync::Arc<VersionRegistries>,
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
            let (packet, failed) = match incoming::decode_packet(&body, &registries) {
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
    /// The registry id of the stack in the main hand (air when empty).
    fn main_hand_item(&self) -> photon_core::ids::ItemId {
        self.world
            .local_player()
            .and_then(|state| state.inventory_slot((36 + self.local.status.selected_slot) as i16))
            .map_or(AIR_ITEM, |stack| stack.item_registry_id())
    }

    /// `MultiPlayerGameMode.tick`: the carried item is sent if it changed, then the connection ticks.
    fn tick_game_mode(&mut self) -> Result<(), ConnectionError> {
        self.ensure_has_sent_carried_item()?;
        self.link.traffic.tick(|| self.link.writer.take_sent());
        Ok(())
    }

    /// One client tick: the local player, its packets, then the tick end.
    pub fn tick(&mut self, keys: Input) -> Result<(), ConnectionError> {
        // `Minecraft.tick`: the level's tick rate manager first.
        self.level_info.tick_rate.tick();
        // The game mode's tick runs before the player moves.
        self.tick_game_mode()?;
        self.tick_level_load()?;
        if self.local.loaded {
            self.local.drop_spam = (self.local.drop_spam - 1).max(0);
        }
        let main_hand = self.main_hand_item();
        // The level ticks a vehicle before its passengers: a boat the player rides moves first
        // (and tells the server of its paddles), then the player sits in it.
        self.set_rider_controls(main_hand);
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
            && let hands = self.hand_state()
            && let Some(player) = self.local.player.as_mut()
        {
            // `Player.tick` counts the item cooldowns down.
            self.local.item_cooldowns.tick();
            player.set_hand_state(hands);
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
                Err(unsupported) => self.warn_unsupported(&unsupported),
            }
        }
        self.tell_world_where_the_player_is();
        self.tick_level_entities(ridden.is_none());
        if let Err(unsupported) = self.world.tick_after_entities(self.local.player.as_mut()) {
            self.warn_unsupported(&unsupported);
        }
        // ClientLevel.tickTime: the game time and the clocks move on.
        self.tick_clocks(self.level_info.game_time + 1);
        self.send(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, |_| Ok(()))
    }

    /// A branch of the movement the port does not carry yet, said once.
    fn warn_unsupported(&mut self, unsupported: &photon_world::local_player::Unsupported) {
        let message = unsupported.to_string();
        if self.link.warned.as_deref() != Some(message.as_str()) {
            log::warn!("movement: {message}");
            self.link.warned = Some(message);
        }
    }

    /// What the rider steers with: its input, its look, and the stick in its hands.
    fn set_rider_controls(&mut self, main_hand: photon_core::ids::ItemId) {
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
    }

    /// `ClientLevel.tickEntities`: the entities the server added move on. The player's effects
    /// tick with it unless it rides a vehicle, which ticks the player as its passenger.
    fn tick_level_entities(&mut self, player_ticks_itself: bool) {
        self.refresh_friendly_ghosts();
        if player_ticks_itself {
            self.world.tick_local_player_effects();
            self.world.tick_entities();
        }
        let rocket = photon_game::item::id_of("minecraft:firework_rocket");
        let hand = photon_world::RocketHand {
            main_arm_left: self.link.client_information.main_hand
                == photon_protocol::configuration::MainHand::Left,
            only_off_hand: rocket == Some(self.off_hand_item())
                && rocket != Some(self.main_hand_item()),
        };
        if let Some(player) = self.local.player.as_mut() {
            self.world.boost_from_rockets(player, hand);
        }
    }

    /// Which invisible entities a team lets the local player see (`Entity.isInvisibleTo`): the
    /// ones on its own team when the team sees its friendly invisibles.
    fn refresh_friendly_ghosts(&mut self) {
        let board = &self.social.scoreboard;
        let local_team = board.team_of(self.local_name()).map(|(name, _)| name);
        let ghosts = local_team.map_or_else(Default::default, |local| {
            self.world
                .entities()
                .all()
                .filter(|entity| entity.is_invisible())
                .filter(|entity| {
                    let name = if entity.is_player() {
                        self.player_name(entity.uuid).map(str::to_owned)
                    } else {
                        Some(entity.uuid.to_string_java())
                    };
                    name.and_then(|name| {
                        board
                            .team_of(&name)
                            .map(|(team, rules)| team == local && rules.sees_friendly_invisibles)
                    })
                    .unwrap_or(false)
                })
                .map(|entity| entity.id())
                .collect()
        });
        self.world.entities_mut().set_friendly_ghosts(ghosts);
    }

    /// Where the client's own player stands after it moved this tick, for what the world's own
    /// ticks measure from it.
    fn tell_world_where_the_player_is(&mut self) {
        let Some(player) = self.local.player.as_ref() else {
            return;
        };
        let position = player.position();
        self.world.set_local_player_chunk(position.x, position.z);
        self.world
            .set_local_player_spot((position.x, position.y, position.z), player.is_spectator());
    }

    pub(super) fn send<F>(&mut self, id: i32, body: F) -> Result<(), ConnectionError>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), ConnectionError>,
    {
        // Nothing is sent over a connection that is over.
        if self.link.disconnect.is_some() || self.link.switching.is_some() {
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

    /// Like [`Self::connect`], as an account that the server may check with the session server.
    pub fn connect_signed_in(
        address: SocketAddr,
        account: crate::account::Account,
        timeout: Duration,
        auto_jump: bool,
        information: photon_protocol::configuration::ClientInformation,
    ) -> Result<Self, ConnectionError> {
        let username = account.name.clone();
        Self::connect_through(
            (address, None),
            (&username, auto_jump),
            timeout,
            information,
            ConnectionState {
                account: Some(account),
                ..ConnectionState::default()
            },
        )
    }

    /// Like [`Self::connect`], with what `first` carries into the connection (the account, the
    /// server packs the client keeps).
    pub fn connect_with(
        address: SocketAddr,
        (username, auto_jump): (&str, bool),
        timeout: Duration,
        information: photon_protocol::configuration::ClientInformation,
        first: ConnectionState,
    ) -> Result<Self, ConnectionError> {
        Self::connect_through(
            (address, None),
            (username, auto_jump),
            timeout,
            information,
            first,
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
                account: carried.account.clone(),
                cookies: carried.cookies.clone(),
                packs: carried.packs.clone(),
                handshake_host: host.clone(),
                transferring: carried.transferring,
                login_stage: carried.login_stage.clone(),
                ..ConnectionState::default()
            };
            state.reach(crate::connection::state::stage::CONNECTING);
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
                    carried = transfer_state(state);
                }
                opened => {
                    let (stream, compression_threshold) = opened?;
                    stream.set_read_timeout(None)?;
                    let registries = VersionRegistries::newest();
                    let incoming =
                        spawn_reader(stream.try_clone()?, compression_threshold, registries);
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
        writer: Transport,
        compression_threshold: Option<usize>,
        incoming: Receiver<Result<Incoming, ConnectionError>>,
        state: ConnectionState,
        login: (&str, bool),
    ) -> Self {
        let writer = PacketWriter::start(writer).expect("a thread to write on");
        let social = Social {
            seen_players: state.seen_players.clone(),
            ..Social::default()
        };
        let effects = Effects {
            seen_unsecure_warning: state.seen_insecure_warning,
            ..Effects::default()
        };
        Self {
            link: Link::new(writer, compression_threshold, incoming, login),
            registries: VersionRegistries::newest(),
            world: state.new_world(),
            state,
            local: LocalState::default(),
            menus: Menus::default(),
            social,
            effects,
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
        self.world.set_game_time(game_time as u64);
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
            // The reader stopped at the terminal packet; the switch is what the connection does.
            if self.link.switching.is_some() {
                return self.poll_switch(Duration::ZERO);
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
        if self.link.switching.is_some() {
            return self.poll_switch(limit);
        }
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
