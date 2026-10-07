use super::*;

impl PlaySession {
    pub(super) fn on_chunk(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        self.apply_chunk(&parse_chunk_packet(payload)?);
        Ok(())
    }

    pub(super) fn on_light_update(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        self.apply_light_update(&parse_light_update_packet(payload)?);
        Ok(())
    }

    pub(super) fn on_forget_level_chunk(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let (x, z) = parse_forget_level_chunk_packet(payload)?;
        self.trace.world.unload_chunk(x, z);
        self.forgotten_chunks.push((x, z));
        self.mark_columns_around(x, z);
        Ok(())
    }

    pub(super) fn on_block_update(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let update = parse_block_update_packet(payload)?;
        self.mark_block(update.x, update.y, update.z);
        self.trace
            .world
            .apply_server_block_state(update.x, update.y, update.z, update.state_id);
        Ok(())
    }

    pub(super) fn on_section_blocks_update(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let mut input = payload;
        let section = read_i64(&mut input)?;
        let section_x = (section >> 42) as i32;
        let section_y = ((section << 44) >> 44) as i32;
        let section_z = ((section << 22) >> 42) as i32;
        let count = wire::read_varint(&mut input)
            .map_err(|_| ConnectionError::Codec("section blocks update"))?;
        for _ in 0..count {
            let packed = wire::read_varlong(&mut input)
                .map_err(|_| ConnectionError::Codec("section blocks update"))?;
            let state = (packed as u64 >> 12) as i32;
            let local = (packed & 0xfff) as i32;
            let (bx, by, bz) = (
                section_x * 16 + ((local >> 8) & 15),
                section_y * 16 + (local & 15),
                section_z * 16 + ((local >> 4) & 15),
            );
            self.mark_block(bx, by, bz);
            self.trace.world.apply_server_block_state(bx, by, bz, state);
        }
        Ok(())
    }

    pub(super) fn on_set_chunk_cache_radius(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let radius = parse_set_chunk_cache_radius_packet(payload)?;
        self.trace.world.update_chunk_cache_radius(radius);
        Ok(())
    }

    pub(super) fn on_set_chunk_cache_center(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let (x, z) = parse_set_chunk_cache_center_packet(payload)?;
        self.trace.world.update_chunk_cache_center(x, z);
        Ok(())
    }

    pub(super) fn on_chunk_batch_finished(
        &mut self,
        _payload: &[u8],
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, |output| {
            // The chunk batch calculator's starting estimate.
            output.extend_from_slice(&3.5_f32.to_be_bytes());
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn on_level_event(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let event = read_level_event_packet(&mut input)?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec("level event trailing payload"));
        }
        self.trace
            .world
            .apply_level_event(event.event, event.pos, event.data);
        Ok(())
    }

    pub(super) fn on_sound(
        &mut self,
        payload: &[u8],
        packet_id: i32,
    ) -> Result<(), ConnectionError> {
        let mut input = payload;
        let command = match packet_id {
            PLAY_CLIENTBOUND_SOUND_PACKET_ID => {
                voxel_game::sounds::SoundCommand::At(read_sound_packet(&mut input)?)
            }
            PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID => {
                voxel_game::sounds::SoundCommand::FollowingEntity(read_sound_entity_packet(
                    &mut input,
                )?)
            }
            _ => voxel_game::sounds::SoundCommand::Stop(read_stop_sound_packet(&mut input)?),
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("sound packet trailing payload"));
        }
        self.sounds.push(command);
        Ok(())
    }

    pub(super) fn on_set_time(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let game_time = read_i64(&mut input)?;
        let count =
            wire::read_varint(&mut input).map_err(|_| ConnectionError::Codec("set time"))?;
        self.trace.world.set_game_time(game_time as u64);
        // handleUpdates: the clocks catch up first, then take the
        // server's values.
        self.tick_clocks(game_time);
        for _ in 0..count {
            let clock = wire::read_varint(&mut input)
                .map_err(|_| ConnectionError::Codec("set time clock"))?;
            let total_ticks = wire::read_varlong(&mut input)
                .map_err(|_| ConnectionError::Codec("set time ticks"))?;
            let partial_tick = read_f32(&mut input)?;
            let rate = read_f32(&mut input)?;
            self.clocks.insert(
                clock,
                ClockState {
                    total_ticks,
                    partial_tick,
                    rate,
                },
            );
        }
        Ok(())
    }

    pub(super) fn on_block_changed_ack(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let sequence = read_varint(&mut &payload[..], "block changed ack sequence")?;
        for synced in self.trace.world.handle_block_changed_ack(sequence) {
            self.mark_block(synced.x, synced.y, synced.z);
            // `syncBlockState`: a block put back into the player
            // moves it back to where it stood when it predicted.
            let pos = voxel_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
            if let Some(position) = synced.player_position
                && let Some(player) = self.player.as_mut()
                && player.is_colliding(&self.trace.world, pos, synced.state)
            {
                player.abs_snap_to(position);
            }
        }
        Ok(())
    }

    pub(super) fn on_block_destruction(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_block_destruction_packet(payload)?;
        self.trace.world.destroy_block_progress(
            packet.entity_id,
            packet.x,
            packet.y,
            packet.z,
            i32::from(packet.progress),
        );
        Ok(())
    }

    pub(super) fn on_block_entity_data(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let entity = parse_block_entity_data_packet(payload)?;
        self.trace
            .world
            .apply_block_entity_update(BlockEntityInput {
                x: entity.x,
                y: entity.y,
                z: entity.z,
                type_id: entity.type_id,
                tag: entity.tag.as_deref(),
            });
        Ok(())
    }

    pub(super) fn on_block_event(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let (pos, event, data) = parse_block_event_packet(payload)?;
        self.trace.world.apply_chest_block_event(pos, event, data);
        Ok(())
    }

    pub(super) fn on_initialize_border(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_initialize_border_packet(payload)?;
        self.trace.world.apply_world_border_initialize(packet);
        Ok(())
    }

    pub(super) fn on_set_border_center(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let x = read_f64(&mut input, "set border center")?;
        let z = read_f64(&mut input, "set border center")?;
        self.trace.world.apply_world_border_center(x, z);
        Ok(())
    }

    pub(super) fn on_set_border_lerp_size(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let (old_size, new_size, time) = parse_set_border_lerp_size_packet(payload)?;
        self.trace
            .world
            .apply_world_border_lerp_size(old_size, new_size, time);
        Ok(())
    }

    pub(super) fn on_set_border_size(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let size = read_f64(&mut input, "set border size")?;
        self.trace.world.apply_world_border_size(size);
        Ok(())
    }

    pub(super) fn on_set_border_warning_delay(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let mut input = payload;
        let time = read_varint(&mut input, "set border warning delay")?;
        self.trace.world.apply_world_border_warning_time(time);
        Ok(())
    }

    pub(super) fn on_set_border_warning_distance(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let mut input = payload;
        let blocks = read_varint(&mut input, "set border warning distance")?;
        self.trace.world.apply_world_border_warning_blocks(blocks);
        Ok(())
    }

    pub(super) fn on_hud(&mut self, payload: &[u8], packet_id: i32) -> Result<(), ConnectionError> {
        match parse_hud_packet(packet_id, payload) {
            Ok(Some(event)) => self.hud_events.push(event),
            Ok(None) => {}
            Err(error) => self.warn(format!("hud packet {packet_id}: {error:?}")),
        };
        Ok(())
    }

    pub(super) fn on_scoreboard(
        &mut self,
        payload: &[u8],
        packet_id: i32,
    ) -> Result<(), ConnectionError> {
        match parse_scoreboard_packet(packet_id, payload) {
            Ok(Some(event)) => self.scoreboard.apply(event),
            Ok(None) => {}
            Err(error) => self.warn(format!("scoreboard packet {packet_id}: {error:?}")),
        }
        Ok(())
    }
}
