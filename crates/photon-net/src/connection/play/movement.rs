use super::*;

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
        self.aimed_entity = match aim {
            photon_game::game_mode::Aim::Entity { id, .. } => Some(id),
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
            cooldowns: &self.item_cooldowns,
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
        let Some(player) = self.player.as_ref() else {
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
        if self.loaded {
            return Ok(());
        }
        let Some(player) = self.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let chunk_x = photon_core::math::floor_f64(position.x).div_euclid(16);
        let chunk_z = photon_core::math::floor_f64(position.z).div_euclid(16);
        if !self.trace.world.contains_chunk(chunk_x, chunk_z) {
            return Ok(());
        }
        self.send(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, |_| Ok(()))?;
        self.loaded = true;
        Ok(())
    }

    /// `ClientPacketListener.handleMovePlayer`.
    /// `setValuesFromPositionPacket` for the local player: a rider stays where its vehicle puts it.
    pub(super) fn place_player(&mut self, packet: &PlayerPositionPacket) {
        let passenger = self.trace.world.local_player_is_passenger();
        let current = self.player.as_ref().map(|player| {
            let position = player.position();
            let (y_rot, x_rot) = player.rotation();
            (position.x, position.y, position.z, y_rot, x_rot)
        });
        let absolute = resolve_player_position(*packet, current);
        let position = Vec3::new(absolute.0, absolute.1, absolute.2);
        let known_health = self.health_received.then_some(self.status.health);
        let player = self.player.get_or_insert_with(|| {
            let mut player = LocalPlayer::new(
                self.player_id.unwrap_or(0),
                position,
                absolute.3,
                absolute.4,
                self.game_mode,
                self.auto_jump,
            );
            // A health update that came before the player did is its first.
            if let Some(health) = known_health {
                player.hurt_to(health);
            }
            player
        });
        if let Some(abilities) = self.abilities {
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
        self.death.take()
    }

    /// `LocalPlayer.respawn`: ask the server to bring the player back.
    pub fn respawn(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_perform_respawn,
        )
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
            .filter(|view| view.kind == photon_world::VehicleKind::Boat)
            .map(|view| view.y_rot);
        if let Some(player) = self.player.as_mut() {
            player.turn(y_rot_delta, x_rot_delta);
            if let Some(y_rot) = boat_y_rot {
                player.clamp_in_boat(y_rot);
            }
        }
    }

    /// How many times the server has placed the player: once on joining,
    /// and once more for every movement it refused.
    pub fn teleports(&self) -> u32 {
        self.teleports
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

    /// How many swings the session has sent.
    pub fn swings(&self) -> u32 {
        self.swings
    }

    /// Select a hotbar slot, 0 to 8; the server hears of it next tick.
    pub fn select_slot(&mut self, slot: i32) {
        if (0..9).contains(&slot) {
            self.status.selected_slot = slot;
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
}
