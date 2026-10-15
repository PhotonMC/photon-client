use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_CHUNK_PACKET_ID,
        typed::<ChunkPacketObservation>,
    ),
    (
        PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID,
        typed::<LightUpdateObservation>,
    ),
    (
        PLAY_CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID,
        typed::<InitializeBorder>,
    ),
    (
        PLAY_CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID,
        typed::<SetBorderCenter>,
    ),
    (
        PLAY_CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID,
        typed::<SetBorderLerpSize>,
    ),
    (
        PLAY_CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID,
        typed::<SetBorderSize>,
    ),
    (
        PLAY_CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID,
        typed::<SetBorderWarningDelay>,
    ),
    (
        PLAY_CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID,
        typed::<SetBorderWarningDistance>,
    ),
    (
        PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID,
        typed::<LevelEventPacket>,
    ),
    (PLAY_CLIENTBOUND_EXPLODE_PACKET_ID, typed::<Explode>),
    (
        PLAY_CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID,
        typed::<LevelParticlesPacket>,
    ),
    (PLAY_CLIENTBOUND_SOUND_PACKET_ID, decode_sound),
    (PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID, decode_sound),
    (PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID, decode_sound),
    (PLAY_CLIENTBOUND_SET_TIME_PACKET_ID, typed::<SetTime>),
    (PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID, decode_hud),
    (PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, decode_scoreboard),
    (PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID, decode_scoreboard),
    (PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID, decode_scoreboard),
    (
        PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
        decode_scoreboard,
    ),
    (
        PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID,
        decode_scoreboard,
    ),
];

/// `handleLevelChunkWithLight`: the chunk, then its light.
impl ClientboundPacket for ChunkPacketObservation {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.apply_chunk(&self);
        Ok(())
    }

    /// A chunk never ended its packet by checking whether loading is done.
    fn checks_loading(&self) -> bool {
        false
    }
}

/// `handleLightUpdatePacket`.
impl ClientboundPacket for LightUpdateObservation {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.apply_light_update(&self);
        Ok(())
    }

    fn checks_loading(&self) -> bool {
        false
    }
}

/// `handleInitializeBorder`.
impl ClientboundPacket for InitializeBorder {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_world_border_initialize(self.0);
        Ok(())
    }
}

/// `handleSetBorderCenter`.
impl ClientboundPacket for SetBorderCenter {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_world_border_center(self.x, self.z);
        Ok(())
    }
}

/// `handleSetBorderLerpSize`.
impl ClientboundPacket for SetBorderLerpSize {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session
            .world
            .apply_world_border_lerp_size(self.old_size, self.new_size, self.time);
        Ok(())
    }
}

/// `handleSetBorderSize`.
impl ClientboundPacket for SetBorderSize {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_world_border_size(self.size);
        Ok(())
    }
}

/// `handleSetBorderWarningDelay`.
impl ClientboundPacket for SetBorderWarningDelay {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_world_border_warning_time(self.time);
        Ok(())
    }
}

/// `handleSetBorderWarningDistance`.
impl ClientboundPacket for SetBorderWarningDistance {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_world_border_warning_blocks(self.blocks);
        Ok(())
    }
}

/// `handleLevelEvent`: a global event (a wither's or a dragon's, heard from afar) is heard two
/// blocks from the camera.
impl ClientboundPacket for LevelEventPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let event = self.0;
        if event.global {
            session
                .world
                .apply_global_level_event(event.event, event.pos);
        } else {
            session
                .world
                .apply_level_event(event.event, event.pos, event.data);
        }
        Ok(())
    }
}

/// `handleExplosion`: the local player is pushed now; the sound and particles wait for the
/// particle engine, in order with the level events.
impl ClientboundPacket for Explode {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let Some((explosion, knockback)) = self.0 else {
            return Ok(());
        };
        if let (Some((x, y, z)), Some(player)) = (knockback, session.local.player.as_mut()) {
            player.add_delta_movement(photon_core::math::Vec3::new(x, y, z));
        }
        session.world.apply_explosion(explosion);
        Ok(())
    }
}

impl ClientboundPacket for LevelParticlesPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.effects.level_particles.extend(self.0);
        Ok(())
    }
}

fn decode_sound(packet_id: i32, payload: &[u8]) -> Result<Incoming, ConnectionError> {
    Ok(Box::new(Sound::decode_packet(packet_id, payload)?))
}

impl ClientboundPacket for Sound {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.effects.sounds.push(self.0);
        Ok(())
    }
}

/// `handleUpdates` of the clock manager: the clocks catch up first, then take the server's
/// values.
impl ClientboundPacket for SetTime {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.set_game_time(self.game_time as u64);
        session.tick_clocks(self.game_time);
        for (clock, total_ticks, partial_tick, rate) in self.clocks {
            session.level_info.clocks.insert(
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
}

/// A title, a bar, a boss bar or the tab list: what the HUD shows.
struct Hud(HudEvent);

fn decode_hud(packet_id: i32, payload: &[u8]) -> Result<Incoming, ConnectionError> {
    Ok(match parse_hud_packet(packet_id, payload) {
        Ok(Some(event)) => Box::new(Hud(event)),
        Ok(None) => Box::new(Ignored),
        Err(error) => Box::new(Malformed(format!("hud packet {packet_id}: {error:?}"))),
    })
}

impl ClientboundPacket for Hud {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.hud_events.push(self.0);
        Ok(())
    }
}

/// An objective, a score or a team.
struct Scoreboard(photon_game::scoreboard::ScoreboardEvent);

fn decode_scoreboard(packet_id: i32, payload: &[u8]) -> Result<Incoming, ConnectionError> {
    Ok(match parse_scoreboard_packet(packet_id, payload) {
        Ok(Some(event)) => Box::new(Scoreboard(event)),
        Ok(None) => Box::new(Ignored),
        Err(error) => Box::new(Malformed(format!(
            "scoreboard packet {packet_id}: {error:?}"
        ))),
    })
}

impl ClientboundPacket for Scoreboard {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.scoreboard.apply(self.0);
        Ok(())
    }
}
