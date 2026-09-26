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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientBlockEntity {
    pub type_id: i32,
    pub tag: Option<Vec<u8>>,
    pub(crate) spawner_state: Option<ClientSpawnerState>,
}

impl ClientBlockEntity {
    pub(crate) fn from_wire(type_id: i32, tag: Option<Vec<u8>>) -> Self {
        let spawner_state = parse_spawner_state(type_id, tag.as_deref());
        Self {
            type_id,
            tag,
            spawner_state,
        }
    }

    pub fn spawner_state(&self) -> Option<&ClientSpawnerState> {
        self.spawner_state.as_ref()
    }
}

pub(crate) fn parse_spawner_state(type_id: i32, tag: Option<&[u8]>) -> Option<ClientSpawnerState> {
    if type_id != 9 {
        return None;
    }
    let mut input = tag?;
    let fields = crate::nbt::read_nbt(&mut input).ok()??;
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

pub(crate) fn spawner_short_number(fields: &[crate::nbt::Field], key: &str, default: i32) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .map_or(default, |value| value as i16 as i32)
}

pub(crate) fn spawner_int_number(fields: &[crate::nbt::Field], key: &str, default: i32) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .unwrap_or(default)
}

pub(crate) fn numeric_nbt_value(value: &crate::nbt::FieldValue) -> Option<i32> {
    match value {
        crate::nbt::FieldValue::Byte(value) => Some(i32::from(*value)),
        crate::nbt::FieldValue::Short(value) => Some(i32::from(*value)),
        crate::nbt::FieldValue::Int(value) => Some(*value),
        crate::nbt::FieldValue::Long(value) => Some(*value as i32),
        crate::nbt::FieldValue::Float(value) => Some(*value as i32),
        crate::nbt::FieldValue::Double(value) => Some(*value as i32),
        _ => None,
    }
}
