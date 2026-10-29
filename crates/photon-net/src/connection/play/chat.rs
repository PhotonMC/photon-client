use super::chat_validation::Verdict;
use super::*;

impl PlaySession {
    /// `ClientPacketListener.applyPlayerInfoUpdate`: an `Add` makes the
    /// player; the rest change one that is known and are dropped for one
    /// that is not.
    pub(super) fn apply_player_info(&mut self, id: (u64, u64), actions: Vec<InfoAction>) {
        for action in actions {
            if let InfoAction::Add { name, properties } = action {
                self.social
                    .player_infos
                    .entry(id)
                    .or_insert_with(|| PlayerInfo {
                        name,
                        properties,
                        ..PlayerInfo::default()
                    });
                self.remember_entity_profile_if_new(id);
                continue;
            }
            let Some(info) = self.social.player_infos.get_mut(&id) else {
                continue;
            };
            match action {
                InfoAction::GameMode(mode) => info.game_mode = mode,
                InfoAction::Listed(listed) => info.listed = listed,
                InfoAction::Latency(latency) => info.latency = latency,
                InfoAction::DisplayName(name) => info.display_name = name,
                InfoAction::ListOrder(order) => info.list_order = order,
                InfoAction::ChatSession(session) => self.social.senders.set_session(id, session),
                InfoAction::Add { .. } => {}
            }
        }
    }

    /// A player entity was added: it takes the tab entry of its uuid.
    pub(super) fn remember_added_player(
        &mut self,
        entity_type: photon_core::ids::EntityTypeId,
        uuid: photon_core::binary::Uuid,
    ) {
        if photon_data::entity_type::id("minecraft:player") == Some(entity_type) {
            self.remember_entity_profile((uuid.most, uuid.least));
        }
    }

    /// `handleAddEntity`: a player in the tab list is remembered as seen, and
    /// `AbstractClientPlayer.getPlayerInfo`: a player entity takes the tab list entry it is
    /// added with and keeps its name and skin whatever the list does afterwards.
    pub(super) fn remember_entity_profile(&mut self, id: (u64, u64)) {
        let Some(info) = self.social.player_infos.get(&id) else {
            return;
        };
        self.social.seen_players.insert(id, info.name.clone());
        let profile = super::groups::EntityProfile {
            name: info.name.clone(),
            properties: info.properties.clone(),
        };
        self.social.entity_profiles.insert(id, profile);
    }

    /// `getPlayerInfo` also caches on the first query that finds an entry: a player entity added
    /// before its tab entry takes the entry when it arrives, and never replaces a cached one.
    fn remember_entity_profile_if_new(&mut self, id: (u64, u64)) {
        let present = self
            .world
            .entities()
            .all()
            .any(|entity| entity.is_player() && (entity.uuid.most, entity.uuid.least) == id);
        if present && !self.social.entity_profiles.contains_key(&id) {
            self.remember_entity_profile(id);
        }
    }

    /// The entities are gone: so are the profiles they kept.
    pub(super) fn forget_entity_profiles(&mut self, entity_ids: &[photon_core::ids::EntityId]) {
        for &entity_id in entity_ids {
            let uuid = self
                .world
                .entities()
                .at(entity_id)
                .filter(|entity| entity.is_player())
                .map(|entity| (entity.uuid.most, entity.uuid.least));
            if let Some(uuid) = uuid {
                self.social.entity_profiles.remove(&uuid);
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
        self.social.suggestions.take()
    }

    /// The title, action bar and tab list packets since the last call.
    pub fn drain_hud(&mut self) -> Vec<HudEvent> {
        std::mem::take(&mut self.social.hud_events)
    }

    /// Whether the player may use entity selectors
    /// (`Permissions.COMMANDS_ENTITY_SELECTORS`: game master and above).
    pub fn can_use_selectors(&self) -> bool {
        self.local.permission_level >= 2
    }

    /// The waypoints the server has the client track.
    pub fn waypoints(&self) -> &photon_game::waypoint::WaypointManager {
        &self.social.waypoints
    }

    /// What the server's scoreboard packets built.
    pub fn scoreboard(&self) -> &photon_game::scoreboard::Scoreboard {
        &self.social.scoreboard
    }

    /// The command tree the server sent, once it has.
    pub fn commands(&self) -> Option<&std::sync::Arc<photon_game::commands::CommandTree>> {
        self.social.commands.as_ref()
    }

    /// The local player's name, which scoreboard teams and scores go by: the one the server's
    /// profile has (`getLocalGameProfile`), else the one the client logged in with.
    pub fn local_name(&self) -> &str {
        self.state
            .profile_name
            .as_deref()
            .unwrap_or(&self.link.username)
    }

    /// `ClientSuggestionProvider.getSelectedEntities`: the string UUID of the
    /// entity the crosshair is on.
    pub fn selected_entities(&self) -> Vec<String> {
        self.local
            .aimed_entity
            .and_then(|id| self.world.entities().at(id))
            .map(|entity| entity.uuid.to_string_java())
            .into_iter()
            .collect()
    }

    /// `getOnlinePlayers`: the names of every player the server told of.
    pub fn online_player_names(&self) -> Vec<String> {
        self.social
            .player_infos
            .values()
            .map(|info| info.name.clone())
            .collect()
    }

    /// `AbstractClientPlayer.getPlayerInfo`'s profile name: what a remote player is called, as
    /// the entity was added (the tab list entry may be gone since), else as the server told of
    /// them.
    pub fn player_name(&self, uuid: photon_core::binary::Uuid) -> Option<&str> {
        let id = (uuid.most, uuid.least);
        if let Some(profile) = self.social.entity_profiles.get(&id) {
            return Some(profile.name.as_str());
        }
        self.social
            .player_infos
            .get(&id)
            .map(|info| info.name.as_str())
    }

    /// A player's profile properties (`GameProfile.properties`), among them the packed textures;
    /// none for one the server did not tell of.
    pub fn player_properties(
        &self,
        uuid: photon_core::binary::Uuid,
    ) -> &[photon_protocol::play::ProfileProperty] {
        let id = (uuid.most, uuid.least);
        if let Some(profile) = self.social.entity_profiles.get(&id) {
            return &profile.properties;
        }
        self.social
            .player_infos
            .get(&id)
            .map_or(&[], |info| info.properties.as_slice())
    }

    /// `LocalPlayerResolver.fetchByName`: the tab list's player of this name (any case), with the
    /// profile's properties.
    pub fn tab_profile_by_name(
        &self,
        name: &str,
    ) -> Option<(
        photon_core::binary::Uuid,
        &[photon_protocol::play::ProfileProperty],
    )> {
        self.social
            .player_infos
            .iter()
            .find(|(_, info)| info.name.eq_ignore_ascii_case(name))
            .map(|(id, info)| {
                (
                    photon_core::binary::Uuid::new(id.0, id.1),
                    info.properties.as_slice(),
                )
            })
    }

    /// `LocalPlayerResolver.fetchById`: the properties of the tab list's player with this id.
    pub fn tab_profile_by_id(
        &self,
        uuid: photon_core::binary::Uuid,
    ) -> Option<&[photon_protocol::play::ProfileProperty]> {
        self.social
            .player_infos
            .get(&(uuid.most, uuid.least))
            .map(|info| info.properties.as_slice())
    }

    /// `getOnlinePlayerIds` with their names, and the seen players who have left
    /// (`getSeenPlayers`), for the social interactions screen.
    pub fn social_players(&self) -> SocialPlayers {
        let id = |(most, least): (u64, u64)| photon_core::binary::Uuid::new(most, least);
        let online: Vec<_> = self
            .social
            .player_infos
            .iter()
            .map(|(key, info)| (id(*key), info.name.clone()))
            .collect();
        let gone = self
            .social
            .seen_players
            .iter()
            .filter(|(key, _)| !self.social.player_infos.contains_key(*key))
            .map(|(key, name)| (id(*key), name.clone()))
            .collect();
        SocialPlayers { online, gone }
    }

    /// The players the tab list names (`getOnlinePlayers`, listed ones).
    pub fn listed_players(&self) -> Vec<&PlayerInfo> {
        self.social
            .player_infos
            .values()
            .filter(|info| info.listed)
            .collect()
    }

    /// `ClientPacketListener.handlePlayerChat`: the count, the signatures
    /// looked up and pushed into the cache, and the sender looked up.
    pub(super) fn receive_player_chat(&mut self, packet: PlayerChatPacket) {
        // The reference drops the connection when the count skips; this
        // client counts on from what it saw.
        if packet.global_index != self.social.next_chat_index {
            self.warn(format!(
                "chat index {}, expected {}",
                packet.global_index, self.social.next_chat_index
            ));
        }
        self.social.next_chat_index = packet.global_index + 1;
        let mut queue = std::collections::VecDeque::new();
        for seen in &packet.last_seen {
            if let Some(signature) = self.social.signature_cache.unpack(seen) {
                queue.push_back(signature);
            } else {
                self.warn("player chat: unknown signature".to_owned());
                return;
            }
        }
        let last_seen: Vec<Vec<u8>> = queue.iter().cloned().collect();
        if let Some(signature) = &packet.signature {
            queue.push_back(signature.clone());
        }
        self.social.signature_cache.push(queue);
        let sender_is_local = self
            .state
            .profile_id
            .is_some_and(|id| (id.most, id.least) == packet.sender);
        let mut packet = packet;
        let verified = self.social.player_infos.contains_key(&packet.sender)
            && match self.validate_player_chat(&packet, &last_seen) {
                Verdict::Accepted { signed } => {
                    // An unsigned message is shown without the signature it came with.
                    if !signed {
                        packet.signature = None;
                    }
                    true
                }
                Verdict::Rejected => false,
            };
        self.social.chat_events.push(ChatEvent::Player {
            packet,
            verified,
            sender_is_local,
        });
    }

    /// `PlayerInfo.getMessageValidator().updateAndValidate`.
    fn validate_player_chat(
        &mut self,
        packet: &PlayerChatPacket,
        last_seen: &[Vec<u8>],
    ) -> Verdict {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        self.social.senders.validate(packet, last_seen, now_ms)
    }

    /// The chat the server sent since the last call.
    pub fn drain_chat(&mut self) -> Vec<ChatEvent> {
        std::mem::take(&mut self.social.chat_events)
    }

    /// The leave-bed button and Escape in bed (`InBedChatScreen.sendWakeUp`): the player asks the
    /// server to get up.
    pub fn send_stop_sleeping(&mut self) -> Result<(), ConnectionError> {
        let Some(entity_id) = self.world.local_player().map(|player| player.id()) else {
            return Ok(());
        };
        self.send_movement(
            photon_world::local_player::ServerboundMovement::PlayerCommand {
                entity_id,
                action: photon_world::local_player::PlayerCommandAction::StopSleeping,
                data: 0,
            },
        )
    }

    /// `MultiPlayerGameMode.isServerControlledInventory`: the player rides something whose
    /// inventory the server opens.
    pub fn is_server_controlled_inventory(&self) -> bool {
        self.world
            .local_player()
            .and_then(|player| player.vehicle_id())
            .and_then(|id| self.world.entities().at(id))
            .is_some_and(|vehicle| vehicle.has_custom_inventory_screen())
    }

    /// `LocalPlayer.sendOpenInventory`: the inventory key on a mount asks the server to open its
    /// inventory.
    pub fn send_open_inventory(&mut self) -> Result<(), ConnectionError> {
        let Some(entity_id) = self.world.local_player().map(|player| player.id()) else {
            return Ok(());
        };
        self.send_movement(
            photon_world::local_player::ServerboundMovement::PlayerCommand {
                entity_id,
                action: photon_world::local_player::PlayerCommandAction::OpenInventory,
                data: 0,
            },
        )
    }

    /// `ClientPacketListener.sendChat` and `sendCommand`: a line typed into
    /// the chat, sent as a command when `command` (without its slash).
    /// Without a chat session the message goes unsigned.
    pub fn send_chat(&mut self, text: &str, command: bool) -> Result<(), ConnectionError> {
        if command {
            return self.send_command(text);
        }
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let salt = i64::from(rand_salt());
        let update = self.social.last_seen.generate_update();
        let sender = self
            .state
            .account
            .as_ref()
            .map(|account| account.profile_id);
        let signature = sender.and_then(|sender| {
            let timing = (millis, salt);
            self.social
                .chat_session
                .sign(sender, text, timing, &update.seen)
        });
        self.send(PLAY_SERVERBOUND_CHAT_PACKET_ID, |output| {
            let seen = (update.offset, update.acknowledged, update.checksum);
            encode_chat(text, (millis, salt), signature.as_deref(), seen, output);
            Ok(())
        })
    }

    /// `ClientPacketListener.sendCommand`: a command with a message in it is signed (each
    /// message by the chat session, if there is one); any other goes as plain text.
    fn send_command(&mut self, command: &str) -> Result<(), ConnectionError> {
        let selectors = self.can_use_selectors();
        let signable = self.social.commands.as_ref().map(|tree| {
            let parse = tree.parse_as(command, 0, (selectors, true));
            photon_game::commands::signable_arguments(tree, &parse, command)
        });
        let Some(arguments) = signable.filter(|arguments| !arguments.is_empty()) else {
            return self.send(PLAY_SERVERBOUND_CHAT_COMMAND_PACKET_ID, |output| {
                let units: Vec<u16> = command.encode_utf16().collect();
                photon_core::utf8::write_utf8(output, &units, 32767)
                    .map_err(|_| ConnectionError::Codec("chat command"))
            });
        };
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let salt = i64::from(rand_salt());
        let update = self.social.last_seen.generate_update();
        let sender = self
            .state
            .account
            .as_ref()
            .map(|account| account.profile_id);
        let signatures: Vec<(String, Vec<u8>)> = arguments
            .into_iter()
            .filter_map(|(name, value)| {
                let signature =
                    self.social
                        .chat_session
                        .sign(sender?, &value, (millis, salt), &update.seen)?;
                Some((name, signature))
            })
            .collect();
        self.send(PLAY_SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, |output| {
            let seen = (update.offset, update.acknowledged, update.checksum);
            encode_chat_command_signed(command, (millis, salt), &signatures, seen, output);
            Ok(())
        })
    }

    /// `ClientPacketListener.markMessageAsProcessed`: a player's message was shown or left out.
    /// Past 64 messages the server is told how many came (`sendChatAcknowledgement`).
    pub fn mark_message_processed(
        &mut self,
        signature: &[u8],
        was_shown: bool,
    ) -> Result<(), ConnectionError> {
        let social = &mut self.social;
        if social.last_seen.add_pending(signature, was_shown) && social.last_seen.offset() > 64 {
            let offset = social.last_seen.take_offset();
            return self.send(PLAY_SERVERBOUND_CHAT_ACK_PACKET_ID, |output| {
                photon_core::wire::write_varint(offset, output);
                Ok(())
            });
        }
        Ok(())
    }
}

/// The players the social interactions screen lists: who is on the server, and who was seen and
/// has left, by id and name.
pub struct SocialPlayers {
    pub online: Vec<(photon_core::binary::Uuid, String)>,
    pub gone: Vec<(photon_core::binary::Uuid, String)>,
}
