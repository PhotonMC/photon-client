//! Queryable client-side world state built from decoded chunk sections.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::binary::BitSet;
use crate::math::{
    mth_atan2, mth_cos_f64, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32, mth_rot_lerp_f64,
    mth_sin_f64,
};

pub(crate) const PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
pub(crate) const PLAYER_RESPAWN_KEEP_ENTITY_DATA: u8 = 2;
const VANILLA_PLAYER_ENTITY_TYPE_ID: i32 = 156;
pub(crate) const VANILLA_SNOWBALL_ENTITY_TYPE_ID: i32 = 120;
const VANILLA_AIR_BLOCK_STATE_ID: i32 = 0;
const ENTITY_SHARED_FLAGS_DATA_ID: u8 = 0;
const ENTITY_AIR_SUPPLY_DATA_ID: u8 = 1;
const ENTITY_CUSTOM_NAME_DATA_ID: u8 = 2;
const ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID: u8 = 3;
const ENTITY_SILENT_DATA_ID: u8 = 4;
const ENTITY_NO_GRAVITY_DATA_ID: u8 = 5;
const ENTITY_POSE_DATA_ID: u8 = 6;
const ENTITY_TICKS_FROZEN_DATA_ID: u8 = 7;
const ENTITY_LIVING_FLAGS_DATA_ID: u8 = 8;
const ENTITY_HEALTH_DATA_ID: u8 = 9;
const ENTITY_EFFECT_PARTICLES_DATA_ID: u8 = 10;
const ENTITY_EFFECT_AMBIENCE_DATA_ID: u8 = 11;
const ENTITY_ARROW_COUNT_DATA_ID: u8 = 12;
const ENTITY_STINGER_COUNT_DATA_ID: u8 = 13;
const ENTITY_SLEEPING_POS_DATA_ID: u8 = 14;
const ENTITY_MOB_FLAGS_DATA_ID: u8 = 15;
const ENTITY_DATA_SERIALIZER_BYTE_ID: i32 = 0;
const ENTITY_DATA_SERIALIZER_INT_ID: i32 = 1;
const ENTITY_DATA_SERIALIZER_FLOAT_ID: i32 = 3;
const ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID: i32 = 6;
const ENTITY_DATA_SERIALIZER_BOOLEAN_ID: i32 = 8;
const ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID: i32 = 11;
const ENTITY_DATA_SERIALIZER_PARTICLES_ID: i32 = 17;
const ENTITY_DATA_SERIALIZER_POSE_ID: i32 = 20;
const ENTITY_EFFECT_PARTICLE_TYPE_ID: i32 = 28;
const DEFAULT_AIR_SUPPLY: i32 = 300;
const DEFAULT_TICKS_REQUIRED_TO_FREEZE: i32 = 140;
const LIVING_ENTITY_HEAD_LERP_STEPS: u8 = 3;
const LIVING_ENTITY_POSITION_LERP_STEPS: u8 = 3;
const REMOTE_PLAYER_MOTION_LERP_STEPS: u8 = 3;
const ENTITY_INTERPOLATION_DISTANCE_SQR: f64 = 4096.0;
const ENTITY_RELATIVE_X_FLAG: i32 = 1 << 0;
const ENTITY_RELATIVE_Y_FLAG: i32 = 1 << 1;
const ENTITY_RELATIVE_Z_FLAG: i32 = 1 << 2;
const ENTITY_RELATIVE_Y_ROT_FLAG: i32 = 1 << 3;
const ENTITY_RELATIVE_X_ROT_FLAG: i32 = 1 << 4;
const ENTITY_RELATIVE_DELTA_X_FLAG: i32 = 1 << 5;
const ENTITY_RELATIVE_DELTA_Y_FLAG: i32 = 1 << 6;
const ENTITY_RELATIVE_DELTA_Z_FLAG: i32 = 1 << 7;
const ENTITY_RELATIVE_ROTATE_DELTA_FLAG: i32 = 1 << 8;
const SNOWBALL_GRAVITY: f64 = 0.03;
const LIVING_FLAG_USING_ITEM: u8 = 0;
const LIVING_FLAG_OFF_HAND: u8 = 1;
const LIVING_FLAG_SPIN_ATTACK: u8 = 2;
const MOB_FLAG_NO_AI: u8 = 0;
const MOB_FLAG_LEFT_HANDED: u8 = 1;
const MOB_FLAG_AGGRESSIVE: u8 = 2;
const SHARED_FLAG_ON_FIRE: u8 = 0;
const SHARED_FLAG_SHIFT_KEY_DOWN: u8 = 1;
const SHARED_FLAG_SPRINTING: u8 = 3;
const SHARED_FLAG_SWIMMING: u8 = 4;
const SHARED_FLAG_INVISIBLE: u8 = 5;
const SHARED_FLAG_GLOWING: u8 = 6;
const SHARED_FLAG_FALL_FLYING: u8 = 7;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClientEntityPose {
    #[default]
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

impl ClientEntityPose {
    pub const fn id(self) -> u8 {
        match self {
            Self::Standing => 0,
            Self::FallFlying => 1,
            Self::Sleeping => 2,
            Self::Swimming => 3,
            Self::SpinAttack => 4,
            Self::Crouching => 5,
            Self::LongJumping => 6,
            Self::Dying => 7,
            Self::Croaking => 8,
            Self::UsingTongue => 9,
            Self::Sitting => 10,
            Self::Roaring => 11,
            Self::Sniffing => 12,
            Self::Emerging => 13,
            Self::Digging => 14,
            Self::Sliding => 15,
            Self::Shooting => 16,
            Self::Inhaling => 17,
        }
    }

    fn from_wire_id(id: i32) -> Self {
        match id {
            1 => Self::FallFlying,
            2 => Self::Sleeping,
            3 => Self::Swimming,
            4 => Self::SpinAttack,
            5 => Self::Crouching,
            6 => Self::LongJumping,
            7 => Self::Dying,
            8 => Self::Croaking,
            9 => Self::UsingTongue,
            10 => Self::Sitting,
            11 => Self::Roaring,
            12 => Self::Sniffing,
            13 => Self::Emerging,
            14 => Self::Digging,
            15 => Self::Sliding,
            16 => Self::Shooting,
            17 => Self::Inhaling,
            _ => Self::Standing,
        }
    }
}

fn shared_flag_is_set(flags: u8, flag: u8) -> bool {
    flags & (1_u8 << flag) != 0
}

fn advance_head_rotation(current_bits: &mut u32, target_bits: u32, steps: &mut u8) {
    if *steps == 0 {
        return;
    }
    let step_count = f32::from(*steps);
    let current = f32::from_bits(*current_bits);
    let target = f32::from_bits(target_bits);
    *current_bits = mth_rot_lerp_f32(1.0 / step_count, current, target).to_bits();
    *steps -= 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntityInterpolationState {
    target_x_bits: u64,
    target_y_bits: u64,
    target_z_bits: u64,
    target_y_rot_bits: u32,
    target_x_rot_bits: u32,
    previous_x_bits: u64,
    previous_y_bits: u64,
    previous_z_bits: u64,
    previous_y_rot_bits: u32,
    previous_x_rot_bits: u32,
    steps: u8,
}

#[derive(Debug, Clone, Copy)]
struct ResolvedEntityPosition {
    current: (f64, f64, f64),
    target: (f64, f64, f64),
    ticking: bool,
}

impl EntityInterpolationState {
    fn target_position(self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.target_x_bits),
            f64::from_bits(self.target_y_bits),
            f64::from_bits(self.target_z_bits),
        )
    }

    fn target_rotation(self) -> (f32, f32) {
        (
            f32::from_bits(self.target_y_rot_bits),
            f32::from_bits(self.target_x_rot_bits),
        )
    }
}

fn read_effect_particle_type_ids(metadata: &mut &[u8]) -> Option<Vec<i32>> {
    let count = crate::wire::read_varint(metadata).ok()?;
    let count = usize::try_from(count).ok()?;
    if count > metadata.len() {
        return None;
    }
    let mut type_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let type_id = crate::wire::read_varint(metadata).ok()?;
        if type_id != ENTITY_EFFECT_PARTICLE_TYPE_ID {
            return None;
        }
        *metadata = metadata.get(4..)?;
        type_ids.push(type_id);
    }
    Some(type_ids)
}

fn skip_metadata_value(metadata: &mut &[u8], serializer_id: i32) -> Option<()> {
    match serializer_id {
        ENTITY_DATA_SERIALIZER_BYTE_ID | ENTITY_DATA_SERIALIZER_BOOLEAN_ID => {
            *metadata = metadata.get(1..)?;
        }
        ENTITY_DATA_SERIALIZER_INT_ID | ENTITY_DATA_SERIALIZER_POSE_ID => {
            crate::wire::read_varint(metadata).ok()?;
        }
        ENTITY_DATA_SERIALIZER_FLOAT_ID => {
            *metadata = metadata.get(4..)?;
        }
        ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                let component = crate::nbt::read_any_nbt(metadata).ok()?;
                component.as_ref()?;
            }
        }
        ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                crate::binary::read_blockpos(metadata).ok()?;
            }
        }
        ENTITY_DATA_SERIALIZER_PARTICLES_ID => {
            read_effect_particle_type_ids(metadata)?;
        }
        _ => return None,
    }
    Some(())
}

fn metadata_byte_value(metadata: &[u8], target_data_id: u8) -> Option<u8> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            if serializer_id != ENTITY_DATA_SERIALIZER_BYTE_ID {
                return None;
            }
            let (&value, rest) = metadata.split_first()?;
            if rest.is_empty() {
                return None;
            }
            return Some(value);
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn metadata_int_value(metadata: &[u8], target_data_id: u8) -> Option<i32> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            if serializer_id != ENTITY_DATA_SERIALIZER_INT_ID {
                return None;
            }
            let mut value_input = metadata;
            let value = crate::wire::read_varint(&mut value_input).ok()?;
            if value_input.is_empty() {
                return None;
            }
            return Some(value);
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn metadata_float_value(metadata: &[u8], target_data_id: u8) -> Option<f32> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            if serializer_id != ENTITY_DATA_SERIALIZER_FLOAT_ID {
                return None;
            }
            let value = u32::from_be_bytes(metadata.get(..4)?.try_into().ok()?);
            if metadata.get(4..)?.is_empty() {
                return None;
            }
            return Some(f32::from_bits(value));
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn metadata_pose_value(metadata: &[u8]) -> Option<ClientEntityPose> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == ENTITY_POSE_DATA_ID {
            if serializer_id != ENTITY_DATA_SERIALIZER_POSE_ID {
                return None;
            }
            let mut value_input = metadata;
            let value = crate::wire::read_varint(&mut value_input).ok()?;
            if value_input.is_empty() {
                return None;
            }
            return Some(ClientEntityPose::from_wire_id(value));
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn shared_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_SHARED_FLAGS_DATA_ID)
}

fn living_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_LIVING_FLAGS_DATA_ID)
}

fn mob_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_MOB_FLAGS_DATA_ID)
}

fn air_supply_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, ENTITY_AIR_SUPPLY_DATA_ID)
}

fn health_from_metadata(metadata: &[u8]) -> Option<f32> {
    metadata_float_value(metadata, ENTITY_HEALTH_DATA_ID)
}

fn living_count_from_metadata(metadata: &[u8], target_data_id: u8) -> Option<i32> {
    metadata_int_value(metadata, target_data_id)
}

fn arrow_count_from_metadata(metadata: &[u8]) -> Option<i32> {
    living_count_from_metadata(metadata, ENTITY_ARROW_COUNT_DATA_ID)
}

fn stinger_count_from_metadata(metadata: &[u8]) -> Option<i32> {
    living_count_from_metadata(metadata, ENTITY_STINGER_COUNT_DATA_ID)
}

fn effect_ambience_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_EFFECT_AMBIENCE_DATA_ID)
}

fn effect_particle_type_ids_from_metadata(metadata: &[u8]) -> Option<Vec<i32>> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == ENTITY_EFFECT_PARTICLES_DATA_ID {
            if serializer_id != ENTITY_DATA_SERIALIZER_PARTICLES_ID {
                return None;
            }
            return read_effect_particle_type_ids(&mut metadata);
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn sleeping_pos_from_metadata(metadata: &[u8]) -> Option<Option<crate::binary::BlockPos>> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == ENTITY_SLEEPING_POS_DATA_ID {
            if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID {
                return None;
            }
            let (&present, rest) = metadata.split_first()?;
            metadata = rest;
            if present == 0 {
                return Some(None);
            }
            let value = crate::binary::read_blockpos(&mut metadata).ok()?;
            if metadata.is_empty() {
                return None;
            }
            return Some(Some(value));
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn ticks_frozen_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, ENTITY_TICKS_FROZEN_DATA_ID)
}

fn metadata_boolean_value(metadata: &[u8], target_data_id: u8) -> Option<bool> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            if serializer_id != ENTITY_DATA_SERIALIZER_BOOLEAN_ID {
                return None;
            }
            let (&value, rest) = metadata.split_first()?;
            if rest.is_empty() {
                return None;
            }
            return Some(value != 0);
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

fn no_gravity_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_NO_GRAVITY_DATA_ID)
}

fn silent_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_SILENT_DATA_ID)
}

fn custom_name_visible_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID)
}

fn custom_name_present_from_metadata(metadata: &[u8]) -> Option<bool> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = crate::wire::read_varint(&mut metadata).ok()?;
        if data_id == ENTITY_CUSTOM_NAME_DATA_ID {
            if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID {
                return None;
            }
            let (&present, rest) = metadata.split_first()?;
            metadata = rest;
            if present == 0 {
                return Some(false);
            }
            let component = crate::nbt::read_any_nbt(&mut metadata).ok()?;
            return Some(component.is_some());
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

/// The vertical bounds used when mapping a chunk packet's section order to Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionBounds {
    pub min_y: i32,
    pub height: i32,
}

impl DimensionBounds {
    pub const fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    pub(crate) fn from_dimension_type(min_y: i32, height: i32) -> Option<Self> {
        // These limits and alignment checks mirror DimensionType's network codec
        // and record constructor in the pinned Java reference.
        const Y_SIZE: i32 = (1 << 12) - 32;
        const MAX_Y: i32 = (Y_SIZE >> 1) - 1;
        const MIN_Y: i32 = MAX_Y - Y_SIZE + 1;

        if !(MIN_Y..=MAX_Y).contains(&min_y)
            || !(16..=Y_SIZE).contains(&height)
            || height % 16 != 0
            || min_y % 16 != 0
        {
            return None;
        }
        let top = min_y.checked_add(height)?;
        if top > MAX_Y + 1 {
            return None;
        }
        Some(Self::new(min_y, height))
    }

    pub fn section_count(self) -> usize {
        if self.height <= 0 || self.height % 16 != 0 {
            return 0;
        }
        (self.height / 16) as usize
    }

    fn min_section_y(self) -> i32 {
        self.min_y.div_euclid(16)
    }
}

/// Raw item-stack state retained by the client inventory copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientItemStackState {
    item_count: i32,
    item_registry_id: i32,
    positive_component_type_ids: Vec<i32>,
    negative_component_type_ids: Vec<i32>,
}

impl ClientItemStackState {
    pub(crate) fn from_wire(
        item_count: i32,
        item_registry_id: i32,
        positive_component_type_ids: Vec<i32>,
        negative_component_type_ids: Vec<i32>,
    ) -> Self {
        Self {
            item_count,
            item_registry_id,
            positive_component_type_ids,
            negative_component_type_ids,
        }
    }

    pub fn item_count(&self) -> i32 {
        self.item_count
    }

    pub fn item_registry_id(&self) -> i32 {
        self.item_registry_id
    }

    pub fn positive_component_type_ids(&self) -> &[i32] {
        &self.positive_component_type_ids
    }

    pub fn negative_component_type_ids(&self) -> &[i32] {
        &self.negative_component_type_ids
    }
}

/// Raw state for the currently open non-player container menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientContainerState {
    container_id: i32,
    menu_type_id: i32,
    title_nbt: Vec<u8>,
    slot_count: usize,
    top_slot_count: Option<usize>,
    state_id: i32,
    slots: BTreeMap<i16, Option<ClientItemStackState>>,
    carried_item: Option<ClientItemStackState>,
}

impl ClientContainerState {
    fn new(container_id: i32, menu_type_id: i32, title_nbt: &[u8]) -> Self {
        Self {
            container_id,
            menu_type_id,
            title_nbt: title_nbt.to_vec(),
            slot_count: 0,
            top_slot_count: menu_top_slot_count(menu_type_id),
            state_id: 0,
            slots: BTreeMap::new(),
            carried_item: None,
        }
    }

    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    pub fn menu_type_id(&self) -> i32 {
        self.menu_type_id
    }

    pub fn title_nbt(&self) -> &[u8] {
        &self.title_nbt
    }

    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub fn top_slot_count(&self) -> Option<usize> {
        self.top_slot_count
    }

    pub fn state_id(&self) -> i32 {
        self.state_id
    }

    pub fn slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }
}

impl Default for DimensionBounds {
    fn default() -> Self {
        Self::new(-64, 384)
    }
}

impl DimensionBounds {
    /// Return the bounds of one of Java's built-in dimensions.
    pub fn for_builtin_dimension(dimension_key: &str) -> Self {
        match dimension_key {
            "minecraft:overworld" | "minecraft:overworld_caves" => Self::new(-64, 384),
            "minecraft:the_nether" | "minecraft:the_end" => Self::new(0, 256),
            _ => Self::default(),
        }
    }
}

/// The dimension-type fields consumed by the client world and renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionEnvironment {
    has_fixed_time: bool,
    has_skylight: bool,
    has_ceiling: bool,
    has_ender_dragon_fight: bool,
    coordinate_scale_bits: u64,
    logical_height: i32,
    ambient_light_bits: u32,
    skybox: DimensionSkybox,
    cardinal_light_type: CardinalLightType,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DimensionEnvironmentInput {
    pub(crate) has_fixed_time: bool,
    pub(crate) has_skylight: bool,
    pub(crate) has_ceiling: bool,
    pub(crate) has_ender_dragon_fight: bool,
    pub(crate) coordinate_scale: f64,
    pub(crate) logical_height: i32,
    pub(crate) ambient_light: f32,
    pub(crate) skybox: DimensionSkybox,
    pub(crate) cardinal_light_type: CardinalLightType,
}

impl DimensionEnvironment {
    fn new(input: DimensionEnvironmentInput) -> Self {
        Self {
            has_fixed_time: input.has_fixed_time,
            has_skylight: input.has_skylight,
            has_ceiling: input.has_ceiling,
            has_ender_dragon_fight: input.has_ender_dragon_fight,
            coordinate_scale_bits: input.coordinate_scale.to_bits(),
            logical_height: input.logical_height,
            ambient_light_bits: input.ambient_light.to_bits(),
            skybox: input.skybox,
            cardinal_light_type: input.cardinal_light_type,
        }
    }

    pub(crate) fn from_wire(input: DimensionEnvironmentInput, height: i32) -> Option<Self> {
        if !input.coordinate_scale.is_finite()
            || !(1.0e-5..=3.0e7).contains(&input.coordinate_scale)
            || !(0..=height).contains(&input.logical_height)
        {
            return None;
        }
        Some(Self::new(input))
    }

    /// Return the default environment for a built-in dimension.
    pub fn for_builtin_dimension(dimension_key: &str) -> Self {
        match dimension_key {
            "minecraft:the_nether" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: true,
                has_skylight: false,
                has_ceiling: true,
                has_ender_dragon_fight: false,
                coordinate_scale: 8.0,
                logical_height: 128,
                ambient_light: 0.1,
                skybox: DimensionSkybox::None,
                cardinal_light_type: CardinalLightType::Nether,
            }),
            "minecraft:the_end" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: true,
                has_skylight: true,
                has_ceiling: false,
                has_ender_dragon_fight: true,
                coordinate_scale: 1.0,
                logical_height: 256,
                ambient_light: 0.25,
                skybox: DimensionSkybox::End,
                cardinal_light_type: CardinalLightType::Default,
            }),
            "minecraft:overworld_caves" => Self::new(DimensionEnvironmentInput {
                has_fixed_time: false,
                has_skylight: true,
                has_ceiling: true,
                has_ender_dragon_fight: false,
                coordinate_scale: 1.0,
                logical_height: 384,
                ambient_light: 0.0,
                skybox: DimensionSkybox::Overworld,
                cardinal_light_type: CardinalLightType::Default,
            }),
            _ => Self::new(DimensionEnvironmentInput {
                has_fixed_time: false,
                has_skylight: true,
                has_ceiling: false,
                has_ender_dragon_fight: false,
                coordinate_scale: 1.0,
                logical_height: 384,
                ambient_light: 0.0,
                skybox: DimensionSkybox::Overworld,
                cardinal_light_type: CardinalLightType::Default,
            }),
        }
    }

    pub fn has_fixed_time(&self) -> bool {
        self.has_fixed_time
    }

    pub fn has_skylight(&self) -> bool {
        self.has_skylight
    }

    pub fn has_ceiling(&self) -> bool {
        self.has_ceiling
    }

    pub fn has_ender_dragon_fight(&self) -> bool {
        self.has_ender_dragon_fight
    }

    pub fn coordinate_scale(&self) -> f64 {
        f64::from_bits(self.coordinate_scale_bits)
    }

    pub fn logical_height(&self) -> i32 {
        self.logical_height
    }

    pub fn ambient_light(&self) -> f32 {
        f32::from_bits(self.ambient_light_bits)
    }

    pub fn skybox(&self) -> DimensionSkybox {
        self.skybox
    }

    pub fn cardinal_light_type(&self) -> CardinalLightType {
        self.cardinal_light_type
    }
}

/// The skybox selected by Java's dimension type codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionSkybox {
    None,
    Overworld,
    End,
}

impl DimensionSkybox {
    pub(crate) fn from_wire(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "overworld" => Some(Self::Overworld),
            "end" => Some(Self::End),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Overworld => "overworld",
            Self::End => "end",
        }
    }
}

/// The cardinal-light preset selected by Java's dimension type codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalLightType {
    Default,
    Nether,
}

impl CardinalLightType {
    pub(crate) fn from_wire(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "nether" => Some(Self::Nether),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Nether => "nether",
        }
    }
}

/// The client-side copy of the currently loaded world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientWorld {
    dimension: DimensionBounds,
    dimension_key: String,
    dimension_type_id: i32,
    seed: i64,
    is_debug: bool,
    is_flat: bool,
    sea_level: i32,
    dimension_environment: DimensionEnvironment,
    chunks: BTreeMap<(i32, i32), ClientChunk>,
    pending_light_updates: VecDeque<PendingLightUpdate>,
    entities: BTreeMap<i32, ClientEntity>,
    local_player: Option<ClientPlayerState>,
    game_time: u64,
}

impl Default for ClientWorld {
    fn default() -> Self {
        Self::new(DimensionBounds::default())
    }
}

impl ClientWorld {
    pub fn new(dimension: DimensionBounds) -> Self {
        Self {
            dimension,
            dimension_key: "minecraft:overworld".to_owned(),
            dimension_type_id: 0,
            seed: 0,
            is_debug: false,
            is_flat: false,
            sea_level: 63,
            dimension_environment: DimensionEnvironment::for_builtin_dimension(
                "minecraft:overworld",
            ),
            chunks: BTreeMap::new(),
            pending_light_updates: VecDeque::new(),
            entities: BTreeMap::new(),
            local_player: None,
            game_time: 0,
        }
    }

    pub fn dimension(&self) -> DimensionBounds {
        self.dimension
    }

    pub fn dimension_key(&self) -> &str {
        &self.dimension_key
    }

    pub fn dimension_type_id(&self) -> i32 {
        self.dimension_type_id
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn sea_level(&self) -> i32 {
        self.sea_level
    }

    pub fn dimension_environment(&self) -> &DimensionEnvironment {
        &self.dimension_environment
    }

    pub fn local_player(&self) -> Option<&ClientPlayerState> {
        self.local_player.as_ref()
    }

    pub(crate) fn set_local_player_id(&mut self, player_id: i32) {
        let mut player = ClientPlayerState::new(player_id);
        if let Some(entity) = self.entities.get(&player_id) {
            player.y_head_rot_bits = entity.y_head_rot_bits;
            player.old_y_head_rot_bits = entity.old_y_head_rot_bits;
            player.head_rotation_target_bits = entity.y_head_rot_bits;
        }
        self.local_player = Some(player);
    }

    pub(crate) fn apply_local_player_open_menu(
        &mut self,
        container_id: i32,
        menu_type_id: i32,
        title_nbt: &[u8],
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.open_container = Some(ClientContainerState::new(
            container_id,
            menu_type_id,
            title_nbt,
        ));
        true
    }

    pub(crate) fn apply_local_player_container_close(&mut self, _container_id: i32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if player.open_container.is_some() {
            player.open_container = None;
            true
        } else {
            false
        }
    }

    pub(crate) fn apply_local_player_position(
        &mut self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.x_bits = position.0.to_bits();
        player.y_bits = position.1.to_bits();
        player.z_bits = position.2.to_bits();
        player.motion_x_bits = motion.0.to_bits();
        player.motion_y_bits = motion.1.to_bits();
        player.motion_z_bits = motion.2.to_bits();
        player.y_rot_bits = rotation.0.to_bits();
        player.x_rot_bits = rotation.1.to_bits();
        true
    }

    pub(crate) fn apply_local_player_health(
        &mut self,
        health: f32,
        food: i32,
        saturation: f32,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.health_bits = health.to_bits();
        player.food = food;
        player.saturation_bits = saturation.to_bits();
        true
    }

    pub(crate) fn apply_local_player_container_content(
        &mut self,
        container_id: i32,
        state_id: i32,
        items: &[Option<ClientItemStackState>],
        carried_item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if container_id == 0 {
            player.inventory_state_id = state_id;
            player.inventory_slots = items
                .iter()
                .enumerate()
                .map(|(slot, item)| (slot as i16, item.clone()))
                .collect();
            player.carried_item = carried_item.cloned();
            return true;
        }

        let Some(container) = player
            .open_container
            .as_mut()
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.state_id = state_id;
        container.slot_count = items.len();
        container.slots = items
            .iter()
            .enumerate()
            .map(|(slot, item)| (slot as i16, item.clone()))
            .collect();
        container.carried_item = carried_item.cloned();
        true
    }

    pub(crate) fn apply_local_player_container_slot(
        &mut self,
        container_id: i32,
        state_id: i32,
        slot: i16,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if container_id == 0 {
            player.inventory_state_id = state_id;
            player.inventory_slots.insert(slot, item.cloned());
            return true;
        }

        let Some(container) = player
            .open_container
            .as_mut()
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.state_id = state_id;
        container.slots.insert(slot, item.cloned());
        true
    }

    pub(crate) fn apply_local_player_carried_item(
        &mut self,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        if let Some(container) = player.open_container.as_mut() {
            container.carried_item = item.cloned();
        } else {
            player.carried_item = item.cloned();
        }
        true
    }

    pub(crate) fn apply_local_player_inventory_slot(
        &mut self,
        slot: i32,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(menu_slot) = player_inventory_menu_slot(slot) else {
            return false;
        };
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.inventory_slots.insert(menu_slot, item.cloned());
        true
    }

    pub(crate) fn recreate_local_player(&mut self, data_to_keep: u8) -> bool {
        let Some(previous) = self.local_player.take() else {
            return false;
        };
        let keep_entity_data = data_to_keep & PLAYER_RESPAWN_KEEP_ENTITY_DATA != 0;
        let keep_attribute_modifiers = data_to_keep & PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS != 0;
        let mut player = ClientPlayerState::new(previous.id);
        player.generation = previous.generation.saturating_add(1);
        player.attributes = previous
            .attributes
            .iter()
            .map(|attribute| {
                let mut attribute = attribute.clone();
                if !keep_attribute_modifiers {
                    attribute.modifiers.clear();
                }
                attribute
            })
            .collect();
        if keep_entity_data {
            player.motion_x_bits = previous.motion_x_bits;
            player.motion_y_bits = previous.motion_y_bits;
            player.motion_z_bits = previous.motion_z_bits;
            player.y_rot_bits = previous.y_rot_bits;
            player.x_rot_bits = previous.x_rot_bits;
            player.y_head_rot_bits = previous.y_head_rot_bits;
            player.old_y_head_rot_bits = previous.old_y_head_rot_bits;
            player.head_rotation_target_bits = previous.head_rotation_target_bits;
            player.head_lerp_steps = previous.head_lerp_steps;
            player.shared_flags = previous.shared_flags;
            player.living_flags = previous.living_flags;
            player.arrow_count = previous.arrow_count;
            player.stinger_count = previous.stinger_count;
            player.effect_ambience = previous.effect_ambience;
            player.effect_particle_type_ids = previous.effect_particle_type_ids;
            player.sleeping_pos = previous.sleeping_pos;
            player.air_supply = previous.air_supply;
            player.custom_name_present = previous.custom_name_present;
            player.custom_name_visible = previous.custom_name_visible;
            player.no_gravity = previous.no_gravity;
            player.silent = previous.silent;
            player.pose = previous.pose;
            player.ticks_frozen = previous.ticks_frozen;
            player.entity_data = previous.entity_data;
        } else {
            player.y_rot_bits = (-180.0_f32).to_bits();
        }
        self.local_player = Some(player);
        true
    }

    pub(crate) fn apply_local_player_attributes(
        &mut self,
        player_id: i32,
        attributes: &[ClientAttributeState],
    ) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        for attribute in attributes {
            if let Some(existing) = player
                .attributes
                .iter_mut()
                .find(|existing| existing.attribute_id == attribute.attribute_id)
            {
                *existing = attribute.clone();
            } else {
                player.attributes.push(attribute.clone());
            }
        }
        true
    }

    pub(crate) fn apply_local_player_mob_effect(
        &mut self,
        player_id: i32,
        effect_id: i32,
        amplifier: i32,
        duration_ticks: i32,
        flags: u8,
    ) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        let effect = ClientMobEffectState {
            effect_id,
            amplifier,
            duration_ticks,
            flags,
        };
        if let Some(existing) = player
            .effects
            .iter_mut()
            .find(|existing| existing.effect_id == effect_id)
        {
            *existing = effect;
        } else {
            player.effects.push(effect);
        }
        true
    }

    pub(crate) fn remove_local_player_mob_effect(
        &mut self,
        player_id: i32,
        effect_id: i32,
    ) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        player
            .effects
            .retain(|effect| effect.effect_id != effect_id);
        true
    }

    pub(crate) fn replace_dimension_with_environment(
        &mut self,
        input: DimensionInput<'_>,
        bounds: DimensionBounds,
        environment: DimensionEnvironment,
    ) -> bool {
        let changed = self.dimension_key != input.key;
        self.dimension = bounds;
        self.dimension_key = input.key.to_owned();
        self.dimension_type_id = input.dimension_type_id;
        self.seed = input.seed;
        self.is_debug = input.is_debug;
        self.is_flat = input.is_flat;
        self.sea_level = input.sea_level;
        self.dimension_environment = environment;
        if changed {
            self.chunks.clear();
            self.pending_light_updates.clear();
            self.entities.clear();
            if let Some(player) = self.local_player.as_mut() {
                player.vehicle_id = None;
                player.passenger_ids.clear();
            }
            self.game_time = 0;
        }
        changed
    }

    pub fn game_time(&self) -> u64 {
        self.game_time
    }

    pub fn tick(&mut self) {
        self.game_time = self.game_time.wrapping_add(1);
        if let Some(player) = self.local_player.as_mut() {
            player.capture_old_transform();
            player.tick_head_rotation();
            player.tick_count = player.tick_count.wrapping_add(1);
            for effect in &mut player.effects {
                effect.tick_client();
            }
        }

        let ticking_entity_ids: Vec<i32> = self
            .entities
            .iter()
            .filter(|(_, entity)| entity.ticking)
            .map(|(&entity_id, _)| entity_id)
            .collect();
        for entity_id in ticking_entity_ids {
            let can_tick_snowball = self.entities.get(&entity_id).is_some_and(|entity| {
                entity.type_id == VANILLA_SNOWBALL_ENTITY_TYPE_ID
                    && self.snowball_path_is_open(
                        entity.position(),
                        entity.motion(),
                        entity.no_gravity,
                    )
            });
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("ticking entity was collected from the world");
            entity.capture_old_transform();
            entity.tick_count = entity.tick_count.wrapping_add(1);
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_position_interpolation();
            }
            entity.tick_head_rotation();
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_motion_interpolation();
            }
            if can_tick_snowball {
                entity.tick_snowball();
            }
        }
    }

    fn snowball_path_is_open(
        &self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        no_gravity: bool,
    ) -> bool {
        let air_drag = f64::from(0.99_f32);
        let gravity = if no_gravity { 0.0 } else { SNOWBALL_GRAVITY };
        let next_position = (
            position.0 + motion.0 * air_drag,
            position.1 + (motion.1 - gravity) * air_drag,
            position.2 + motion.2 * air_drag,
        );
        let current_block = (
            position.0.floor() as i32,
            position.1.floor() as i32,
            position.2.floor() as i32,
        );
        let next_block = (
            next_position.0.floor() as i32,
            next_position.1.floor() as i32,
            next_position.2.floor() as i32,
        );
        self.block_state_at(current_block.0, current_block.1, current_block.2)
            == Some(VANILLA_AIR_BLOCK_STATE_ID)
            && self.block_state_at(next_block.0, next_block.1, next_block.2)
                == Some(VANILLA_AIR_BLOCK_STATE_ID)
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.sections.len()).sum()
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunks.contains_key(&(chunk_x, chunk_z))
    }

    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        let removed = self.chunks.remove(&(chunk_x, chunk_z)).is_some();
        self.pending_light_updates
            .retain(|update| update.chunk_x != chunk_x || update.chunk_z != chunk_z);
        if removed {
            for entity in self.entities.values_mut() {
                if !entity.always_ticking
                    && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                {
                    entity.ticking = false;
                }
            }
        }
        removed
    }

    pub fn loaded_light_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.lights.len()).sum()
    }

    pub fn pending_light_update_count(&self) -> usize {
        self.pending_light_updates.len()
    }

    pub fn sky_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Sky, x, y, z)
    }

    pub fn block_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Block, x, y, z)
    }

    pub fn loaded_block_entity_count(&self) -> usize {
        self.chunks
            .values()
            .map(|chunk| chunk.block_entities.len())
            .sum()
    }

    pub fn block_entity_at(&self, x: i32, y: i32, z: i32) -> Option<&ClientBlockEntity> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        self.chunks.get(&chunk_key)?.block_entities.get(&(x, y, z))
    }

    pub fn loaded_entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn entity_at(&self, entity_id: i32) -> Option<&ClientEntity> {
        self.entities.get(&entity_id)
    }

    pub fn entities(&self) -> impl Iterator<Item = &ClientEntity> {
        self.entities.values()
    }

    pub fn entity_metadata_at(&self, entity_id: i32) -> Option<&[u8]> {
        self.entities
            .get(&entity_id)
            .and_then(|entity| entity.metadata())
    }

    pub fn remove_entity(&mut self, entity_id: i32) -> bool {
        if !self.entities.contains_key(&entity_id) {
            return false;
        }
        self.detach_entity_from_vehicle(entity_id);
        self.eject_entity_passengers(entity_id);
        self.entities.remove(&entity_id).is_some()
    }

    pub(crate) fn remove_entities(&mut self, entity_ids: &[i32]) -> usize {
        entity_ids
            .iter()
            .filter(|&&entity_id| self.remove_entity(entity_id))
            .count()
    }

    pub(crate) fn apply_light_updates(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        if !self.chunks.contains_key(&(chunk_x, chunk_z)) {
            let light_section_count = self.dimension.section_count().saturating_add(2);
            let invalid_sky_updates = count_invalid_light_updates(
                light_section_count,
                updates.sky.mask,
                updates.sky.updates,
            );
            let invalid_block_updates = count_invalid_light_updates(
                light_section_count,
                updates.block.mask,
                updates.block.updates,
            );
            if invalid_sky_updates != 0 || invalid_block_updates != 0 {
                return LightApplication {
                    invalid_sky_updates,
                    invalid_block_updates,
                    ..LightApplication::default()
                };
            }
            self.pending_light_updates
                .push_back(PendingLightUpdate::new(chunk_x, chunk_z, updates));
            return LightApplication {
                queued: true,
                ..LightApplication::default()
            };
        }

        self.apply_light_layers(chunk_x, chunk_z, updates)
    }

    fn apply_light_layers(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) else {
            return LightApplication::default();
        };
        let min_light_section_y = self.dimension.min_section_y() - 1;
        let light_section_count = self.dimension.section_count().saturating_add(2);
        let (applied_sky_sections, invalid_sky_updates) = apply_light_layer(
            &mut chunk.lights,
            min_light_section_y,
            light_section_count,
            LightLayer::Sky,
            updates.sky.mask,
            updates.sky.empty_mask,
            updates.sky.updates,
        );
        let (applied_block_sections, invalid_block_updates) = apply_light_layer(
            &mut chunk.lights,
            min_light_section_y,
            light_section_count,
            LightLayer::Block,
            updates.block.mask,
            updates.block.empty_mask,
            updates.block.updates,
        );
        LightApplication {
            applied_sky_sections,
            applied_block_sections,
            invalid_sky_updates,
            invalid_block_updates,
            queued: false,
        }
    }

    fn apply_pending_light_updates_for(&mut self, chunk_x: i32, chunk_z: i32) {
        let pending = std::mem::take(&mut self.pending_light_updates);
        for update in pending {
            if update.chunk_x == chunk_x && update.chunk_z == chunk_z {
                let application = self.apply_light_layers(chunk_x, chunk_z, update.as_input());
                debug_assert_eq!(application.invalid_sky_updates, 0);
                debug_assert_eq!(application.invalid_block_updates, 0);
            } else {
                self.pending_light_updates.push_back(update);
            }
        }
    }

    /// Return the resolved global block-state ID at a world coordinate.
    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get(&chunk_key)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        section.block_states.get(index).copied()
    }

    /// Apply a server-verified block-state update to an already loaded section.
    pub(crate) fn update_block_state(&mut self, x: i32, y: i32, z: i32, state_id: i32) -> bool {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunks.get_mut(&chunk_key) else {
            return false;
        };
        let section_y = y.div_euclid(16);
        let Some(section) = chunk.sections.get_mut(&section_y) else {
            return false;
        };
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let Some(block_state) = section.block_states.get_mut(index) else {
            return false;
        };
        *block_state = state_id;
        true
    }

    pub(crate) fn apply_block_entities<'a, I>(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        entities: I,
    ) -> usize
    where
        I: IntoIterator<Item = BlockEntityInput<'a>>,
    {
        let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) else {
            return 0;
        };
        let mut applied = 0;
        for entity in entities {
            if entity.x.div_euclid(16) != chunk_x || entity.z.div_euclid(16) != chunk_z {
                continue;
            }
            chunk.block_entities.insert(
                (entity.x, entity.y, entity.z),
                ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec)),
            );
            applied += 1;
        }
        applied
    }

    pub(crate) fn apply_block_entity_update(&mut self, entity: BlockEntityInput<'_>) -> bool {
        let chunk_key = (entity.x.div_euclid(16), entity.z.div_euclid(16));
        let Some(chunk) = self.chunks.get_mut(&chunk_key) else {
            return false;
        };
        chunk.block_entities.insert(
            (entity.x, entity.y, entity.z),
            ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec)),
        );
        true
    }

    pub(crate) fn apply_entity(&mut self, entity: EntityInput) -> bool {
        if self.entities.contains_key(&entity.entity_id) {
            self.remove_entity(entity.entity_id);
        }
        let (chunk_x, chunk_z) = entity_chunk_key((entity.x, entity.y, entity.z));
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        let head_rotation_bits = entity.rotation.2.to_bits();
        self.entities.insert(
            entity.entity_id,
            ClientEntity {
                uuid: entity.uuid,
                type_id: entity.type_id,
                tick_count: 0,
                x_bits: entity.x.to_bits(),
                y_bits: entity.y.to_bits(),
                z_bits: entity.z.to_bits(),
                old_x_bits: entity.x.to_bits(),
                old_y_bits: entity.y.to_bits(),
                old_z_bits: entity.z.to_bits(),
                motion_x_bits: entity.movement.0.to_bits(),
                motion_y_bits: entity.movement.1.to_bits(),
                motion_z_bits: entity.movement.2.to_bits(),
                motion_lerp_target_x_bits: entity.movement.0.to_bits(),
                motion_lerp_target_y_bits: entity.movement.1.to_bits(),
                motion_lerp_target_z_bits: entity.movement.2.to_bits(),
                motion_lerp_steps: 0,
                x_rot_bits: entity.rotation.1.to_bits(),
                y_rot_bits: entity.rotation.0.to_bits(),
                y_head_rot_bits: entity.rotation.2.to_bits(),
                old_x_rot_bits: entity.rotation.1.to_bits(),
                old_y_rot_bits: entity.rotation.0.to_bits(),
                old_y_head_rot_bits: entity.rotation.2.to_bits(),
                head_rotation_target_bits: entity.rotation.2.to_bits(),
                head_lerp_steps: 0,
                interpolation: None,
                on_ground: false,
                always_ticking: entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                ticking: ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                data: entity.data,
                metadata: None,
                shared_flags: 0,
                living_flags: 0,
                mob_flags: 0,
                air_supply: DEFAULT_AIR_SUPPLY,
                health_bits: None,
                arrow_count: 0,
                stinger_count: 0,
                effect_ambience: false,
                effect_particle_type_ids: Vec::new(),
                sleeping_pos: None,
                custom_name_present: false,
                custom_name_visible: false,
                no_gravity: false,
                silent: false,
                pose: ClientEntityPose::Standing,
                ticks_frozen: 0,
                vehicle_id: None,
                passenger_ids: Vec::new(),
            },
        );
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity.entity_id)
        {
            player.y_head_rot_bits = head_rotation_bits;
            player.old_y_head_rot_bits = head_rotation_bits;
            player.head_rotation_target_bits = head_rotation_bits;
            player.head_lerp_steps = 0;
        }
        true
    }

    pub(crate) fn apply_entity_passengers(
        &mut self,
        vehicle_id: i32,
        passenger_ids: &[i32],
    ) -> bool {
        if !self.entity_exists(vehicle_id) {
            return false;
        }

        self.eject_entity_passengers(vehicle_id);
        for &passenger_id in passenger_ids {
            if !self.entity_exists(passenger_id)
                || self.would_create_passenger_cycle(passenger_id, vehicle_id)
                || self.entity_vehicle_id(passenger_id) == Some(vehicle_id)
            {
                continue;
            }
            self.detach_entity_from_vehicle(passenger_id);
            self.set_entity_vehicle_id(passenger_id, Some(vehicle_id));
            if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
                vehicle.passenger_ids.push(passenger_id);
            } else if let Some(player) = self
                .local_player
                .as_mut()
                .filter(|player| player.id == vehicle_id)
            {
                player.passenger_ids.push(passenger_id);
            }
        }
        true
    }

    fn entity_exists(&self, entity_id: i32) -> bool {
        self.entities.contains_key(&entity_id)
            || self
                .local_player
                .as_ref()
                .is_some_and(|player| player.id == entity_id)
    }

    pub(crate) fn entity_vehicle_id(&self, entity_id: i32) -> Option<i32> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return entity.vehicle_id;
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .and_then(|player| player.vehicle_id)
    }

    pub(crate) fn entity_passengers(&self, entity_id: i32) -> Option<&[i32]> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return Some(&entity.passenger_ids);
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| player.passenger_ids.as_slice())
    }

    fn set_entity_vehicle_id(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.vehicle_id = vehicle_id;
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.vehicle_id = vehicle_id;
        }
    }

    fn detach_entity_from_vehicle(&mut self, entity_id: i32) {
        let Some(vehicle_id) = self.entity_vehicle_id(entity_id) else {
            return;
        };
        if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
            vehicle
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            player
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        }
        self.set_entity_vehicle_id(entity_id, None);
    }

    fn eject_entity_passengers(&mut self, vehicle_id: i32) {
        let passengers = if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
            Some(std::mem::take(&mut vehicle.passenger_ids))
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            Some(std::mem::take(&mut player.passenger_ids))
        } else {
            None
        };
        let Some(passengers) = passengers else {
            return;
        };
        for passenger_id in passengers {
            if self.entity_vehicle_id(passenger_id) == Some(vehicle_id) {
                self.set_entity_vehicle_id(passenger_id, None);
            }
        }
    }

    fn would_create_passenger_cycle(&self, passenger_id: i32, vehicle_id: i32) -> bool {
        if passenger_id == vehicle_id {
            return true;
        }
        let mut visited = BTreeSet::new();
        let mut current = Some(vehicle_id);
        while let Some(entity_id) = current {
            if !visited.insert(entity_id) {
                return true;
            }
            if entity_id == passenger_id {
                return true;
            }
            current = self.entity_vehicle_id(entity_id);
        }
        false
    }

    pub(crate) fn apply_entity_metadata(&mut self, entity_id: i32, metadata: &[u8]) -> bool {
        let shared_flags = shared_flags_from_metadata(metadata);
        let living_flags = living_flags_from_metadata(metadata);
        let mob_flags = mob_flags_from_metadata(metadata);
        let air_supply = air_supply_from_metadata(metadata);
        let health = health_from_metadata(metadata);
        let arrow_count = arrow_count_from_metadata(metadata);
        let stinger_count = stinger_count_from_metadata(metadata);
        let effect_ambience = effect_ambience_from_metadata(metadata);
        let effect_particle_type_ids = effect_particle_type_ids_from_metadata(metadata);
        let sleeping_pos = sleeping_pos_from_metadata(metadata);
        let custom_name_present = custom_name_present_from_metadata(metadata);
        let custom_name_visible = custom_name_visible_from_metadata(metadata);
        let no_gravity = no_gravity_from_metadata(metadata);
        let silent = silent_from_metadata(metadata);
        let pose = metadata_pose_value(metadata);
        let ticks_frozen = ticks_frozen_from_metadata(metadata);
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.entity_data = Some(metadata.to_vec());
            if let Some(shared_flags) = shared_flags {
                player.shared_flags = shared_flags;
            }
            if let Some(living_flags) = living_flags {
                player.living_flags = living_flags;
            }
            if let Some(air_supply) = air_supply {
                player.air_supply = air_supply;
            }
            if let Some(health) = health {
                player.health_bits = health.to_bits();
            }
            if let Some(arrow_count) = arrow_count {
                player.arrow_count = arrow_count;
            }
            if let Some(stinger_count) = stinger_count {
                player.stinger_count = stinger_count;
            }
            if let Some(effect_ambience) = effect_ambience {
                player.effect_ambience = effect_ambience;
            }
            if let Some(effect_particle_type_ids) = effect_particle_type_ids.as_ref() {
                player.effect_particle_type_ids = effect_particle_type_ids.clone();
            }
            if let Some(sleeping_pos) = sleeping_pos {
                player.sleeping_pos = sleeping_pos;
            }
            if let Some(custom_name_present) = custom_name_present {
                player.custom_name_present = custom_name_present;
            }
            if let Some(custom_name_visible) = custom_name_visible {
                player.custom_name_visible = custom_name_visible;
            }
            if let Some(no_gravity) = no_gravity {
                player.no_gravity = no_gravity;
            }
            if let Some(silent) = silent {
                player.silent = silent;
            }
            if let Some(pose) = pose {
                player.pose = pose;
            }
            if let Some(ticks_frozen) = ticks_frozen {
                player.ticks_frozen = ticks_frozen;
            }
            return true;
        }
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.metadata = Some(metadata.to_vec());
        if let Some(shared_flags) = shared_flags {
            entity.shared_flags = shared_flags;
        }
        if let Some(living_flags) = living_flags {
            entity.living_flags = living_flags;
        }
        if let Some(mob_flags) = mob_flags {
            entity.mob_flags = mob_flags;
        }
        if let Some(air_supply) = air_supply {
            entity.air_supply = air_supply;
        }
        if let Some(health) = health {
            entity.health_bits = Some(health.to_bits());
        }
        if let Some(arrow_count) = arrow_count {
            entity.arrow_count = arrow_count;
        }
        if let Some(stinger_count) = stinger_count {
            entity.stinger_count = stinger_count;
        }
        if let Some(effect_ambience) = effect_ambience {
            entity.effect_ambience = effect_ambience;
        }
        if let Some(effect_particle_type_ids) = effect_particle_type_ids {
            entity.effect_particle_type_ids = effect_particle_type_ids;
        }
        if let Some(sleeping_pos) = sleeping_pos {
            entity.sleeping_pos = sleeping_pos;
        }
        if let Some(custom_name_present) = custom_name_present {
            entity.custom_name_present = custom_name_present;
        }
        if let Some(custom_name_visible) = custom_name_visible {
            entity.custom_name_visible = custom_name_visible;
        }
        if let Some(no_gravity) = no_gravity {
            entity.no_gravity = no_gravity;
        }
        if let Some(silent) = silent {
            entity.silent = silent;
        }
        if let Some(pose) = pose {
            entity.pose = pose;
        }
        if let Some(ticks_frozen) = ticks_frozen {
            entity.ticks_frozen = ticks_frozen;
        }
        true
    }

    pub(crate) fn apply_entity_motion(&mut self, input: EntityMotionInput) -> bool {
        let Some(entity) = self.entities.get_mut(&input.entity_id) else {
            return false;
        };
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            entity.motion_lerp_target_x_bits = input.x.to_bits();
            entity.motion_lerp_target_y_bits = input.y.to_bits();
            entity.motion_lerp_target_z_bits = input.z.to_bits();
            entity.motion_lerp_steps = REMOTE_PLAYER_MOTION_LERP_STEPS;
        } else {
            entity.motion_x_bits = input.x.to_bits();
            entity.motion_y_bits = input.y.to_bits();
            entity.motion_z_bits = input.z.to_bits();
        }
        true
    }

    pub(crate) fn apply_entity_ground_state(&mut self, entity_id: i32, on_ground: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_ground = on_ground;
        true
    }

    pub(crate) fn apply_entity_rotation(&mut self, input: EntityRotationInput) -> bool {
        {
            let Some(entity) = self.entities.get_mut(&input.entity_id) else {
                return false;
            };
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID
                && (input.y_rot.is_some() || input.x_rot.is_some())
            {
                let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
                let y_rot = input.y_rot.map_or(current_y_rot, |y_rot| {
                    if input.relative_y_rot {
                        current_y_rot + y_rot
                    } else {
                        y_rot
                    }
                });
                let x_rot = input.x_rot.map_or(current_x_rot, |x_rot| {
                    if input.relative_x_rot {
                        current_x_rot + x_rot
                    } else {
                        x_rot
                    }
                });
                entity.interpolate_to(entity.interpolation_position(), y_rot, x_rot);
            } else {
                if let Some(y_rot) = input.y_rot {
                    let current = f32::from_bits(entity.y_rot_bits);
                    let value = if input.relative_y_rot {
                        current + y_rot
                    } else {
                        y_rot
                    };
                    entity.y_rot_bits = (value % 360.0).to_bits();
                }
                if let Some(x_rot) = input.x_rot {
                    let current = f32::from_bits(entity.x_rot_bits);
                    let value = if input.relative_x_rot {
                        current + x_rot
                    } else {
                        x_rot
                    };
                    entity.x_rot_bits = (value % 360.0).clamp(-90.0, 90.0).to_bits();
                }
            }
            if let Some(y_head_rot) = input.y_head_rot {
                if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                    entity.head_rotation_target_bits = y_head_rot.to_bits();
                    entity.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
                } else {
                    entity.y_head_rot_bits = y_head_rot.to_bits();
                }
            }
        }
        if let Some(y_head_rot) = input.y_head_rot
            && let Some(player) = self
                .local_player
                .as_mut()
                .filter(|player| player.id == input.entity_id)
        {
            player.head_rotation_target_bits = y_head_rot.to_bits();
            player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn apply_entity_position(&mut self, input: EntityPositionInput) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            None,
            EntityPositionUpdateMode::PositionSync,
        )
    }

    pub(crate) fn apply_entity_position_and_rotation(
        &mut self,
        input: EntityPositionInput,
        rotation: EntityRotationInput,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        let Some(entity) = self.entities.get(&input.entity_id) else {
            return false;
        };
        let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
        let target_y_rot = rotation.y_rot.map_or(current_y_rot, |y_rot| {
            if rotation.relative_y_rot {
                current_y_rot + y_rot
            } else {
                y_rot
            }
        });
        let target_x_rot = rotation.x_rot.map_or(current_x_rot, |x_rot| {
            if rotation.relative_x_rot {
                current_x_rot + x_rot
            } else {
                x_rot
            }
        });
        let target_x_rot = if mode == EntityPositionUpdateMode::Teleport {
            target_x_rot.clamp(-90.0, 90.0)
        } else {
            target_x_rot
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            Some((target_y_rot, target_x_rot)),
            mode,
        )
    }

    pub(crate) fn apply_entity_teleport(&mut self, input: EntityTeleportInput) -> bool {
        let Some((current_motion, current_y_rot, current_x_rot)) =
            self.entities.get(&input.entity_id).map(|entity| {
                (
                    entity.motion(),
                    entity.interpolation_rotation().0,
                    entity.interpolation_rotation().1,
                )
            })
        else {
            return false;
        };
        let target_y_rot = if input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0 {
            current_y_rot + input.y_rot
        } else {
            input.y_rot
        };
        let target_x_rot = if input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0 {
            current_x_rot + input.x_rot
        } else {
            input.x_rot
        }
        .clamp(-90.0, 90.0);
        let target_motion = resolve_entity_teleport_motion(
            current_motion,
            (current_y_rot, current_x_rot),
            (target_y_rot, target_x_rot),
            (input.delta_x, input.delta_y, input.delta_z),
            input.relative_flags,
        );
        let applied = self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: input.entity_id,
                x: input.x,
                y: input.y,
                z: input.z,
                relative_flags: input.relative_flags,
            },
            EntityRotationInput {
                entity_id: input.entity_id,
                y_rot: Some(input.y_rot),
                x_rot: Some(input.x_rot),
                y_head_rot: None,
                relative_y_rot: input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0,
                relative_x_rot: input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0,
            },
            EntityPositionUpdateMode::Teleport,
        );
        if applied && let Some(entity) = self.entities.get_mut(&input.entity_id) {
            entity.set_motion_if_finite(target_motion);
        }
        applied
    }

    pub(crate) fn apply_entity_move(&mut self, input: EntityMoveInput) -> bool {
        let Some(current) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.interpolation_position())
        else {
            return false;
        };
        let new_position = input.has_position.then(|| {
            (
                f64::from_bits(decode_entity_delta(current.0, input.delta_x)),
                f64::from_bits(decode_entity_delta(current.1, input.delta_y)),
                f64::from_bits(decode_entity_delta(current.2, input.delta_z)),
            )
        });
        let ticking = new_position.map(|position| {
            let (chunk_x, chunk_z) = entity_chunk_key(position);
            self.contains_chunk(chunk_x, chunk_z)
        });
        let entity = self
            .entities
            .get_mut(&input.entity_id)
            .expect("entity checked above");
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            let target_position = new_position.unwrap_or(current);
            let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
            let target_y_rot = input.y_rot.unwrap_or(current_y_rot);
            let target_x_rot = input.x_rot.unwrap_or(current_x_rot);
            entity.interpolate_to(target_position, target_y_rot, target_x_rot);
        } else {
            if input.has_position {
                let (x, y, z) = new_position.expect("position was requested");
                entity.x_bits = x.to_bits();
                entity.y_bits = y.to_bits();
                entity.z_bits = z.to_bits();
                entity.ticking = entity.always_ticking || ticking.expect("position ticking state");
            }
            if let Some(y_rot) = input.y_rot {
                entity.y_rot_bits = (y_rot % 360.0).to_bits();
            }
            if let Some(x_rot) = input.x_rot {
                entity.x_rot_bits = (x_rot % 360.0).clamp(-90.0, 90.0).to_bits();
            }
        }
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            entity.ticking = true;
        }
        true
    }

    fn resolve_entity_position(
        &self,
        input: &EntityPositionInput,
    ) -> Option<ResolvedEntityPosition> {
        let current = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.interpolation_position())?;
        let target = (
            if input.relative_flags & ENTITY_RELATIVE_X_FLAG != 0 {
                current.0 + input.x
            } else {
                input.x
            },
            if input.relative_flags & ENTITY_RELATIVE_Y_FLAG != 0 {
                current.1 + input.y
            } else {
                input.y
            },
            if input.relative_flags & ENTITY_RELATIVE_Z_FLAG != 0 {
                current.2 + input.z
            } else {
                input.z
            },
        );
        let (chunk_x, chunk_z) = entity_chunk_key(target);
        Some(ResolvedEntityPosition {
            current,
            target,
            ticking: self.contains_chunk(chunk_x, chunk_z),
        })
    }

    fn apply_entity_position_target(
        &mut self,
        entity_id: i32,
        current_position: (f64, f64, f64),
        target_position: (f64, f64, f64),
        ticking: bool,
        target_rotation: Option<(f32, f32)>,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            let rotation = target_rotation.unwrap_or_else(|| entity.interpolation_rotation());
            let distance_position = match mode {
                EntityPositionUpdateMode::PositionSync => entity.position(),
                EntityPositionUpdateMode::Teleport => current_position,
            };
            let delta_x = target_position.0 - distance_position.0;
            let delta_y = target_position.1 - distance_position.1;
            let delta_z = target_position.2 - distance_position.2;
            let can_interpolate = match mode {
                EntityPositionUpdateMode::PositionSync => entity.ticking,
                EntityPositionUpdateMode::Teleport => true,
            } && delta_x * delta_x + delta_y * delta_y + delta_z * delta_z
                <= ENTITY_INTERPOLATION_DISTANCE_SQR;
            if can_interpolate {
                entity.interpolate_to(target_position, rotation.0, rotation.1);
            } else {
                entity.snap_to(target_position, rotation.0, rotation.1);
            }
        } else {
            entity.x_bits = target_position.0.to_bits();
            entity.y_bits = target_position.1.to_bits();
            entity.z_bits = target_position.2.to_bits();
            if let Some((y_rot, x_rot)) = target_rotation {
                entity.y_rot_bits = (y_rot % 360.0).to_bits();
                entity.x_rot_bits = (x_rot % 360.0).clamp(-90.0, 90.0).to_bits();
            }
        }
        entity.ticking = entity.always_ticking || ticking;
        true
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunks.get(&(chunk_x, chunk_z))?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// Return the resolved biome registry ID at a world coordinate.
    pub fn biome_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get(&chunk_key)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize / 4;
        let local_y = y.rem_euclid(16) as usize / 4;
        let local_z = z.rem_euclid(16) as usize / 4;
        let index = local_y * 16 + local_z * 4 + local_x;
        section.biomes.get(index).copied()
    }

    fn light_at(&self, layer: LightLayer, x: i32, y: i32, z: i32) -> Option<u8> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunks.get(&chunk_key)?;
        let section_y = y.div_euclid(16);
        let section = chunk.lights.get(&section_y)?;
        let data = match layer {
            LightLayer::Sky => section.sky.as_ref()?,
            LightLayer::Block => section.block.as_ref()?,
        };
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        Some((data[index / 2] >> ((index & 1) * 4)) & 0x0f)
    }

    pub(crate) fn replace_chunk<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, sections: I)
    where
        I: IntoIterator<Item = SectionInput<'a>>,
    {
        let section_base_y = self.dimension.min_section_y();
        let sections = sections
            .into_iter()
            .enumerate()
            .map(|(index, input)| {
                (
                    section_base_y + index as i32,
                    ClientSection {
                        non_empty_block_count: input.non_empty_block_count,
                        fluid_count: input.fluid_count,
                        block_states: input.block_states.to_vec(),
                        biomes: input.biomes.to_vec(),
                    },
                )
            })
            .collect();
        self.chunks.insert(
            (chunk_x, chunk_z),
            ClientChunk {
                sections,
                lights: BTreeMap::new(),
                block_entities: BTreeMap::new(),
            },
        );
        self.apply_pending_light_updates_for(chunk_x, chunk_z);
        for entity in self.entities.values_mut() {
            if entity_chunk_key(entity.position()) == (chunk_x, chunk_z) {
                entity.ticking = true;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSpawnerState {
    delay: i32,
    min_spawn_delay: i32,
    max_spawn_delay: i32,
    spawn_count: i32,
    max_nearby_entities: i32,
    required_player_range: i32,
    spawn_range: i32,
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
    spawner_state: Option<ClientSpawnerState>,
}

impl ClientBlockEntity {
    fn from_wire(type_id: i32, tag: Option<Vec<u8>>) -> Self {
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

fn parse_spawner_state(type_id: i32, tag: Option<&[u8]>) -> Option<ClientSpawnerState> {
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

fn spawner_short_number(fields: &[crate::nbt::Field], key: &str, default: i32) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .map_or(default, |value| value as i16 as i32)
}

fn spawner_int_number(fields: &[crate::nbt::Field], key: &str, default: i32) -> i32 {
    fields
        .iter()
        .find(|field| field.key == key)
        .and_then(|field| numeric_nbt_value(&field.value))
        .unwrap_or(default)
}

fn numeric_nbt_value(value: &crate::nbt::FieldValue) -> Option<i32> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientEntity {
    pub uuid: crate::binary::Uuid,
    pub type_id: i32,
    tick_count: i32,
    x_bits: u64,
    y_bits: u64,
    z_bits: u64,
    old_x_bits: u64,
    old_y_bits: u64,
    old_z_bits: u64,
    motion_x_bits: u64,
    motion_y_bits: u64,
    motion_z_bits: u64,
    motion_lerp_target_x_bits: u64,
    motion_lerp_target_y_bits: u64,
    motion_lerp_target_z_bits: u64,
    motion_lerp_steps: u8,
    x_rot_bits: u32,
    y_rot_bits: u32,
    y_head_rot_bits: u32,
    old_x_rot_bits: u32,
    old_y_rot_bits: u32,
    old_y_head_rot_bits: u32,
    head_rotation_target_bits: u32,
    head_lerp_steps: u8,
    interpolation: Option<EntityInterpolationState>,
    on_ground: bool,
    always_ticking: bool,
    ticking: bool,
    pub data: i32,
    metadata: Option<Vec<u8>>,
    shared_flags: u8,
    living_flags: u8,
    mob_flags: u8,
    air_supply: i32,
    health_bits: Option<u32>,
    arrow_count: i32,
    stinger_count: i32,
    effect_ambience: bool,
    effect_particle_type_ids: Vec<i32>,
    sleeping_pos: Option<crate::binary::BlockPos>,
    custom_name_present: bool,
    custom_name_visible: bool,
    no_gravity: bool,
    silent: bool,
    pose: ClientEntityPose,
    ticks_frozen: i32,
    vehicle_id: Option<i32>,
    passenger_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeModifierState {
    id: String,
    amount_bits: u64,
    operation: u8,
}

impl ClientAttributeModifierState {
    pub(crate) fn from_wire(id: String, amount: f64, operation: u8) -> Self {
        Self {
            id,
            amount_bits: amount.to_bits(),
            operation,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        f64::from_bits(self.amount_bits)
    }

    pub fn operation(&self) -> u8 {
        self.operation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeState {
    attribute_id: i32,
    base_value_bits: u64,
    modifiers: Vec<ClientAttributeModifierState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMobEffectState {
    effect_id: i32,
    amplifier: i32,
    duration_ticks: i32,
    flags: u8,
}

impl ClientMobEffectState {
    pub fn effect_id(&self) -> i32 {
        self.effect_id
    }

    pub fn amplifier(&self) -> i32 {
        self.amplifier
    }

    pub fn duration_ticks(&self) -> i32 {
        self.duration_ticks
    }

    pub fn has_remaining_duration(&self) -> bool {
        self.duration_ticks == -1 || self.duration_ticks > 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    fn tick_client(&mut self) {
        if self.has_remaining_duration() && self.duration_ticks != -1 {
            self.duration_ticks -= 1;
        }
    }
}

impl ClientAttributeState {
    pub(crate) fn from_wire(
        attribute_id: i32,
        base_value: f64,
        modifiers: Vec<ClientAttributeModifierState>,
    ) -> Self {
        Self {
            attribute_id,
            base_value_bits: base_value.to_bits(),
            modifiers,
        }
    }

    pub fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    pub fn base_value(&self) -> f64 {
        f64::from_bits(self.base_value_bits)
    }

    pub fn modifiers(&self) -> &[ClientAttributeModifierState] {
        &self.modifiers
    }

    /// Calculate the effective value using Java's AttributeInstance operation order.
    pub fn effective_value(&self, min_value: f64, max_value: f64) -> f64 {
        assert!(min_value <= max_value, "attribute range must be ordered");

        let mut base = self.base_value();
        for modifier in self.modifiers.iter().filter(|modifier| {
            modifier.operation() == 0 || (modifier.operation() != 1 && modifier.operation() != 2)
        }) {
            base += modifier.amount();
        }

        let mut result = base;
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 1)
        {
            result += base * modifier.amount();
        }
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 2)
        {
            result *= 1.0 + modifier.amount();
        }

        if result.is_nan() {
            min_value
        } else {
            result.clamp(min_value, max_value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayerState {
    id: i32,
    generation: u64,
    tick_count: i32,
    x_bits: u64,
    y_bits: u64,
    z_bits: u64,
    old_x_bits: u64,
    old_y_bits: u64,
    old_z_bits: u64,
    motion_x_bits: u64,
    motion_y_bits: u64,
    motion_z_bits: u64,
    y_rot_bits: u32,
    x_rot_bits: u32,
    old_y_rot_bits: u32,
    old_x_rot_bits: u32,
    y_head_rot_bits: u32,
    old_y_head_rot_bits: u32,
    head_rotation_target_bits: u32,
    head_lerp_steps: u8,
    entity_data: Option<Vec<u8>>,
    shared_flags: u8,
    living_flags: u8,
    arrow_count: i32,
    stinger_count: i32,
    effect_ambience: bool,
    effect_particle_type_ids: Vec<i32>,
    sleeping_pos: Option<crate::binary::BlockPos>,
    air_supply: i32,
    custom_name_present: bool,
    custom_name_visible: bool,
    no_gravity: bool,
    silent: bool,
    pose: ClientEntityPose,
    ticks_frozen: i32,
    attributes: Vec<ClientAttributeState>,
    effects: Vec<ClientMobEffectState>,
    health_bits: u32,
    food: i32,
    saturation_bits: u32,
    inventory_state_id: i32,
    inventory_slots: BTreeMap<i16, Option<ClientItemStackState>>,
    carried_item: Option<ClientItemStackState>,
    open_container: Option<ClientContainerState>,
    vehicle_id: Option<i32>,
    passenger_ids: Vec<i32>,
}

impl ClientPlayerState {
    fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_x_rot_bits = self.x_rot_bits;
        self.old_y_head_rot_bits = self.y_head_rot_bits;
    }

    fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot_bits,
            self.head_rotation_target_bits,
            &mut self.head_lerp_steps,
        );
    }

    fn new(id: i32) -> Self {
        Self {
            id,
            generation: 0,
            tick_count: 0,
            x_bits: 0.0_f64.to_bits(),
            y_bits: 0.0_f64.to_bits(),
            z_bits: 0.0_f64.to_bits(),
            old_x_bits: 0.0_f64.to_bits(),
            old_y_bits: 0.0_f64.to_bits(),
            old_z_bits: 0.0_f64.to_bits(),
            motion_x_bits: 0.0_f64.to_bits(),
            motion_y_bits: 0.0_f64.to_bits(),
            motion_z_bits: 0.0_f64.to_bits(),
            y_rot_bits: 0.0_f32.to_bits(),
            x_rot_bits: 0.0_f32.to_bits(),
            old_y_rot_bits: 0.0_f32.to_bits(),
            old_x_rot_bits: 0.0_f32.to_bits(),
            y_head_rot_bits: 0.0_f32.to_bits(),
            old_y_head_rot_bits: 0.0_f32.to_bits(),
            head_rotation_target_bits: 0.0_f32.to_bits(),
            head_lerp_steps: 0,
            entity_data: None,
            shared_flags: 0,
            living_flags: 0,
            arrow_count: 0,
            stinger_count: 0,
            effect_ambience: false,
            effect_particle_type_ids: Vec::new(),
            sleeping_pos: None,
            air_supply: DEFAULT_AIR_SUPPLY,
            custom_name_present: false,
            custom_name_visible: false,
            no_gravity: false,
            silent: false,
            pose: ClientEntityPose::Standing,
            ticks_frozen: 0,
            attributes: Vec::new(),
            effects: Vec::new(),
            health_bits: 20.0_f32.to_bits(),
            food: 20,
            saturation_bits: 5.0_f32.to_bits(),
            inventory_state_id: 0,
            inventory_slots: BTreeMap::new(),
            carried_item: None,
            open_container: None,
            vehicle_id: None,
            passenger_ids: Vec::new(),
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.old_x_bits),
            f64::from_bits(self.old_y_bits),
            f64::from_bits(self.old_z_bits),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.motion_x_bits),
            f64::from_bits(self.motion_y_bits),
            f64::from_bits(self.motion_z_bits),
        )
    }

    pub fn rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
        )
    }

    pub fn previous_rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.old_y_rot_bits),
            f32::from_bits(self.old_x_rot_bits),
        )
    }

    pub fn head_rotation(&self) -> f32 {
        f32::from_bits(self.y_head_rot_bits)
    }

    pub fn previous_head_rotation(&self) -> f32 {
        f32::from_bits(self.old_y_head_rot_bits)
    }

    pub fn interpolated_head_rotation(&self, partial_tick: f32) -> f32 {
        if partial_tick == 1.0 {
            return self.head_rotation();
        }
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_head_rotation(),
            self.head_rotation(),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
        )
    }

    pub fn entity_data(&self) -> Option<&[u8]> {
        self.entity_data.as_deref()
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    pub fn used_item_is_off_hand(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_OFF_HAND)
    }

    pub fn is_auto_spin_attack(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_SPIN_ATTACK)
    }

    pub fn air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    pub fn is_custom_name_visible(&self) -> bool {
        self.custom_name_visible
    }

    pub fn should_show_name(&self) -> bool {
        true
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn percent_frozen(&self) -> f32 {
        (self.ticks_frozen.min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.ticks_frozen > 0
    }

    pub fn is_on_fire(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_ON_FIRE)
    }

    pub fn is_shift_key_down(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SHIFT_KEY_DOWN)
    }

    pub fn is_sprinting(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SPRINTING)
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    pub fn is_invisible(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_INVISIBLE)
    }

    pub fn is_glowing(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_GLOWING)
    }

    pub fn is_fall_flying(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_FALL_FLYING)
    }

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }

    pub fn health(&self) -> f32 {
        f32::from_bits(self.health_bits)
    }

    pub fn arrow_count(&self) -> i32 {
        self.arrow_count
    }

    pub fn stinger_count(&self) -> i32 {
        self.stinger_count
    }

    pub fn is_effect_ambient(&self) -> bool {
        self.effect_ambience
    }

    pub fn effect_particle_type_ids(&self) -> &[i32] {
        &self.effect_particle_type_ids
    }

    pub fn sleeping_pos(&self) -> Option<crate::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn food(&self) -> i32 {
        self.food
    }

    pub fn saturation(&self) -> f32 {
        f32::from_bits(self.saturation_bits)
    }

    pub fn inventory_state_id(&self) -> i32 {
        self.inventory_state_id
    }

    pub fn inventory_slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.inventory_slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn inventory_slot_count(&self) -> usize {
        self.inventory_slots.len()
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }

    pub fn open_container(&self) -> Option<&ClientContainerState> {
        self.open_container.as_ref()
    }
}

impl ClientEntity {
    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_x_rot_bits = self.x_rot_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_y_head_rot_bits = self.y_head_rot_bits;
    }

    fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot_bits,
            self.head_rotation_target_bits,
            &mut self.head_lerp_steps,
        );
    }

    fn interpolation_position(&self) -> (f64, f64, f64) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.position(),
                EntityInterpolationState::target_position,
            )
    }

    fn interpolation_rotation(&self) -> (f32, f32) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.body_rotation(),
                EntityInterpolationState::target_rotation,
            )
    }

    fn interpolate_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        let target_matches = self.interpolation.is_some_and(|interpolation| {
            interpolation.steps > 0
                && interpolation.target_x_bits == position.0.to_bits()
                && interpolation.target_y_bits == position.1.to_bits()
                && interpolation.target_z_bits == position.2.to_bits()
                && interpolation.target_y_rot_bits == y_rot.to_bits()
                && interpolation.target_x_rot_bits == x_rot.to_bits()
        });
        if target_matches {
            return;
        }

        let current_position = self.position();
        let current_rotation = self.body_rotation();
        self.interpolation = Some(EntityInterpolationState {
            target_x_bits: position.0.to_bits(),
            target_y_bits: position.1.to_bits(),
            target_z_bits: position.2.to_bits(),
            target_y_rot_bits: y_rot.to_bits(),
            target_x_rot_bits: x_rot.to_bits(),
            previous_x_bits: current_position.0.to_bits(),
            previous_y_bits: current_position.1.to_bits(),
            previous_z_bits: current_position.2.to_bits(),
            previous_y_rot_bits: current_rotation.0.to_bits(),
            previous_x_rot_bits: current_rotation.1.to_bits(),
            steps: LIVING_ENTITY_POSITION_LERP_STEPS,
        });
    }

    fn snap_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        self.x_bits = position.0.to_bits();
        self.y_bits = position.1.to_bits();
        self.z_bits = position.2.to_bits();
        self.y_rot_bits = y_rot.to_bits();
        self.x_rot_bits = x_rot.to_bits();
        self.old_x_bits = position.0.to_bits();
        self.old_y_bits = position.1.to_bits();
        self.old_z_bits = position.2.to_bits();
        self.old_y_rot_bits = y_rot.to_bits();
        self.old_x_rot_bits = x_rot.to_bits();
        self.interpolation = None;
    }

    fn tick_position_interpolation(&mut self) {
        let current_position = self.position();
        let current_rotation = self.body_rotation();
        let Some(interpolation) = self
            .interpolation
            .as_mut()
            .filter(|interpolation| interpolation.steps > 0)
        else {
            return;
        };

        let alpha = 1.0 / f64::from(interpolation.steps);
        let previous_position = (
            f64::from_bits(interpolation.previous_x_bits),
            f64::from_bits(interpolation.previous_y_bits),
            f64::from_bits(interpolation.previous_z_bits),
        );
        let delta_position = (
            current_position.0 - previous_position.0,
            current_position.1 - previous_position.1,
            current_position.2 - previous_position.2,
        );
        interpolation.target_x_bits =
            (f64::from_bits(interpolation.target_x_bits) + delta_position.0).to_bits();
        interpolation.target_y_bits =
            (f64::from_bits(interpolation.target_y_bits) + delta_position.1).to_bits();
        interpolation.target_z_bits =
            (f64::from_bits(interpolation.target_z_bits) + delta_position.2).to_bits();

        let previous_rotation = (
            f32::from_bits(interpolation.previous_y_rot_bits),
            f32::from_bits(interpolation.previous_x_rot_bits),
        );
        let delta_rotation = (
            current_rotation.0 - previous_rotation.0,
            current_rotation.1 - previous_rotation.1,
        );
        interpolation.target_y_rot_bits =
            (f32::from_bits(interpolation.target_y_rot_bits) + delta_rotation.0).to_bits();
        interpolation.target_x_rot_bits =
            (f32::from_bits(interpolation.target_x_rot_bits) + delta_rotation.1).to_bits();

        let target_position = interpolation.target_position();
        let target_rotation = interpolation.target_rotation();
        let new_position = (
            mth_lerp_f64(alpha, current_position.0, target_position.0),
            mth_lerp_f64(alpha, current_position.1, target_position.1),
            mth_lerp_f64(alpha, current_position.2, target_position.2),
        );
        let new_rotation = (
            mth_rot_lerp_f64(
                alpha,
                f64::from(current_rotation.0),
                f64::from(target_rotation.0),
            ) as f32,
            mth_lerp_f64(
                alpha,
                f64::from(current_rotation.1),
                f64::from(target_rotation.1),
            ) as f32,
        );
        self.x_bits = new_position.0.to_bits();
        self.y_bits = new_position.1.to_bits();
        self.z_bits = new_position.2.to_bits();
        self.y_rot_bits = (new_rotation.0 % 360.0).to_bits();
        self.x_rot_bits = (new_rotation.1 % 360.0).clamp(-90.0, 90.0).to_bits();
        interpolation.steps -= 1;
        interpolation.previous_x_bits = new_position.0.to_bits();
        interpolation.previous_y_bits = new_position.1.to_bits();
        interpolation.previous_z_bits = new_position.2.to_bits();
        interpolation.previous_y_rot_bits = new_rotation.0.to_bits();
        interpolation.previous_x_rot_bits = new_rotation.1.to_bits();
    }

    fn tick_motion_interpolation(&mut self) {
        if self.motion_lerp_steps == 0 {
            return;
        }

        let steps = f64::from(self.motion_lerp_steps);
        let current = self.motion();
        let target = (
            f64::from_bits(self.motion_lerp_target_x_bits),
            f64::from_bits(self.motion_lerp_target_y_bits),
            f64::from_bits(self.motion_lerp_target_z_bits),
        );
        self.motion_x_bits = (current.0 + (target.0 - current.0) / steps).to_bits();
        self.motion_y_bits = (current.1 + (target.1 - current.1) / steps).to_bits();
        self.motion_z_bits = (current.2 + (target.2 - current.2) / steps).to_bits();
        self.motion_lerp_steps -= 1;
    }

    fn tick_snowball(&mut self) {
        let air_drag = f64::from(0.99_f32);
        let gravity = if self.no_gravity {
            0.0
        } else {
            SNOWBALL_GRAVITY
        };
        let current_motion = self.motion();
        let motion = (
            current_motion.0 * air_drag,
            (current_motion.1 - gravity) * air_drag,
            current_motion.2 * air_drag,
        );
        self.motion_x_bits = motion.0.to_bits();
        self.motion_y_bits = motion.1.to_bits();
        self.motion_z_bits = motion.2.to_bits();

        let current_position = self.position();
        self.x_bits = (current_position.0 + motion.0).to_bits();
        self.y_bits = (current_position.1 + motion.1).to_bits();
        self.z_bits = (current_position.2 + motion.2).to_bits();

        const PI_F32: f32 = f32::from_bits(0x4049_0fdb);
        let horizontal_distance = (motion.0 * motion.0 + motion.2 * motion.2).sqrt();
        let target_x_rot = (mth_atan2(motion.1, horizontal_distance) * f64::from(180.0_f32)
            / f64::from(PI_F32)) as f32;
        let target_y_rot =
            (mth_atan2(motion.0, motion.2) * f64::from(180.0_f32) / f64::from(PI_F32)) as f32;
        self.x_rot_bits =
            mth_rot_lerp_f32(0.2_f32, f32::from_bits(self.old_x_rot_bits), target_x_rot).to_bits();
        self.y_rot_bits =
            mth_rot_lerp_f32(0.2_f32, f32::from_bits(self.old_y_rot_bits), target_y_rot).to_bits();
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.old_x_bits),
            f64::from_bits(self.old_y_bits),
            f64::from_bits(self.old_z_bits),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.motion_x_bits),
            f64::from_bits(self.motion_y_bits),
            f64::from_bits(self.motion_z_bits),
        )
    }

    fn set_motion_if_finite(&mut self, motion: (f64, f64, f64)) {
        if motion.0.is_finite() && motion.1.is_finite() && motion.2.is_finite() {
            self.motion_x_bits = motion.0.to_bits();
            self.motion_y_bits = motion.1.to_bits();
            self.motion_z_bits = motion.2.to_bits();
        }
    }

    pub fn motion_interpolation_target(&self) -> Option<(f64, f64, f64)> {
        (self.motion_lerp_steps > 0).then(|| {
            (
                f64::from_bits(self.motion_lerp_target_x_bits),
                f64::from_bits(self.motion_lerp_target_y_bits),
                f64::from_bits(self.motion_lerp_target_z_bits),
            )
        })
    }

    pub fn motion_interpolation_steps(&self) -> u8 {
        self.motion_lerp_steps
    }

    pub fn rotation(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
            f32::from_bits(self.y_head_rot_bits),
        )
    }

    fn body_rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
        )
    }

    pub fn interpolation_target_position(&self) -> Option<(f64, f64, f64)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_position)
    }

    pub fn interpolation_target_rotation(&self) -> Option<(f32, f32)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_rotation)
    }

    pub fn interpolation_steps(&self) -> u8 {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or(0, |interpolation| interpolation.steps)
    }

    pub fn previous_rotation(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.old_y_rot_bits),
            f32::from_bits(self.old_x_rot_bits),
            f32::from_bits(self.old_y_head_rot_bits),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
            mth_rot_lerp_f32(partial_tick, previous.2, current.2),
        )
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn is_ticking(&self) -> bool {
        self.ticking
    }

    pub fn is_always_ticking(&self) -> bool {
        self.always_ticking
    }

    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn mob_flags(&self) -> u8 {
        self.mob_flags
    }

    pub fn is_no_ai(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_NO_AI)
    }

    pub fn is_left_handed(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_LEFT_HANDED)
    }

    pub fn is_aggressive(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_AGGRESSIVE)
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    pub fn used_item_is_off_hand(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_OFF_HAND)
    }

    pub fn is_auto_spin_attack(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_SPIN_ATTACK)
    }

    pub fn air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn health(&self) -> Option<f32> {
        self.health_bits.map(f32::from_bits)
    }

    pub fn arrow_count(&self) -> i32 {
        self.arrow_count
    }

    pub fn stinger_count(&self) -> i32 {
        self.stinger_count
    }

    pub fn is_effect_ambient(&self) -> bool {
        self.effect_ambience
    }

    pub fn effect_particle_type_ids(&self) -> &[i32] {
        &self.effect_particle_type_ids
    }

    pub fn sleeping_pos(&self) -> Option<crate::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    pub fn is_custom_name_visible(&self) -> bool {
        self.custom_name_visible
    }

    pub fn should_show_name(&self) -> bool {
        self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID || self.custom_name_visible
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn percent_frozen(&self) -> f32 {
        (self.ticks_frozen.min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.ticks_frozen > 0
    }

    pub fn is_on_fire(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_ON_FIRE)
    }

    pub fn is_shift_key_down(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SHIFT_KEY_DOWN)
    }

    pub fn is_sprinting(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SPRINTING)
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    pub fn is_invisible(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_INVISIBLE)
    }

    pub fn is_glowing(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_GLOWING)
    }

    pub fn is_fall_flying(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_FALL_FLYING)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClientChunk {
    sections: BTreeMap<i32, ClientSection>,
    lights: BTreeMap<i32, ClientLightSection>,
    block_entities: BTreeMap<(i32, i32, i32), ClientBlockEntity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ClientLightSection {
    sky: Option<Vec<u8>>,
    block: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingLightUpdate {
    chunk_x: i32,
    chunk_z: i32,
    sky: OwnedLightLayerInput,
    block: OwnedLightLayerInput,
}

impl PendingLightUpdate {
    fn new(chunk_x: i32, chunk_z: i32, input: LightUpdateInput<'_>) -> Self {
        Self {
            chunk_x,
            chunk_z,
            sky: OwnedLightLayerInput::new(input.sky),
            block: OwnedLightLayerInput::new(input.block),
        }
    }

    fn as_input(&self) -> LightUpdateInput<'_> {
        LightUpdateInput {
            sky: self.sky.as_input(),
            block: self.block.as_input(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedLightLayerInput {
    mask: BitSet,
    empty_mask: BitSet,
    updates: Vec<Vec<u8>>,
}

impl OwnedLightLayerInput {
    fn new(input: LightLayerInput<'_>) -> Self {
        Self {
            mask: input.mask.clone(),
            empty_mask: input.empty_mask.clone(),
            updates: input.updates.to_vec(),
        }
    }

    fn as_input(&self) -> LightLayerInput<'_> {
        LightLayerInput {
            mask: &self.mask,
            empty_mask: &self.empty_mask,
            updates: &self.updates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClientSection {
    non_empty_block_count: i16,
    fluid_count: i16,
    block_states: Vec<i32>,
    biomes: Vec<i32>,
}

pub(crate) struct SectionInput<'a> {
    pub(crate) non_empty_block_count: i16,
    pub(crate) fluid_count: i16,
    pub(crate) block_states: &'a [i32],
    pub(crate) biomes: &'a [i32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DimensionInput<'a> {
    pub(crate) key: &'a str,
    pub(crate) dimension_type_id: i32,
    pub(crate) seed: i64,
    pub(crate) is_debug: bool,
    pub(crate) is_flat: bool,
    pub(crate) sea_level: i32,
}

pub(crate) struct LightUpdateInput<'a> {
    pub(crate) sky: LightLayerInput<'a>,
    pub(crate) block: LightLayerInput<'a>,
}

pub(crate) struct LightLayerInput<'a> {
    pub(crate) mask: &'a BitSet,
    pub(crate) empty_mask: &'a BitSet,
    pub(crate) updates: &'a [Vec<u8>],
}

pub(crate) struct BlockEntityInput<'a> {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z: i32,
    pub(crate) type_id: i32,
    pub(crate) tag: Option<&'a [u8]>,
}

pub(crate) struct EntityInput {
    pub(crate) entity_id: i32,
    pub(crate) uuid: crate::binary::Uuid,
    pub(crate) type_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) movement: (f64, f64, f64),
    pub(crate) rotation: (f32, f32, f32),
    pub(crate) data: i32,
}

pub(crate) struct EntityMotionInput {
    pub(crate) entity_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

pub(crate) struct EntityRotationInput {
    pub(crate) entity_id: i32,
    pub(crate) y_rot: Option<f32>,
    pub(crate) x_rot: Option<f32>,
    pub(crate) y_head_rot: Option<f32>,
    pub(crate) relative_y_rot: bool,
    pub(crate) relative_x_rot: bool,
}

pub(crate) struct EntityPositionInput {
    pub(crate) entity_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) relative_flags: i32,
}

pub(crate) struct EntityTeleportInput {
    pub(crate) entity_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) delta_x: f64,
    pub(crate) delta_y: f64,
    pub(crate) delta_z: f64,
    pub(crate) y_rot: f32,
    pub(crate) x_rot: f32,
    pub(crate) relative_flags: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntityPositionUpdateMode {
    PositionSync,
    Teleport,
}

pub(crate) struct EntityMoveInput {
    pub(crate) entity_id: i32,
    pub(crate) delta_x: i16,
    pub(crate) delta_y: i16,
    pub(crate) delta_z: i16,
    pub(crate) has_position: bool,
    pub(crate) y_rot: Option<f32>,
    pub(crate) x_rot: Option<f32>,
}

fn entity_chunk_key(position: (f64, f64, f64)) -> (i32, i32) {
    (
        (position.0.floor() as i32).div_euclid(16),
        (position.2.floor() as i32).div_euclid(16),
    )
}

fn player_inventory_menu_slot(slot: i32) -> Option<i16> {
    match slot {
        0..=8 => Some((36 + slot) as i16),
        9..=35 => Some(slot as i16),
        _ => None,
    }
}

const VANILLA_MENU_TOP_SLOT_COUNTS: [usize; 25] = [
    9, 18, 27, 36, 45, 54, 9, 10, 3, 1, 3, 5, 10, 2, 3, 3, 5, 1, 4, 3, 27, 4, 3, 3, 2,
];

fn menu_top_slot_count(menu_type_id: i32) -> Option<usize> {
    usize::try_from(menu_type_id)
        .ok()
        .and_then(|id| VANILLA_MENU_TOP_SLOT_COUNTS.get(id).copied())
}

fn decode_entity_delta(base: f64, delta: i16) -> u64 {
    if delta == 0 {
        base.to_bits()
    } else {
        let encoded_base = (base * 4096.0 + 0.5).floor() as i64;
        ((encoded_base + i64::from(delta)) as f64 / 4096.0).to_bits()
    }
}

fn resolve_entity_teleport_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
    delta_movement: (f64, f64, f64),
    relative_flags: i32,
) -> (f64, f64, f64) {
    let current_motion = if relative_flags & ENTITY_RELATIVE_ROTATE_DELTA_FLAG != 0 {
        rotate_entity_motion(current_motion, current_rotation, target_rotation)
    } else {
        current_motion
    };
    (
        if relative_flags & ENTITY_RELATIVE_DELTA_X_FLAG != 0 {
            current_motion.0 + delta_movement.0
        } else {
            delta_movement.0
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Y_FLAG != 0 {
            current_motion.1 + delta_movement.1
        } else {
            delta_movement.1
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Z_FLAG != 0 {
            current_motion.2 + delta_movement.2
        } else {
            delta_movement.2
        },
    )
}

fn rotate_entity_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
) -> (f64, f64, f64) {
    let diff_y_rot = current_rotation.0 - target_rotation.0;
    let diff_x_rot = current_rotation.1 - target_rotation.1;
    let x_radians = (f64::from(diff_x_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let y_radians = (f64::from(diff_y_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let x_cos = f64::from(mth_cos_f64(f64::from(x_radians)));
    let x_sin = f64::from(mth_sin_f64(f64::from(x_radians)));
    let x_rotated = (
        current_motion.0,
        current_motion.1 * x_cos + current_motion.2 * x_sin,
        current_motion.2 * x_cos - current_motion.1 * x_sin,
    );
    let y_cos = f64::from(mth_cos_f64(f64::from(y_radians)));
    let y_sin = f64::from(mth_sin_f64(f64::from(y_radians)));
    (
        x_rotated.0 * y_cos + x_rotated.2 * y_sin,
        x_rotated.1,
        x_rotated.2 * y_cos - x_rotated.0 * y_sin,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LightLayer {
    Sky,
    Block,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct LightApplication {
    pub(crate) applied_sky_sections: usize,
    pub(crate) applied_block_sections: usize,
    pub(crate) invalid_sky_updates: usize,
    pub(crate) invalid_block_updates: usize,
    pub(crate) queued: bool,
}

fn count_invalid_light_updates(section_count: usize, mask: &BitSet, updates: &[Vec<u8>]) -> usize {
    let mut update_index = 0;
    let mut invalid_updates = 0;
    for section_index in 0..section_count {
        if mask.contains(section_index as u64) {
            if updates
                .get(update_index)
                .is_none_or(|update| update.len() != 2_048)
            {
                invalid_updates += 1;
            }
            update_index += 1;
        }
    }
    invalid_updates
}

fn apply_light_layer(
    lights: &mut BTreeMap<i32, ClientLightSection>,
    min_section_y: i32,
    section_count: usize,
    layer: LightLayer,
    mask: &BitSet,
    empty_mask: &BitSet,
    updates: &[Vec<u8>],
) -> (usize, usize) {
    let mut update_index = 0;
    let mut applied_sections = 0;
    let mut invalid_updates = 0;
    for section_index in 0..section_count {
        let bit = section_index as u64;
        if mask.contains(bit) {
            let update = updates.get(update_index);
            update_index += 1;
            if let Some(update) = update.filter(|update| update.len() == 2_048) {
                let section = lights
                    .entry(min_section_y + section_index as i32)
                    .or_default();
                match layer {
                    LightLayer::Sky => section.sky = Some(update.clone()),
                    LightLayer::Block => section.block = Some(update.clone()),
                }
                applied_sections += 1;
            } else {
                invalid_updates += 1;
            }
        } else if empty_mask.contains(bit) {
            let section = lights
                .entry(min_section_y + section_index as i32)
                .or_default();
            let empty = vec![0; 2_048];
            match layer {
                LightLayer::Sky => section.sky = Some(empty),
                LightLayer::Block => section.block = Some(empty),
            }
            applied_sections += 1;
        }
    }
    (applied_sections, invalid_updates)
}

#[cfg(test)]
mod tests {
    use super::{
        BlockEntityInput, ClientAttributeModifierState, ClientAttributeState, ClientItemStackState,
        ClientWorld, DimensionBounds, DimensionEnvironment, DimensionInput,
        ENTITY_DATA_SERIALIZER_BOOLEAN_ID, ENTITY_NO_GRAVITY_DATA_ID, ENTITY_RELATIVE_DELTA_X_FLAG,
        ENTITY_RELATIVE_DELTA_Y_FLAG, ENTITY_RELATIVE_DELTA_Z_FLAG,
        ENTITY_RELATIVE_ROTATE_DELTA_FLAG, ENTITY_RELATIVE_X_FLAG, ENTITY_RELATIVE_X_ROT_FLAG,
        ENTITY_RELATIVE_Y_FLAG, ENTITY_RELATIVE_Y_ROT_FLAG, ENTITY_RELATIVE_Z_FLAG, EntityInput,
        EntityMotionInput, EntityMoveInput, EntityPositionInput, EntityPositionUpdateMode,
        EntityRotationInput, EntityTeleportInput, LightLayerInput, LightUpdateInput,
        PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA, SNOWBALL_GRAVITY,
        SectionInput, VANILLA_PLAYER_ENTITY_TYPE_ID, VANILLA_SNOWBALL_ENTITY_TYPE_ID,
        menu_top_slot_count,
    };
    use crate::binary::{BitSet, Uuid};
    use crate::math::{mth_cos_f64, mth_sin_f64};

    fn section(block_value: i32, biome_value: i32) -> (Vec<i32>, Vec<i32>) {
        let mut blocks = vec![block_value; 4_096];
        blocks[0] = 11;
        blocks[15] = 12;
        blocks[16] = 13;
        blocks[255] = 14;
        blocks[256] = 15;
        blocks[4_095] = 16;

        let mut biomes = vec![biome_value; 64];
        biomes[0] = 21;
        biomes[3] = 22;
        biomes[4] = 23;
        biomes[63] = 24;
        (blocks, biomes)
    }

    #[test]
    fn chunk_sections_are_queryable_with_java_local_order() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
        world.replace_chunk(
            -1,
            2,
            [SectionInput {
                non_empty_block_count: 6,
                fluid_count: 1,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_section_count(), 1);
        assert_eq!(world.section_counts_at(-1, -4, 2), Some((6, 1)));
        assert_eq!(world.block_state_at(-16, -64, 32), Some(11));
        assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
        assert_eq!(world.block_state_at(-16, -64, 33), Some(13));
        assert_eq!(world.block_state_at(-1, -64, 47), Some(14));
        assert_eq!(world.block_state_at(-16, -63, 32), Some(15));
        assert_eq!(world.block_state_at(-1, -49, 47), Some(16));
        assert_eq!(world.block_state_at(-17, -64, 32), None);
        assert_eq!(world.biome_at(-16, -64, 32), Some(21));
        assert_eq!(world.biome_at(-4, -64, 32), Some(22));
        assert_eq!(world.biome_at(-16, -64, 36), Some(23));
        assert_eq!(world.biome_at(-1, -49, 47), Some(24));
    }

    #[test]
    fn server_block_update_replaces_loaded_state() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            -1,
            2,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
        assert!(world.update_block_state(-1, -64, 32, 86));
        assert_eq!(world.block_state_at(-1, -64, 32), Some(86));
        assert!(!world.update_block_state(-17, -64, 32, 86));
        assert!(!world.update_block_state(-1, -48, 32, 86));
    }

    #[test]
    fn replacing_and_unloading_a_chunk_updates_the_section_cache() {
        let (first_blocks, first_biomes) = section(7, 8);
        let (mut second_blocks, mut second_biomes) = section(17, 18);
        second_blocks[0] = 31;
        second_biomes[0] = 32;
        let mut world = ClientWorld::default();
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &first_blocks,
                biomes: &first_biomes,
            }],
        );
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 2,
                fluid_count: 0,
                block_states: &second_blocks,
                biomes: &second_biomes,
            }],
        );

        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_section_count(), 1);
        assert_eq!(world.block_state_at(0, -64, 0), Some(31));
        assert_eq!(world.biome_at(0, -64, 0), Some(32));
        assert!(world.unload_chunk(0, 0));
        assert!(!world.contains_chunk(0, 0));
        assert_eq!(world.loaded_section_count(), 0);
        assert_eq!(world.block_state_at(0, -64, 0), None);
        assert!(!world.unload_chunk(0, 0));
    }

    #[test]
    fn light_updates_store_java_nibble_order_and_empty_sections() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        let mut sky = vec![0; 2_048];
        sky[0] = 0xc3;
        sky[8] = 0x05;
        sky[128] = 0x09;
        let mut block = vec![0; 2_048];
        block[0] = 0xa6;
        let application = world.apply_light_updates(
            0,
            0,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &BitSet::from_bits(&[1]),
                    empty_mask: &BitSet::from_bits(&[2]),
                    updates: &[sky],
                },
                block: LightLayerInput {
                    mask: &BitSet::from_bits(&[1]),
                    empty_mask: &BitSet::from_bits(&[2]),
                    updates: &[block],
                },
            },
        );

        assert_eq!(application.applied_sky_sections, 2);
        assert_eq!(application.applied_block_sections, 2);
        assert_eq!(application.invalid_sky_updates, 0);
        assert_eq!(application.invalid_block_updates, 0);
        assert_eq!(world.loaded_light_section_count(), 2);
        assert_eq!(world.sky_light_at(0, -64, 0), Some(3));
        assert_eq!(world.sky_light_at(1, -64, 0), Some(12));
        assert_eq!(world.sky_light_at(0, -64, 1), Some(5));
        assert_eq!(world.sky_light_at(0, -63, 0), Some(9));
        assert_eq!(world.block_light_at(0, -64, 0), Some(6));
        assert_eq!(world.block_light_at(1, -64, 0), Some(10));
        assert_eq!(world.sky_light_at(0, -48, 0), Some(0));
        assert_eq!(world.block_light_at(0, -48, 0), Some(0));

        assert!(world.unload_chunk(0, 0));
        assert_eq!(world.sky_light_at(0, -64, 0), None);
        assert_eq!(world.loaded_light_section_count(), 0);
    }

    #[test]
    fn light_updates_for_unloaded_chunks_wait_in_fifo_until_chunk_load() {
        let mut world = ClientWorld::default();
        let mut first_sky = vec![0; 2_048];
        first_sky[0] = 0x05;
        let mut second_sky = vec![0; 2_048];
        second_sky[0] = 0x0a;

        let first_application = world.apply_light_updates(
            3,
            -2,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &BitSet::from_bits(&[1]),
                    empty_mask: &BitSet::new(),
                    updates: &[first_sky],
                },
                block: LightLayerInput {
                    mask: &BitSet::new(),
                    empty_mask: &BitSet::new(),
                    updates: &[],
                },
            },
        );
        let second_application = world.apply_light_updates(
            3,
            -2,
            LightUpdateInput {
                sky: LightLayerInput {
                    mask: &BitSet::from_bits(&[1]),
                    empty_mask: &BitSet::new(),
                    updates: &[second_sky],
                },
                block: LightLayerInput {
                    mask: &BitSet::new(),
                    empty_mask: &BitSet::new(),
                    updates: &[],
                },
            },
        );

        assert!(first_application.queued);
        assert!(second_application.queued);
        assert_eq!(world.pending_light_update_count(), 2);
        assert_eq!(world.sky_light_at(48, -64, -32), None);

        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            3,
            -2,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );

        assert_eq!(world.pending_light_update_count(), 0);
        assert_eq!(world.loaded_light_section_count(), 1);
        assert_eq!(world.sky_light_at(48, -64, -32), Some(10));
    }

    #[test]
    fn unloading_a_chunk_discards_pending_light_updates_for_that_chunk() {
        let mut world = ClientWorld::default();
        let mut sky = vec![0; 2_048];
        sky[0] = 0x0f;
        assert!(
            world
                .apply_light_updates(
                    3,
                    -2,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &BitSet::from_bits(&[1]),
                            empty_mask: &BitSet::new(),
                            updates: &[sky],
                        },
                        block: LightLayerInput {
                            mask: &BitSet::new(),
                            empty_mask: &BitSet::new(),
                            updates: &[],
                        },
                    },
                )
                .queued
        );
        assert_eq!(world.pending_light_update_count(), 1);
        assert!(!world.unload_chunk(3, -2));
        assert_eq!(world.pending_light_update_count(), 0);

        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            3,
            -2,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert_eq!(world.sky_light_at(48, -64, -32), None);
    }

    #[test]
    fn block_entities_are_queryable_and_removed_with_their_chunk() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            -1,
            2,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        let tag = [10, 0];

        assert_eq!(
            world.apply_block_entities(
                -1,
                2,
                [BlockEntityInput {
                    x: -1,
                    y: -63,
                    z: 32,
                    type_id: 7,
                    tag: Some(&tag),
                }],
            ),
            1
        );
        assert_eq!(world.loaded_block_entity_count(), 1);
        assert_eq!(
            world.block_entity_at(-1, -63, 32),
            Some(&super::ClientBlockEntity {
                type_id: 7,
                tag: Some(tag.to_vec()),
                spawner_state: None,
            })
        );

        let replacement = [10, 0, 0];
        assert!(world.apply_block_entity_update(BlockEntityInput {
            x: -1,
            y: -63,
            z: 32,
            type_id: 8,
            tag: Some(&replacement),
        }));
        assert_eq!(world.block_entity_at(-1, -63, 32).unwrap().type_id, 8);
        assert_eq!(
            world.block_entity_at(-1, -63, 32).unwrap().tag,
            Some(replacement.to_vec())
        );
        assert!(world.unload_chunk(-1, 2));
        assert_eq!(world.loaded_block_entity_count(), 0);
        assert_eq!(world.block_entity_at(-1, -63, 32), None);
    }

    #[test]
    fn spawner_state_decodes_java_update_tag_and_defaults() {
        use crate::nbt::{Field, FieldValue};

        let mut tag = Vec::new();
        crate::nbt::write_nbt(
            Some(&[
                Field {
                    key: "Delay".to_owned(),
                    value: FieldValue::Int(65_535),
                },
                Field {
                    key: "MaxSpawnDelay".to_owned(),
                    value: FieldValue::Short(640),
                },
                Field {
                    key: "MinSpawnDelay".to_owned(),
                    value: FieldValue::Short(120),
                },
                Field {
                    key: "SpawnCount".to_owned(),
                    value: FieldValue::Str("invalid numeric tag".to_owned()),
                },
            ]),
            &mut tag,
        );

        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert_eq!(
            world.apply_block_entities(
                0,
                0,
                [BlockEntityInput {
                    x: 1,
                    y: -63,
                    z: 2,
                    type_id: 9,
                    tag: Some(&tag),
                }],
            ),
            1
        );

        let state = world
            .block_entity_at(1, -63, 2)
            .and_then(super::ClientBlockEntity::spawner_state)
            .expect("spawner tag should be retained as semantic state");
        assert_eq!(state.delay(), -1);
        assert_eq!(state.min_spawn_delay(), 120);
        assert_eq!(state.max_spawn_delay(), 640);
        assert_eq!(state.spawn_count(), 4);
        assert_eq!(state.max_nearby_entities(), 6);
        assert_eq!(state.required_player_range(), 16);
        assert_eq!(state.spawn_range(), 4);

        let non_spawner = world.block_entity_at(1, -63, 2).unwrap();
        assert_eq!(non_spawner.type_id, 9);
    }

    #[test]
    fn entities_are_queryable_and_replaced_by_entity_id() {
        let mut world = ClientWorld::default();
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.125, -0.25, 0.5),
            rotation: (12.5, -30.0, 45.0),
            data: 6,
        }));
        assert_eq!(world.loaded_entity_count(), 1);
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((1.25, 80.0, -35.5))
        );
        assert_eq!(world.entity_at(37).map(|entity| entity.type_id), Some(120));
        assert_eq!(world.entity_at(37).map(|entity| entity.data), Some(6));
        assert_eq!(
            world.entity_at(37).map(|entity| entity.uuid),
            Some(Uuid::new(1, 2))
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::motion),
            Some((0.125, -0.25, 0.5))
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::rotation),
            Some((12.5, -30.0, 45.0))
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::on_ground),
            Some(false)
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::is_ticking),
            Some(true)
        );
        assert!(world.unload_chunk(0, -3));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::is_ticking),
            Some(false)
        );
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::is_ticking),
            Some(true)
        );
        assert_eq!(world.entity_metadata_at(37), None);
        assert!(world.apply_entity_metadata(37, &[1, 0, 255]));
        assert_eq!(world.entity_metadata_at(37), Some(&[1, 0, 255][..]));
        assert!(!world.apply_entity_metadata(999, &[255]));
        assert!(world.apply_entity_motion(EntityMotionInput {
            entity_id: 37,
            x: -0.5,
            y: 0.25,
            z: 0.75,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::motion),
            Some((-0.5, 0.25, 0.75))
        );
        assert!(!world.apply_entity_motion(EntityMotionInput {
            entity_id: 999,
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }));
        assert!(world.apply_entity_ground_state(37, true));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::on_ground),
            Some(true)
        );
        assert!(world.apply_entity_ground_state(37, false));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::on_ground),
            Some(false)
        );
        assert!(!world.apply_entity_ground_state(999, true));
        assert!(world.apply_entity_rotation(EntityRotationInput {
            entity_id: 37,
            y_rot: Some(90.0),
            x_rot: Some(-15.0),
            y_head_rot: Some(135.0),
            relative_y_rot: false,
            relative_x_rot: false,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::rotation),
            Some((90.0, -15.0, 135.0))
        );
        assert!(world.apply_entity_rotation(EntityRotationInput {
            entity_id: 37,
            y_rot: Some(45.0),
            x_rot: Some(10.0),
            y_head_rot: None,
            relative_y_rot: true,
            relative_x_rot: true,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::rotation),
            Some((135.0, -5.0, 135.0))
        );
        assert!(!world.apply_entity_rotation(EntityRotationInput {
            entity_id: 999,
            y_rot: Some(1.0),
            x_rot: Some(2.0),
            y_head_rot: Some(3.0),
            relative_y_rot: false,
            relative_x_rot: false,
        }));

        assert!(world.apply_entity_position(EntityPositionInput {
            entity_id: 37,
            x: 10.0,
            y: 20.0,
            z: 30.0,
            relative_flags: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((10.0, 20.0, 30.0))
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::is_ticking),
            Some(false)
        );
        assert!(world.apply_entity_position(EntityPositionInput {
            entity_id: 37,
            x: 0.5,
            y: -1.0,
            z: 2.0,
            relative_flags: 0b111,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((10.5, 19.0, 32.0))
        );
        assert!(!world.apply_entity_position(EntityPositionInput {
            entity_id: 999,
            x: 1.0,
            y: 2.0,
            z: 3.0,
            relative_flags: 0,
        }));
        assert!(world.apply_entity_move(EntityMoveInput {
            entity_id: 37,
            delta_x: 2_048,
            delta_y: -4_096,
            delta_z: 0,
            has_position: true,
            y_rot: None,
            x_rot: None,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((11.0, 18.0, 32.0))
        );
        assert!(world.apply_entity_move(EntityMoveInput {
            entity_id: 37,
            delta_x: 7,
            delta_y: 8,
            delta_z: 9,
            has_position: false,
            y_rot: None,
            x_rot: None,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((11.0, 18.0, 32.0))
        );
        world.replace_chunk(
            1,
            2,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity_move(EntityMoveInput {
            entity_id: 37,
            delta_x: 32_767,
            delta_y: 0,
            delta_z: 0,
            has_position: true,
            y_rot: None,
            x_rot: None,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::is_ticking),
            Some(true)
        );

        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(3, 4),
            type_id: 71,
            x: -4.0,
            y: 65.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(world.loaded_entity_count(), 1);
        assert_eq!(world.entity_at(37).map(|entity| entity.type_id), Some(71));
        assert_eq!(
            world.entity_at(37).map(|entity| entity.uuid),
            Some(Uuid::new(3, 4))
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((-4.0, 65.0, 2.0))
        );
        assert_eq!(world.entity_metadata_at(37), None);
        assert!(world.remove_entity(37));
        assert_eq!(world.loaded_entity_count(), 0);
        assert!(!world.remove_entity(37));
    }

    #[test]
    fn entity_passenger_state_replaces_previous_vehicle_and_cleans_relationships() {
        let mut world = ClientWorld::default();
        for entity_id in [37, 38, 39, 40] {
            assert!(world.apply_entity(EntityInput {
                entity_id,
                uuid: Uuid::new(entity_id as u64, 0),
                type_id: 71,
                x: 0.0,
                y: 64.0,
                z: 0.0,
                movement: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                data: 0,
            }));
        }
        world.set_local_player_id(42);

        assert!(world.apply_entity_passengers(39, &[38]));
        assert_eq!(world.entity_passengers(39), Some(&[38][..]));
        assert_eq!(world.entity_vehicle_id(38), Some(39));

        assert!(world.apply_entity_passengers(37, &[38, 39, 999, 38, 42]));
        assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));
        assert_eq!(world.entity_passengers(39), Some(&[][..]));
        assert_eq!(world.entity_vehicle_id(38), Some(37));
        assert_eq!(world.entity_vehicle_id(39), Some(37));
        assert_eq!(world.entity_vehicle_id(42), Some(37));
        assert!(world.apply_entity_passengers(39, &[37]));
        assert_eq!(world.entity_passengers(39), Some(&[][..]));
        assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));

        assert!(world.apply_entity_passengers(37, &[]));
        assert_eq!(world.entity_passengers(37), Some(&[][..]));
        assert_eq!(world.entity_vehicle_id(38), None);
        assert_eq!(world.entity_vehicle_id(39), None);
        assert_eq!(world.entity_vehicle_id(42), None);
        assert!(!world.apply_entity_passengers(999, &[38]));

        assert!(world.apply_entity_passengers(37, &[38]));
        assert!(world.remove_entity(37));
        assert_eq!(world.entity_vehicle_id(38), None);
        assert_eq!(world.entity_at(37), None);
    }

    #[test]
    fn local_player_head_rotation_follows_entity_packet_and_respawn_keep_data() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 42,
            uuid: Uuid::new(1, 2),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (15.0, -10.0, 35.0),
            data: 0,
        }));
        world.set_local_player_id(42);
        assert_eq!(
            world.local_player().map(|player| player.head_rotation()),
            Some(35.0)
        );
        assert!(world.apply_entity_rotation(EntityRotationInput {
            entity_id: 42,
            y_rot: None,
            x_rot: None,
            y_head_rot: Some(135.0),
            relative_y_rot: false,
            relative_x_rot: false,
        }));
        assert_eq!(
            world.local_player().map(|player| player.head_rotation()),
            Some(35.0)
        );
        assert_eq!(
            world.entity_at(42).map(|entity| entity.rotation().2),
            Some(35.0)
        );
        world.tick();
        assert_eq!(
            world.entity_at(42).map(|entity| entity.rotation().2),
            Some(68.333336)
        );
        world.tick();
        world.tick();
        assert_eq!(
            world.local_player().map(|player| player.head_rotation()),
            Some(135.0)
        );
        assert_eq!(
            world.entity_at(42).map(|entity| entity.rotation().2),
            Some(135.0)
        );
        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world.local_player().map(|player| player.head_rotation()),
            Some(135.0)
        );
        assert!(world.recreate_local_player(0));
        assert_eq!(
            world.local_player().map(|player| player.head_rotation()),
            Some(0.0)
        );
    }

    #[test]
    fn entity_shared_flags_follow_java_byte_metadata_and_preserve_unspecified_values() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let all_defined_flags = 0b1111_1011;
        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_SHARED_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                all_defined_flags,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("entity with shared flags");
        assert_eq!(entity.shared_flags(), all_defined_flags);
        assert!(entity.is_on_fire());
        assert!(entity.is_shift_key_down());
        assert!(entity.is_sprinting());
        assert!(entity.is_swimming());
        assert!(entity.is_invisible());
        assert!(entity.is_glowing());
        assert!(entity.is_fall_flying());

        assert!(world.apply_entity_metadata(37, &[1, 0, 7, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::shared_flags),
            Some(all_defined_flags)
        );

        assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
        let entity = world.entity_at(37).expect("cleared shared flags");
        assert_eq!(entity.shared_flags(), 0);
        assert!(!entity.is_on_fire());
        assert!(!entity.is_shift_key_down());
        assert!(!entity.is_sprinting());
        assert!(!entity.is_swimming());
        assert!(!entity.is_invisible());
        assert!(!entity.is_glowing());
        assert!(!entity.is_fall_flying());
    }

    #[test]
    fn entity_air_supply_follows_java_int_metadata_and_preserves_shared_flags() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::air_supply),
            Some(300)
        );

        let mut air_metadata = vec![
            super::ENTITY_AIR_SUPPLY_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(250, &mut air_metadata);
        air_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &air_metadata));
        let entity = world.entity_at(37).expect("entity with air supply");
        assert_eq!(entity.air_supply(), 250);
        assert_eq!(entity.shared_flags(), 0);

        let mut combined_metadata = vec![
            super::ENTITY_SHARED_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            64,
            super::ENTITY_AIR_SUPPLY_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(-1, &mut combined_metadata);
        combined_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &combined_metadata));
        let entity = world.entity_at(37).expect("entity with combined metadata");
        assert_eq!(entity.air_supply(), -1);
        assert_eq!(entity.shared_flags(), 64);
        assert!(entity.is_glowing());

        assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::air_supply),
            Some(-1)
        );
    }

    #[test]
    fn entity_no_gravity_follows_java_boolean_metadata_and_preserves_air_supply() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 78,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let mut no_gravity_metadata = vec![
            super::ENTITY_NO_GRAVITY_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ];
        assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
        let entity = world.entity_at(37).expect("no-gravity entity");
        assert!(entity.is_no_gravity());
        assert_eq!(entity.air_supply(), 300);

        no_gravity_metadata = vec![
            super::ENTITY_SHARED_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            64,
            super::ENTITY_NO_GRAVITY_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            0,
            255,
        ];
        assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
        let entity = world.entity_at(37).expect("cleared no-gravity entity");
        assert!(!entity.is_no_gravity());
        assert_eq!(entity.shared_flags(), 64);
        assert!(entity.is_glowing());

        let mut air_metadata = vec![1, 1];
        crate::wire::write_varint(250, &mut air_metadata);
        air_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &air_metadata));
        let entity = world.entity_at(37).expect("air supply entity");
        assert_eq!(entity.air_supply(), 250);
        assert!(!entity.is_no_gravity());
    }

    #[test]
    fn entity_silent_follows_java_boolean_metadata_and_preserves_other_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 5,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(!world.entity_at(37).expect("entity").is_silent());

        let metadata = [4, 8, 1, 255];
        assert!(world.apply_entity_metadata(37, &metadata));
        let entity = world.entity_at(37).expect("silent entity");
        assert!(entity.is_silent());
        assert!(!entity.is_no_gravity());
        assert_eq!(entity.air_supply(), 300);

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        let entity = world.entity_at(37).expect("silent no-gravity entity");
        assert!(entity.is_silent());
        assert!(entity.is_no_gravity());

        assert!(world.apply_entity_metadata(37, &[4, 8, 0, 255]));
        let entity = world.entity_at(37).expect("audible entity");
        assert!(!entity.is_silent());
        assert!(entity.is_no_gravity());
    }

    #[test]
    fn entity_custom_name_visibility_follows_java_boolean_metadata() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 5,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default name visibility entity");
        assert!(!entity.is_custom_name_visible());
        assert!(!entity.should_show_name());

        let mut visible_metadata = vec![
            2,
            super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
            1,
        ];
        crate::nbt::write_any_nbt(
            Some(&crate::nbt::FieldValue::Str("Display name".to_owned())),
            &mut visible_metadata,
        );
        visible_metadata.extend([
            super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ]);
        assert!(world.apply_entity_metadata(37, &visible_metadata));
        let entity = world.entity_at(37).expect("visible name entity");
        assert!(entity.is_custom_name_visible());
        assert!(entity.should_show_name());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
                0,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("hidden name entity");
        assert!(!entity.is_custom_name_visible());
        assert!(!entity.should_show_name());
    }

    #[test]
    fn entity_custom_name_presence_follows_java_optional_component_metadata() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 5,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default custom-name entity");
        assert!(!entity.has_custom_name());
        assert!(!entity.is_custom_name_visible());

        let mut custom_name_metadata = vec![
            super::ENTITY_CUSTOM_NAME_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
            1,
        ];
        crate::nbt::write_any_nbt(
            Some(&crate::nbt::FieldValue::Str("Display name".to_owned())),
            &mut custom_name_metadata,
        );
        custom_name_metadata.extend([
            super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ]);
        assert!(world.apply_entity_metadata(37, &custom_name_metadata));
        let entity = world.entity_at(37).expect("named entity");
        assert!(entity.has_custom_name());
        assert!(entity.is_custom_name_visible());
        assert!(entity.should_show_name());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_CUSTOM_NAME_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
                0,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("unnamed entity");
        assert!(!entity.has_custom_name());
        assert!(entity.is_custom_name_visible());
        assert!(entity.should_show_name());
    }

    #[test]
    fn entity_health_follows_java_float_metadata_and_preserves_partial_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.entity_at(37).and_then(super::ClientEntity::health),
            None
        );

        let mut health_metadata = vec![
            8,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0,
            super::ENTITY_HEALTH_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
        ];
        health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
        health_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &health_metadata));
        let entity = world.entity_at(37).expect("living entity health");
        assert_eq!(entity.health(), Some(7.5));
        assert_eq!(entity.shared_flags(), 0);

        let mut zero_health_metadata = vec![
            super::ENTITY_HEALTH_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
        ];
        zero_health_metadata.extend_from_slice(&0.0_f32.to_be_bytes());
        zero_health_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &zero_health_metadata));
        assert_eq!(
            world.entity_at(37).and_then(super::ClientEntity::health),
            Some(0.0)
        );

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        let entity = world.entity_at(37).expect("health after partial metadata");
        assert_eq!(entity.health(), Some(0.0));
        assert!(entity.is_no_gravity());
    }

    #[test]
    fn local_player_health_metadata_updates_and_respawn_resets_health() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);

        let mut health_metadata = vec![
            8,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0,
            super::ENTITY_HEALTH_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
        ];
        health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
        health_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &health_metadata));
        assert_eq!(
            world.local_player().map(|player| player.health()),
            Some(7.5)
        );

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world.local_player().map(|player| player.health()),
            Some(20.0)
        );
    }

    #[test]
    fn entity_arrow_count_follows_java_int_metadata_and_preserves_partial_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::arrow_count),
            Some(0)
        );

        let mut arrow_metadata = vec![
            super::ENTITY_ARROW_COUNT_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(3, &mut arrow_metadata);
        arrow_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &arrow_metadata));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::arrow_count),
            Some(3)
        );

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::arrow_count),
            Some(3)
        );

        assert!(world.apply_entity_metadata(37, &[12, 0, 7, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::arrow_count),
            Some(3)
        );
    }

    #[test]
    fn local_player_arrow_count_metadata_follows_entity_data_keep_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);

        let mut arrow_metadata = vec![
            super::ENTITY_ARROW_COUNT_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(5, &mut arrow_metadata);
        arrow_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &arrow_metadata));
        assert_eq!(
            world.local_player().map(|player| player.arrow_count()),
            Some(5)
        );

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world.local_player().map(|player| player.arrow_count()),
            Some(5)
        );

        assert!(world.recreate_local_player(0));
        assert_eq!(
            world.local_player().map(|player| player.arrow_count()),
            Some(0)
        );
    }

    #[test]
    fn entity_stinger_count_follows_java_int_metadata_and_preserves_partial_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::stinger_count),
            Some(0)
        );

        let mut stinger_metadata = vec![
            super::ENTITY_STINGER_COUNT_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(2, &mut stinger_metadata);
        stinger_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &stinger_metadata));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::stinger_count),
            Some(2)
        );

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::stinger_count),
            Some(2)
        );

        assert!(world.apply_entity_metadata(37, &[13, 0, 4, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::stinger_count),
            Some(2)
        );
    }

    #[test]
    fn local_player_stinger_count_metadata_follows_entity_data_keep_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);

        let mut stinger_metadata = vec![
            super::ENTITY_STINGER_COUNT_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(4, &mut stinger_metadata);
        stinger_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &stinger_metadata));
        assert_eq!(
            world.local_player().map(|player| player.stinger_count()),
            Some(4)
        );

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world.local_player().map(|player| player.stinger_count()),
            Some(4)
        );

        assert!(world.recreate_local_player(0));
        assert_eq!(
            world.local_player().map(|player| player.stinger_count()),
            Some(0)
        );
    }

    #[test]
    fn entity_effect_ambience_follows_java_boolean_metadata_with_empty_particles() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(!world.entity_at(37).expect("entity").is_effect_ambient());

        assert!(world.apply_entity_metadata(37, &[10, 17, 0, 11, 8, 1, 255]));
        assert!(
            world
                .entity_at(37)
                .expect("ambient entity")
                .is_effect_ambient()
        );

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        assert!(
            world
                .entity_at(37)
                .expect("ambient entity after partial metadata")
                .is_effect_ambient()
        );

        assert!(world.apply_entity_metadata(37, &[11, 0, 0, 255]));
        assert!(
            world
                .entity_at(37)
                .expect("ambient entity after wrong serializer")
                .is_effect_ambient()
        );

        assert!(world.apply_entity_metadata(37, &[10, 17, 1, 0, 11, 8, 0, 255]));
        assert!(
            world
                .entity_at(37)
                .expect("ambient entity after unsupported particles")
                .is_effect_ambient()
        );
    }

    #[test]
    fn local_player_effect_ambience_metadata_follows_entity_data_keep_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        assert!(world.apply_entity_metadata(42, &[10, 17, 0, 11, 8, 1, 255]));
        assert!(
            world
                .local_player()
                .expect("ambient player")
                .is_effect_ambient()
        );

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert!(
            world
                .local_player()
                .expect("respawned ambient player")
                .is_effect_ambient()
        );

        assert!(world.recreate_local_player(0));
        assert!(
            !world
                .local_player()
                .expect("reset ambient player")
                .is_effect_ambient()
        );
    }

    #[test]
    fn entity_effect_particle_types_follow_java_particle_list_and_preserve_partial_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(
            world
                .entity_at(37)
                .expect("entity")
                .effect_particle_type_ids()
                .is_empty()
        );

        let mut particle_metadata = vec![
            super::ENTITY_EFFECT_PARTICLES_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
            1,
        ];
        crate::wire::write_varint(
            super::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            &mut particle_metadata,
        );
        particle_metadata.extend_from_slice(&0x1122_3344_i32.to_be_bytes());
        particle_metadata.extend([11, 8, 0, 255]);
        assert!(world.apply_entity_metadata(37, &particle_metadata));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity with effect particle")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity after partial particle metadata")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );

        assert!(world.apply_entity_metadata(37, &[10, 0, 1, 255]));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity after wrong particle serializer")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );

        assert!(world.apply_entity_metadata(37, &[10, 17, 1, 0, 255]));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity after unsupported particle type")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );
    }

    #[test]
    fn local_player_effect_particle_types_follow_entity_data_keep_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        let mut particle_metadata = vec![
            super::ENTITY_EFFECT_PARTICLES_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
            1,
        ];
        crate::wire::write_varint(
            super::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            &mut particle_metadata,
        );
        particle_metadata.extend_from_slice(&0x5566_7788_i32.to_be_bytes());
        particle_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &particle_metadata));
        assert_eq!(
            world
                .local_player()
                .expect("particle player")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world
                .local_player()
                .expect("respawned particle player")
                .effect_particle_type_ids(),
            &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
        );

        assert!(world.recreate_local_player(0));
        assert!(
            world
                .local_player()
                .expect("reset particle player")
                .effect_particle_type_ids()
                .is_empty()
        );
    }

    #[test]
    fn entity_sleeping_position_follows_java_optional_block_pos_metadata() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default sleeping entity");
        assert_eq!(entity.sleeping_pos(), None);
        assert!(!entity.is_sleeping());

        let bed = crate::binary::BlockPos::new(-12, 90, 37);
        let mut sleeping_metadata = vec![
            super::ENTITY_SLEEPING_POS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
            1,
        ];
        crate::binary::write_blockpos(bed, &mut sleeping_metadata);
        sleeping_metadata.extend([3, 8, 1, 255]);
        assert!(world.apply_entity_metadata(37, &sleeping_metadata));
        let entity = world.entity_at(37).expect("sleeping entity");
        assert_eq!(entity.sleeping_pos(), Some(bed));
        assert!(entity.is_sleeping());
        assert!(entity.is_custom_name_visible());

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        assert_eq!(
            world
                .entity_at(37)
                .expect("sleeping entity after partial metadata")
                .sleeping_pos(),
            Some(bed)
        );

        assert!(world.apply_entity_metadata(37, &[14, 11, 0, 255]));
        assert_eq!(
            world.entity_at(37).expect("awake entity").sleeping_pos(),
            None
        );

        let mut wrong_serializer = vec![14, 10, 1];
        crate::binary::write_blockpos(bed, &mut wrong_serializer);
        wrong_serializer.push(255);
        assert!(world.apply_entity_metadata(37, &wrong_serializer));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity after wrong serializer")
                .sleeping_pos(),
            None
        );

        assert!(world.apply_entity_metadata(37, &[14, 11, 1, 0, 255]));
        assert_eq!(
            world
                .entity_at(37)
                .expect("entity after truncated sleeping position")
                .sleeping_pos(),
            None
        );
    }

    #[test]
    fn local_player_sleeping_position_follows_entity_data_keep_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        let bed = crate::binary::BlockPos::new(14, 90, -23);
        let mut sleeping_metadata = vec![
            super::ENTITY_SLEEPING_POS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
            1,
        ];
        crate::binary::write_blockpos(bed, &mut sleeping_metadata);
        sleeping_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &sleeping_metadata));
        assert_eq!(
            world
                .local_player()
                .expect("sleeping player")
                .sleeping_pos(),
            Some(bed)
        );
        assert!(world.local_player().expect("sleeping player").is_sleeping());

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(
            world
                .local_player()
                .expect("respawned sleeping player")
                .sleeping_pos(),
            Some(bed)
        );

        assert!(world.recreate_local_player(0));
        assert_eq!(
            world
                .local_player()
                .expect("reset sleeping player")
                .sleeping_pos(),
            None
        );
    }

    #[test]
    fn entity_living_flags_follow_java_byte_metadata_masks() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 54,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default living flags entity");
        assert_eq!(entity.living_flags(), 0);
        assert!(!entity.is_using_item());
        assert!(!entity.used_item_is_off_hand());
        assert!(!entity.is_auto_spin_attack());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_LIVING_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                0b111,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("all living flags entity");
        assert_eq!(entity.living_flags(), 0b111);
        assert!(entity.is_using_item());
        assert!(entity.used_item_is_off_hand());
        assert!(entity.is_auto_spin_attack());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_LIVING_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                0b010,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("off-hand living flags entity");
        assert_eq!(entity.living_flags(), 0b010);
        assert!(!entity.is_using_item());
        assert!(entity.used_item_is_off_hand());
        assert!(!entity.is_auto_spin_attack());

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        let entity = world
            .entity_at(37)
            .expect("living flags after partial metadata");
        assert_eq!(entity.living_flags(), 0b010);
        assert!(entity.is_no_gravity());
    }

    #[test]
    fn local_player_living_flags_follow_metadata_and_respawn_keep_mask() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        assert_eq!(
            world.local_player().map(|player| player.living_flags()),
            Some(0)
        );

        assert!(world.apply_entity_metadata(
            42,
            &[
                super::ENTITY_LIVING_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                0b111,
                255,
            ],
        ));
        let player = world.local_player().expect("living flags player");
        assert_eq!(player.living_flags(), 0b111);
        assert!(player.is_using_item());
        assert!(player.used_item_is_off_hand());
        assert!(player.is_auto_spin_attack());

        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        let player = world.local_player().expect("respawned living flags player");
        assert_eq!(player.living_flags(), 0b111);
        assert!(player.is_using_item());

        assert!(world.recreate_local_player(0));
        let player = world.local_player().expect("reset living flags player");
        assert_eq!(player.living_flags(), 0);
        assert!(!player.is_using_item());
        assert!(!player.used_item_is_off_hand());
        assert!(!player.is_auto_spin_attack());
    }

    #[test]
    fn mob_flags_follow_java_byte_metadata_masks_and_preserve_partial_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 70,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default mob flags entity");
        assert_eq!(entity.mob_flags(), 0);
        assert!(!entity.is_no_ai());
        assert!(!entity.is_left_handed());
        assert!(!entity.is_aggressive());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_MOB_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                0b111,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("all mob flags entity");
        assert_eq!(entity.mob_flags(), 0b111);
        assert!(entity.is_no_ai());
        assert!(entity.is_left_handed());
        assert!(entity.is_aggressive());

        assert!(world.apply_entity_metadata(
            37,
            &[
                super::ENTITY_MOB_FLAGS_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
                0b001,
                255,
            ],
        ));
        let entity = world.entity_at(37).expect("no-ai mob flags entity");
        assert_eq!(entity.mob_flags(), 0b001);
        assert!(entity.is_no_ai());
        assert!(!entity.is_left_handed());
        assert!(!entity.is_aggressive());

        assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
        let entity = world
            .entity_at(37)
            .expect("mob flags after partial metadata");
        assert_eq!(entity.mob_flags(), 0b001);
        assert!(entity.is_no_ai());
        assert!(entity.is_no_gravity());

        assert!(world.apply_entity_metadata(37, &[15, 8, 0, 255]));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::mob_flags),
            Some(0b001)
        );
    }

    #[test]
    fn entity_pose_follows_java_pose_metadata_and_maps_unknown_ids_to_standing() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 5,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::pose),
            Some(super::ClientEntityPose::Standing)
        );

        let expected_poses = [
            super::ClientEntityPose::Standing,
            super::ClientEntityPose::FallFlying,
            super::ClientEntityPose::Sleeping,
            super::ClientEntityPose::Swimming,
            super::ClientEntityPose::SpinAttack,
            super::ClientEntityPose::Crouching,
            super::ClientEntityPose::LongJumping,
            super::ClientEntityPose::Dying,
            super::ClientEntityPose::Croaking,
            super::ClientEntityPose::UsingTongue,
            super::ClientEntityPose::Sitting,
            super::ClientEntityPose::Roaring,
            super::ClientEntityPose::Sniffing,
            super::ClientEntityPose::Emerging,
            super::ClientEntityPose::Digging,
            super::ClientEntityPose::Sliding,
            super::ClientEntityPose::Shooting,
            super::ClientEntityPose::Inhaling,
        ];
        for (id, expected) in expected_poses.into_iter().enumerate() {
            let mut metadata = vec![
                super::ENTITY_POSE_DATA_ID,
                super::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
            ];
            crate::wire::write_varint(id as i32, &mut metadata);
            metadata.push(255);
            assert!(world.apply_entity_metadata(37, &metadata));
            let entity = world.entity_at(37).expect("entity with pose");
            assert_eq!(entity.pose(), expected);
            assert_eq!(entity.pose().id(), id as u8);
        }

        let combined_metadata = [0, 0, 64, 5, 8, 1, 6, 20, 10, 255];
        assert!(world.apply_entity_metadata(37, &combined_metadata));
        let entity = world.entity_at(37).expect("combined pose entity");
        assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
        assert!(entity.is_glowing());
        assert!(entity.is_no_gravity());

        assert!(world.apply_entity_metadata(37, &[6, 20, 99, 255]));
        let entity = world.entity_at(37).expect("fallback pose entity");
        assert_eq!(entity.pose(), super::ClientEntityPose::Standing);
        assert!(entity.is_glowing());
        assert!(entity.is_no_gravity());
    }

    #[test]
    fn entity_ticks_frozen_follows_java_int_metadata_and_derived_state() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 1,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world.entity_at(37).expect("default freeze entity");
        assert_eq!(entity.ticks_frozen(), 0);
        assert_eq!(entity.percent_frozen(), 0.0);
        assert!(!entity.is_freezing());
        assert!(!entity.is_fully_frozen());

        let mut partial_metadata = vec![
            super::ENTITY_TICKS_FROZEN_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(70, &mut partial_metadata);
        partial_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &partial_metadata));
        let entity = world.entity_at(37).expect("partially frozen entity");
        assert_eq!(entity.ticks_frozen(), 70);
        assert_eq!(entity.percent_frozen(), 0.5);
        assert!(entity.is_freezing());
        assert!(!entity.is_fully_frozen());

        let mut combined_metadata = vec![0, 0, 64, 6, 20, 10, 7, 1];
        crate::wire::write_varint(140, &mut combined_metadata);
        combined_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &combined_metadata));
        let entity = world.entity_at(37).expect("fully frozen entity");
        assert_eq!(entity.ticks_frozen(), 140);
        assert_eq!(entity.percent_frozen(), 1.0);
        assert!(entity.is_freezing());
        assert!(entity.is_fully_frozen());
        assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
        assert!(entity.is_glowing());

        let mut negative_metadata = vec![
            super::ENTITY_TICKS_FROZEN_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
        ];
        crate::wire::write_varint(-1, &mut negative_metadata);
        negative_metadata.push(255);
        assert!(world.apply_entity_metadata(37, &negative_metadata));
        let entity = world.entity_at(37).expect("negative freeze entity");
        assert_eq!(entity.ticks_frozen(), -1);
        assert_eq!(entity.percent_frozen(), -1.0 / 140.0);
        assert!(!entity.is_freezing());
        assert!(!entity.is_fully_frozen());
        assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
    }

    #[test]
    fn player_entities_keep_ticking_after_their_chunk_unloads() {
        let mut world = ClientWorld::default();
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 42,
            uuid: Uuid::new(5, 6),
            type_id: super::VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(
            world
                .entity_at(42)
                .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
        );

        assert!(world.unload_chunk(0, -3));
        assert!(
            world
                .entity_at(42)
                .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
        );
        world.tick();
        assert_eq!(
            world.entity_at(42).map(super::ClientEntity::tick_count),
            Some(1)
        );
    }

    #[test]
    fn tick_counter_is_independent_of_chunk_storage() {
        let mut world = ClientWorld::default();
        assert_eq!(world.game_time(), 0);
        world.tick();
        world.tick();
        assert_eq!(world.game_time(), 2);
        assert_eq!(world.loaded_chunk_count(), 0);
    }

    #[test]
    fn ticking_snowballs_applies_java_free_flight_gravity_drag_and_history() {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            x: 2.0,
            y: 1.0,
            z: 2.0,
            movement: (0.25, 0.5, 0.25),
            rotation: (10.0, -20.0, 30.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: 38,
            uuid: Uuid::new(3, 4),
            type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            x: 4.0,
            y: 1.0,
            z: 2.0,
            movement: (0.0, 0.0, 1.0),
            rotation: (45.0, 30.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_metadata(
            38,
            &[
                ENTITY_NO_GRAVITY_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
                1,
                255,
            ],
        ));

        world.tick();

        let air_drag = f64::from(0.99_f32);
        let expected_motion = (
            0.25 * air_drag,
            (0.5 - SNOWBALL_GRAVITY) * air_drag,
            0.25 * air_drag,
        );
        let entity = world.entity_at(37).expect("gravity snowball");
        assert_eq!(entity.previous_position(), (2.0, 1.0, 2.0));
        assert_eq!(entity.motion(), expected_motion);
        assert_eq!(
            entity.position(),
            (
                2.0 + expected_motion.0,
                1.0 + expected_motion.1,
                2.0 + expected_motion.2
            )
        );
        assert_eq!(entity.tick_count(), 1);

        let entity = world.entity_at(38).expect("no-gravity snowball");
        assert_eq!(entity.motion(), (0.0, 0.0, air_drag));
        assert_eq!(entity.position(), (4.0, 1.0, 2.0 + air_drag));
        assert_eq!(entity.previous_position(), (4.0, 1.0, 2.0));
        assert_eq!(entity.rotation(), (36.0, 24.0, 0.0));
    }

    #[test]
    fn tick_counters_follow_local_player_and_entity_ticking_eligibility() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert_eq!(
            world.local_player().map(|player| player.tick_count()),
            Some(0)
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::tick_count),
            Some(0)
        );

        world.tick();
        assert_eq!(
            world.local_player().map(|player| player.tick_count()),
            Some(1)
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::tick_count),
            Some(1)
        );

        assert!(world.unload_chunk(0, -3));
        world.tick();
        assert_eq!(
            world.local_player().map(|player| player.tick_count()),
            Some(2)
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::tick_count),
            Some(1)
        );

        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        world.tick();
        assert_eq!(
            world.local_player().map(|player| player.tick_count()),
            Some(3)
        );
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::tick_count),
            Some(2)
        );

        assert!(world.recreate_local_player(0));
        assert_eq!(
            world.local_player().map(|player| player.tick_count()),
            Some(0)
        );
        assert_eq!(
            world.local_player().map(|player| player.generation()),
            Some(1)
        );
    }

    #[test]
    fn tick_captures_previous_transform_for_position_and_rotation_interpolation() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        assert!(world.apply_local_player_position(
            (10.0, 64.0, -2.0),
            (0.0, 0.0, 0.0),
            (350.0, -20.0),
        ));

        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 120,
            x: 1.25,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (350.0, -20.0, 350.0),
            data: 0,
        }));

        world.tick();
        assert_eq!(
            world
                .local_player()
                .map(super::ClientPlayerState::previous_position),
            Some((10.0, 64.0, -2.0))
        );
        assert_eq!(
            world
                .entity_at(37)
                .map(super::ClientEntity::previous_position),
            Some((1.25, 80.0, -35.5))
        );

        assert!(world.apply_local_player_position(
            (14.0, 68.0, 6.0),
            (0.0, 0.0, 0.0),
            (10.0, 20.0),
        ));
        let player = world.local_player().expect("local player transform");
        assert_eq!(player.previous_position(), (10.0, 64.0, -2.0));
        assert_eq!(player.interpolated_position(0.5), (12.0, 66.0, 2.0));
        assert_eq!(player.previous_rotation(), (350.0, -20.0));
        assert_eq!(player.interpolated_rotation(0.5), (360.0, 0.0));
        assert_eq!(player.interpolated_rotation(1.0), (10.0, 20.0));

        assert!(world.apply_entity_position(EntityPositionInput {
            entity_id: 37,
            x: 3.0,
            y: 82.0,
            z: -34.5,
            relative_flags: 0,
        }));
        assert!(world.apply_entity_rotation(EntityRotationInput {
            entity_id: 37,
            y_rot: Some(10.0),
            x_rot: Some(20.0),
            y_head_rot: Some(10.0),
            relative_y_rot: false,
            relative_x_rot: false,
        }));
        let entity = world.entity_at(37).expect("remote entity transform");
        assert_eq!(entity.previous_position(), (1.25, 80.0, -35.5));
        assert_eq!(entity.interpolated_position(0.5), (2.125, 81.0, -35.0));
        assert_eq!(entity.previous_rotation(), (350.0, -20.0, 350.0));
        assert_eq!(entity.interpolated_rotation(0.5), (360.0, 0.0, 360.0));

        assert!(world.unload_chunk(0, -3));
        assert!(world.apply_entity_position(EntityPositionInput {
            entity_id: 37,
            x: 4.0,
            y: 82.0,
            z: -34.5,
            relative_flags: 0,
        }));
        world.tick();
        assert_eq!(
            world
                .entity_at(37)
                .map(super::ClientEntity::previous_position),
            Some((1.25, 80.0, -35.5))
        );

        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        world.tick();
        assert_eq!(
            world
                .entity_at(37)
                .map(super::ClientEntity::previous_position),
            Some((4.0, 82.0, -34.5))
        );
    }

    #[test]
    fn remote_player_position_interpolation_follows_java_three_step_handler() {
        let mut world = ClientWorld::default();
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 1.0,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (350.0, -20.0, 350.0),
            data: 0,
        }));

        assert!(world.apply_entity_position(EntityPositionInput {
            entity_id: 37,
            x: 4.0,
            y: 82.0,
            z: -34.5,
            relative_flags: 0,
        }));
        assert_eq!(
            world.entity_at(37).map(super::ClientEntity::position),
            Some((1.0, 80.0, -35.5))
        );
        assert_eq!(
            world
                .entity_at(37)
                .and_then(super::ClientEntity::interpolation_target_position),
            Some((4.0, 82.0, -34.5))
        );
        assert!(world.apply_entity_rotation(EntityRotationInput {
            entity_id: 37,
            y_rot: Some(10.0),
            x_rot: Some(20.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        }));
        assert_eq!(
            world
                .entity_at(37)
                .and_then(super::ClientEntity::interpolation_target_rotation),
            Some((10.0, 20.0))
        );

        world.tick();
        let first = world.entity_at(37).expect("first interpolation step");
        assert_eq!(first.interpolation_steps(), 2);
        assert_eq!(
            first.position(),
            (2.0, 80.66666666666667, -35.166666666666664)
        );
        assert_eq!(first.rotation(), (356.666_66_f32, -6.6666665_f32, 350.0));

        world.tick();
        let second = world.entity_at(37).expect("second interpolation step");
        assert_eq!(second.interpolation_steps(), 1);
        assert_eq!(
            second.position(),
            (3.0, 81.33333333333334, -34.83333333333333)
        );
        assert_eq!(second.rotation(), (3.333313_f32, 6.666667_f32, 350.0));

        world.tick();
        let third = world.entity_at(37).expect("final interpolation step");
        assert_eq!(third.interpolation_steps(), 0);
        assert_eq!(third.position(), (4.0, 82.0, -34.5));
        assert_eq!(third.rotation(), (10.0, 20.0, 350.0));
    }

    #[test]
    fn remote_player_motion_packet_interpolates_for_three_ticks() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 1.0,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));

        let target = (0.3, -0.6, 0.9);
        assert!(world.apply_entity_motion(EntityMotionInput {
            entity_id: 37,
            x: target.0,
            y: target.1,
            z: target.2,
        }));
        let entity = world.entity_at(37).expect("remote player motion target");
        assert_eq!(entity.motion(), (0.0, 0.0, 0.0));
        assert_eq!(entity.motion_interpolation_target(), Some(target));
        assert_eq!(entity.motion_interpolation_steps(), 3);

        let first = (target.0 / 3.0, target.1 / 3.0, target.2 / 3.0);
        world.tick();
        let entity = world
            .entity_at(37)
            .expect("first remote player motion step");
        assert_eq!(entity.motion(), first);
        assert_eq!(entity.motion_interpolation_steps(), 2);

        let second = (
            first.0 + (target.0 - first.0) / 2.0,
            first.1 + (target.1 - first.1) / 2.0,
            first.2 + (target.2 - first.2) / 2.0,
        );
        world.tick();
        let entity = world
            .entity_at(37)
            .expect("second remote player motion step");
        assert_eq!(entity.motion(), second);
        assert_eq!(entity.motion_interpolation_steps(), 1);

        let third = (
            second.0 + (target.0 - second.0),
            second.1 + (target.1 - second.1),
            second.2 + (target.2 - second.2),
        );
        world.tick();
        let entity = world
            .entity_at(37)
            .expect("final remote player motion step");
        assert_eq!(entity.motion(), third);
        assert_eq!(entity.motion(), target);
        assert_eq!(entity.motion_interpolation_target(), None);
        assert_eq!(entity.motion_interpolation_steps(), 0);
    }

    #[test]
    fn entity_teleport_applies_relative_delta_motion_and_rotation() {
        let mut world = ClientWorld::default();
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: 1,
            x: 10.0,
            y: 64.0,
            z: 20.0,
            movement: (1.0, 2.0, 3.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let relative_flags = ENTITY_RELATIVE_X_FLAG
            | ENTITY_RELATIVE_Y_FLAG
            | ENTITY_RELATIVE_Z_FLAG
            | ENTITY_RELATIVE_Y_ROT_FLAG
            | ENTITY_RELATIVE_X_ROT_FLAG
            | ENTITY_RELATIVE_DELTA_X_FLAG
            | ENTITY_RELATIVE_DELTA_Y_FLAG
            | ENTITY_RELATIVE_DELTA_Z_FLAG;
        assert!(world.apply_entity_teleport(EntityTeleportInput {
            entity_id: 37,
            x: 2.0,
            y: 3.0,
            z: 4.0,
            delta_x: 0.25,
            delta_y: -0.5,
            delta_z: 0.75,
            y_rot: 15.0,
            x_rot: 10.0,
            relative_flags,
        }));
        let entity = world.entity_at(37).expect("relative teleport entity");
        assert_eq!(entity.position(), (12.0, 67.0, 24.0));
        assert_eq!(entity.rotation(), (15.0, 10.0, 0.0));
        assert_eq!(entity.motion(), (1.25, 1.5, 3.75));

        assert!(world.apply_entity(EntityInput {
            entity_id: 38,
            uuid: Uuid::new(3, 4),
            type_id: 1,
            x: 10.0,
            y: 64.0,
            z: 20.0,
            movement: (1.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let rotate_flags = ENTITY_RELATIVE_Y_ROT_FLAG
            | ENTITY_RELATIVE_DELTA_X_FLAG
            | ENTITY_RELATIVE_DELTA_Z_FLAG
            | ENTITY_RELATIVE_ROTATE_DELTA_FLAG;
        assert!(world.apply_entity_teleport(EntityTeleportInput {
            entity_id: 38,
            x: 10.0,
            y: 64.0,
            z: 20.0,
            delta_x: 0.0,
            delta_y: 0.0,
            delta_z: 0.0,
            y_rot: 90.0,
            x_rot: 0.0,
            relative_flags: rotate_flags,
        }));
        let entity = world.entity_at(38).expect("rotated teleport entity");
        let radians = (f64::from(-90.0_f32) * (std::f64::consts::PI / 180.0)) as f32;
        let expected_x = f64::from(mth_cos_f64(f64::from(radians)));
        let expected_z = -f64::from(mth_sin_f64(f64::from(radians)));
        assert_eq!(entity.rotation(), (90.0, 0.0, 0.0));
        assert_eq!(entity.motion(), (expected_x, 0.0, expected_z));

        assert!(world.apply_entity(EntityInput {
            entity_id: 39,
            uuid: Uuid::new(5, 6),
            type_id: 1,
            x: 10.0,
            y: 64.0,
            z: 20.0,
            movement: (0.5, 0.25, -0.75),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_teleport(EntityTeleportInput {
            entity_id: 39,
            x: 10.0,
            y: 64.0,
            z: 20.0,
            delta_x: f64::NAN,
            delta_y: 0.0,
            delta_z: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
            relative_flags: ENTITY_RELATIVE_DELTA_X_FLAG,
        }));
        assert_eq!(
            world
                .entity_at(39)
                .expect("nonfinite teleport entity")
                .motion(),
            (0.5, 0.25, -0.75)
        );
    }

    #[test]
    fn remote_player_position_sync_snaps_when_distance_exceeds_java_threshold() {
        let mut world = ClientWorld::default();
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 2),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 1.0,
            y: 80.0,
            z: -35.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (350.0, -20.0, 350.0),
            data: 0,
        }));

        assert!(world.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: 37,
                x: 66.0,
                y: 80.0,
                z: -35.5,
                relative_flags: 0,
            },
            EntityRotationInput {
                entity_id: 37,
                y_rot: Some(10.0),
                x_rot: Some(20.0),
                y_head_rot: None,
                relative_y_rot: false,
                relative_x_rot: false,
            },
            EntityPositionUpdateMode::PositionSync,
        ));
        let entity = world.entity_at(37).expect("snapped remote player");
        assert_eq!(entity.position(), (66.0, 80.0, -35.5));
        assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
        assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));
        assert_eq!(entity.previous_rotation(), (10.0, 20.0, 350.0));
        assert_eq!(entity.interpolation_steps(), 0);
        assert_eq!(entity.interpolation_target_position(), None);
        assert_eq!(entity.interpolation_target_rotation(), None);

        world.tick();
        let entity = world.entity_at(37).expect("snapped remote player tick");
        assert_eq!(entity.position(), (66.0, 80.0, -35.5));
        assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
        assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));

        assert!(world.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: 37,
                x: 130.0,
                y: 80.0,
                z: -35.5,
                relative_flags: 0,
            },
            EntityRotationInput {
                entity_id: 37,
                y_rot: Some(30.0),
                x_rot: Some(10.0),
                y_head_rot: None,
                relative_y_rot: false,
                relative_x_rot: false,
            },
            EntityPositionUpdateMode::PositionSync,
        ));
        let entity = world.entity_at(37).expect("threshold remote player");
        assert_eq!(entity.position(), (66.0, 80.0, -35.5));
        assert_eq!(
            entity.interpolation_target_position(),
            Some((130.0, 80.0, -35.5))
        );
        assert_eq!(entity.interpolation_target_rotation(), Some((30.0, 10.0)));
        assert_eq!(entity.interpolation_steps(), 3);

        assert!(world.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: 37,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                relative_flags: 0b111,
            },
            EntityRotationInput {
                entity_id: 37,
                y_rot: Some(20.0),
                x_rot: Some(100.0),
                y_head_rot: None,
                relative_y_rot: true,
                relative_x_rot: true,
            },
            EntityPositionUpdateMode::Teleport,
        ));
        let entity = world.entity_at(37).expect("relative teleport target");
        assert_eq!(
            entity.interpolation_target_position(),
            Some((130.0, 80.0, -35.5))
        );
        assert_eq!(entity.interpolation_target_rotation(), Some((50.0, 90.0)));
    }

    #[test]
    fn local_player_respawn_recreates_identity_and_applies_entity_data_mask() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        assert_eq!(world.local_player().map(|player| player.id()), Some(42));
        assert_eq!(
            world.local_player().map(|player| player.generation()),
            Some(0)
        );

        assert!(world.apply_local_player_position(
            (10.5, 64.0, -3.25),
            (0.125, -0.25, 0.5),
            (135.0, -30.0),
        ));
        assert!(world.apply_entity_metadata(42, &[0, 0, 32, 255]));
        let mut air_metadata = vec![1, 1];
        crate::wire::write_varint(250, &mut air_metadata);
        air_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &air_metadata));
        let no_gravity_metadata = [5, 8, 1, 255];
        assert!(world.apply_entity_metadata(42, &no_gravity_metadata));
        let silent_metadata = [4, 8, 1, 255];
        assert!(world.apply_entity_metadata(42, &silent_metadata));
        let mut custom_name_metadata = vec![2, 6, 1];
        crate::nbt::write_any_nbt(
            Some(&crate::nbt::FieldValue::Str("Player name".to_owned())),
            &mut custom_name_metadata,
        );
        custom_name_metadata.push(255);
        assert!(world.apply_entity_metadata(42, &custom_name_metadata));
        let custom_name_visible_metadata = [3, 8, 1, 255];
        assert!(world.apply_entity_metadata(42, &custom_name_visible_metadata));
        let pose_metadata = [6, 20, 10, 255];
        assert!(world.apply_entity_metadata(42, &pose_metadata));
        let ticks_frozen_metadata = [7, 1, 70, 255];
        assert!(world.apply_entity_metadata(42, &ticks_frozen_metadata));

        assert!(world.recreate_local_player(
            PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS | PLAYER_RESPAWN_KEEP_ENTITY_DATA
        ));
        let player = world.local_player().expect("respawned local player");
        assert_eq!(player.id(), 42);
        assert_eq!(player.generation(), 1);
        assert_eq!(player.position(), (0.0, 0.0, 0.0));
        assert_eq!(player.motion(), (0.125, -0.25, 0.5));
        assert_eq!(player.rotation(), (135.0, -30.0));
        assert_eq!(player.entity_data(), Some(ticks_frozen_metadata.as_slice()));
        assert_eq!(player.shared_flags(), 32);
        assert!(player.is_invisible());
        assert_eq!(player.air_supply(), 250);
        assert!(player.is_no_gravity());
        assert!(player.is_silent());
        assert!(player.has_custom_name());
        assert!(player.is_custom_name_visible());
        assert!(player.should_show_name());
        assert_eq!(player.pose(), super::ClientEntityPose::Sitting);
        assert_eq!(player.ticks_frozen(), 70);
        assert!(player.is_freezing());

        assert!(world.recreate_local_player(0));
        let player = world.local_player().expect("reset local player");
        assert_eq!(player.id(), 42);
        assert_eq!(player.generation(), 2);
        assert_eq!(player.motion(), (0.0, 0.0, 0.0));
        assert_eq!(player.rotation(), (-180.0, 0.0));
        assert_eq!(player.entity_data(), None);
        assert_eq!(player.shared_flags(), 0);
        assert!(!player.is_invisible());
        assert_eq!(player.air_supply(), 300);
        assert!(!player.is_no_gravity());
        assert!(!player.is_silent());
        assert!(!player.has_custom_name());
        assert!(!player.is_custom_name_visible());
        assert!(player.should_show_name());
        assert_eq!(player.pose(), super::ClientEntityPose::Standing);
        assert_eq!(player.ticks_frozen(), 0);
        assert!(!player.is_freezing());
    }

    #[test]
    fn local_player_respawn_preserves_attribute_bases_and_applies_modifier_mask() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        let modifier =
            ClientAttributeModifierState::from_wire("minecraft:test_modifier".to_owned(), 0.25, 2);
        let attribute = ClientAttributeState::from_wire(17, 2.5, vec![modifier.clone()]);

        assert!(world.apply_local_player_attributes(42, &[attribute]));
        let player = world.local_player().expect("initial local player");
        assert_eq!(player.attributes().len(), 1);
        assert_eq!(player.attributes()[0].base_value(), 2.5);
        assert_eq!(
            player.attributes()[0].modifiers(),
            std::slice::from_ref(&modifier)
        );

        assert!(world.recreate_local_player(0));
        let player = world.local_player().expect("base-only respawned player");
        assert_eq!(player.attributes().len(), 1);
        assert_eq!(player.attributes()[0].base_value(), 2.5);
        assert!(player.attributes()[0].modifiers().is_empty());

        assert!(world.apply_local_player_attributes(
            42,
            &[ClientAttributeState::from_wire(
                17,
                3.5,
                vec![modifier.clone()],
            )],
        ));
        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
        let player = world.local_player().expect("modifier-preserving player");
        assert_eq!(player.attributes()[0].base_value(), 3.5);
        assert_eq!(player.attributes()[0].modifiers(), &[modifier]);
    }

    #[test]
    fn attribute_effective_value_matches_java_operation_order_and_sanitization() {
        let attribute = ClientAttributeState::from_wire(
            26,
            0.25,
            vec![
                ClientAttributeModifierState::from_wire("minecraft:oracle_add".to_owned(), 0.1, 0),
                ClientAttributeModifierState::from_wire("minecraft:oracle_base".to_owned(), 0.2, 1),
                ClientAttributeModifierState::from_wire(
                    "minecraft:oracle_total".to_owned(),
                    0.5,
                    2,
                ),
            ],
        );
        assert_eq!(
            attribute.effective_value(0.0, 1024.0).to_bits(),
            0x3fe4_28f5_c28f_5c29
        );

        let unknown_operation = ClientAttributeState::from_wire(
            26,
            1.0,
            vec![ClientAttributeModifierState::from_wire(
                "minecraft:oracle_unknown".to_owned(),
                0.5,
                255,
            )],
        );
        assert_eq!(unknown_operation.effective_value(0.0, 1024.0), 1.5);

        let clamped = ClientAttributeState::from_wire(26, 2048.0, Vec::new());
        assert_eq!(clamped.effective_value(0.0, 1024.0), 1024.0);

        let nan = ClientAttributeState::from_wire(26, f64::NAN, Vec::new());
        assert_eq!(nan.effective_value(0.0, 1024.0), 0.0);
    }

    #[test]
    fn local_player_mob_effects_replace_remove_and_clear_on_respawn() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);

        assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
        let player = world.local_player().expect("initial local player");
        assert_eq!(player.effects().len(), 1);
        assert_eq!(player.effects()[0].effect_id(), 18);
        assert_eq!(player.effects()[0].amplifier(), 0);
        assert_eq!(player.effects()[0].duration_ticks(), 1_200);
        assert_eq!(player.effects()[0].flags(), 0b1000);

        assert!(world.apply_local_player_mob_effect(42, 18, 1, 60, 0b1111));
        let player = world.local_player().expect("updated local player");
        assert_eq!(player.effects().len(), 1);
        assert_eq!(player.effects()[0].amplifier(), 1);
        assert_eq!(player.effects()[0].duration_ticks(), 60);
        assert_eq!(player.effects()[0].flags(), 0b1111);

        assert!(!world.apply_local_player_mob_effect(7, 19, 0, 20, 0));
        assert!(world.remove_local_player_mob_effect(42, 18));
        assert!(
            world
                .local_player()
                .expect("local player after removal")
                .effects()
                .is_empty()
        );
        assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
        assert!(
            world
                .local_player()
                .expect("respawned local player")
                .effects()
                .is_empty()
        );
    }

    #[test]
    fn local_player_mob_effect_durations_tick_without_removing_expired_entries() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(42);
        assert!(world.apply_local_player_mob_effect(42, 18, 0, 2, 0));
        assert!(world.apply_local_player_mob_effect(42, 19, 0, -1, 0));
        assert!(world.apply_local_player_mob_effect(42, 20, 0, 0, 0));
        assert!(world.apply_local_player_mob_effect(42, 21, 0, -2, 0));

        let player = world.local_player().expect("local player effects");
        assert!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 18)
                .is_some_and(|effect| effect.has_remaining_duration())
        );
        assert!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 19)
                .is_some_and(|effect| effect.has_remaining_duration())
        );
        assert!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 20)
                .is_some_and(|effect| !effect.has_remaining_duration())
        );

        world.tick();
        let player = world
            .local_player()
            .expect("local player after first effect tick");
        assert_eq!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 18)
                .map(|effect| effect.duration_ticks()),
            Some(1)
        );
        assert_eq!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 19)
                .map(|effect| effect.duration_ticks()),
            Some(-1)
        );

        world.tick();
        let player = world
            .local_player()
            .expect("local player after effect expiry");
        assert_eq!(player.effects().len(), 4);
        assert_eq!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 18)
                .map(|effect| effect.duration_ticks()),
            Some(0)
        );
        assert!(
            player
                .effects()
                .iter()
                .all(|effect| effect.effect_id() != 18 || !effect.has_remaining_duration())
        );

        world.tick();
        let player = world
            .local_player()
            .expect("local player after expired effect tick");
        assert_eq!(
            player
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == 18)
                .map(|effect| effect.duration_ticks()),
            Some(0)
        );
    }

    #[test]
    fn local_player_health_state_applies_and_resets_with_new_player() {
        let mut world = ClientWorld::default();
        assert!(!world.apply_local_player_health(0.0, 0, 0.0));
        world.set_local_player_id(42);
        let player = world.local_player().expect("initial local player");
        assert_eq!(player.health(), 20.0);
        assert_eq!(player.food(), 20);
        assert_eq!(player.saturation(), 5.0);

        assert!(world.apply_local_player_health(0.0, 7, 1.5));
        let player = world.local_player().expect("updated local player");
        assert_eq!(player.health(), 0.0);
        assert_eq!(player.food(), 7);
        assert_eq!(player.saturation(), 1.5);

        assert!(world.recreate_local_player(0));
        let player = world.local_player().expect("respawned local player");
        assert_eq!(player.health(), 20.0);
        assert_eq!(player.food(), 20);
        assert_eq!(player.saturation(), 5.0);
    }

    #[test]
    fn local_player_inventory_state_applies_slot_cursor_and_respawn_reset() {
        let mut world = ClientWorld::default();
        let stone = ClientItemStackState::from_wire(8, 1, vec![], vec![]);
        let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
        let mut items = vec![None; 46];
        items[36] = Some(stone.clone());

        assert!(!world.apply_local_player_container_content(0, 1, &items, Some(&stone),));
        world.set_local_player_id(42);
        assert!(world.apply_local_player_container_content(0, 2, &items, Some(&stone),));
        let player = world.local_player().expect("local player inventory");
        assert_eq!(player.inventory_state_id(), 2);
        assert_eq!(player.inventory_slot_count(), 46);
        assert_eq!(player.inventory_slot(36), Some(&stone));
        assert_eq!(player.carried_item(), Some(&stone));

        assert!(world.apply_local_player_container_slot(0, 3, 36, Some(&bottle)));
        assert_eq!(
            world
                .local_player()
                .expect("local player after slot update")
                .inventory_slot(36),
            Some(&bottle)
        );
        assert_eq!(
            world
                .local_player()
                .expect("local player after slot update")
                .inventory_state_id(),
            3
        );

        assert!(world.apply_local_player_carried_item(None));
        assert!(
            world
                .local_player()
                .expect("local player after cursor clear")
                .carried_item()
                .is_none()
        );

        assert!(world.apply_local_player_inventory_slot(0, Some(&stone)));
        assert_eq!(
            world
                .local_player()
                .expect("local player after direct inventory update")
                .inventory_slot(36),
            Some(&stone)
        );
        assert!(!world.apply_local_player_inventory_slot(36, Some(&bottle)));

        assert!(world.recreate_local_player(0));
        let player = world
            .local_player()
            .expect("respawned local player inventory");
        assert_eq!(player.inventory_state_id(), 0);
        assert_eq!(player.inventory_slot_count(), 0);
        assert!(player.inventory_slot(36).is_none());
        assert!(player.carried_item().is_none());
    }

    #[test]
    fn local_player_container_state_applies_open_content_slot_cursor_close_and_reset() {
        let mut world = ClientWorld::default();
        let stone = ClientItemStackState::from_wire(7, 1, vec![], vec![]);
        let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
        let title = [10, 0, 0, 0];
        let mut items = vec![None; 63];
        items[0] = Some(stone.clone());
        items[27] = Some(bottle.clone());

        assert!(!world.apply_local_player_open_menu(7, 2, &title));
        world.set_local_player_id(42);
        assert!(world.apply_local_player_open_menu(7, 2, &title));
        assert!(world.apply_local_player_container_content(7, 4, &items, None));
        let container = world
            .local_player()
            .expect("local player with open container")
            .open_container()
            .expect("open container");
        assert_eq!(container.container_id(), 7);
        assert_eq!(container.menu_type_id(), 2);
        assert_eq!(container.title_nbt(), title);
        assert_eq!(container.top_slot_count(), Some(27));
        assert_eq!(container.slot_count(), 63);
        assert_eq!(container.state_id(), 4);
        assert_eq!(container.slot(0), Some(&stone));
        assert_eq!(container.slot(27), Some(&bottle));
        assert!(container.carried_item().is_none());

        assert!(world.apply_local_player_container_slot(7, 5, 0, None));
        assert!(
            world
                .local_player()
                .expect("local player after custom slot update")
                .open_container()
                .expect("container after slot update")
                .slot(0)
                .is_none()
        );
        assert!(world.apply_local_player_carried_item(Some(&stone)));
        assert_eq!(
            world
                .local_player()
                .expect("local player after custom cursor update")
                .open_container()
                .expect("container after cursor update")
                .carried_item(),
            Some(&stone)
        );
        assert!(world.apply_local_player_container_close(99));
        assert!(
            world
                .local_player()
                .expect("local player after custom container close")
                .open_container()
                .is_none()
        );
        assert!(!world.apply_local_player_container_close(7));

        assert!(world.apply_local_player_open_menu(7, 2, &title));
        assert!(world.recreate_local_player(0));
        let player = world
            .local_player()
            .expect("respawned local player without container");
        assert!(player.open_container().is_none());
    }

    #[test]
    fn vanilla_menu_registry_slot_counts_cover_all_builtin_menu_types() {
        let expected = [
            (0, 9),
            (1, 18),
            (2, 27),
            (3, 36),
            (4, 45),
            (5, 54),
            (6, 9),
            (7, 10),
            (8, 3),
            (9, 1),
            (10, 3),
            (11, 5),
            (12, 10),
            (13, 2),
            (14, 3),
            (15, 3),
            (16, 5),
            (17, 1),
            (18, 4),
            (19, 3),
            (20, 27),
            (21, 4),
            (22, 3),
            (23, 3),
            (24, 2),
        ];

        for (menu_type_id, slot_count) in expected {
            assert_eq!(menu_top_slot_count(menu_type_id), Some(slot_count));
        }
        assert_eq!(menu_top_slot_count(-1), None);
        assert_eq!(menu_top_slot_count(25), None);
    }

    #[test]
    fn changing_dimension_keeps_local_player_identity_until_respawn_replacement() {
        let mut world = ClientWorld::default();
        world.set_local_player_id(17);
        let input = DimensionInput {
            key: "minecraft:the_nether",
            dimension_type_id: 3,
            seed: 776,
            is_debug: false,
            is_flat: false,
            sea_level: 32,
        };

        assert!(world.replace_dimension_with_environment(
            input,
            DimensionBounds::new(0, 256),
            DimensionEnvironment::for_builtin_dimension(input.key),
        ));
        assert_eq!(world.local_player().map(|player| player.id()), Some(17));
        assert_eq!(
            world.local_player().map(|player| player.generation()),
            Some(0)
        );
        assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
        assert_eq!(world.local_player().map(|player| player.id()), Some(17));
        assert_eq!(
            world.local_player().map(|player| player.generation()),
            Some(1)
        );
    }

    #[test]
    fn changing_dimension_replaces_world_storage_and_builtin_bounds() {
        let (blocks, biomes) = section(7, 8);
        let mut world = ClientWorld::default();
        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 9,
            uuid: Uuid::new(5, 6),
            type_id: 71,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.tick();

        let input = DimensionInput {
            key: "minecraft:the_nether",
            dimension_type_id: 1,
            seed: 776,
            is_debug: false,
            is_flat: false,
            sea_level: 32,
        };
        assert!(world.replace_dimension_with_environment(
            input,
            DimensionBounds::for_builtin_dimension(input.key),
            DimensionEnvironment::for_builtin_dimension(input.key),
        ));
        assert_eq!(world.dimension_key(), "minecraft:the_nether");
        assert_eq!(world.dimension(), DimensionBounds::new(0, 256));
        assert_eq!(world.dimension_type_id(), 1);
        assert_eq!(world.seed(), 776);
        assert_eq!(world.sea_level(), 32);
        assert_eq!(world.loaded_chunk_count(), 0);
        assert_eq!(world.loaded_entity_count(), 0);
        assert_eq!(world.game_time(), 0);

        world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(!world.replace_dimension_with_environment(
            input,
            DimensionBounds::for_builtin_dimension(input.key),
            DimensionEnvironment::for_builtin_dimension(input.key),
        ));
        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_entity_count(), 0);
    }
}
