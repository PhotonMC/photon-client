use super::*;

impl PlaySession {
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
    pub(in super::super) fn apply_player_info(&mut self, id: (u64, u64), actions: Vec<InfoAction>) {
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
    pub(in super::super) fn receive_player_chat(&mut self, packet: PlayerChatPacket) {
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
    pub(in super::super) fn open_container_id(&self) -> i32 {
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
