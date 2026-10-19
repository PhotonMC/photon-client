use super::*;
use photon_core::ids::{EntityId, ItemId};
use photon_data::item::registry::AIR_ITEM;

impl PlaySession {
    /// The attack and use keys' part of `handleKeybinds`, with what the
    /// player aims at. Runs before the tick, as the reference handles keys
    /// before the level ticks the player.
    pub fn keys(
        &mut self,
        keys: photon_game::game_mode::Keys,
        aim: photon_game::game_mode::Aim,
    ) -> Result<(), ConnectionError> {
        use photon_game::game_mode::{Context, Miner, Outgoing};
        self.local.aimed_entity = match aim {
            photon_game::game_mode::Aim::Entity { id, .. } => Some(id),
            _ => None,
        };
        self.ensure_has_sent_carried_item()?;
        let Some(player) = self.local.player.as_mut() else {
            return Ok(());
        };
        let miner = Miner::of(&self.world, self.local.status.selected_slot);
        let mut ctx = Context {
            world: &mut self.world,
            player,
            miner: &miner,
            mode: self.local.game_mode,
            selected_slot: self.local.status.selected_slot,
            cooldowns: &self.local.item_cooldowns,
        };
        let sent = self.local.game_mode_state.keys_tick(&mut ctx, keys, aim);
        for packet in sent {
            let id = match packet {
                Outgoing::Swing { .. } => {
                    self.local.swings += 1;
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
                        photon_core::direction::Direction::from_3d_data_value(i32::from(face))
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

    /// `handleMovePlayer`: where the server puts the player, which it accepts and answers with
    /// where the player now is.
    pub(super) fn handle_player_position(
        &mut self,
        packet: PlayerPositionPacket,
    ) -> Result<(), ConnectionError> {
        self.place_player(&packet);
        self.send(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, |output| {
            wire::write_varint(packet.teleport_id, output);
            Ok(())
        })?;
        self.send_player_pos_rot()
    }

    /// `handleTeleportEntity` for a vehicle that was removed under its rider: the rider is moved
    /// in its place and tells the server where it is.
    pub(super) fn move_former_rider(
        &mut self,
        packet: &EntityTeleportObservation,
    ) -> Result<(), ConnectionError> {
        self.place_player(&PlayerPositionPacket {
            teleport_id: 0,
            x: packet.x,
            y: packet.y,
            z: packet.z,
            delta_x: packet.delta_x,
            delta_y: packet.delta_y,
            delta_z: packet.delta_z,
            y_rot: packet.y_rot,
            x_rot: packet.x_rot,
            relative_flags: packet.relative_flags,
        });
        self.send_player_pos_rot()
    }

    /// The `PosRot` the player sends after a teleport: where it is now, in the air, free of walls.
    pub(super) fn send_player_pos_rot(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.local.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let (y_rot, x_rot) = player.rotation();
        let absolute = (position.x, position.y, position.z, y_rot, x_rot);
        self.send(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, |output| {
            encode_move_player_pos_rot(output, absolute, 0)
        })
    }

    /// The level counts as loaded once the player has a position and the
    /// chunk under it has arrived; then the client says so and starts ticking.
    pub(super) fn maybe_finish_loading(&mut self) -> Result<(), ConnectionError> {
        if self.local.loaded {
            return Ok(());
        }
        let Some(player) = self.local.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let chunk_x = photon_core::math::floor_f64(position.x).div_euclid(16);
        let chunk_z = photon_core::math::floor_f64(position.z).div_euclid(16);
        if !self.world.contains_chunk(chunk_x, chunk_z) {
            return Ok(());
        }
        self.send(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, |_| Ok(()))?;
        self.local.loaded = true;
        Ok(())
    }

    /// `ClientPacketListener.handleMovePlayer`.
    /// `setValuesFromPositionPacket` for the local player: a rider stays where its vehicle puts it.
    pub(super) fn place_player(&mut self, packet: &PlayerPositionPacket) {
        let passenger = self.world.local_player_is_passenger();
        let current = self.local.player.as_ref().map(|player| {
            let position = player.position();
            let (y_rot, x_rot) = player.rotation();
            (position.x, position.y, position.z, y_rot, x_rot)
        });
        let absolute = resolve_player_position(*packet, current);
        let position = Vec3::new(absolute.0, absolute.1, absolute.2);
        let known_health = self
            .local
            .health_received
            .then_some(self.local.status.health);
        let player = self.local.player.get_or_insert_with(|| {
            let mut player = LocalPlayer::new(
                self.local.player_id.unwrap_or(EntityId(0)),
                position,
                absolute.3,
                absolute.4,
                self.local.game_mode,
                self.link.auto_jump,
            );
            // A health update that came before the player did is its first.
            if let Some(health) = known_health {
                player.hurt_to(health);
            }
            if let Some((carry, cooldown)) = self.local.respawn_portal.take() {
                player.restore_portal(carry, cooldown);
            }
            player
        });
        if let Some(abilities) = self.local.abilities {
            player.set_abilities(abilities);
        }
        let current_delta = player.delta_movement();
        let delta = Vec3::new(
            packet.delta_x
                + if packet.relative_flags & (1 << 5) != 0 {
                    current_delta.x
                } else {
                    0.0
                },
            packet.delta_y
                + if packet.relative_flags & (1 << 6) != 0 {
                    current_delta.y
                } else {
                    0.0
                },
            packet.delta_z
                + if packet.relative_flags & (1 << 7) != 0 {
                    current_delta.z
                } else {
                    0.0
                },
        );
        if !passenger {
            player.apply_teleport(position, delta, absolute.3, absolute.4);
        }
    }

    /// A death the death screen is yet to show: the server's message.
    pub fn take_death(&mut self) -> Option<photon_game::text::Component> {
        self.local.death.take()
    }

    /// Whether the server ended the game (`WIN_GAME`) since this was last asked: the credits are
    /// yet to be shown.
    pub fn take_win_game(&mut self) -> bool {
        std::mem::take(&mut self.local.win_game)
    }

    /// `LocalPlayer.respawn`: ask the server to bring the player back.
    pub fn respawn(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_perform_respawn,
        )
    }

    /// `StatsScreen`'s request: ask the server for the statistics.
    pub fn request_stats(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_request_stats,
        )
    }

    /// `ClientAdvancements`.
    pub fn advancements(&self) -> &photon_game::advancements::ClientAdvancements {
        &self.local.advancements
    }

    /// `ClientAdvancements`, for the screen that listens to it.
    pub fn advancements_mut(&mut self) -> &mut photon_game::advancements::ClientAdvancements {
        &mut self.local.advancements
    }

    /// `ServerboundSeenAdvancementsPacket`: the tab that was opened, or (`None`) the screen closed.
    pub fn send_seen_advancements(&mut self, tab: Option<&str>) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID, |output| {
            encode_seen_advancements(output, tab)
        })
    }

    /// The toasts the packets asked for since this was last asked, in the order they came.
    pub fn take_pending_toasts(&mut self) -> Vec<photon_game::toast::PendingToast> {
        std::mem::take(&mut self.local.pending_toasts)
    }

    /// `LocalPlayer.getStats`: what the server said of the statistics.
    pub fn stats(&self) -> &std::collections::HashMap<StatKey, i32> {
        &self.local.stats
    }

    /// `StatsCounter.getValue` of the statistic with this key: none counted is zero.
    pub fn stat(&self, key: StatKey) -> i32 {
        self.local.stats.get(&key).copied().unwrap_or(0)
    }

    /// Whether statistics arrived since this was last asked (`StatsScreen.onStatsUpdated`'s cue).
    pub fn take_stats_updated(&mut self) -> bool {
        std::mem::take(&mut self.local.stats_updated)
    }

    /// `ClientLevel.getMarkerParticleTarget`: the block a creative player's barrier or light in
    /// the main hand stands for, which the level shows markers on.
    pub fn marker_particle_block(&self) -> Option<usize> {
        if self.game_mode() != photon_world::ClientPlayerGameMode::Creative {
            return None;
        }
        let held = self
            .world
            .local_player()
            .and_then(|state| state.inventory_slot((36 + self.local.status.selected_slot) as i16))
            .map(|stack| stack.item_registry_id())?;
        let item = photon_world::block::rules::item(held)?;
        matches!(item.name, "minecraft:barrier" | "minecraft:light")
            .then(|| photon_data::block::registry::block_index(item.name))
            .flatten()
    }

    /// The registry id of what the off hand holds (air when empty).
    pub(super) fn off_hand_item(&self) -> ItemId {
        self.world
            .local_player()
            .and_then(|state| state.inventory_slot(45))
            .map_or(AIR_ITEM, |stack| stack.item_registry_id())
    }

    /// The mouse turns the player (`Entity.turn`), who in a boat is then held within a quarter
    /// turn of where the boat faces (`onPassengerTurned`).
    pub fn turn_player(&mut self, y_rot_delta: f64, x_rot_delta: f64) {
        let boat_y_rot = self
            .world
            .ridden_vehicle_of_local_player()
            .and_then(|id| self.world.vehicle_view(id))
            .filter(|view| view.kind == photon_world::VehicleKind::Boat)
            .map(|view| view.y_rot);
        if let Some(player) = self.local.player.as_mut() {
            player.turn(y_rot_delta, x_rot_delta);
            if let Some(y_rot) = boat_y_rot {
                player.clamp_in_boat(y_rot);
            }
        }
    }

    /// How many times the server has placed the player: once on joining,
    /// and once more for every movement it refused.
    pub fn teleports(&self) -> u32 {
        self.local.teleports
    }

    /// `LocalPlayer.swing` with the main hand: the arm swings, and the
    /// server hears of it.
    pub fn swing(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.local.player.as_mut() else {
            return Ok(());
        };
        player.swing();
        self.local.swings += 1;
        self.send(PLAY_SERVERBOUND_SWING_PACKET_ID, |output| {
            wire::write_varint(0, output);
            Ok(())
        })
    }

    /// How many swings the session has sent.
    pub fn swings(&self) -> u32 {
        self.local.swings
    }

    /// Select a hotbar slot, 0 to 8; the server hears of it next tick.
    pub fn select_slot(&mut self, slot: i32) {
        if (0..9).contains(&slot) {
            self.local.status.selected_slot = slot;
        }
    }

    /// `MultiPlayerGameMode.ensureHasSentCarriedItem`: the server hears of
    /// a new hotbar slot before anything the slot's item does.
    pub(super) fn ensure_has_sent_carried_item(&mut self) -> Result<(), ConnectionError> {
        if self.local.status.selected_slot != self.local.sent_slot {
            self.local.sent_slot = self.local.status.selected_slot;
            let slot = self.local.sent_slot as i16;
            self.send(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, |output| {
                output.extend_from_slice(&slot.to_be_bytes());
                Ok(())
            })?;
        }
        Ok(())
    }
}
