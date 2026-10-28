use super::*;
use photon_core::ids::EntityId;

pub fn parse_player_position_packet(input: &[u8]) -> Result<PlayerPositionPacket, ConnectionError> {
    let mut input = input;
    let packet = PlayerPositionPacket {
        teleport_id: read_varint(&mut input, "player position teleport id")?,
        x: read_f64(&mut input, "player position x")?,
        y: read_f64(&mut input, "player position y")?,
        z: read_f64(&mut input, "player position z")?,
        delta_x: read_f64(&mut input, "player position delta x")?,
        delta_y: read_f64(&mut input, "player position delta y")?,
        delta_z: read_f64(&mut input, "player position delta z")?,
        y_rot: read_f32(&mut input, "player position y rotation")?,
        x_rot: read_f32(&mut input, "player position x rotation")?,
        relative_flags: read_i32(&mut input, "player position relative flags")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player position trailing payload"));
    }
    Ok(packet)
}

pub fn resolve_player_position(
    packet: PlayerPositionPacket,
    current: Option<(f64, f64, f64, f32, f32)>,
) -> (f64, f64, f64, f32, f32) {
    let current = current.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));
    let x = if packet.relative_flags & PLAYER_RELATIVE_X_FLAG != 0 {
        current.0 + packet.x
    } else {
        packet.x
    };
    let y = if packet.relative_flags & PLAYER_RELATIVE_Y_FLAG != 0 {
        current.1 + packet.y
    } else {
        packet.y
    };
    let z = if packet.relative_flags & PLAYER_RELATIVE_Z_FLAG != 0 {
        current.2 + packet.z
    } else {
        packet.z
    };
    let y_rot = if packet.relative_flags & PLAYER_RELATIVE_Y_ROT_FLAG != 0 {
        current.3 + packet.y_rot
    } else {
        packet.y_rot
    };
    let x_rot = if packet.relative_flags & PLAYER_RELATIVE_X_ROT_FLAG != 0 {
        (current.4 + packet.x_rot).clamp(-90.0, 90.0)
    } else {
        packet.x_rot.clamp(-90.0, 90.0)
    };
    (x, y, z, y_rot, x_rot)
}

/// `ClientboundPlayerCombatKillPacket`: who died and the message for them.
pub fn parse_player_combat_kill_packet(
    input: &[u8],
) -> Result<(EntityId, photon_core::text::Component), ConnectionError> {
    let mut input = input;
    let id = EntityId(read_varint(&mut input, "combat kill player id")?);
    Ok((id, read_chat_component(&mut input, "combat kill message")?))
}

pub fn parse_login_packet(input: &[u8]) -> Result<LoginObservation, ConnectionError> {
    let mut input = input;
    let packet = LoginObservation {
        player_id: EntityId(read_i32(&mut input, "login player id")?),
        hardcore: read_bool(&mut input, "login hardcore")?,
        level_count: {
            let count = read_count(&mut input, "login level count")?;
            for _ in 0..count {
                read_identifier_string(&mut input, "login level identifier")?;
            }
            count
        },
        max_players: read_varint(&mut input, "login max players")?,
        chunk_radius: read_varint(&mut input, "login chunk radius")?,
        simulation_distance: read_varint(&mut input, "login simulation distance")?,
        reduced_debug_info: read_bool(&mut input, "login reduced debug info")?,
        show_death_screen: read_bool(&mut input, "login show death screen")?,
        do_limited_crafting: read_bool(&mut input, "login limited crafting")?,
        spawn_info: parse_spawn_info(&mut input)?,
        online_mode: read_bool(&mut input, "login online mode")?,
        enforces_secure_chat: read_bool(&mut input, "login secure chat")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("login trailing payload"));
    }
    Ok(packet)
}

pub fn parse_respawn_packet(input: &[u8]) -> Result<(SpawnInfoObservation, u8), ConnectionError> {
    let mut input = input;
    let spawn_info = parse_spawn_info(&mut input)?;
    let data_to_keep = read_byte(&mut input, "respawn data to keep")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("respawn trailing payload"));
    }
    Ok((spawn_info, data_to_keep))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPositionPacket {
    pub teleport_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: i32,
}

/// Fields shared by the login and respawn spawn-info payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnInfoObservation {
    pub dimension_type_id: i32,
    pub dimension: String,
    pub seed: i64,
    pub game_type: u8,
    pub previous_game_type: Option<u8>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<GlobalPositionObservation>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

/// The dimension and player-state fields carried by the initial login packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginObservation {
    pub player_id: EntityId,
    pub hardcore: bool,
    pub level_count: usize,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub spawn_info: SpawnInfoObservation,
    pub online_mode: bool,
    pub enforces_secure_chat: bool,
}

/// One clientbound respawn packet plus its client-world transition result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnObservation {
    pub spawn_info: SpawnInfoObservation,
    pub data_to_keep: u8,
    pub local_player_id: Option<EntityId>,
    pub local_player_recreated: bool,
    pub local_player_generation: Option<u64>,
    pub keep_entity_data: bool,
    pub keep_attribute_modifiers: bool,
    pub local_player_attribute_count: usize,
    pub local_player_attribute_modifier_count: usize,
    pub dimension_changed: bool,
    pub cleared_chunk_count: usize,
    pub cleared_entity_count: usize,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound health update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthObservation {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
    pub applied_to_world: bool,
}

/// Wire-level values decoded from one clientbound player-abilities packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub can_instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

/// One clientbound player-abilities packet and its local-player application result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilitiesObservation {
    pub packet: PlayerAbilitiesPacket,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound game-event packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEventPacket {
    pub event_id: u8,
    pub param: f32,
}

/// One clientbound game-event packet and its local game-mode application result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEventObservation {
    pub packet: GameEventPacket,
    pub applied_to_world: bool,
    pub local_game_mode: Option<ClientPlayerGameMode>,
}

/// A serialized global position embedded in spawn info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPositionObservation {
    pub dimension: String,
    pub position: i64,
}

/// Wire-level fields observed from a clientbound player-position correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPositionObservation {
    pub teleport_id: i32,
    pub relative_flags: i32,
    pub applied_to_world: bool,
}

pub fn parse_set_health_packet(input: &[u8]) -> Result<HealthObservation, ConnectionError> {
    let mut input = input;
    let packet = HealthObservation {
        health: read_f32(&mut input, "set health health")?,
        food: read_varint(&mut input, "set health food")?,
        saturation: read_f32(&mut input, "set health saturation")?,
        applied_to_world: false,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("set health trailing payload"));
    }
    Ok(packet)
}

pub fn parse_player_abilities_packet(
    input: &[u8],
) -> Result<PlayerAbilitiesPacket, ConnectionError> {
    let mut input = input;
    let flags = read_byte(&mut input, "player abilities flags")?;
    let packet = PlayerAbilitiesPacket {
        invulnerable: flags & 0x01 != 0,
        flying: flags & 0x02 != 0,
        can_fly: flags & 0x04 != 0,
        can_instabuild: flags & 0x08 != 0,
        flying_speed: read_f32(&mut input, "player abilities flying speed")?,
        walking_speed: read_f32(&mut input, "player abilities walking speed")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player abilities trailing payload"));
    }
    Ok(packet)
}

pub fn parse_game_event_packet(input: &[u8]) -> Result<GameEventPacket, ConnectionError> {
    let mut input = input;
    let packet = GameEventPacket {
        event_id: read_byte(&mut input, "game event id")?,
        param: read_f32(&mut input, "game event parameter")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("game event trailing payload"));
    }
    Ok(packet)
}

pub(super) fn parse_spawn_info(input: &mut &[u8]) -> Result<SpawnInfoObservation, ConnectionError> {
    let dimension_type_id = read_varint(input, "spawn dimension type")?;
    let dimension = read_identifier_string(input, "spawn dimension")?;
    let seed = read_i64(input, "spawn seed")?;
    let game_type = read_byte(input, "spawn game type")?;
    let previous_game_type = match read_byte(input, "spawn previous game type")? {
        u8::MAX => None,
        value => Some(value),
    };
    let is_debug = read_bool(input, "spawn debug flag")?;
    let is_flat = read_bool(input, "spawn flat flag")?;
    let last_death_location = if read_bool(input, "spawn death-location presence")? {
        Some(GlobalPositionObservation {
            dimension: read_identifier_string(input, "spawn death-location dimension")?,
            position: read_i64(input, "spawn death-location position")?,
        })
    } else {
        None
    };
    let portal_cooldown = read_varint(input, "spawn portal cooldown")?;
    let sea_level = read_varint(input, "spawn sea level")?;
    Ok(SpawnInfoObservation {
        dimension_type_id,
        dimension,
        seed,
        game_type,
        previous_game_type,
        is_debug,
        is_flat,
        last_death_location,
        portal_cooldown,
        sea_level,
    })
}

wrapped_packet!(
    /// `ClientboundLoginPacket`.
    Login(LoginObservation) = parse_login_packet
);
wrapped_packet!(
    /// `ClientboundRespawnPacket`: where the player respawns, and what it keeps.
    Respawn((SpawnInfoObservation, u8)) = parse_respawn_packet
);
wrapped_packet!(
    /// `ClientboundSetHealthPacket`.
    SetHealth(HealthObservation) = parse_set_health_packet
);
wrapped_packet!(
    /// `ClientboundPlayerCombatKillPacket`: who died, and the message of the death.
    PlayerCombatKill((EntityId, photon_core::text::Component)) = parse_player_combat_kill_packet
);
wrapped_packet!(
    /// `ClientboundGameEventPacket`.
    GameEvent(GameEventPacket) = parse_game_event_packet
);
wrapped_packet!(
    /// `ClientboundPlayerAbilitiesPacket`.
    PlayerAbilities(PlayerAbilitiesPacket) = parse_player_abilities_packet
);
wrapped_packet!(
    /// `ClientboundPlayerPositionPacket`.
    PlayerPosition(PlayerPositionPacket) = parse_player_position_packet
);

/// `ClientboundSetExperiencePacket`.
pub struct SetExperience {
    pub progress: f32,
    pub level: i32,
    pub total: i32,
}

impl Decode for SetExperience {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        Ok(Self {
            progress: read_f32(&mut input, "float")?,
            level: read_varint(&mut input, "experience level")?,
            total: read_varint(&mut input, "total experience")?,
        })
    }
}

/// `ClientboundSetHeldSlotPacket`.
pub struct SetHeldSlot(pub i32);

impl Decode for SetHeldSlot {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        read_varint(&mut input, "held slot").map(Self)
    }
}

/// `ClientboundTickingStatePacket`: the tick rate and whether the level is frozen.
pub struct TickingState {
    pub tick_rate: f32,
    pub frozen: bool,
}

impl Decode for TickingState {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        Ok(Self {
            tick_rate: read_f32(&mut input, "tick rate")?,
            frozen: read_bool(&mut input, "frozen")?,
        })
    }
}

/// `ClientboundTickingStepPacket`: how many frozen ticks to run.
pub struct TickingStep(pub i32);

impl Decode for TickingStep {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        read_varint(&mut input, "tick steps").map(Self)
    }
}
