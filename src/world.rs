//! Queryable client-side world state built from decoded chunk sections.

use std::collections::BTreeMap;

use crate::binary::BitSet;
use crate::math::{mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32};

pub(crate) const PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
pub(crate) const PLAYER_RESPAWN_KEEP_ENTITY_DATA: u8 = 2;

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
        self.local_player = Some(ClientPlayerState::new(player_id));
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
            self.entities.clear();
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
            player.tick_count = player.tick_count.wrapping_add(1);
        }
        for entity in self.entities.values_mut().filter(|entity| entity.ticking) {
            entity.capture_old_transform();
            entity.tick_count = entity.tick_count.wrapping_add(1);
        }
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
        if removed {
            for entity in self.entities.values_mut() {
                if entity_chunk_key(entity.position()) == (chunk_x, chunk_z) {
                    entity.ticking = false;
                }
            }
        }
        removed
    }

    pub fn loaded_light_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.lights.len()).sum()
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
        let (chunk_x, chunk_z) = entity_chunk_key((entity.x, entity.y, entity.z));
        let ticking = self.contains_chunk(chunk_x, chunk_z);
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
                x_rot_bits: entity.rotation.1.to_bits(),
                y_rot_bits: entity.rotation.0.to_bits(),
                y_head_rot_bits: entity.rotation.2.to_bits(),
                old_x_rot_bits: entity.rotation.1.to_bits(),
                old_y_rot_bits: entity.rotation.0.to_bits(),
                old_y_head_rot_bits: entity.rotation.2.to_bits(),
                on_ground: false,
                ticking,
                data: entity.data,
                metadata: None,
            },
        );
        true
    }

    pub(crate) fn apply_entity_metadata(&mut self, entity_id: i32, metadata: &[u8]) -> bool {
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.entity_data = Some(metadata.to_vec());
            return true;
        }
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.metadata = Some(metadata.to_vec());
        true
    }

    pub(crate) fn apply_entity_motion(&mut self, input: EntityMotionInput) -> bool {
        let Some(entity) = self.entities.get_mut(&input.entity_id) else {
            return false;
        };
        entity.motion_x_bits = input.x.to_bits();
        entity.motion_y_bits = input.y.to_bits();
        entity.motion_z_bits = input.z.to_bits();
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
        let Some(entity) = self.entities.get_mut(&input.entity_id) else {
            return false;
        };
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
        if let Some(y_head_rot) = input.y_head_rot {
            entity.y_head_rot_bits = y_head_rot.to_bits();
        }
        true
    }

    pub(crate) fn apply_entity_position(&mut self, input: EntityPositionInput) -> bool {
        let Some(current) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.position())
        else {
            return false;
        };
        let x = if input.relative_flags & (1 << 0) != 0 {
            current.0 + input.x
        } else {
            input.x
        };
        let y = if input.relative_flags & (1 << 1) != 0 {
            current.1 + input.y
        } else {
            input.y
        };
        let z = if input.relative_flags & (1 << 2) != 0 {
            current.2 + input.z
        } else {
            input.z
        };
        let (chunk_x, chunk_z) = entity_chunk_key((x, y, z));
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        let entity = self
            .entities
            .get_mut(&input.entity_id)
            .expect("entity checked above");
        entity.x_bits = x.to_bits();
        entity.y_bits = y.to_bits();
        entity.z_bits = z.to_bits();
        entity.ticking = ticking;
        true
    }

    pub(crate) fn apply_entity_move(&mut self, input: EntityMoveInput) -> bool {
        let Some(current) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.position())
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
        if input.has_position {
            let (x, y, z) = new_position.expect("position was requested");
            entity.x_bits = x.to_bits();
            entity.y_bits = y.to_bits();
            entity.z_bits = z.to_bits();
            entity.ticking = ticking.expect("position ticking state");
        }
        if let Some(y_rot) = input.y_rot {
            entity.y_rot_bits = (y_rot % 360.0).to_bits();
        }
        if let Some(x_rot) = input.x_rot {
            entity.x_rot_bits = (x_rot % 360.0).clamp(-90.0, 90.0).to_bits();
        }
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
    x_rot_bits: u32,
    y_rot_bits: u32,
    y_head_rot_bits: u32,
    old_x_rot_bits: u32,
    old_y_rot_bits: u32,
    old_y_head_rot_bits: u32,
    on_ground: bool,
    ticking: bool,
    pub data: i32,
    metadata: Option<Vec<u8>>,
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

    pub fn flags(&self) -> u8 {
        self.flags
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
    entity_data: Option<Vec<u8>>,
    attributes: Vec<ClientAttributeState>,
    effects: Vec<ClientMobEffectState>,
    health_bits: u32,
    food: i32,
    saturation_bits: u32,
    inventory_state_id: i32,
    inventory_slots: BTreeMap<i16, Option<ClientItemStackState>>,
    carried_item: Option<ClientItemStackState>,
    open_container: Option<ClientContainerState>,
}

impl ClientPlayerState {
    fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_x_rot_bits = self.x_rot_bits;
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
            entity_data: None,
            attributes: Vec::new(),
            effects: Vec::new(),
            health_bits: 20.0_f32.to_bits(),
            food: 20,
            saturation_bits: 5.0_f32.to_bits(),
            inventory_state_id: 0,
            inventory_slots: BTreeMap::new(),
            carried_item: None,
            open_container: None,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
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

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }

    pub fn health(&self) -> f32 {
        f32::from_bits(self.health_bits)
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
    fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_x_rot_bits = self.x_rot_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_y_head_rot_bits = self.y_head_rot_bits;
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

    pub fn rotation(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
            f32::from_bits(self.y_head_rot_bits),
        )
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

    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
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
        ClientWorld, DimensionBounds, DimensionEnvironment, DimensionInput, EntityInput,
        EntityMotionInput, EntityMoveInput, EntityPositionInput, EntityRotationInput,
        LightLayerInput, LightUpdateInput, PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS,
        PLAYER_RESPAWN_KEEP_ENTITY_DATA, SectionInput, menu_top_slot_count,
    };
    use crate::binary::{BitSet, Uuid};

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
    fn tick_counter_is_independent_of_chunk_storage() {
        let mut world = ClientWorld::default();
        assert_eq!(world.game_time(), 0);
        world.tick();
        world.tick();
        assert_eq!(world.game_time(), 2);
        assert_eq!(world.loaded_chunk_count(), 0);
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
        assert!(world.apply_entity_metadata(42, &[1, 0, 255]));

        assert!(world.recreate_local_player(
            PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS | PLAYER_RESPAWN_KEEP_ENTITY_DATA
        ));
        let player = world.local_player().expect("respawned local player");
        assert_eq!(player.id(), 42);
        assert_eq!(player.generation(), 1);
        assert_eq!(player.position(), (0.0, 0.0, 0.0));
        assert_eq!(player.motion(), (0.125, -0.25, 0.5));
        assert_eq!(player.rotation(), (135.0, -30.0));
        assert_eq!(player.entity_data(), Some(&[1, 0, 255][..]));

        assert!(world.recreate_local_player(0));
        let player = world.local_player().expect("reset local player");
        assert_eq!(player.id(), 42);
        assert_eq!(player.generation(), 2);
        assert_eq!(player.motion(), (0.0, 0.0, 0.0));
        assert_eq!(player.rotation(), (-180.0, 0.0));
        assert_eq!(player.entity_data(), None);
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
