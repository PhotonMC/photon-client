use super::*;

impl ClientWorld {
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
}
