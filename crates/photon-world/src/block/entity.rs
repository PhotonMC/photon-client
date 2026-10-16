//! `ClientWorld` block_entity layer split out of the former single-file module.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSpawnerState {
    pub(crate) delay: i32,
    pub(crate) min_spawn_delay: i32,
    pub(crate) max_spawn_delay: i32,
    pub(crate) spawn_count: i32,
    pub(crate) max_nearby_entities: i32,
    pub(crate) required_player_range: i32,
    pub(crate) spawn_range: i32,
}

impl Default for ClientSpawnerState {
    /// A spawner with none of its data: `BaseSpawner`'s own defaults.
    fn default() -> Self {
        Self {
            delay: 20,
            min_spawn_delay: 200,
            max_spawn_delay: 800,
            spawn_count: 4,
            max_nearby_entities: 6,
            required_player_range: 16,
            spawn_range: 4,
        }
    }
}

impl ClientSpawnerState {
    pub fn delay(&self) -> i32 {
        self.delay
    }

    pub fn min_spawn_delay(&self) -> i32 {
        self.min_spawn_delay
    }

    pub fn max_spawn_delay(&self) -> i32 {
        self.max_spawn_delay
    }

    pub fn spawn_count(&self) -> i32 {
        self.spawn_count
    }

    pub fn max_nearby_entities(&self) -> i32 {
        self.max_nearby_entities
    }

    pub fn required_player_range(&self) -> i32 {
        self.required_player_range
    }

    pub fn spawn_range(&self) -> i32 {
        self.spawn_range
    }
}

/// `PistonMovingBlockEntity`: a block on its way between two places.
#[derive(Debug, Clone, PartialEq)]
pub struct MovingPiston {
    /// `movedState`: the block (or the piston) being moved.
    pub moved_state: photon_core::ids::BlockStateId,
    pub direction: photon_core::direction::Direction,
    pub extending: bool,
    /// `isSourcePiston`: the arm of the piston that moves, not a block it pushes.
    pub source: bool,
    pub progress: f32,
    /// `progressO`: the progress of the tick before.
    pub progress_old: f32,
    /// How many ticks the finished move has waited before the block settles.
    pub death_ticks: i32,
}

impl Eq for MovingPiston {}

impl MovingPiston {
    /// `getMovementDirection`: the way the block moves.
    pub fn movement_direction(&self) -> photon_core::direction::Direction {
        if self.extending {
            self.direction
        } else {
            self.direction.opposite()
        }
    }

    /// `getExtendedProgress`: how far the block is from where it ends, as a part of a block.
    pub fn extended_progress(&self) -> f32 {
        if self.extending {
            self.progress - 1.0
        } else {
            1.0 - self.progress
        }
    }
}

/// What block events set in a block entity of the client's own (`BlockEntity.triggerEvent`),
/// for the renderers and tickers that read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityEvents {
    /// `DecoratedPotBlockEntity.wobbleStartedAtTick` and the style's number
    /// (`lastWobbleStyle`), once a pot has wobbled.
    pub wobble: Option<(i64, i32)>,
    /// `TheEndGatewayBlockEntity.teleportCooldown`.
    pub teleport_cooldown: i32,
    /// `PotentSulfurBlockEntity.eruptionTick`: -1 until an eruption.
    pub eruption_tick: i64,
    /// The move a moving piston block is in the middle of.
    pub moving: Option<MovingPiston>,
}

impl Default for BlockEntityEvents {
    fn default() -> Self {
        Self {
            wobble: None,
            teleport_cooldown: 0,
            eruption_tick: -1,
            moving: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBlockEntity {
    pub type_id: i32,
    pub tag: Option<Vec<u8>>,
    pub(crate) spawner_state: Option<ClientSpawnerState>,
    pub(crate) events: BlockEntityEvents,
}

impl ClientBlockEntity {
    pub fn from_wire(type_id: i32, tag: Option<Vec<u8>>) -> Self {
        let spawner_state = parse_spawner_state(type_id, tag.as_deref());
        Self {
            type_id,
            tag,
            spawner_state,
            events: BlockEntityEvents::default(),
        }
    }

    /// The block entity as the level makes it (`BlockEntity.setLevel`): a potent sulfur block
    /// notes the game time as when it last erupted.
    pub fn made_at(mut self, game_time: i64) -> Self {
        if photon_data::block_entity_type::name(self.type_id) == Some("minecraft:potent_sulfur")
            && self.events.eruption_tick == -1
        {
            self.events.eruption_tick = game_time;
        }
        self
    }

    /// What block events set in the block entity.
    pub fn events(&self) -> &BlockEntityEvents {
        &self.events
    }

    pub fn spawner_state(&self) -> Option<&ClientSpawnerState> {
        self.spawner_state.as_ref()
    }
}

pub(crate) fn parse_spawner_state(type_id: i32, tag: Option<&[u8]>) -> Option<ClientSpawnerState> {
    if photon_data::block_entity_type::name(type_id) != Some("minecraft:mob_spawner") {
        return None;
    }
    let mut input = tag?;
    let fields = photon_core::nbt::read_nbt(&mut input).ok()??;
    if !input.is_empty() {
        return None;
    }
    Some(ClientSpawnerState {
        delay: spawner_short_number(&fields, "Delay", 20),
        min_spawn_delay: spawner_int_number(&fields, "MinSpawnDelay", 200),
        max_spawn_delay: spawner_int_number(&fields, "MaxSpawnDelay", 800),
        spawn_count: spawner_int_number(&fields, "SpawnCount", 4),
        max_nearby_entities: spawner_int_number(&fields, "MaxNearbyEntities", 6),
        required_player_range: spawner_int_number(&fields, "RequiredPlayerRange", 16),
        spawn_range: spawner_int_number(&fields, "SpawnRange", 4),
    })
}

pub(crate) fn spawner_short_number(
    fields: &[photon_core::nbt::Field],
    key: &str,
    default: i32,
) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .map_or(default, |value| value as i16 as i32)
}

pub(crate) fn spawner_int_number(
    fields: &[photon_core::nbt::Field],
    key: &str,
    default: i32,
) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .unwrap_or(default)
}

pub(crate) fn numeric_nbt_value(value: &photon_core::nbt::FieldValue) -> Option<i32> {
    match value {
        photon_core::nbt::FieldValue::Byte(value) => Some(i32::from(*value)),
        photon_core::nbt::FieldValue::Short(value) => Some(i32::from(*value)),
        photon_core::nbt::FieldValue::Int(value) => Some(*value),
        photon_core::nbt::FieldValue::Long(value) => Some(*value as i32),
        photon_core::nbt::FieldValue::Float(value) => Some(*value as i32),
        photon_core::nbt::FieldValue::Double(value) => Some(*value as i32),
        _ => None,
    }
}
