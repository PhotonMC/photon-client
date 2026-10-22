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
    /// The `id` of the entity of `SpawnData`, the one the spawner shows spinning; none when the
    /// spawner names no entity.
    pub(crate) entity_id: Option<String>,
    /// A trial spawner's `nextMobSpawnsAt`, in game time; zero for a mob spawner.
    pub(crate) next_mob_spawns_at: i64,
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
            entity_id: None,
            next_mob_spawns_at: 0,
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

    /// The type of the entity the spawner shows (`SpawnData.entity.id`).
    pub fn entity_id(&self) -> Option<&str> {
        self.entity_id.as_deref()
    }
}

/// `BaseSpawner.oSpin` and `spin`, in degrees, kept as bits so the block entity stays `Eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpawnerSpin {
    previous: u64,
    current: u64,
}

impl SpawnerSpin {
    pub fn previous(&self) -> f64 {
        f64::from_bits(self.previous)
    }

    pub fn current(&self) -> f64 {
        f64::from_bits(self.current)
    }

    /// One tick of `BaseSpawner.clientTick` with a player near: the spin turns by a rate that
    /// quickens as the delay runs down (float arithmetic, as in the reference).
    pub fn turn(&mut self, delay: i32) {
        self.previous = self.current;
        let step = 1000.0_f32 / (delay as f32 + 200.0_f32);
        self.current = ((self.current() + f64::from(step)) % 360.0).to_bits();
    }

    /// One tick of `TrialSpawner.tickClient` in a state that shows a mob: the spin turns by the
    /// state's speed over the ticks to the next spawn (double arithmetic).
    pub fn turn_trial(&mut self, speed: f64, delay: i64) {
        self.previous = self.current;
        self.current = ((self.current() + speed / (delay as f64 + 200.0)) % 360.0).to_bits();
    }

    /// A tick with no player near: the old spin catches up.
    pub fn rest(&mut self) {
        self.previous = self.current;
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

/// What an enchanting table's book keeps of its animation (`EnchantingTableBlockEntity`'s
/// `time`, `flip`, `oFlip`, `flipT`, `flipA`, `open`, `oOpen`, `rot`, `oRot` and `tRot`).
#[derive(Debug, Clone, Copy, Default)]
pub struct BookState {
    pub time: i32,
    pub flip: f32,
    pub o_flip: f32,
    pub flip_t: f32,
    pub flip_a: f32,
    pub open: f32,
    pub o_open: f32,
    pub rot: f32,
    pub o_rot: f32,
    pub t_rot: f32,
}

impl BookState {
    /// Every number as the bits it is held in: two states are the same only if all of them are.
    fn bits(&self) -> ([u32; 9], i32) {
        let floats = [
            self.flip,
            self.o_flip,
            self.flip_t,
            self.flip_a,
            self.open,
            self.o_open,
            self.rot,
            self.o_rot,
            self.t_rot,
        ];
        (floats.map(f32::to_bits), self.time)
    }
}

impl PartialEq for BookState {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }
}

impl Eq for BookState {}

/// One stretch of a beacon's beam (`BeaconBeamOwner.Section`): its colour (ARGB) and how many
/// blocks tall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeamSection {
    pub color: u32,
    pub height: i32,
}

/// What a beacon keeps of its beam (`BeaconBlockEntity`'s `beamSections`, the ones it is working
/// out, and how high up the column it has looked).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BeaconState {
    pub sections: Vec<BeamSection>,
    /// `BeaconBlockEntity.levels`: how many layers of the pyramid under it the client has seen.
    pub levels: i32,
    pub(crate) checking: Vec<BeamSection>,
    pub(crate) last_check_y: i32,
}

impl BeaconState {
    /// How high up the column the scan has looked (`lastCheckY`).
    pub fn last_check_y(&self) -> i32 {
        self.last_check_y
    }
}

/// A conduit's client state (`ConduitBlockEntity`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConduitState {
    /// `tickCount`.
    pub tick_count: i32,
    /// `activeRotation`, which only ever counts up by one.
    pub rotation: i32,
    /// `isActive`.
    pub active: bool,
    /// `isHunting`.
    pub hunting: bool,
    /// `effectBlocks`: the frame blocks the last look found.
    pub frame: Vec<(i32, i32, i32)>,
    /// `destroyTarget`: the mob the update tag says the conduit is attacking.
    pub target: Option<photon_core::binary::Uuid>,
}

/// A vault's display item's spin (`VaultClientData`), in whole degrees: it starts at none and
/// turns ten at a time, wrapped to a half turn either way, so it is never fractional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VaultSpin {
    pub previous: i32,
    pub current: i32,
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
    /// `TheEndGatewayBlockEntity.age`, in ticks.
    pub age: i64,
    /// `PotentSulfurBlockEntity.eruptionTick`: -1 until an eruption.
    pub eruption_tick: i64,
    /// The move a moving piston block is in the middle of.
    pub moving: Option<MovingPiston>,
    /// An enchanting table's book.
    pub book: BookState,
    /// A beacon's beam.
    pub beacon: BeaconState,
    /// A conduit's shell.
    pub conduit: ConduitState,
    /// A vault's display item.
    pub vault: VaultSpin,
    /// A spawner's spinning mob.
    pub spawner: SpawnerSpin,
}

impl Default for BlockEntityEvents {
    fn default() -> Self {
        Self {
            wobble: None,
            teleport_cooldown: 0,
            age: 0,
            eruption_tick: -1,
            moving: None,
            book: BookState::default(),
            beacon: BeaconState::default(),
            conduit: ConduitState::default(),
            vault: VaultSpin::default(),
            spawner: SpawnerSpin::default(),
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
        let events = BlockEntityEvents {
            age: parse_gateway_age(type_id, tag.as_deref()),
            conduit: ConduitState {
                target: parse_conduit_target(type_id, tag.as_deref()),
                ..ConduitState::default()
            },
            ..BlockEntityEvents::default()
        };
        Self {
            type_id,
            tag,
            spawner_state,
            events,
        }
    }

    /// The block entity as the level makes it (`BlockEntity.setLevel`) in a level whose lowest
    /// block is `min_y`: a potent sulfur block notes the game time as when it last erupted, and a
    /// beacon begins its scan from the bottom of the world.
    pub fn made_at(mut self, game_time: i64, min_y: i32) -> Self {
        match photon_data::block_entity_type::name(self.type_id) {
            Some("minecraft:potent_sulfur") if self.events.eruption_tick == -1 => {
                self.events.eruption_tick = game_time;
            }
            Some("minecraft:beacon") => self.events.beacon.last_check_y = min_y - 1,
            _ => {}
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

/// `ConduitBlockEntity.loadAdditional`'s `Target`: the mob it attacks, none if the tag names none.
fn parse_conduit_target(type_id: i32, tag: Option<&[u8]>) -> Option<photon_core::binary::Uuid> {
    if photon_data::block_entity_type::name(type_id) != Some("minecraft:conduit") {
        return None;
    }
    let mut input = tag?;
    let fields = photon_core::nbt::read_nbt(&mut input).ok()??;
    let target = fields.iter().find(|field| field.key == "Target")?;
    crate::particles::uuid_of_tag(&target.value)
}

/// `TheEndGatewayBlockEntity.loadAdditional`'s `Age`: none is none.
fn parse_gateway_age(type_id: i32, tag: Option<&[u8]>) -> i64 {
    if photon_data::block_entity_type::name(type_id) != Some("minecraft:end_gateway") {
        return 0;
    }
    let Some(mut input) = tag else { return 0 };
    let Ok(Some(fields)) = photon_core::nbt::read_nbt(&mut input) else {
        return 0;
    };
    fields
        .iter()
        .find(|field| field.key == "Age")
        .and_then(|field| match field.value {
            photon_core::nbt::FieldValue::Long(age) => Some(age),
            _ => None,
        })
        .unwrap_or(0)
}

pub(crate) fn parse_spawner_state(type_id: i32, tag: Option<&[u8]>) -> Option<ClientSpawnerState> {
    let name = photon_data::block_entity_type::name(type_id);
    if !matches!(
        name,
        Some("minecraft:mob_spawner" | "minecraft:trial_spawner")
    ) {
        return None;
    }
    let mut input = tag?;
    let fields = photon_core::nbt::read_nbt(&mut input).ok()??;
    if !input.is_empty() {
        return None;
    }
    if name == Some("minecraft:trial_spawner") {
        // `TrialSpawnerStateData.getUpdateTag`: the next spawn time while active and the spawn data.
        let next = fields.iter().find_map(|field| match field.value {
            photon_core::nbt::FieldValue::Long(time) if field.key == "next_mob_spawns_at" => {
                Some(time)
            }
            _ => None,
        });
        return Some(ClientSpawnerState {
            entity_id: spawn_data_entity(&fields, "spawn_data"),
            next_mob_spawns_at: next.unwrap_or(0),
            ..ClientSpawnerState::default()
        });
    }
    Some(ClientSpawnerState {
        delay: spawner_short_number(&fields, "Delay", 20),
        min_spawn_delay: spawner_int_number(&fields, "MinSpawnDelay", 200),
        max_spawn_delay: spawner_int_number(&fields, "MaxSpawnDelay", 800),
        spawn_count: spawner_int_number(&fields, "SpawnCount", 4),
        max_nearby_entities: spawner_int_number(&fields, "MaxNearbyEntities", 6),
        required_player_range: spawner_int_number(&fields, "RequiredPlayerRange", 16),
        spawn_range: spawner_int_number(&fields, "SpawnRange", 4),
        entity_id: spawn_data_entity(&fields, "SpawnData"),
        next_mob_spawns_at: 0,
    })
}

/// `SpawnData.entityToSpawn`'s `id`: none when `SpawnData` or its `entity` is missing or has no id.
fn spawn_data_entity(fields: &[photon_core::nbt::Field], key: &str) -> Option<String> {
    use photon_core::nbt::FieldValue;
    let compound = |fields: &[photon_core::nbt::Field], key: &str| {
        fields.iter().find_map(|field| match &field.value {
            FieldValue::Compound(inner) if field.key == key => Some(inner.clone()),
            _ => None,
        })
    };
    let entity = compound(&compound(fields, key)?, "entity")?;
    entity.iter().find_map(|field| match &field.value {
        FieldValue::Str(id) if field.key == "id" && !id.is_empty() => Some(id.clone()),
        _ => None,
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
