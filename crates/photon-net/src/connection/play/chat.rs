use super::*;

impl PlaySession {
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
            photon_core::wire::write_varint(id, output);
            let units: Vec<u16> = text.encode_utf16().collect();
            photon_core::utf8::write_utf8(output, &units, 32500)
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

    /// Whether the player may use entity selectors
    /// (`Permissions.COMMANDS_ENTITY_SELECTORS`: game master and above).
    pub fn can_use_selectors(&self) -> bool {
        self.permission_level >= 2
    }

    /// The waypoints the server has the client track.
    pub fn waypoints(&self) -> &photon_game::waypoint::WaypointManager {
        &self.waypoints
    }

    /// What the server's scoreboard packets built.
    pub fn scoreboard(&self) -> &photon_game::scoreboard::Scoreboard {
        &self.scoreboard
    }

    /// The command tree the server sent, once it has.
    pub fn commands(&self) -> Option<&std::sync::Arc<photon_game::commands::CommandTree>> {
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
            .and_then(|id| self.trace.world.entities().at(id))
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
    pub fn player_name(&self, uuid: photon_core::binary::Uuid) -> Option<&str> {
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
                photon_core::utf8::write_utf8(output, &units, 32767)
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
}
