//! Queryable client-side world state built from decoded chunk sections.

use std::collections::BTreeMap;

use crate::binary::BitSet;

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

    pub fn local_player(&self) -> Option<&ClientPlayerState> {
        self.local_player.as_ref()
    }

    pub(crate) fn set_local_player_id(&mut self, player_id: i32) {
        self.local_player = Some(ClientPlayerState::new(player_id));
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

    pub(crate) fn replace_dimension(
        &mut self,
        input: DimensionInput<'_>,
        bounds: DimensionBounds,
    ) -> bool {
        let changed = self.dimension_key != input.key;
        self.dimension = bounds;
        self.dimension_key = input.key.to_owned();
        self.dimension_type_id = input.dimension_type_id;
        self.seed = input.seed;
        self.is_debug = input.is_debug;
        self.is_flat = input.is_flat;
        self.sea_level = input.sea_level;
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
                ClientBlockEntity {
                    type_id: entity.type_id,
                    tag: entity.tag.map(<[u8]>::to_vec),
                },
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
            ClientBlockEntity {
                type_id: entity.type_id,
                tag: entity.tag.map(<[u8]>::to_vec),
            },
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
                x_bits: entity.x.to_bits(),
                y_bits: entity.y.to_bits(),
                z_bits: entity.z.to_bits(),
                motion_x_bits: entity.movement.0.to_bits(),
                motion_y_bits: entity.movement.1.to_bits(),
                motion_z_bits: entity.movement.2.to_bits(),
                x_rot_bits: entity.rotation.1.to_bits(),
                y_rot_bits: entity.rotation.0.to_bits(),
                y_head_rot_bits: entity.rotation.2.to_bits(),
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
pub struct ClientBlockEntity {
    pub type_id: i32,
    pub tag: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientEntity {
    pub uuid: crate::binary::Uuid,
    pub type_id: i32,
    x_bits: u64,
    y_bits: u64,
    z_bits: u64,
    motion_x_bits: u64,
    motion_y_bits: u64,
    motion_z_bits: u64,
    x_rot_bits: u32,
    y_rot_bits: u32,
    y_head_rot_bits: u32,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPlayerState {
    id: i32,
    generation: u64,
    x_bits: u64,
    y_bits: u64,
    z_bits: u64,
    motion_x_bits: u64,
    motion_y_bits: u64,
    motion_z_bits: u64,
    y_rot_bits: u32,
    x_rot_bits: u32,
    entity_data: Option<Vec<u8>>,
    attributes: Vec<ClientAttributeState>,
    effects: Vec<ClientMobEffectState>,
}

impl ClientPlayerState {
    fn new(id: i32) -> Self {
        Self {
            id,
            generation: 0,
            x_bits: 0.0_f64.to_bits(),
            y_bits: 0.0_f64.to_bits(),
            z_bits: 0.0_f64.to_bits(),
            motion_x_bits: 0.0_f64.to_bits(),
            motion_y_bits: 0.0_f64.to_bits(),
            motion_z_bits: 0.0_f64.to_bits(),
            y_rot_bits: 0.0_f32.to_bits(),
            x_rot_bits: 0.0_f32.to_bits(),
            entity_data: None,
            attributes: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
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

    pub fn entity_data(&self) -> Option<&[u8]> {
        self.entity_data.as_deref()
    }

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }
}

impl ClientEntity {
    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
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
        BlockEntityInput, ClientAttributeModifierState, ClientAttributeState, ClientWorld,
        DimensionBounds, DimensionInput, EntityInput, EntityMotionInput, EntityMoveInput,
        EntityPositionInput, EntityRotationInput, LightLayerInput, LightUpdateInput,
        PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA, SectionInput,
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

        assert!(world.replace_dimension(input, DimensionBounds::new(0, 256)));
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
        assert!(world.replace_dimension(input, DimensionBounds::for_builtin_dimension(input.key)));
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
        assert!(!world.replace_dimension(input, DimensionBounds::for_builtin_dimension(input.key)));
        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.loaded_entity_count(), 0);
    }
}
