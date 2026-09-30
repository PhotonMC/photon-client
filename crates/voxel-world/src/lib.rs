//! Queryable client-side world state built from decoded chunk sections.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::input::{
    Input, KeyboardInput, LocalPlayerHandleRelativeFrictionParameters, LocalPlayerInputModifiers,
    LocalPlayerSprintTriggerConditions, LocalPlayerSprintTriggerInput,
    LocalPlayerTravelDispatchParameters, LocalPlayerTravelDispatchResult,
    LocalPlayerTravelFlyingParameters, LocalPlayerTravelInAirParameters,
    LocalPlayerTravelInLavaParameters, LocalPlayerTravelInWaterParameters,
    LocalPlayerUpdateFallFlyingMovementParameters, advance_local_player_sprint_trigger,
};
use voxel_core::math::{Vec2, Vec3, abs_max_f64, mth_cos_f64, mth_sin_f64};
use voxel_core::{
    aabb::Aabb,
    binary::BlockPos,
    shape::{BooleanOp, Shapes, VoxelShape},
};

pub mod block;
mod block_entity;
pub mod block_rules;
pub mod block_state;
mod border;
mod chunk;
mod collision;
mod constants;
mod container;
mod dimension;
mod entity;
mod java_math;
pub mod light;
mod metadata;
mod player;
mod prediction;
mod region_copy;

pub use block_entity::*;
pub use block_state::*;
pub use border::*;
pub use chunk::*;
pub use collision::*;
pub use constants::*;
pub use container::*;
pub use dimension::*;
pub use entity::*;
pub(crate) use metadata::*;
pub use player::*;
pub use prediction::{DestroyProgress, SyncedBlock};
pub use region_copy::RegionCopy;

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
    block_state_registry: ClientBlockStateRegistry,
    chunks: BTreeMap<(i32, i32), ClientChunk>,
    chunk_cache: Option<ClientChunkCacheState>,
    /// The client level's light engine and the light tasks waiting for a
    /// frame, as `ClientLevel.queueLightUpdate` keeps them.
    light: light::LevelLightEngine,
    light_tasks: VecDeque<LightTask>,
    /// Sections the light tasks asked to redraw, besides the engine's own.
    light_dirty: Vec<light::SectionKey>,
    entities: BTreeMap<i32, ClientEntity>,
    entity_sections: BTreeMap<(i32, i32, i32), BTreeSet<i32>>,
    local_player: Option<ClientPlayerState>,
    world_border: ClientWorldBorder,
    game_time: u64,
    predictions: prediction::BlockPredictions,
    destroying_blocks: BTreeMap<i32, DestroyProgress>,
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
            block_state_registry: ClientBlockStateRegistry::vanilla_26_2(),
            chunks: BTreeMap::new(),
            chunk_cache: None,
            light: new_light_engine(dimension, true),
            light_tasks: VecDeque::new(),
            light_dirty: Vec::new(),
            entities: BTreeMap::new(),
            entity_sections: BTreeMap::new(),
            local_player: None,
            world_border: ClientWorldBorder::default(),
            game_time: 0,
            predictions: prediction::BlockPredictions::new(),
            destroying_blocks: BTreeMap::new(),
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

    pub fn block_state_registry(&self) -> &ClientBlockStateRegistry {
        &self.block_state_registry
    }

    /// Create the bounded chunk cache used by a Java client level.
    pub fn configure_chunk_cache(&mut self, server_view_radius: i32) {
        self.chunks.clear();
        self.light = new_light_engine(self.dimension, self.dimension_environment.has_skylight);
        self.light_tasks.clear();
        self.chunk_cache = Some(ClientChunkCacheState::new(server_view_radius));
    }

    /// Return the Java cache range and ring index for a chunk coordinate.
    pub fn chunk_cache_location(&self, chunk_x: i32, chunk_z: i32) -> Option<(bool, usize)> {
        self.chunk_cache.as_ref().map(|cache| {
            (
                cache.in_range(chunk_x, chunk_z),
                cache.index(chunk_x, chunk_z),
            )
        })
    }

    /// Move the Java client chunk-cache view center without evicting slots.
    pub fn update_chunk_cache_center(&mut self, chunk_x: i32, chunk_z: i32) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.view_center_x = chunk_x;
            cache.view_center_z = chunk_z;
        }
    }

    /// Resize the Java client chunk cache and retain chunks in the new range.
    pub fn update_chunk_cache_radius(&mut self, server_view_radius: i32) {
        let Some(cache) = self.chunk_cache.as_mut() else {
            return;
        };
        let new_storage_radius = server_view_radius.max(2) + 3;
        cache.server_view_radius = server_view_radius;
        if cache.storage_radius == new_storage_radius {
            return;
        }

        let mut new_cache = ClientChunkCacheState::new(server_view_radius);
        new_cache.view_center_x = cache.view_center_x;
        new_cache.view_center_z = cache.view_center_z;
        let retained = self
            .chunks
            .iter()
            .filter(|&(&(chunk_x, chunk_z), _)| new_cache.in_range(chunk_x, chunk_z))
            .map(|(&(chunk_x, chunk_z), chunk)| ((chunk_x, chunk_z), chunk.clone()))
            .collect::<Vec<_>>();
        self.chunks.clear();
        for ((chunk_x, chunk_z), chunk) in retained {
            let index = new_cache.index(chunk_x, chunk_z);
            new_cache.slots.insert(index, (chunk_x, chunk_z));
            new_cache.on_chunk_added(
                chunk_x,
                chunk_z,
                chunk
                    .sections
                    .iter()
                    .map(|(&section_y, section)| (section_y, section.has_only_air())),
            );
            self.chunks.insert((chunk_x, chunk_z), chunk);
        }
        self.chunk_cache = Some(new_cache);
    }

    /// Apply the cache's section-emptiness notification path.
    pub fn update_chunk_section_emptiness(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        empty: bool,
    ) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.on_section_emptiness_changed(section_x, section_y, section_z, empty);
        }
    }

    /// Swap the Java cache's double-buffered update sets.
    pub fn flip_chunk_cache_update_sets(&mut self) {
        if let Some(cache) = self.chunk_cache.as_mut() {
            cache.flip_update_tracking_sets();
        }
    }

    /// Snapshot the loaded chunks and cache update sets in deterministic order.
    pub fn client_chunk_cache_snapshot(&self) -> Option<ClientChunkCacheSnapshot> {
        let cache = self.chunk_cache.as_ref()?;
        let index = cache.updating_sets_index;
        Some(ClientChunkCacheSnapshot {
            storage_radius: cache.storage_radius,
            view_range: cache.view_range,
            view_center_x: cache.view_center_x,
            view_center_z: cache.view_center_z,
            loaded_chunks: self.chunks.keys().copied().collect(),
            added_empty_sections: cache.added_empty_sections[index].iter().copied().collect(),
            removed_empty_sections: cache.removed_empty_sections[index]
                .iter()
                .copied()
                .collect(),
            added_loaded_chunks: cache.added_loaded_chunks[index].iter().copied().collect(),
            removed_loaded_chunks: cache.removed_loaded_chunks[index].iter().copied().collect(),
        })
    }

    /// Apply a clientbound common update-tags packet to the world registry copy.
    pub fn apply_update_tags_packet(
        &mut self,
        packet: &voxel_core::tags::ClientboundUpdateTagsPacket,
    ) {
        self.block_state_registry
            .apply_update_tags(&packet.registries);
    }

    pub fn world_border(&self) -> &ClientWorldBorder {
        &self.world_border
    }

    /// Apply the client listener's initialize-border packet sequence.
    pub fn apply_world_border_initialize(&mut self, input: ClientWorldBorderInitialization) {
        self.world_border.set_center(input.center_x, input.center_z);
        if input.lerp_time > 0 {
            self.world_border
                .lerp_size_between(input.old_size, input.new_size, input.lerp_time);
        } else {
            self.world_border.set_size(input.new_size);
        }
        self.world_border
            .set_absolute_max_size(input.absolute_max_size);
        self.world_border.set_warning_blocks(input.warning_blocks);
        self.world_border.set_warning_time(input.warning_time);
    }

    pub fn apply_world_border_center(&mut self, center_x: f64, center_z: f64) {
        self.world_border.set_center(center_x, center_z);
    }

    pub fn apply_world_border_lerp_size(&mut self, old_size: f64, new_size: f64, lerp_time: i64) {
        self.world_border
            .lerp_size_between(old_size, new_size, lerp_time);
    }

    pub fn apply_world_border_size(&mut self, size: f64) {
        self.world_border.set_size(size);
    }

    pub fn apply_world_border_warning_blocks(&mut self, warning_blocks: i32) {
        self.world_border.set_warning_blocks(warning_blocks);
    }

    pub fn apply_world_border_warning_time(&mut self, warning_time: i32) {
        self.world_border.set_warning_time(warning_time);
    }

    pub fn local_player(&self) -> Option<&ClientPlayerState> {
        self.local_player.as_ref()
    }

    /// Create or replace the local player state for a clientbound player ID.
    pub fn set_local_player_id(&mut self, player_id: i32) {
        let mut player = ClientPlayerState::new(player_id);
        if let Some(entity) = self.entities.get(&player_id) {
            player.y_head_rot_bits = entity.y_head_rot_bits;
            player.old_y_head_rot_bits = entity.old_y_head_rot_bits;
            player.head_rotation_target_bits = entity.y_head_rot_bits;
            player.y_body_rot_bits = entity.y_body_rot_bits;
            player.old_y_body_rot_bits = entity.old_y_body_rot_bits;
        }
        self.local_player = Some(player);
    }

    pub fn apply_local_player_open_menu(
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

    pub fn apply_local_player_container_close(&mut self, _container_id: i32) -> bool {
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

    pub fn apply_local_player_position(
        &mut self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        let initialize_transform_history = player.tick_count == 0;
        player.x_bits = position.0.to_bits();
        player.y_bits = position.1.to_bits();
        player.z_bits = position.2.to_bits();
        player.motion_x_bits = motion.0.to_bits();
        player.motion_y_bits = motion.1.to_bits();
        player.motion_z_bits = motion.2.to_bits();
        player.y_rot_bits = rotation.0.to_bits();
        player.x_rot_bits = rotation.1.to_bits();
        if initialize_transform_history {
            player.old_x_bits = player.x_bits;
            player.old_y_bits = player.y_bits;
            player.old_z_bits = player.z_bits;
            player.old_y_rot_bits = player.y_rot_bits;
            player.old_x_rot_bits = player.x_rot_bits;
        }
        true
    }

    pub fn apply_local_player_health(&mut self, health: f32, food: i32, saturation: f32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.health_bits = health.to_bits();
        player.food = food;
        player.saturation_bits = saturation.to_bits();
        true
    }

    pub fn apply_local_player_abilities(&mut self, abilities: ClientPlayerAbilities) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.abilities = ClientPlayerAbilities {
            may_build: player.abilities.may_build,
            ..abilities
        };
        true
    }

    pub fn apply_local_player_game_mode(&mut self, game_mode: ClientPlayerGameMode) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.game_mode = game_mode;
        player.abilities = game_mode.update_player_abilities(player.abilities);
        true
    }

    /// Apply one LocalPlayer input snapshot to its queryable movement state.
    pub fn apply_local_player_input(
        &mut self,
        move_vector: voxel_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input(move_vector, jump_pressed, controlled_camera, modifiers);
        true
    }

    /// Apply LocalPlayer input while resolving the sneaking factor from its
    /// current movement attribute state.
    pub fn apply_local_player_input_from_attributes(
        &mut self,
        move_vector: voxel_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input_from_attributes(move_vector, jump_pressed, controlled_camera, modifiers);
        true
    }

    /// Sample keyboard input and apply the resulting LocalPlayer input state.
    pub fn apply_local_player_keyboard_input(
        &mut self,
        key_states: Input,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let mut keyboard = KeyboardInput::new(key_states);
        keyboard.tick();
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input_from_attributes(
            keyboard.move_vector(),
            keyboard.key_presses.jump,
            controlled_camera,
            modifiers,
        );
        true
    }

    /// Advance LocalPlayer's sprint double-tap window from one input snapshot.
    pub fn apply_local_player_sprint_trigger(
        &mut self,
        current_input: Input,
        current_move_vector: Vec2,
        sprint_window: i32,
        conditions: LocalPlayerSprintTriggerConditions,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        let result = advance_local_player_sprint_trigger(LocalPlayerSprintTriggerInput {
            sprinting: player.is_sprinting(),
            sprint_trigger_time: player.sprint_trigger_time,
            sprint_window,
            previous_input: player.input_key_presses,
            previous_move_vector: player.input_move_vector(),
            current_input,
            current_move_vector,
            conditions,
        });
        player.input_key_presses = current_input;
        player.input_move_vector_x_bits = current_move_vector.x.to_bits();
        player.input_move_vector_y_bits = current_move_vector.y.to_bits();
        player.sprint_trigger_time = result.sprint_trigger_time;
        if result.sprinting != player.is_sprinting() {
            player.apply_sprinting(result.sprinting);
        }
        true
    }

    /// Apply the fluid falling adjustment to the local player's motion state.
    pub fn apply_local_player_fluid_falling_adjustment(
        &mut self,
        base_gravity: f64,
        is_falling: bool,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_fluid_falling_adjustment(base_gravity, is_falling);
        true
    }

    /// Set local player climbable environment state for the next movement step.
    pub fn apply_local_player_climbable_state(
        &mut self,
        on_climbable: bool,
        scaffolding: bool,
        suppress_sliding_down_ladder: bool,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.on_climbable = on_climbable;
        player.in_scaffolding = scaffolding;
        player.suppress_sliding_down_ladder = suppress_sliding_down_ladder;
        true
    }

    /// Apply Java's climbable movement clamp to the local player's motion.
    pub fn apply_local_player_climbable_movement(&mut self) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_climbable_movement();
        true
    }

    /// Apply Java's friction-influenced speed selection to local player state.
    pub fn apply_local_player_friction_influenced_speed(
        &mut self,
        on_ground: bool,
        block_friction: f32,
        speed: f32,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_friction_influenced_speed(on_ground, block_friction, speed);
        true
    }

    /// Apply Java's friction-influenced speed using Player.getSpeed's
    /// movement-speed attribute lookup.
    pub fn apply_local_player_friction_influenced_speed_from_attributes(
        &mut self,
        on_ground: bool,
        block_friction: f32,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_friction_influenced_speed_from_attributes(on_ground, block_friction);
        true
    }

    /// Apply Java's modified block friction to local player state.
    pub fn apply_local_player_modified_friction(&mut self, friction: f32, modifier: f32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_modified_friction(friction, modifier);
        true
    }

    /// Apply Java's no-collision travelFlying path to local player motion.
    pub fn apply_local_player_travel_flying(
        &mut self,
        parameters: LocalPlayerTravelFlyingParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_travel_flying(parameters);
        true
    }

    /// Apply Java's resolved-environment travelInWater path to local motion.
    pub fn apply_local_player_travel_in_water(
        &mut self,
        parameters: LocalPlayerTravelInWaterParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_travel_in_water(parameters);
        true
    }

    /// Apply Java's travelInWater path using Player.getSpeed's movement-speed
    /// attribute lookup.
    pub fn apply_local_player_travel_in_water_from_attributes(
        &mut self,
        parameters: LocalPlayerTravelInWaterParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_travel_in_water_from_attributes(parameters);
        true
    }

    /// Apply Java's resolved-environment travelInLava path to local motion.
    pub fn apply_local_player_travel_in_lava(
        &mut self,
        parameters: LocalPlayerTravelInLavaParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_travel_in_lava(parameters);
        true
    }

    /// Apply Java's relative friction and climbable movement path to local motion.
    pub fn apply_local_player_handle_relative_friction(
        &mut self,
        parameters: LocalPlayerHandleRelativeFrictionParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_handle_relative_friction(parameters);
        true
    }

    /// Apply Java's resolved-environment travelInAir path to local motion.
    pub fn apply_local_player_travel_in_air(
        &mut self,
        parameters: LocalPlayerTravelInAirParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_travel_in_air(parameters);
        true
    }

    /// Apply Java's fall-flying movement update to local player motion.
    pub fn apply_local_player_update_fall_flying_movement(
        &mut self,
        parameters: LocalPlayerUpdateFallFlyingMovementParameters,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_update_fall_flying_movement(parameters);
        true
    }

    /// Apply Java's top-level travel dispatch to local player motion.
    pub fn apply_local_player_travel_dispatch(
        &mut self,
        parameters: LocalPlayerTravelDispatchParameters,
    ) -> Option<LocalPlayerTravelDispatchResult> {
        let player = self.local_player.as_mut()?;
        Some(player.apply_travel_dispatch(parameters))
    }

    pub fn apply_local_player_container_content(
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

    /// The local player's inventory menu: its 46 slots, the carried stack
    /// and the menu's state id.
    pub fn local_player_menu(
        &self,
    ) -> Option<(
        Vec<Option<ClientItemStackState>>,
        Option<ClientItemStackState>,
        i32,
    )> {
        let player = self.local_player.as_ref()?;
        let slots = (0..46)
            .map(|slot| player.inventory_slots.get(&slot).cloned().flatten())
            .collect();
        Some((
            slots,
            player.carried_item.clone(),
            player.inventory_state_id,
        ))
    }

    /// Write back the local player's inventory menu after a predicted click.
    pub fn set_local_player_menu(
        &mut self,
        slots: &[Option<ClientItemStackState>],
        carried: Option<ClientItemStackState>,
    ) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        for (slot, stack) in slots.iter().enumerate() {
            player.inventory_slots.insert(slot as i16, stack.clone());
        }
        player.carried_item = carried;
    }

    /// `ItemStack.shrink` on a stack in the local player's inventory menu,
    /// as the client predicts using it up. An emptied stack leaves the slot.
    pub fn shrink_local_player_slot(&mut self, slot: i16, amount: i32) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        if let Some(Some(stack)) = player.inventory_slots.get_mut(&slot) {
            stack.item_count -= amount;
            if stack.item_count <= 0 {
                player.inventory_slots.insert(slot, None);
            }
        }
    }

    pub fn apply_local_player_container_slot(
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

    pub fn apply_local_player_carried_item(&mut self, item: Option<&ClientItemStackState>) -> bool {
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

    pub fn apply_local_player_inventory_slot(
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

    pub fn recreate_local_player(&mut self, data_to_keep: u8) -> bool {
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
            player.old_y_rot_bits = previous.old_y_rot_bits;
            player.old_x_rot_bits = previous.old_x_rot_bits;
            player.y_body_rot_bits = previous.y_body_rot_bits;
            player.old_y_body_rot_bits = previous.old_y_body_rot_bits;
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

    /// Apply a clientbound attribute snapshot to the matching local player.
    pub fn apply_local_player_attributes(
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

    /// Apply Java LivingEntity.setSprinting to the local player state.
    pub fn apply_local_player_sprinting(&mut self, player_id: i32, sprinting: bool) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        player.apply_sprinting(sprinting);
        true
    }

    pub fn apply_local_player_mob_effect(
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

    /// Apply the three-tick head rotation interpolation target used by the
    /// clientbound rotate-head path for the local player.
    pub fn apply_local_player_head_rotation(&mut self, y_head_rot: f32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.head_rotation_target_bits = y_head_rot.to_bits();
        player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        true
    }

    /// Apply the three-tick head rotation interpolation target from a
    /// clientbound RotateHead packet to an entity in the client world.
    pub fn apply_entity_head_rotation(&mut self, entity_id: i32, y_head_rot: f32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            entity.head_rotation_target_bits = y_head_rot.to_bits();
            entity.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        } else {
            entity.y_head_rot_bits = y_head_rot.to_bits();
        }
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.head_rotation_target_bits = y_head_rot.to_bits();
            player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        }
        true
    }

    pub fn remove_local_player_mob_effect(&mut self, player_id: i32, effect_id: i32) -> bool {
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

    pub fn replace_dimension_with_environment(
        &mut self,
        input: DimensionInput<'_>,
        bounds: DimensionBounds,
        environment: DimensionEnvironment,
    ) -> bool {
        let changed = self.dimension_key != input.key;
        // A new level has its own light engine.
        if changed
            || self.dimension.min_y != bounds.min_y
            || self.dimension.height != bounds.height
            || self.dimension_environment.has_skylight != environment.has_skylight
        {
            self.light = new_light_engine(bounds, environment.has_skylight);
            self.light_tasks.clear();
            self.light_dirty.clear();
        }
        self.dimension = bounds;
        self.dimension_key = input.key.to_owned();
        self.dimension_type_id = input.dimension_type_id;
        self.seed = input.seed;
        self.is_debug = input.is_debug;
        self.is_flat = input.is_flat;
        self.sea_level = input.sea_level;
        self.dimension_environment = environment;
        if changed {
            let server_view_radius = self
                .chunk_cache
                .as_ref()
                .map(|cache| cache.server_view_radius);
            self.chunks.clear();
            self.chunk_cache = server_view_radius.map(ClientChunkCacheState::new);
            self.entities.clear();
            if let Some(player) = self.local_player.as_mut() {
                player.vehicle_id = None;
                player.passenger_ids.clear();
            }
            self.world_border = ClientWorldBorder::default();
            self.game_time = 0;
            // A new level starts with no predictions and no cracks.
            self.predictions = prediction::BlockPredictions::new();
            self.destroying_blocks.clear();
        }
        changed
    }

    pub fn game_time(&self) -> u64 {
        self.game_time
    }

    pub fn tick(&mut self) {
        self.world_border.tick();
        self.game_time = self.game_time.wrapping_add(1);
        let local_player_is_passenger = self.local_player.as_ref().is_some_and(|player| {
            self.entity_vehicle_id(player.id).is_some_and(|vehicle_id| {
                self.entity_passengers(vehicle_id)
                    .is_some_and(|ids| ids.contains(&player.id))
            })
        });
        if !local_player_is_passenger {
            self.tick_local_player(None);
        }

        let ticking_entity_ids: Vec<i32> = self
            .entities
            .iter()
            .filter(|(_, entity)| entity.ticking && entity.vehicle_id.is_none())
            .map(|(&entity_id, _)| entity_id)
            .collect();
        for entity_id in ticking_entity_ids {
            self.tick_entity_tree(entity_id, None);
        }
    }

    /// Apply Java's `Entity.turn` deltas and notify a boat passenger's vehicle.
    pub fn turn_entity(&mut self, entity_id: i32, mouse_x: f64, mouse_y: f64) -> bool {
        let vehicle_id = self.entity_vehicle_id(entity_id);
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.turn(mouse_x, mouse_y);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.turn(mouse_x, mouse_y);
        } else {
            return false;
        }
        if let Some(vehicle_id) = vehicle_id {
            self.clamp_entity_passenger_rotation_after_turn(vehicle_id, entity_id);
        }
        true
    }

    /// Copy local player movement flags into a client-authoritative boat.
    pub fn set_boat_input(&mut self, vehicle_id: i32, input: BoatInput) -> bool {
        let Some(boat) = self.entities.get_mut(&vehicle_id) else {
            return false;
        };
        if vanilla_boat_max_passengers(boat.type_id).is_none() {
            return false;
        }
        boat.boat_input = input;
        true
    }

    fn tick_local_player(&mut self, vehicle_id: Option<i32>) {
        let passenger_ids = {
            let Some(player) = self.local_player.as_mut() else {
                return;
            };
            if vehicle_id.is_some() {
                player.motion_x_bits = 0.0_f64.to_bits();
                player.motion_y_bits = 0.0_f64.to_bits();
                player.motion_z_bits = 0.0_f64.to_bits();
            }
            player.capture_old_transform();
            player.tick_head_rotation();
            player.tick_count = player.tick_count.wrapping_add(1);
            for effect in &mut player.effects {
                effect.tick_client();
            }
            player.passenger_ids.clone()
        };
        let Some(player_id) = self.local_player.as_ref().map(|player| player.id) else {
            return;
        };
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, player_id);
        }
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(player_id, passenger_id);
        }
    }

    fn tick_entity_tree(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
        if vehicle_id.is_some()
            && let Some(entity) = self.entities.get_mut(&entity_id)
        {
            entity.motion_x_bits = 0.0_f64.to_bits();
            entity.motion_y_bits = 0.0_f64.to_bits();
            entity.motion_z_bits = 0.0_f64.to_bits();
        }
        self.tick_entity_once(entity_id);
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, entity_id);
        }
        let passenger_ids = self
            .entity_passengers(entity_id)
            .map(<[i32]>::to_vec)
            .unwrap_or_default();
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(entity_id, passenger_id);
        }
    }

    fn tick_entity_passenger_tree(&mut self, vehicle_id: i32, passenger_id: i32) {
        if self.entity_vehicle_id(passenger_id) != Some(vehicle_id) {
            return;
        }
        if self
            .local_player
            .as_ref()
            .is_some_and(|player| player.id == passenger_id)
        {
            self.tick_local_player(Some(vehicle_id));
        } else if self
            .entities
            .get(&passenger_id)
            .is_some_and(|entity| entity.ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID)
        {
            self.tick_entity_tree(passenger_id, Some(vehicle_id));
        }
    }

    fn position_entity_passenger(&mut self, vehicle_id: i32, passenger_id: i32) {
        let Some(passenger_type_id) = self
            .entities
            .get(&passenger_id)
            .map(|entity| entity.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })
        else {
            return;
        };
        let Some((
            vehicle_type_id,
            vehicle_position,
            vehicle_y_rot,
            passenger_index,
            passenger_count,
        )) = self.entities.get(&vehicle_id).and_then(|vehicle| {
            let passenger_index = vehicle
                .passenger_ids
                .iter()
                .position(|&entity_id| entity_id == passenger_id)?;
            Some((
                vehicle.type_id,
                vehicle.position(),
                f32::from_bits(vehicle.y_rot_bits),
                passenger_index,
                vehicle.passenger_ids.len(),
            ))
        })
        else {
            return;
        };
        let Some((ride_height, single_passenger_z_offset)) = VANILLA_BOAT_PASSENGER_GEOMETRY
            .iter()
            .find(|&&(type_id, _, _)| type_id == vehicle_type_id)
            .map(|&(_, ride_height, single_passenger_z_offset)| {
                (f64::from(ride_height), f64::from(single_passenger_z_offset))
            })
        else {
            return;
        };
        let passenger_is_baby = self
            .entities
            .get(&passenger_id)
            .is_some_and(|passenger| passenger.baby);
        let passenger_vehicle_attachment_y =
            vanilla_passenger_vehicle_attachment_y(passenger_type_id, passenger_is_baby);

        let passenger_z_offset = if passenger_count > 1 {
            let base_offset = if passenger_index == 0 {
                0.2_f32
            } else {
                -0.6_f32
            };
            let animal_offset = if vanilla_passenger_is_animal(passenger_type_id) {
                0.2_f32
            } else {
                0.0_f32
            };
            f64::from(base_offset + animal_offset)
        } else {
            single_passenger_z_offset
        };
        let radians = f64::from(-vehicle_y_rot * (std::f32::consts::PI / 180.0_f32));
        let rotated_x = passenger_z_offset * f64::from(mth_sin_f64(radians));
        let rotated_z = passenger_z_offset * f64::from(mth_cos_f64(radians));
        let target_position = (
            vehicle_position.0 + rotated_x,
            vehicle_position.1 + ride_height - passenger_vehicle_attachment_y,
            vehicle_position.2 + rotated_z,
        );
        let previous_entity_position = self.entities.get(&passenger_id).map(ClientEntity::position);
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.set_position(target_position);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.set_position(target_position);
        }
        if let Some(previous_position) = previous_entity_position {
            self.update_entity_section(passenger_id, previous_position, target_position);
        }
        self.clamp_entity_passenger_rotation(vehicle_id, passenger_id);
    }

    fn clamp_entity_passenger_rotation(&mut self, vehicle_id: i32, passenger_id: i32) {
        let Some((vehicle_type_id, vehicle_y_rot, delta_rotation, passenger_count)) =
            self.entities.get(&vehicle_id).map(|vehicle| {
                (
                    vehicle.type_id,
                    f32::from_bits(vehicle.y_rot_bits),
                    f32::from_bits(vehicle.delta_rotation_bits),
                    vehicle.passenger_ids.len(),
                )
            })
        else {
            return;
        };
        let Some(passenger_type_id) = self
            .entities
            .get(&passenger_id)
            .map(|passenger| passenger.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })
        else {
            return;
        };
        if vanilla_passenger_can_turn_in_boat(passenger_type_id) {
            return;
        }
        let Some(max_passenger_count) = vanilla_boat_max_passengers(vehicle_type_id) else {
            return;
        };
        let animal_rotation_offset = (vanilla_passenger_is_animal(passenger_type_id)
            && passenger_count == max_passenger_count)
            .then_some({
                if passenger_id % 2 == 0 {
                    90.0_f32
                } else {
                    270.0_f32
                }
            });
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.add_vanilla_boat_passenger_rotation(delta_rotation);
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, animal_rotation_offset);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.add_vanilla_boat_passenger_rotation(delta_rotation);
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    fn clamp_entity_passenger_rotation_after_turn(&mut self, vehicle_id: i32, passenger_id: i32) {
        let Some(vehicle_y_rot) = self.entities.get(&vehicle_id).and_then(|vehicle| {
            VANILLA_BOAT_PASSENGER_GEOMETRY
                .iter()
                .any(|&(type_id, _, _)| type_id == vehicle.type_id)
                .then(|| f32::from_bits(vehicle.y_rot_bits))
        }) else {
            return;
        };
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, None);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    fn tick_entity_once(&mut self, entity_id: i32) {
        // One read of the entity decides every branch below; the reference
        // evaluates these conditions against the pre-tick state, so they are
        // resolved before the mutable tick borrow is taken.
        let Some((
            boat_is_non_authoritative,
            projectile_kind,
            projectile_path_is_open,
            boat_in_air,
            boat_control,
        )) = self.entities.get(&entity_id).map(|entity| {
            let is_boat = vanilla_boat_max_passengers(entity.type_id).is_some();
            let authoritative = is_boat && self.boat_is_local_instance_authoritative(entity);
            let kind = vanilla_projectile_kind(entity.type_id);
            let path_open = kind.is_some_and(|kind| {
                self.projectile_path_is_open(
                    entity.position(),
                    kind.next_position(entity.position(), entity.motion(), entity.no_gravity),
                )
            });
            (
                is_boat && !authoritative,
                kind,
                path_open,
                authoritative && self.boat_is_in_air(entity),
                authoritative.then_some((entity.boat_input, !entity.passenger_ids.is_empty())),
            )
        })
        else {
            return;
        };

        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return;
        };
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            entity.capture_old_transform();
            entity.tick_count = entity.tick_count.wrapping_add(1);
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_position_interpolation();
            }
            entity.tick_head_rotation();
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_motion_interpolation();
            }
            if boat_is_non_authoritative {
                entity.motion_x_bits = 0.0_f64.to_bits();
                entity.motion_y_bits = 0.0_f64.to_bits();
                entity.motion_z_bits = 0.0_f64.to_bits();
            }
            match projectile_kind {
                // The orb ticks either way and takes the blocked path as input;
                // the others only advance while their path stays open.
                Some(VanillaProjectileKind::ExperienceOrb) => {
                    entity.tick_experience_orb(projectile_path_is_open);
                }
                Some(kind) if projectile_path_is_open => match kind {
                    VanillaProjectileKind::Snowball => entity.tick_snowball(),
                    VanillaProjectileKind::Arrow => entity.tick_arrow(),
                    VanillaProjectileKind::PrimedTnt => entity.tick_primed_tnt(),
                    VanillaProjectileKind::FallingBlock => entity.tick_falling_block(),
                    VanillaProjectileKind::Firework => entity.tick_firework(),
                    VanillaProjectileKind::ExperienceOrb => unreachable!("handled above"),
                },
                _ => {}
            }
            if boat_in_air {
                entity.tick_boat_float_in_air();
            }
            if let Some((input, has_passengers)) = boat_control {
                entity.tick_boat_control(input, has_passengers);
            }
            if boat_in_air {
                entity.move_by_motion();
            }
        }
        let position = self
            .entities
            .get(&entity_id)
            .expect("entity remains after tick")
            .position();
        self.update_entity_section(entity_id, previous_position, position);
    }

    fn boat_is_in_air(&self, entity: &ClientEntity) -> bool {
        let position = entity.position();
        let min_x = voxel_core::math::floor_f64(position.0 - VANILLA_BOAT_HALF_WIDTH);
        let max_x = voxel_core::math::ceil_f64(position.0 + VANILLA_BOAT_HALF_WIDTH);
        let min_y = voxel_core::math::floor_f64(position.1 - 0.001) - 1;
        let max_y = voxel_core::math::ceil_f64(position.1 + VANILLA_BOAT_HEIGHT) + 1;
        let min_z = voxel_core::math::floor_f64(position.2 - VANILLA_BOAT_HALF_WIDTH);
        let max_z = voxel_core::math::ceil_f64(position.2 + VANILLA_BOAT_HALF_WIDTH);

        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let Some((_, fluid_count)) = self.section_counts_at(
                        x.div_euclid(16),
                        y.div_euclid(16),
                        z.div_euclid(16),
                    ) else {
                        return false;
                    };
                    if fluid_count != 0
                        || self.block_state_at(x, y, z) != Some(VANILLA_AIR_BLOCK_STATE_ID)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn boat_is_local_instance_authoritative(&self, entity: &ClientEntity) -> bool {
        self.local_player
            .as_ref()
            .is_some_and(|player| entity.passenger_ids.first() == Some(&player.id))
    }

    /// Java advances these entities only while the block they occupy and the
    /// block they would reach are both air. Each kind contributes its own
    /// one-tick step; this shared check keeps the two block lookups identical.
    fn projectile_path_is_open(
        &self,
        position: (f64, f64, f64),
        next_position: (f64, f64, f64),
    ) -> bool {
        let current_block = (
            voxel_core::math::floor_f64(position.0),
            voxel_core::math::floor_f64(position.1),
            voxel_core::math::floor_f64(position.2),
        );
        let next_block = (
            voxel_core::math::floor_f64(next_position.0),
            voxel_core::math::floor_f64(next_position.1),
            voxel_core::math::floor_f64(next_position.2),
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

    fn cached_chunk_is_current(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let Some(cache) = self.chunk_cache.as_ref() else {
            return true;
        };
        cache.in_range(chunk_x, chunk_z)
            && cache.slots.get(&cache.index(chunk_x, chunk_z)) == Some(&(chunk_x, chunk_z))
    }

    fn chunk_for_query(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        self.cached_chunk_is_current(chunk_x, chunk_z)
            .then(|| self.chunks.get(&(chunk_x, chunk_z)))
            .flatten()
    }

    fn chunk_for_update(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ClientChunk> {
        if !self.cached_chunk_is_current(chunk_x, chunk_z) {
            return None;
        }
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk_for_query(chunk_x, chunk_z).is_some()
    }

    /// `handleForgetLevelChunk`: the chunk is dropped if the cache holds it,
    /// and its light is queued for removal either way.
    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        self.queue_light_removal(chunk_x, chunk_z);
        if let Some(cache) = self.chunk_cache.as_ref() {
            if !cache.in_range(chunk_x, chunk_z) {
                return false;
            }
            let index = cache.index(chunk_x, chunk_z);
            if cache.slots.get(&index) != Some(&(chunk_x, chunk_z)) {
                return false;
            }
            let removed = self.chunks.remove(&(chunk_x, chunk_z));
            if let Some(chunk) = removed.as_ref() {
                let section_ys = chunk.sections.keys().copied().collect::<Vec<_>>();
                let cache = self
                    .chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during unload");
                cache.slots.remove(&index);
                cache.on_chunk_removed(chunk_x, chunk_z, section_ys);
            }
            if removed.is_some() {
                self.unload_chunk_light(chunk_x, chunk_z);
                for entity in self.entities.values_mut() {
                    if !entity.always_ticking
                        && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                    {
                        entity.ticking = false;
                    }
                }
            }
            return removed.is_some();
        }

        let removed = self.chunks.remove(&(chunk_x, chunk_z)).is_some();
        if removed {
            self.unload_chunk_light(chunk_x, chunk_z);
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

    /// Sections of loaded chunks with published light data in either layer.
    pub fn loaded_light_section_count(&self) -> usize {
        let mut sections: BTreeSet<light::SectionKey> = BTreeSet::new();
        for layer in [light::LightLayer::Sky, light::LightLayer::Block] {
            sections.extend(
                self.light
                    .visible_sections(layer)
                    .into_iter()
                    .filter(|key| self.contains_chunk(key.0, key.2)),
            );
        }
        sections.len()
    }

    /// Light tasks waiting for a frame.
    pub fn pending_light_update_count(&self) -> usize {
        self.light_tasks.len()
    }

    /// The layer's value where the engine holds or has queued a layer for the
    /// section, as `getDataLayerData` finds it.
    pub fn sky_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Sky, x, y, z)
    }

    pub fn block_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Block, x, y, z)
    }

    /// `getBrightness(LightLayer.SKY, pos)`: above the highest section with
    /// data the sky is full, and a section with no data takes the bottom row
    /// of the nearest one above.
    pub fn sky_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Sky, (x, y, z))
    }

    /// `getBrightness(LightLayer.BLOCK, pos)`: nothing where no data arrived.
    pub fn block_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Block, (x, y, z))
    }

    pub fn light_engine(&self) -> &light::LevelLightEngine {
        &self.light
    }

    pub fn loaded_block_entity_count(&self) -> usize {
        self.chunks
            .values()
            .map(|chunk| chunk.block_entities.len())
            .sum()
    }

    pub fn block_entity_at(&self, x: i32, y: i32, z: i32) -> Option<&ClientBlockEntity> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        self.chunk_for_query(chunk_key.0, chunk_key.1)?
            .block_entities
            .get(&(x, y, z))
    }

    pub fn loaded_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Return the number of non-empty entity sections in the client index.
    pub fn indexed_entity_section_count(&self) -> usize {
        self.entity_sections.len()
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
        let Some(position) = self.entities.get(&entity_id).map(ClientEntity::position) else {
            return false;
        };
        self.detach_entity_from_vehicle(entity_id);
        self.eject_entity_passengers(entity_id);
        self.remove_entity_from_section(entity_id, position);
        self.entities.remove(&entity_id).is_some()
    }

    /// Update one live entity position and its persistent section membership.
    pub fn update_entity_position(&mut self, entity_id: i32, position: (f64, f64, f64)) -> bool {
        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.set_position(position);
            entity.ticking = entity.always_ticking || ticking;
        }
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    pub fn remove_entities(&mut self, entity_ids: &[i32]) -> usize {
        entity_ids
            .iter()
            .filter(|&&entity_id| self.remove_entity(entity_id))
            .count()
    }

    /// A light update packet: `handleLightUpdatePacket` queues its layers
    /// for the next frame, and their sections are redrawn when it runs.
    pub fn apply_light_updates(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, true, false)
    }

    /// The light that came with a chunk: `handleLevelChunkWithLight` queues
    /// it, then switches the chunk's sections on for light.
    pub fn apply_chunk_light(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, false, true)
    }

    fn queue_light_data(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
        schedule_rebuild: bool,
        enable_chunk: bool,
    ) -> LightApplication {
        let light_section_count = self.dimension.section_count().saturating_add(2);
        let invalid_sky_updates =
            count_invalid_light_updates(light_section_count, updates.sky.mask, updates.sky.updates);
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
        let application = LightApplication {
            applied_sky_sections: count_light_sections(
                light_section_count,
                updates.sky.mask,
                updates.sky.empty_mask,
            ),
            applied_block_sections: count_light_sections(
                light_section_count,
                updates.block.mask,
                updates.block.empty_mask,
            ),
            queued: !self.contains_chunk(chunk_x, chunk_z),
            ..LightApplication::default()
        };
        self.light_tasks.push_back(LightTask::Data {
            update: PendingLightUpdate::new(chunk_x, chunk_z, updates),
            schedule_rebuild,
            enable_chunk,
        });
        application
    }

    /// `ClientLevel.unload`: the chunk's column stops lighting at once.
    fn unload_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light
            .set_light_enabled(&view, (chunk_x, chunk_z), false);
    }

    /// `queueLightRemoval`: whether or not the chunk was loaded, its light
    /// is dropped on a later frame.
    fn queue_light_removal(&mut self, chunk_x: i32, chunk_z: i32) {
        self.light_tasks
            .push_back(LightTask::Removal { chunk_x, chunk_z });
    }

    /// `enableChunkLight`: each of the loaded chunk's sections tells the
    /// engine whether it is empty, and the chunk and its neighbours redraw.
    pub fn enable_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let Some(chunk) = self.chunk_for_query(chunk_x, chunk_z) else {
            return;
        };
        let statuses: Vec<(i32, bool)> = chunk
            .sections
            .iter()
            .map(|(y, section)| (*y, section.has_only_air()))
            .collect();
        for (section_y, empty) in statuses {
            self.light
                .update_section_status((chunk_x, section_y, chunk_z), empty);
        }
        let min_section = self.dimension.min_section_y();
        let max_section = min_section + self.dimension.section_count() as i32 - 1;
        for x in chunk_x - 1..=chunk_x + 1 {
            for z in chunk_z - 1..=chunk_z + 1 {
                for y in min_section..=max_section {
                    self.light_dirty.push((x, y, z));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: some queued light
    /// tasks, then the engine. Returns the sections to redraw.
    pub fn update_light(&mut self) -> Vec<light::SectionKey> {
        // `pollLightUpdates`: a tenth of the queue, at least ten, or all of
        // it once it reaches a thousand.
        let size = self.light_tasks.len();
        let budget = if size < 1000 {
            (size / 10).max(10)
        } else {
            size
        };
        for _ in 0..budget {
            let Some(task) = self.light_tasks.pop_front() else {
                break;
            };
            self.run_light_task(task);
        }
        let mut dirty = std::mem::take(&mut self.light_dirty);
        dirty.extend(self.run_light_updates());
        dirty
    }

    /// Every queued light task and the engine, as enough frames would.
    pub fn flush_light(&mut self) -> Vec<light::SectionKey> {
        let mut dirty = Vec::new();
        while !self.light_tasks.is_empty() || self.light.has_light_work() {
            dirty.extend(self.update_light());
        }
        dirty
    }

    /// `LevelLightEngine.runLightUpdates`: the sections whose light changed.
    pub fn run_light_updates(&mut self) -> Vec<light::SectionKey> {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light.run_light_updates(&view)
    }

    /// Light every loaded chunk from its own sources, as the server first
    /// lights a chunk: the sections holding blocks are marked, every column
    /// is switched on and the sources spread. Not what the client does with
    /// a server's light; it measures the engine on its own.
    pub fn light_from_sources(&mut self) {
        let chunks: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        for &(chunk_x, chunk_z) in &chunks {
            let filled: Vec<i32> = self.chunks[&(chunk_x, chunk_z)]
                .sections
                .iter()
                .filter(|(_, section)| !section.has_only_air())
                .map(|(y, _)| *y)
                .collect();
            for section_y in filled {
                self.light
                    .update_section_status((chunk_x, section_y, chunk_z), false);
            }
        }
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        for &chunk in &chunks {
            self.light.set_light_enabled(&view, chunk, true);
        }
        for &chunk in &chunks {
            self.light.propagate_light_sources(&view, chunk);
        }
    }

    fn run_light_task(&mut self, task: LightTask) {
        match task {
            LightTask::Data {
                update,
                schedule_rebuild,
                enable_chunk,
            } => {
                // `applyLightData`.
                let (chunk_x, chunk_z) = (update.chunk_x, update.chunk_z);
                let min = self.light.min_light_section();
                let count = self.light.light_section_count();
                for (layer, input) in [
                    (light::LightLayer::Sky, &update.sky),
                    (light::LightLayer::Block, &update.block),
                ] {
                    let mut updates = input.updates.iter();
                    for index in 0..count {
                        let bit = index as u64;
                        let have_data = input.mask.contains(bit);
                        if !have_data && !input.empty_mask.contains(bit) {
                            continue;
                        }
                        let data = if have_data {
                            light::DataLayer::from_bytes(updates.next().expect("counted"))
                        } else {
                            light::DataLayer::new(0)
                        };
                        let section_y = min + index;
                        self.light.queue_section_data(
                            layer,
                            (chunk_x, section_y, chunk_z),
                            Some(data),
                        );
                        if schedule_rebuild {
                            // `setSectionDirtyWithNeighbors`.
                            for dx in -1..=1 {
                                for dy in -1..=1 {
                                    for dz in -1..=1 {
                                        self.light_dirty.push((
                                            chunk_x + dx,
                                            section_y + dy,
                                            chunk_z + dz,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), true);
                if enable_chunk {
                    self.enable_chunk_light(chunk_x, chunk_z);
                }
            }
            LightTask::Removal { chunk_x, chunk_z } => {
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), false);
                let min = self.light.min_light_section();
                for section_y in min..min + self.light.light_section_count() {
                    for layer in [light::LightLayer::Block, light::LightLayer::Sky] {
                        self.light
                            .queue_section_data(layer, (chunk_x, section_y, chunk_z), None);
                    }
                }
                let min_section = self.dimension.min_section_y();
                for section_y in min_section..min_section + self.dimension.section_count() as i32 {
                    self.light
                        .update_section_status((chunk_x, section_y, chunk_z), true);
                }
            }
        }
    }

    /// Return the resolved global block-state ID at a world coordinate.
    /// `Level.getHeight(MOTION_BLOCKING, x, z)`: one above the highest
    /// state in the column that blocks motion or holds a fluid, or the
    /// bottom of the world where there is none or no chunk.
    pub fn motion_blocking_height(&self, x: i32, z: i32) -> i32 {
        let bottom = self.dimension().min_y;
        let Some(chunk) = self.chunk_for_query(x.div_euclid(16), z.div_euclid(16)) else {
            return bottom;
        };
        let column = z.rem_euclid(16) as usize * 16 + x.rem_euclid(16) as usize;
        for (section_y, section) in chunk.sections.iter().rev() {
            for local_y in (0..16).rev() {
                let Some(&state) = section.block_states.get(local_y * 256 + column) else {
                    continue;
                };
                if motion_blocking(state) {
                    return section_y * 16 + local_y as i32 + 1;
                }
            }
        }
        bottom
    }

    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunk_for_query(chunk_key.0, chunk_key.1)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        section.block_states.get(index).copied()
    }

    /// Collect loaded block collision shapes using Java's `BlockCollisions` traversal.
    ///
    /// The caller supplies the local collision shape for each resolved block-state ID.
    /// Missing chunks, missing shape entries, and empty shapes produce no collision.
    /// Block-state context rules, entity colliders, and world-border shapes are handled
    /// by later client-world slices.
    pub fn collect_block_collision_shapes(
        &self,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> Vec<ClientBlockCollision> {
        self.collect_block_collisions_with(bounding_box, |state_id| {
            shapes_by_state.get(&state_id).cloned()
        })
    }

    /// The collision walk, asking `shape_of` for each state it meets.
    fn collect_block_collisions_with(
        &self,
        bounding_box: Aabb,
        shape_of: impl Fn(i32) -> Option<VoxelShape>,
    ) -> Vec<ClientBlockCollision> {
        let entity_shape = Shapes::create_from_aabb(bounding_box)
            .expect("ClientWorld collision queries require normalized bounds");
        let x0 = voxel_core::math::floor_f64(bounding_box.min_x - Shapes::EPSILON) - 1;
        let x1 = voxel_core::math::floor_f64(bounding_box.max_x + Shapes::EPSILON) + 1;
        let y0 = voxel_core::math::floor_f64(bounding_box.min_y - Shapes::EPSILON) - 1;
        let y1 = voxel_core::math::floor_f64(bounding_box.max_y + Shapes::EPSILON) + 1;
        let z0 = voxel_core::math::floor_f64(bounding_box.min_z - Shapes::EPSILON) - 1;
        let z1 = voxel_core::math::floor_f64(bounding_box.max_z + Shapes::EPSILON) + 1;
        let mut collisions = Vec::new();

        // Cursor3D advances x fastest, then y, then z. Its corner cells are skipped.
        for z in z0..=z1 {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let cursor_face_type = (x == x0 || x == x1) as u8
                        + (y == y0 || y == y1) as u8
                        + (z == z0 || z == z1) as u8;
                    if cursor_face_type == 3 {
                        continue;
                    }

                    let Some(state_id) = self.block_state_at(x, y, z) else {
                        continue;
                    };
                    let Some(local_shape) = shape_of(state_id) else {
                        continue;
                    };
                    let position = BlockPos::new(x, y, z);
                    let shape = if local_shape.equals_java(&Shapes::block()) {
                        if !bounding_box.intersects_coords(
                            f64::from(x),
                            f64::from(y),
                            f64::from(z),
                            f64::from(x.wrapping_add(1)),
                            f64::from(y.wrapping_add(1)),
                            f64::from(z.wrapping_add(1)),
                        ) {
                            continue;
                        }
                        local_shape.move_by_block_pos(position)
                    } else {
                        let shape = local_shape.move_by_block_pos(position);
                        if shape.is_empty()
                            || !Shapes::join_is_not_empty(
                                &shape,
                                &entity_shape,
                                voxel_core::shape::BooleanOp::And,
                            )
                        {
                            continue;
                        }
                        shape
                    };
                    collisions.push(ClientBlockCollision { position, shape });
                }
            }
        }

        collisions
    }

    /// Collect loaded block collisions through the ClientWorld state registry.
    pub fn collect_registered_block_collision_shapes(
        &self,
        bounding_box: Aabb,
    ) -> Vec<ClientBlockCollision> {
        let registry = &self.block_state_registry;
        self.collect_block_collisions_with(bounding_box, |state_id| {
            Some(registry.collision_shape(state_id))
        })
    }

    /// Check whether an entity dimensions box fits in loaded block collisions.
    ///
    /// Java's player pose check deflates the candidate box by one shape epsilon
    /// before asking the level for block collisions. Entity and world-border
    /// collision sources remain caller-owned until their world slices land.
    pub fn entity_fits_in_blocks(
        &self,
        position: Vec3,
        dimensions: ClientEntityDimensions,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        let bounding_box = dimensions
            .make_bounding_box(position)
            .deflate(Shapes::EPSILON);
        self.collect_block_collision_shapes(bounding_box, shapes_by_state)
            .is_empty()
    }

    /// Return whether the query has no block, entity, or world-border collision.
    ///
    /// The order is the same short-circuit order as Java's `CollisionGetter`:
    /// blocks, then entities, then the border. The source position is optional
    /// because Java skips the border branch for a null source entity.
    pub fn no_collision(
        &self,
        source: Option<ClientWorldCollisionSource>,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
        candidates: &[ClientEntityCollisionCandidate],
    ) -> bool {
        self.no_block_collision(bounding_box, shapes_by_state)
            && self.no_entity_collision(source.is_some(), bounding_box, candidates)
            && self.no_border_collision(source, bounding_box)
    }

    /// Return whether the query has no loaded block collision.
    pub fn no_block_collision(
        &self,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        self.collect_block_collision_shapes(bounding_box, shapes_by_state)
            .is_empty()
    }

    /// Return whether the query has no entity collision.
    pub fn no_entity_collision(
        &self,
        source_present: bool,
        bounding_box: Aabb,
        candidates: &[ClientEntityCollisionCandidate],
    ) -> bool {
        collect_entity_collision_shapes(source_present, bounding_box, candidates).is_empty()
    }

    /// Return whether a query has no collision with the loaded entity state.
    ///
    /// The source ID is resolved against the same entity map that stores live
    /// positions and passenger links. The current type-backed collider table
    /// covers vanilla boat and minecart families and the Happy Ghast's
    /// source-aware override; entity types with Java's base non-collidable
    /// behavior produce no candidate until their dedicated type branch lands.
    pub fn no_entity_collision_for_entity(
        &self,
        source_entity_id: Option<i32>,
        bounding_box: Aabb,
    ) -> bool {
        let source_entity_id = source_entity_id.filter(|entity_id| self.entity_exists(*entity_id));
        let source_type_id = source_entity_id
            .and_then(|entity_id| self.entities.get(&entity_id).map(|entity| entity.type_id));
        let source_position =
            source_entity_id.and_then(|entity_id| self.entity_position(entity_id));
        let candidates = self.indexed_entity_collision_candidates(
            source_entity_id,
            source_type_id,
            source_position,
            bounding_box,
        );
        collect_entity_collision_shapes_for_source(
            source_type_id,
            source_entity_id.is_some(),
            bounding_box,
            &candidates,
        )
        .is_empty()
    }

    /// Return whether a query has no collision in the loaded world state.
    pub fn no_collision_for_entity(
        &self,
        source_entity_id: Option<i32>,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        let source_entity_id = source_entity_id.filter(|entity_id| self.entity_exists(*entity_id));
        let source = source_entity_id
            .and_then(|entity_id| self.entity_position(entity_id))
            .map(|position| ClientWorldCollisionSource { position });
        self.no_block_collision(bounding_box, shapes_by_state)
            && self.no_entity_collision_for_entity(source_entity_id, bounding_box)
            && self.no_border_collision(source, bounding_box)
    }

    fn add_entity_to_section(&mut self, entity_id: i32, position: (f64, f64, f64)) {
        self.entity_sections
            .entry(entity_section_key(position))
            .or_default()
            .insert(entity_id);
    }

    fn remove_entity_from_section(&mut self, entity_id: i32, position: (f64, f64, f64)) {
        let key = entity_section_key(position);
        let Some(entity_ids) = self.entity_sections.get_mut(&key) else {
            return;
        };
        entity_ids.remove(&entity_id);
        if entity_ids.is_empty() {
            self.entity_sections.remove(&key);
        }
    }

    fn update_entity_section(
        &mut self,
        entity_id: i32,
        previous_position: (f64, f64, f64),
        position: (f64, f64, f64),
    ) {
        let previous_key = entity_section_key(previous_position);
        let key = entity_section_key(position);
        if previous_key != key {
            self.remove_entity_from_section(entity_id, previous_position);
            self.entity_sections
                .entry(key)
                .or_default()
                .insert(entity_id);
        }
    }

    fn indexed_entity_collision_candidates(
        &self,
        source_entity_id: Option<i32>,
        source_type_id: Option<i32>,
        source_position: Option<Vec3>,
        bounding_box: Aabb,
    ) -> Vec<ClientEntityCollisionCandidate> {
        let mut entity_ids = BTreeSet::new();
        let x_min = entity_section_coord(bounding_box.min_x - 2.0);
        let y_min = entity_section_coord(bounding_box.min_y - 4.0);
        let z_min = entity_section_coord(bounding_box.min_z - 2.0);
        let x_max = entity_section_coord(bounding_box.max_x + 2.0);
        let y_max = entity_section_coord(bounding_box.max_y);
        let z_max = entity_section_coord(bounding_box.max_z + 2.0);
        for section_x in x_min..=x_max {
            for section_y in y_min..=y_max {
                for section_z in z_min..=z_max {
                    if let Some(ids) = self.entity_sections.get(&(section_x, section_y, section_z))
                    {
                        entity_ids.extend(ids.iter().copied());
                    }
                }
            }
        }
        entity_ids
            .into_iter()
            .filter_map(|entity_id| self.entities.get(&entity_id))
            .filter_map(|entity| {
                let same_vehicle = source_entity_id.is_some_and(|source_id| {
                    source_id != entity.id()
                        && self.root_vehicle_id(source_id) == self.root_vehicle_id(entity.id())
                });
                entity.collision_candidate(
                    source_entity_id,
                    source_type_id,
                    source_position,
                    same_vehicle,
                )
            })
            .collect()
    }

    fn root_vehicle_id(&self, entity_id: i32) -> i32 {
        let mut current = entity_id;
        let mut visited = BTreeSet::new();
        while visited.insert(current) {
            let Some(vehicle_id) = self.entity_vehicle_id(current) else {
                break;
            };
            current = vehicle_id;
        }
        current
    }

    fn entity_position(&self, entity_id: i32) -> Option<Vec3> {
        if let Some(entity) = self.entities.get(&entity_id) {
            let (x, y, z) = entity.position();
            return Some(Vec3::new(x, y, z));
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| {
                let (x, y, z) = player.position();
                Vec3::new(x, y, z)
            })
    }

    /// Return whether the query has no collision with the current world border.
    pub fn no_border_collision(
        &self,
        source: Option<ClientWorldCollisionSource>,
        bounding_box: Aabb,
    ) -> bool {
        let Some(source) = source else {
            return true;
        };

        let border = &self.world_border;
        let margin = abs_max_f64(bounding_box.x_size(), bounding_box.z_size()).max(1.0);
        let min_x = border.min_x(0.0);
        let max_x = border.max_x(0.0);
        let min_z = border.min_z(0.0);
        let max_z = border.max_z(0.0);
        let distance = (source.position.x - min_x)
            .min(max_x - source.position.x)
            .min(source.position.z - min_z)
            .min(max_z - source.position.z);
        let inside_close = distance < margin * 2.0
            && source.position.x >= min_x - margin
            && source.position.x < max_x + margin
            && source.position.z >= min_z - margin
            && source.position.z < max_z + margin;
        if !inside_close {
            return true;
        }

        let border_shape = border.collision_shape();
        let query_shape = Shapes::create_from_aabb(bounding_box)
            .expect("ClientWorld collision queries require normalized bounds");
        !Shapes::join_is_not_empty(&border_shape, &query_shape, BooleanOp::And)
    }

    /// Apply a server-verified block-state update to an already loaded section.
    pub fn update_block_state(&mut self, x: i32, y: i32, z: i32, state_id: i32) -> bool {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunk_for_update(chunk_key.0, chunk_key.1) else {
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
        let Some(previous_state_id) = section.block_states.get(index).copied() else {
            return false;
        };
        if (section.has_only_air() && state_id == VANILLA_AIR_BLOCK_STATE_ID)
            || previous_state_id == state_id
        {
            return false;
        }
        let was_empty = section.has_only_air();
        if section.set_block_state(index, state_id).is_none() {
            return false;
        }
        // `LevelChunk.setBlockState`: the light hears of a section turning
        // empty or not, and of a change in how light passes the block.
        let is_empty = section.has_only_air();
        if was_empty != is_empty {
            self.light
                .update_section_status((chunk_key.0, section_y, chunk_key.1), is_empty);
        }
        if light::has_different_light_properties(previous_state_id, state_id) {
            let chunk = self.chunks.get_mut(&chunk_key).expect("updated above");
            let sections = &chunk.sections;
            let read = |lx: i32, y: i32, lz: i32| {
                sections
                    .get(&y.div_euclid(16))
                    .and_then(|section| {
                        section
                            .block_states
                            .get((y.rem_euclid(16) * 256 + lz * 16 + lx) as usize)
                            .copied()
                    })
                    .unwrap_or(VANILLA_AIR_BLOCK_STATE_ID)
            };
            if let Some(sources) = chunk.sky_sources.as_mut() {
                sources.update(read, local_x as i32, y, local_z as i32);
            }
            self.light.check_block((x, y, z));
        }
        true
    }

    /// Apply the horizontal `StairBlock.updateShape` state transition against
    /// the currently loaded six-neighbor block states.
    pub fn apply_stair_neighbor_shape_update(&mut self, x: i32, y: i32, z: i32) -> bool {
        let Some(state_id) = self.block_state_at(x, y, z) else {
            return false;
        };
        let neighbors = [
            self.block_state_at(x, y - 1, z),
            self.block_state_at(x, y + 1, z),
            self.block_state_at(x, y, z - 1),
            self.block_state_at(x, y, z + 1),
            self.block_state_at(x - 1, y, z),
            self.block_state_at(x + 1, y, z),
        ];
        let Some(updated_state_id) = self
            .block_state_registry
            .stair_shape_update_state_id(state_id, &neighbors)
        else {
            return false;
        };
        if updated_state_id == state_id {
            return true;
        }
        self.update_block_state(x, y, z, updated_state_id)
    }

    pub fn apply_block_entities<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, entities: I) -> usize
    where
        I: IntoIterator<Item = BlockEntityInput<'a>>,
    {
        let Some(chunk) = self.chunk_for_update(chunk_x, chunk_z) else {
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

    pub fn apply_block_entity_update(&mut self, entity: BlockEntityInput<'_>) -> bool {
        let chunk_key = (entity.x.div_euclid(16), entity.z.div_euclid(16));
        let Some(chunk) = self.chunk_for_update(chunk_key.0, chunk_key.1) else {
            return false;
        };
        chunk.block_entities.insert(
            (entity.x, entity.y, entity.z),
            ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec)),
        );
        true
    }

    pub fn apply_entity(&mut self, entity: EntityInput) -> bool {
        if self.entities.contains_key(&entity.entity_id) {
            self.remove_entity(entity.entity_id);
        }
        let (chunk_x, chunk_z) = entity_chunk_key((entity.x, entity.y, entity.z));
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        let head_rotation_bits = entity.rotation.2.to_bits();
        self.entities.insert(
            entity.entity_id,
            ClientEntity {
                id: entity.entity_id,
                uuid: entity.uuid,
                type_id: entity.type_id,
                tick_count: 0,
                first_tick: true,
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
                y_body_rot_bits: entity.rotation.0.to_bits(),
                old_x_rot_bits: entity.rotation.1.to_bits(),
                old_y_rot_bits: entity.rotation.0.to_bits(),
                old_y_head_rot_bits: entity.rotation.2.to_bits(),
                old_y_body_rot_bits: entity.rotation.0.to_bits(),
                delta_rotation_bits: 0.0_f32.to_bits(),
                boat_input: BoatInput::EMPTY,
                paddle_left: false,
                paddle_right: false,
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
                baby: false,
                alive: true,
                spectator: false,
                on_climbable: false,
                flying: false,
                happy_ghast_still_timeout: false,
                happy_ghast_is_vehicle: false,
                ticks_frozen: 0,
                vehicle_id: None,
                passenger_ids: Vec::new(),
            },
        );
        self.add_entity_to_section(entity.entity_id, (entity.x, entity.y, entity.z));
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity.entity_id)
        {
            player.y_head_rot_bits = head_rotation_bits;
            player.old_y_head_rot_bits = head_rotation_bits;
            player.head_rotation_target_bits = head_rotation_bits;
            player.head_lerp_steps = 0;
            player.y_body_rot_bits = entity.rotation.0.to_bits();
            player.old_y_body_rot_bits = entity.rotation.0.to_bits();
        }
        true
    }

    /// Apply the collision-relevant Happy Ghast state used by the client query.
    pub fn apply_happy_ghast_collision_state(
        &mut self,
        entity_id: i32,
        baby: bool,
        alive: bool,
        still_timeout: bool,
        is_vehicle: bool,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id != VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID {
            return false;
        }
        entity.baby = baby;
        entity.alive = alive;
        entity.happy_ghast_still_timeout = still_timeout;
        entity.happy_ghast_is_vehicle = is_vehicle;
        true
    }

    /// Apply the collision-relevant alive and spectator state of a living entity.
    pub fn apply_entity_living_collision_state(
        &mut self,
        entity_id: i32,
        alive: bool,
        spectator: bool,
        on_climbable: bool,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.alive = alive;
        entity.spectator = spectator;
        entity.on_climbable = on_climbable;
        true
    }

    /// Apply the trapdoor and ladder block context used by LivingEntity.onClimbable.
    pub fn apply_entity_living_climbable_trapdoor_state(
        &mut self,
        entity_id: i32,
        trapdoor_open: bool,
        trapdoor_facing: &str,
        below_is_ladder: bool,
        ladder_facing: &str,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = trapdoor_open && below_is_ladder && trapdoor_facing == ladder_facing;
        true
    }

    /// Apply a pinned block-state's BlockTags.CLIMBABLE membership to a living entity.
    pub fn apply_entity_living_climbable_block_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
    ) -> bool {
        let on_climbable = self.block_state_registry.is_climbable_state(state_id);
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        true
    }

    /// Apply the LivingEntity.onClimbable tag and fall-flying context.
    pub fn apply_entity_living_climbable_glide_through_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
        fall_flying: bool,
    ) -> bool {
        let is_climbable = self.block_state_registry.is_climbable_state(state_id);
        let is_glide_through = self.block_state_registry.is_glide_through_state(state_id);
        let on_climbable = if fall_flying && is_glide_through {
            false
        } else {
            is_climbable
        };
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        if fall_flying {
            entity.shared_flags |= 1 << SHARED_FLAG_FALL_FLYING;
        } else {
            entity.shared_flags &= !(1 << SHARED_FLAG_FALL_FLYING);
        }
        true
    }

    /// Apply the Player ability state used by Java's `Player.onClimbable` gate.
    pub fn apply_entity_player_flying_state(&mut self, entity_id: i32, flying: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id != VANILLA_PLAYER_ENTITY_TYPE_ID {
            return false;
        }
        entity.flying = flying;
        true
    }

    /// Apply a clientbound vehicle and ordered passenger relationship.
    pub fn apply_entity_passengers(&mut self, vehicle_id: i32, passenger_ids: &[i32]) -> bool {
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

    pub fn entity_vehicle_id(&self, entity_id: i32) -> Option<i32> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return entity.vehicle_id;
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .and_then(|player| player.vehicle_id)
    }

    pub fn entity_passengers(&self, entity_id: i32) -> Option<&[i32]> {
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

    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "one step per metadata field the client applies"
    )]
    pub fn apply_entity_metadata(&mut self, entity_id: i32, metadata: &[u8]) -> bool {
        let shared_flags = shared_flags_from_metadata(metadata);
        let living_flags = living_flags_from_metadata(metadata);
        let mob_flags = mob_flags_from_metadata(metadata);
        let air_supply = air_supply_from_metadata(metadata);
        let health = health_from_metadata(metadata);
        let arrow_count = arrow_count_from_metadata(metadata);
        let stinger_count = stinger_count_from_metadata(metadata);
        let effect_ambience = effect_ambience_from_metadata(metadata);
        let effect_particle_type_ids = effect_particle_type_ids_from_metadata(metadata);
        let zombie_baby = zombie_baby_from_metadata(metadata);
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
            if let Some(absorption) = player_absorption_from_metadata(metadata) {
                player.absorption_bits = absorption.to_bits();
            }
            if let Some(customisation) = player_model_customisation_from_metadata(metadata) {
                player.model_customisation = customisation;
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
        if entity.type_id == VANILLA_ZOMBIE_ENTITY_TYPE_ID
            && let Some(zombie_baby) = zombie_baby
        {
            entity.baby = zombie_baby;
        }
        if let Some(ticks_frozen) = ticks_frozen {
            entity.ticks_frozen = ticks_frozen;
        }
        true
    }

    pub fn apply_entity_motion(&mut self, input: EntityMotionInput) -> bool {
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

    pub fn apply_entity_ground_state(&mut self, entity_id: i32, on_ground: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_ground = on_ground;
        true
    }

    pub fn apply_entity_rotation(&mut self, input: EntityRotationInput) -> bool {
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
        }
        if let Some(y_head_rot) = input.y_head_rot
            && !self.apply_entity_head_rotation(input.entity_id, y_head_rot)
        {
            return false;
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

    pub fn apply_entity_position_and_rotation(
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

    pub fn apply_entity_teleport(&mut self, input: EntityTeleportInput) -> bool {
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

    pub fn apply_entity_move(&mut self, input: EntityMoveInput) -> bool {
        let Some((previous_position, current)) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| (entity.position(), entity.interpolation_position()))
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
        let position;
        {
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
                    entity.ticking =
                        entity.always_ticking || ticking.expect("position ticking state");
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
            position = entity.position();
        }
        self.update_entity_section(input.entity_id, previous_position, position);
        true
    }

    /// Apply Java's absolute entity position-sync path for a remote entity.
    pub fn apply_entity_position_sync(
        &mut self,
        entity_id: i32,
        position: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id,
                x: position.0,
                y: position.1,
                z: position.2,
                relative_flags: 0,
            },
            EntityRotationInput {
                entity_id,
                y_rot: Some(rotation.0),
                x_rot: Some(rotation.1),
                y_head_rot: None,
                relative_y_rot: false,
                relative_x_rot: false,
            },
            EntityPositionUpdateMode::PositionSync,
        )
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
        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let position;
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                let rotation = target_rotation.unwrap_or_else(|| entity.interpolation_rotation());
                let distance_position = match mode {
                    EntityPositionUpdateMode::PositionSync => entity.position(),
                    EntityPositionUpdateMode::Teleport => current_position,
                };
                let delta_x = target_position.0 - distance_position.0;
                let delta_y = target_position.1 - distance_position.1;
                let delta_z = target_position.2 - distance_position.2;
                let can_interpolate =
                    match mode {
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
            position = entity.position();
        }
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunk_for_query(chunk_x, chunk_z)?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// `getNoiseBiome`: the biome at a quart position, its row clamped into
    /// the dimension.
    pub fn noise_biome(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        self.biome_at(x * 4, self.dimension.clamp_quart_y(y) * 4, z * 4)
    }

    /// Return the resolved biome registry ID at a world coordinate.
    pub fn biome_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunk_for_query(chunk_key.0, chunk_key.1)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize / 4;
        let local_y = y.rem_euclid(16) as usize / 4;
        let local_z = z.rem_euclid(16) as usize / 4;
        let index = local_y * 16 + local_z * 4 + local_x;
        section.biomes.get(index).copied()
    }

    fn light_at(&self, layer: LightLayer, x: i32, y: i32, z: i32) -> Option<u8> {
        let layer = match layer {
            LightLayer::Sky => light::LightLayer::Sky,
            LightLayer::Block => light::LightLayer::Block,
        };
        self.light
            .data_layer(
                layer,
                (x.div_euclid(16), y.div_euclid(16), z.div_euclid(16)),
            )
            .map(|data| data.get(x.rem_euclid(16), y.rem_euclid(16), z.rem_euclid(16)))
    }

    /// Replace a loaded chunk with decoded sections in bottom-to-top order.
    pub fn replace_chunk<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, sections: I) -> bool
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
            .collect::<BTreeMap<_, _>>();
        // `initializeLightSources`.
        let mut sky_sources = light::ChunkSkyLightSources::new(self.dimension.min_y);
        let min_section = self.dimension.min_section_y();
        sky_sources.fill_from(self.dimension.section_count(), |index| {
            sections
                .get(&(min_section + index as i32))
                .filter(|section| !section.has_only_air())
                .map(|section| section.block_states.clone())
        });
        let new_chunk = ClientChunk {
            sections,
            sky_sources: Some(sky_sources),
            block_entities: BTreeMap::new(),
        };
        let new_section_statuses = new_chunk
            .sections
            .iter()
            .map(|(&section_y, section)| (section_y, section.has_only_air()))
            .collect::<Vec<_>>();

        if let Some(cache) = self.chunk_cache.as_ref() {
            if !cache.in_range(chunk_x, chunk_z) {
                return false;
            }
            let index = cache.index(chunk_x, chunk_z);
            let current_chunk = cache.slots.get(&index).copied();
            if current_chunk == Some((chunk_x, chunk_z)) {
                self.chunks.insert((chunk_x, chunk_z), new_chunk);
                self.chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during refresh")
                    .refresh_empty_sections(chunk_x, chunk_z, new_section_statuses);
            } else {
                if let Some((old_x, old_z)) = current_chunk
                    && let Some(old_chunk) = self.chunks.remove(&(old_x, old_z))
                {
                    let old_section_ys = old_chunk.sections.keys().copied().collect::<Vec<_>>();
                    self.chunk_cache
                        .as_mut()
                        .expect("chunk cache exists during replacement")
                        .on_chunk_removed(old_x, old_z, old_section_ys);
                    // `Storage.replace`: the chunk pushed out is unloaded.
                    self.unload_chunk_light(old_x, old_z);
                }
                self.chunks.insert((chunk_x, chunk_z), new_chunk);
                let cache = self
                    .chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during insertion");
                cache.slots.insert(index, (chunk_x, chunk_z));
                cache.on_chunk_added(chunk_x, chunk_z, new_section_statuses);
            }
        } else {
            self.chunks.insert((chunk_x, chunk_z), new_chunk);
        }
        for entity in self.entities.values_mut() {
            if entity_chunk_key(entity.position()) == (chunk_x, chunk_z) {
                entity.ticking = true;
            }
        }
        true
    }
}

pub mod input;
pub mod local_player;

#[cfg(test)]
mod tests;

/// Whether the `MOTION_BLOCKING` heightmap counts a state: it blocks
/// motion or holds a fluid.
pub fn motion_blocking(state: i32) -> bool {
    let state = block::state_or_air(i64::from(state));
    state.blocks_motion || !state.fluid.is_empty()
}

/// A light task waiting for a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LightTask {
    Data {
        update: PendingLightUpdate,
        schedule_rebuild: bool,
        enable_chunk: bool,
    },
    Removal {
        chunk_x: i32,
        chunk_z: i32,
    },
}

fn new_light_engine(dimension: DimensionBounds, has_sky_light: bool) -> light::LevelLightEngine {
    light::LevelLightEngine::new(
        dimension.min_y,
        dimension.section_count() as i32,
        has_sky_light,
    )
}

/// The level as the light engine reads it: `getChunkForLighting` finds only
/// a chunk the cache holds.
struct LightView<'a> {
    chunks: &'a BTreeMap<(i32, i32), ClientChunk>,
    cache: Option<&'a ClientChunkCacheState>,
}

impl LightView<'_> {
    fn chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        if let Some(cache) = self.cache
            && !(cache.in_range(chunk_x, chunk_z)
                && cache.slots.get(&cache.index(chunk_x, chunk_z)) == Some(&(chunk_x, chunk_z)))
        {
            return None;
        }
        self.chunks.get(&(chunk_x, chunk_z))
    }
}

impl light::LightChunks for LightView<'_> {
    fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk(chunk_x, chunk_z).is_some()
    }

    fn block_state(&self, x: i32, y: i32, z: i32) -> i32 {
        self.chunk(x >> 4, z >> 4)
            .and_then(|chunk| chunk.sections.get(&(y >> 4)))
            .and_then(|section| {
                section
                    .block_states
                    .get(((y & 15) * 256 + (z & 15) * 16 + (x & 15)) as usize)
                    .copied()
            })
            .unwrap_or(0)
    }

    fn sky_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<&light::ChunkSkyLightSources> {
        self.chunk(chunk_x, chunk_z)?.sky_sources.as_ref()
    }
}
