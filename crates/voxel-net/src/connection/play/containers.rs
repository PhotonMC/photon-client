use super::*;

impl PlaySession {
    /// `CrafterScreen.updateSlotState`: a slot of the crafter's grid is switched on or off, in the
    /// menu at once and by `ServerboundContainerSlotStateChangedPacket`.
    pub fn set_crafter_slot_state(
        &mut self,
        slot: i32,
        enabled: bool,
    ) -> Result<(), ConnectionError> {
        let Some(container_id) = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id())
        else {
            return Ok(());
        };
        let value = i16::from(!enabled);
        self.trace
            .world
            .apply_local_player_container_data(container_id, slot as i16, value);
        if let Some(entry) = self
            .open_menu
            .as_mut()
            .and_then(|menu| menu.data.get_mut(slot as usize))
        {
            *entry = value;
        }
        self.send(
            PLAY_SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID,
            |output| {
                output.extend_from_slice(&encode_container_slot_state(slot, container_id, enabled));
                Ok(())
            },
        )
    }

    /// `ServerboundSetBeaconPacket`: the effects the beacon's screen chose, by registry id.
    pub fn set_beacon(
        &mut self,
        primary: Option<i32>,
        secondary: Option<i32>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_BEACON_PACKET_ID, |output| {
            output.extend_from_slice(&encode_set_beacon(primary, secondary));
            Ok(())
        })
    }

    /// `LoomMenu.slotsChanged`'s list of patterns: with a banner and a dye in,
    /// those that need no item, or those the pattern item in the loom gives.
    /// Nothing once the banner has six layers.
    pub(super) fn loom_patterns(
        world: &voxel_world::ClientWorld,
        menu: &voxel_game::container_menu::ContainerMenu,
    ) -> (Vec<i32>, usize) {
        let stack = |slot: usize| menu.slots.get(slot).and_then(Option::as_ref);
        let (Some(banner), Some(_dye)) = (stack(0), stack(1)) else {
            return (Vec::new(), 0);
        };
        let layers =
            voxel_game::item_values::Values::new(banner.item_registry_id(), banner.patch_bytes())
                .banner_layers()
                .len();
        let tags = world.block_state_registry().banner_pattern_tags();
        let tag = match stack(2) {
            None => "minecraft:no_item_required".to_owned(),
            Some(pattern) => {
                // `provides_banner_patterns` names `pattern_item/<name>`.
                let name = usize::try_from(pattern.item_registry_id())
                    .ok()
                    .and_then(|id| voxel_data::item::registry::ITEMS.get(id))
                    .map_or("", |item| item.name);
                let short = name
                    .trim_start_matches("minecraft:")
                    .trim_end_matches("_banner_pattern");
                format!("minecraft:pattern_item/{short}")
            }
        };
        (tags.members(&tag).collect(), layers)
    }

    /// `AbstractContainerScreen.slotClicked` into the open menu: the player's
    /// own, or the container the server opened. The click as the client
    /// predicts it, the arm swings its drops cause, then the click packet.
    pub fn container_click(
        &mut self,
        slot: i32,
        button: i32,
        kind: voxel_game::container_menu::ClickType,
    ) -> Result<(), ConnectionError> {
        let player = voxel_game::container_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        let opened = self
            .trace
            .world
            .local_player()
            .is_some_and(|player| player.open_container().is_some());
        let (swings, body) = if opened {
            // Synced before the click; `None` when the container is of a kind
            // with no menu here, which no screen sends clicks for.
            if self.open_container_menu().is_none() {
                return Ok(());
            }
            let menu = self.open_menu.as_mut().expect("synced above");
            let clicked = menu.handle_click(slot, button, kind, player);
            let (slots, carried) = (menu.slots.clone(), menu.carried.clone());
            self.trace
                .world
                .set_local_player_container_menu(&slots, carried);
            clicked
        } else {
            let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
                return Ok(());
            };
            self.container_menu.load(slots, carried, state_id);
            let clicked = self.container_menu.handle_click(slot, button, kind, player);
            let menu = &self.container_menu;
            self.trace
                .world
                .set_local_player_menu(&menu.slots, menu.carried.clone());
            clicked
        };
        for _ in 0..swings {
            // `LocalPlayer.swing`: the arm, then the packet.
            if let Some(player) = self.player.as_mut() {
                player.swing();
            }
            self.swings += 1;
            self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                output.push(0);
                Ok(())
            })?;
        }
        self.send(PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    /// `CreativeModeInventoryScreen.slotClicked` into the player's own menu:
    /// the click as the client predicts it, then what it sends for it.
    pub fn creative_click(
        &mut self,
        creative: &mut voxel_game::creative_menu::CreativeMenu,
        click: voxel_game::creative_menu::Click,
    ) -> Result<(), ConnectionError> {
        use voxel_game::creative_menu::Effect;
        let Some((slots, carried, state_id)) = self.trace.world.local_player_menu() else {
            return Ok(());
        };
        self.container_menu.load(slots, carried, state_id);
        let player = voxel_game::container_menu::Player {
            selected: self.status.selected_slot.clamp(0, 8) as usize,
            infinite_materials: self.game_mode == voxel_world::ClientPlayerGameMode::Creative,
        };
        // `Player.canDropItems`: under the throttler's threshold.
        let can_drop = self.drop_spam < 1280;
        let effects = creative.slot_clicked(click, &mut self.container_menu, player, can_drop);
        let menu = &self.container_menu;
        self.trace
            .world
            .set_local_player_menu(&menu.slots, menu.carried.clone());
        if !player.infinite_materials {
            return Ok(());
        }
        for effect in effects {
            match effect {
                Effect::Swing => {
                    if let Some(player) = self.player.as_mut() {
                        player.swing();
                    }
                    self.swings += 1;
                    self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
                        output.push(0);
                        Ok(())
                    })?;
                }
                Effect::SetSlot(slot, stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, slot, stack.as_ref());
                            Ok(())
                        },
                    )?;
                }
                Effect::Drop(stack) => {
                    self.send(
                        PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                        |output| {
                            encode_creative_slot(output, -1, Some(&stack));
                            Ok(())
                        },
                    )?;
                    self.drop_spam += 20;
                }
            }
        }
        Ok(())
    }

    /// `LocalPlayer.closeContainer` for the inventory: the close packet,
    /// then `ContainerMenu.removed`, which empties the result slot.
    pub fn close_inventory(&mut self) -> Result<(), ConnectionError> {
        let opened = self
            .trace
            .world
            .local_player()
            .and_then(|player| player.open_container())
            .map(|container| container.container_id());
        let id = opened.unwrap_or(0);
        self.send(PLAY_SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, |output| {
            voxel_core::wire::write_varint(id, output);
            Ok(())
        })?;
        if opened.is_some() {
            // `LocalPlayer.clientSideCloseContainer`: back to the player's menu.
            self.trace.world.apply_local_player_container_close(id);
            self.open_menu = None;
            return Ok(());
        }
        if let Some((mut slots, carried, _)) = self.trace.world.local_player_menu() {
            slots[0] = None;
            self.trace.world.set_local_player_menu(&slots, carried);
        }
        Ok(())
    }

    /// The hands whose stack a use spent since the last call.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        self.game_mode_state.take_items_used()
    }

    /// The attack and use keys' part of `handleKeybinds`, with what the
    /// player aims at. Runs before the tick, as the reference handles keys
    /// before the level ticks the player.
    pub fn keys(
        &mut self,
        keys: voxel_game::game_mode::Keys,
        aim: voxel_game::game_mode::Aim,
    ) -> Result<(), ConnectionError> {
        use voxel_game::game_mode::{Context, Miner, Outgoing};
        self.aimed_entity = match aim {
            voxel_game::game_mode::Aim::Entity { id, .. } => Some(id),
            _ => None,
        };
        self.ensure_has_sent_carried_item()?;
        let Some(player) = self.player.as_mut() else {
            return Ok(());
        };
        let miner = Miner::of(&self.trace.world, self.status.selected_slot);
        let mut ctx = Context {
            world: &mut self.trace.world,
            player,
            miner: &miner,
            mode: self.game_mode,
            selected_slot: self.status.selected_slot,
        };
        let sent = self.game_mode_state.keys_tick(&mut ctx, keys, aim);
        for packet in sent {
            let id = match packet {
                Outgoing::Swing { .. } => {
                    self.swings += 1;
                    PLAY_SERVERBOUND_SWING_PACKET_ID
                }
                Outgoing::PlayerAction { x, y, z, .. } => {
                    // A predicted break changes the block it names.
                    self.mark_block(x, y, z);
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID
                }
                Outgoing::UseItemOn { x, y, z, face, .. } => {
                    // A predicted placement changes the block or the one
                    // in front of the face, and its neighbours.
                    let (dx, dy, dz) =
                        voxel_core::direction::Direction::from_3d_data_value(i32::from(face))
                            .step();
                    self.mark_block(x, y, z);
                    self.mark_block(x + dx, y + dy, z + dz);
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID
                }
                Outgoing::UseItem { .. } => PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                Outgoing::Attack { .. } => PLAY_SERVERBOUND_ATTACK_PACKET_ID,
                Outgoing::PickItemFromBlock { .. } => {
                    PLAY_SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID
                }
                Outgoing::PickItemFromEntity { .. } => {
                    PLAY_SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID
                }
                Outgoing::SpectatorAction { .. } => PLAY_SERVERBOUND_SPECTATOR_ACTION_PACKET_ID,
                Outgoing::Interact { .. } => PLAY_SERVERBOUND_INTERACT_PACKET_ID,
            };
            let body = packet.encode_body();
            self.send(id, |output| {
                output.extend_from_slice(&body);
                Ok(())
            })?;
        }
        Ok(())
    }

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
                    voxel_world::block_rules::item(item).is_some_and(|item| item.name == name)
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

    pub(super) fn send_movement(
        &mut self,
        packet: ServerboundMovement,
    ) -> Result<(), ConnectionError> {
        let id = match packet {
            ServerboundMovement::PlayerInput(_) => PLAY_SERVERBOUND_PLAYER_INPUT_PACKET_ID,
            ServerboundMovement::PlayerCommand { .. } => PLAY_SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
            ServerboundMovement::PlayerAbilities { .. } => {
                PLAY_SERVERBOUND_PLAYER_ABILITIES_PACKET_ID
            }
            ServerboundMovement::MovePlayerPos { .. } => PLAY_SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            ServerboundMovement::MovePlayerPosRot { .. } => {
                PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID
            }
            ServerboundMovement::MovePlayerRot { .. } => PLAY_SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID,
            ServerboundMovement::MovePlayerStatusOnly { .. } => {
                PLAY_SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID
            }
            ServerboundMovement::MoveVehicle { .. } => PLAY_SERVERBOUND_MOVE_VEHICLE_PACKET_ID,
            ServerboundMovement::PaddleBoat { .. } => PLAY_SERVERBOUND_PADDLE_BOAT_PACKET_ID,
        };
        let body = packet.encode_body();
        self.send(id, |output| {
            output.extend_from_slice(&body);
            Ok(())
        })
    }

    /// A packet from the reader thread: the heavy ones arrive parsed.
    pub(super) fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        match packet {
            Incoming::Raw(body) => self.handle(&body),
            Incoming::Chunk(chunk) => {
                self.apply_chunk(&chunk);
                Ok(())
            }
            Incoming::Light(update) => {
                self.apply_light_update(&update);
                Ok(())
            }
        }
    }

    /// `handleLevelChunkWithLight`: the chunk, then its light.
    pub(super) fn apply_chunk(&mut self, chunk: &ChunkPacketObservation) {
        self.mark_columns_around(chunk.x, chunk.z);
        self.trace.world.replace_chunk(
            chunk.x,
            chunk.z,
            chunk
                .section_observations
                .iter()
                .map(|section| SectionInput {
                    non_empty_block_count: section.non_empty_block_count,
                    fluid_count: section.fluid_count,
                    block_states: &section.block_state_values,
                    biomes: &section.biome_values,
                }),
        );
        self.trace.world.apply_block_entities(
            chunk.x,
            chunk.z,
            chunk.block_entities.iter().map(|entity| BlockEntityInput {
                x: entity.x,
                y: entity.y,
                z: entity.z,
                type_id: entity.type_id,
                tag: entity.tag.as_deref(),
            }),
        );
        self.trace.world.apply_chunk_light(
            chunk.x,
            chunk.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &chunk.light_data.sky_mask,
                    empty_mask: &chunk.light_data.empty_sky_mask,
                    updates: &chunk.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &chunk.light_data.block_mask,
                    empty_mask: &chunk.light_data.empty_block_mask,
                    updates: &chunk.light_data.block_updates,
                },
            },
        );
    }

    /// `handleLightUpdatePacket`: `readSectionList` redraws the sections
    /// whose light arrived when the queued task runs.
    pub(super) fn apply_light_update(&mut self, update: &LightUpdateObservation) {
        self.trace.world.apply_light_updates(
            update.x,
            update.z,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &update.light_data.sky_mask,
                    empty_mask: &update.light_data.empty_sky_mask,
                    updates: &update.light_data.sky_updates,
                },
                block: LightLayerInput {
                    mask: &update.light_data.block_mask,
                    empty_mask: &update.light_data.empty_block_mask,
                    updates: &update.light_data.block_updates,
                },
            },
        );
    }
}
