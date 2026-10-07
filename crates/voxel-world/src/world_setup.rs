use super::*;

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
            block_state_registry: ClientBlockStateRegistry::default(),
            recipe_property_sets: BTreeMap::new(),
            minecart_improvements: false,
            stonecutter: Arc::default(),
            recipe_book: RecipeBook::default(),
            chunks: BTreeMap::new(),
            chunk_cache: None,
            light: new_light_engine(dimension, true),
            light_tasks: VecDeque::new(),
            light_dirty: Vec::new(),
            entities: BTreeMap::new(),
            maps: BTreeMap::new(),
            entity_sections: BTreeMap::new(),
            local_player: None,
            sounds: local_player::SoundQueue::default(),
            world_border: ClientWorldBorder::default(),
            block_animations: chest::BlockAnimations::default(),
            server_simulation_distance: 3,
            local_chunk: None,
            game_time: 0,
            paddle_packets: Vec::new(),
            rider_controls: mount_ride::Controller::default(),
            sky_flash_time: 0,
            rain_level_bits: 0,
            thunder_level_bits: 0,
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

    /// What the local player's swing and use of items read of the player state: what the main hand
    /// (the selected hotbar slot) and the off hand hold, the effects that change a swing, and the
    /// server's use flags.
    pub fn local_hand_state(&self, selected_slot: i32) -> local_player::HandState {
        const HASTE: i32 = 2;
        const MINING_FATIGUE: i32 = 3;
        const CONDUIT_POWER: i32 = 28;
        let Some(state) = self.local_player() else {
            return local_player::HandState::default();
        };
        let item = |slot: i32| {
            state
                .inventory_slot(slot as i16)
                .map_or(0, |stack| stack.item_registry_id())
        };
        let amplifier = |id: i32| {
            state
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == id)
                .map(|effect| effect.amplifier())
        };
        local_player::HandState {
            main_item: item(36 + selected_slot),
            off_item: item(45),
            dig_speed: match (amplifier(HASTE), amplifier(CONDUIT_POWER)) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0).max(b.unwrap_or(0))),
            },
            mining_fatigue: amplifier(MINING_FATIGUE),
            using_item: state.is_using_item(),
            using_off_hand: state.used_item_is_off_hand(),
        }
    }

    /// Create or replace the local player state for a clientbound player ID.
    pub fn set_local_player_id(&mut self, player_id: i32) {
        let mut player = ClientPlayerState::new(player_id);
        if let Some(entity) = self.entities.get(&player_id) {
            player.y_head_rot = entity.y_head_rot;
            player.old_y_head_rot = entity.old_y_head_rot;
            player.head_rotation_target = entity.y_head_rot;
            player.y_body_rot = entity.y_body_rot;
            player.old_y_body_rot = entity.old_y_body_rot;
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
        player.position.x = JavaF64::new(position.0);
        player.position.y = JavaF64::new(position.1);
        player.position.z = JavaF64::new(position.2);
        player.motion.x = JavaF64::new(motion.0);
        player.motion.y = JavaF64::new(motion.1);
        player.motion.z = JavaF64::new(motion.2);
        player.y_rot = JavaF32::new(rotation.0);
        player.x_rot = JavaF32::new(rotation.1);
        if initialize_transform_history {
            player.old_position.x = player.position.x;
            player.old_position.y = player.position.y;
            player.old_position.z = player.position.z;
            player.old_y_rot = player.y_rot;
            player.old_x_rot = player.x_rot;
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
        player.mirror_container_into_inventory();
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

    /// The menu of the container the server opened: its id and type, every
    /// slot, the carried stack and the state id. `None` with no container
    /// open or before its contents have come.
    pub fn local_player_container_menu(&self) -> Option<OpenContainerMenu> {
        let player = self.local_player.as_ref()?;
        let container = player.open_container.as_ref()?;
        if container.slot_count == 0 {
            return None;
        }
        let mut slots: Vec<Option<ClientItemStackState>> = (0..container.slot_count)
            .map(|slot| container.slots.get(&(slot as i16)).cloned().flatten())
            .collect();
        // The last 36 slots are the player's inventory, which the server can
        // also change without the container's knowing.
        if let Some(top) = container.top_slot_count {
            for index in 0..36 {
                let from = if index < 27 {
                    9 + index
                } else {
                    36 + index - 27
                };
                if let (Some(slot), Some(stack)) = (
                    slots.get_mut(top + index),
                    player.inventory_slots.get(&(from as i16)),
                ) {
                    *slot = stack.clone();
                }
            }
        }
        Some(OpenContainerMenu {
            container_id: container.container_id,
            menu_type_id: container.menu_type_id,
            slots,
            carried: container.carried_item.clone(),
            state_id: container.state_id,
        })
    }

    /// Write back the open container's menu after a predicted click.
    pub fn set_local_player_container_menu(
        &mut self,
        slots: &[Option<ClientItemStackState>],
        carried: Option<ClientItemStackState>,
    ) {
        let Some(player) = self.local_player.as_mut() else {
            return;
        };
        let Some(container) = player.open_container.as_mut() else {
            return;
        };
        for (slot, stack) in slots.iter().enumerate() {
            container.slots.insert(slot as i16, stack.clone());
        }
        container.carried_item = carried;
        player.mirror_container_into_inventory();
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
        player.mirror_container_into_inventory();
        true
    }

    /// `handleContainerSetData`: a value of the open menu's `ContainerData`.
    pub fn apply_local_player_container_data(
        &mut self,
        container_id: i32,
        index: i16,
        value: i16,
    ) -> bool {
        let Some(container) = self
            .local_player
            .as_mut()
            .and_then(|player| player.open_container.as_mut())
            .filter(|container| container.container_id == container_id)
        else {
            return false;
        };
        container.data.insert(index, value);
        true
    }

    /// `handleUpdateRecipes`: the items each recipe property set accepts, by
    /// the set's name (`minecraft:furnace_input`).
    pub fn apply_recipe_property_sets(&mut self, sets: Vec<(String, Vec<i32>)>) {
        self.recipe_property_sets = sets
            .into_iter()
            .map(|(name, items)| (name, Arc::new(items.into_iter().collect())))
            .collect();
    }

    pub fn recipe_book(&self) -> &RecipeBook {
        &self.recipe_book
    }

    pub fn recipe_book_mut(&mut self) -> &mut RecipeBook {
        &mut self.recipe_book
    }

    /// Whether an item is in an item tag (`ItemStack.is(TagKey)`).
    pub fn item_in_tag(&self, item: i32, tag: &str) -> bool {
        self.block_state_registry.item_tags().contains(item, tag)
    }

    /// `handleUpdateRecipes`'s stonecutter half.
    pub fn apply_stonecutter_recipes(&mut self, entries: Vec<StonecutterEntry>) {
        self.stonecutter = Arc::new(entries);
    }

    /// `SingleInputSet.acceptsInput`, as the items it accepts: every entry's
    /// items, and the members of the tags it names.
    pub fn stonecutter_inputs(&self) -> BTreeSet<i32> {
        let tags = self.block_state_registry.item_tags();
        let mut accepted = BTreeSet::new();
        for entry in self.stonecutter.iter() {
            match &entry.input {
                Ingredient::Items(items) => accepted.extend(items.iter().copied()),
                Ingredient::Tag(tag) => accepted.extend(tags.members(tag)),
            }
        }
        accepted
    }

    /// `SingleInputSet.selectByInput`: what each entry that takes the item
    /// shows, in the server's order (the first item of each display).
    pub fn stonecutter_options(&self, item: i32) -> Vec<i32> {
        let tags = self.block_state_registry.item_tags();
        self.stonecutter
            .iter()
            .filter(|entry| match &entry.input {
                Ingredient::Items(items) => items.contains(&item),
                Ingredient::Tag(tag) => tags.contains(item, tag),
            })
            .map(|entry| entry.display.first_item().unwrap_or(-1))
            .collect()
    }

    /// The items a recipe property set accepts (`RecipePropertySet.test`);
    /// none for a set the server has not sent.
    pub fn recipe_property_set(&self, set: &str) -> Arc<BTreeSet<i32>> {
        self.recipe_property_sets
            .get(set)
            .cloned()
            .unwrap_or_default()
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
}
