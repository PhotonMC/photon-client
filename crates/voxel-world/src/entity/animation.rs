use super::*;

impl ClientEntity {
    /// A chicken's `flap` and `flapSpeed` between two ticks.
    pub fn chicken_flap(&self, partial: f32) -> (f32, f32) {
        let animal = &self.living.animal;
        (animal.flap_at(partial), animal.flap_speed_at(partial))
    }

    /// An armor stand's flags and poses.
    pub fn armor_stand(&self) -> &crate::armor_stand::ArmorStandState {
        &self.living.armor_stand
    }

    /// The swing of an arm (`LivingEntity.swing`).
    pub fn swing_state(&self) -> &crate::living::SwingState {
        &self.living.swing
    }

    /// What a living entity's swing does each tick: the last tick's attack
    /// animation is kept, and the swing runs on for a player and a monster
    /// (`Player.aiStep`, `Monster.aiStep`).
    pub(crate) fn tick_swing(&mut self, kind: LivingKind) {
        self.living.swing.begin_tick();
        let monster = voxel_data::entity_type::pick(self.type_id).is_some_and(|pick| pick.monster);
        if kind == LivingKind::Player || monster {
            let durations = self.swing_durations();
            self.living.swing.update(durations);
        }
    }

    /// `ItemStack.getSwingAnimation().duration()` of what each hand holds, the main hand's first.
    pub(crate) fn swing_durations(&self) -> [i32; 2] {
        [EquipmentSlot::MainHand, EquipmentSlot::OffHand].map(|slot| {
            self.equipment(slot)
                .and_then(|stack| crate::block_rules::item(stack.item_registry_id()))
                .and_then(|item| voxel_data::spear::spear_of(item.name))
                .map_or(crate::living::DEFAULT_SWING_DURATION, |spear| {
                    spear.swing_ticks
                })
        })
    }

    /// What a living entity does each tick besides moving: hurt and death,
    /// its pose, its swing, and what its kind does of its own.
    pub(crate) fn tick_living(&mut self, kind: LivingKind, tick_death: bool, game_time: u64) {
        self.living.using_item_ticks = if self.is_using_item() {
            self.living.using_item_ticks.saturating_add(1)
        } else {
            0
        };
        let (mx, my, mz) = self.motion();
        self.living.previous_motion = [mx.to_bits(), my.to_bits(), mz.to_bits()];
        self.living.fall_fly_ticks = if self.is_fall_flying() {
            self.living.fall_fly_ticks.saturating_add(1)
        } else {
            0
        };
        self.living.elytra.tick(crate::elytra::ElytraInput {
            fall_flying: self.is_fall_flying(),
            motion: self.motion(),
            crouching: self.is_crouching(),
        });
        self.tick_hurt_and_death(tick_death);
        self.tick_living_pose(kind);
        self.tick_swing(kind);
        self.tick_animal(game_time);
    }

    /// What a sheep or a chicken does of its own each tick (`aiStep`).
    pub(crate) fn tick_animal(&mut self, game_time: u64) {
        match voxel_data::entity_type::name(self.type_id) {
            Some("minecraft:sheep") => self.living.animal.tick_sheep(),
            Some("minecraft:chicken") => self.living.animal.tick_chicken(self.on_ground),
            Some("minecraft:creeper") => self.living.animal.tick_creeper(self.alive),
            Some("minecraft:iron_golem") => self.living.animal.tick_iron_golem(),
            Some("minecraft:goat") => self.living.animal.tick_goat(),
            Some("minecraft:hoglin" | "minecraft:zoglin") => self.living.animal.tick_hoglin(),
            Some("minecraft:ravager") => {
                let alive = self.alive;
                self.kind
                    .ravager_or_insert_with(Default::default)
                    .tick(alive);
            }
            Some("minecraft:bee") => {
                let rolling = self.living.bee_flags & 2 != 0;
                self.living.animal.tick_bee(rolling);
            }
            Some(
                "minecraft:horse"
                | "minecraft:donkey"
                | "minecraft:mule"
                | "minecraft:skeleton_horse"
                | "minecraft:zombie_horse"
                | "minecraft:llama"
                | "minecraft:trader_llama"
                | "minecraft:camel"
                | "minecraft:camel_husk",
            ) => {
                let (alive, camel) = (
                    self.alive,
                    matches!(
                        voxel_data::entity_type::name(self.type_id),
                        Some("minecraft:camel" | "minecraft:camel_husk")
                    ),
                );
                let (tick_count, on_ground, passenger) =
                    (self.tick_count, self.on_ground, self.vehicle_id.is_some());
                let horse = self
                    .kind
                    .horse_or_insert_with(|| Box::new(crate::horse::HorseState::new()));
                horse.tick(alive);
                if camel {
                    let in_liquid = self.mount_ride.in_liquid;
                    horse.tick_camel(
                        (tick_count, game_time as i64),
                        (on_ground, in_liquid, passenger),
                    );
                }
            }
            Some("minecraft:rabbit") => {
                let (tick_count, no_ai) = (self.tick_count, self.is_no_ai());
                self.kind
                    .rabbit_or_insert_with(|| Box::new(crate::rabbit::RabbitState::new()))
                    .tick(tick_count, no_ai);
            }
            Some("minecraft:copper_golem") => {
                let (action, tick_count, flags) = (
                    self.living.copper_golem_action,
                    self.tick_count,
                    (self.is_no_ai(), self.alive),
                );
                self.kind
                    .copper_golem_or_insert_with(|| {
                        Box::new(crate::copper_golem::CopperGolemAnimations::new())
                    })
                    .tick(action, tick_count, flags);
            }
            Some("minecraft:polar_bear") => self.living.polar_bear.tick(),
            Some("minecraft:fox") => self.living.fox.tick(),
            Some("minecraft:wolf") => {
                let (alive, interested) = (self.alive, self.living.wolf_interested);
                self.living.wolf.tick(alive, interested);
            }
            _ => {}
        }
    }

    /// `getItemHeldByArm`.
    pub fn item_held_by_arm(&self, left: bool) -> Option<&ClientItemStackState> {
        self.equipment(if left == self.main_arm_is_left() {
            EquipmentSlot::MainHand
        } else {
            EquipmentSlot::OffHand
        })
    }

    /// `ItemEntity.bobOffs`.
    pub fn bob_offset(&self) -> f32 {
        self.bob_offset.get()
    }

    /// Whether the entity is a player.
    pub fn is_player(&self) -> bool {
        is_vanilla_player_entity_type(self.type_id)
    }

    /// Whether the player only watches (its game mode, as the server told).
    pub fn is_spectator(&self) -> bool {
        self.spectator
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
