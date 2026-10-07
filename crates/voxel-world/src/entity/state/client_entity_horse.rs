use super::*;

impl ClientEntity {
    /// `LivingEntity.getTicksSinceLastKineticHitFeedback`.
    pub fn ticks_since_kinetic_hit(&self, game_time: u64, partial: f32) -> f32 {
        self.living
            .kinetic_hit_time
            .map_or(0.0, |last| game_time.saturating_sub(last) as f32 + partial)
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

    /// `LivingEntity.isDeadOrDying`: its health, as the server told, is spent.
    pub fn is_dead_or_dying(&self) -> bool {
        self.health().is_some_and(|health| health <= 0.0)
    }

    /// The registry id of the variant (`PigVariant`, `CowVariant`) the server
    /// set, or none while it wears the default.
    pub fn variant(&self) -> Option<i32> {
        self.living.variant
    }

    /// `Piglin.isChargingCrossbow`.
    pub fn is_charging_crossbow(&self) -> bool {
        self.living.charging_crossbow
    }

    /// Whether a piglin dances (`Piglin.isDancing`).
    pub fn is_dancing(&self) -> bool {
        self.living.dancing
    }

    /// An experience orb's value (`ExperienceOrb.getValue`), once told.
    pub fn orb_value(&self) -> Option<i32> {
        self.living.orb_value
    }

    pub fn hurt_time(&self) -> i32 {
        self.living.hurt_time
    }

    pub fn death_time(&self) -> i32 {
        self.living.death_time
    }

    /// `LivingEntity.baseTick`'s share: `hurtTime` counts down, and a dying
    /// entity's `deathTime` up (`tickDeath`; the server removes it later) while
    /// the level says deaths tick here (`shouldTickDeath`).
    pub(crate) fn tick_hurt_and_death(&mut self, tick_death: bool) {
        if self.living.hurt_time > 0 {
            self.living.hurt_time -= 1;
        }
        if self.is_dead_or_dying() && tick_death {
            if self.is_ender_dragon() {
                // `EnderDragon.tickDeath` keeps its own count, not `deathTime`, and
                // lifts the dragon and its parts a tenth of a block.
                let lift = f64::from(0.1_f32);
                self.ender_dragon_mut().tick_death(lift);
                self.position.y = JavaF64::new(self.position.y.get() + lift);
            } else {
                self.living.death_time += 1;
            }
        }
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

    pub fn sleeping_pos(&self) -> Option<voxel_core::binary::BlockPos> {
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

    /// Return player/Avatar dimensions for a remote player entity.
    pub fn player_dimensions(&self) -> Option<ClientEntityDimensions> {
        (self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID).then(|| self.pose.player_dimensions())
    }

    pub fn is_baby(&self) -> bool {
        self.baby
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

    /// The stack a dropped item holds.
    pub fn item_stack(&self) -> Option<&ClientItemStackState> {
        self.item_stack.as_ref()
    }

    /// What the entity holds or wears in a slot (`getItemBySlot`); none for
    /// an empty one.
    pub fn equipment(&self, slot: EquipmentSlot) -> Option<&ClientItemStackState> {
        self.equipment[slot as usize].as_ref()
    }

    /// `getMainArm`: a mob's left-handed flag, an avatar's data.
    pub fn main_arm_is_left(&self) -> bool {
        if self.is_player() {
            self.living.main_arm_left
        } else {
            self.is_left_handed()
        }
    }

    /// `Villager.getVillagerData`: the registry ids of its type and profession
    /// and its level; a plains villager without a profession, at level one,
    /// until the server says otherwise (`createDefaultVillagerData`).
    pub fn villager_data(&self) -> [i32; 3] {
        self.living.villager_data.unwrap_or([2, 0, 1])
    }

    /// Whether a villager or a wandering trader shakes its head
    /// (`getUnhappyCounter() > 0`).
    pub fn is_unhappy(&self) -> bool {
        self.living.unhappy_counter > 0
    }

    /// Whether an enderman carries a block (`getCarriedBlock() != null`).
    pub fn is_carrying_block(&self) -> bool {
        self.living.carried_state != 0
    }

    /// The block state an enderman carries (`getCarriedBlock`), by its id.
    pub fn carried_block_state(&self) -> Option<u32> {
        u32::try_from(self.living.carried_state)
            .ok()
            .filter(|state| *state != 0)
    }

    /// A horse's animations and tail, if it has ticked.
    pub fn horse(&self) -> Option<&crate::horse::HorseState> {
        self.kind.horse()
    }

    /// `Horse.getVariant` and `getMarkings`: ids, each wrapped into its enum
    /// (seven variants, five markings).
    pub fn horse_look(&self) -> (i32, i32) {
        let data = self.living.horse_variant;
        (
            (data & 0xFF).rem_euclid(7),
            ((data & 0xFF00) >> 8).rem_euclid(5),
        )
    }

    /// `Llama.getVariant`'s id, kept within the four variants (an id past the
    /// last takes the last).
    pub fn llama_variant(&self) -> i32 {
        self.living.horse_variant.clamp(0, 3)
    }

    /// `AbstractChestedHorse.hasChest`.
    pub fn horse_has_chest(&self) -> bool {
        self.living.horse_chest
    }

    /// Fixes the source a horse's tail chance is drawn from (see
    /// [`crate::horse::HorseState::fix_seed`]).
    pub fn fix_horse_seed(&mut self, seed: i64) {
        self.kind
            .horse_or_insert_with(|| Box::new(crate::horse::HorseState::new()))
            .fix_seed(seed);
    }

    /// A rabbit's hop and animation states, if it has ticked.
    pub fn rabbit(&self) -> Option<&crate::rabbit::RabbitState> {
        self.kind.rabbit()
    }

    /// Fixes the idle head tilt's wait and random source of a rabbit (see
    /// [`crate::rabbit::RabbitState::fix_idle`]).
    pub fn fix_rabbit_idle(&mut self, timeout: i32, seed: i64) {
        self.kind
            .rabbit_or_insert_with(|| Box::new(crate::rabbit::RabbitState::new()))
            .fix_idle(timeout, seed);
    }

    /// A squid's tentacles and body turns, if it has ticked.
    pub fn squid(&self) -> Option<&crate::squid::SquidState> {
        self.kind.squid()
    }

    pub(crate) fn is_squid(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:squid" | "minecraft:glow_squid")
        )
    }

    /// Whether the entity's water state is tracked (`Entity.isInWater`): the
    /// mobs that draw differently in water.
    pub(crate) fn tracks_water(&self) -> bool {
        self.is_squid()
            || self.is_fish()
            || matches!(
                voxel_data::entity_type::name(self.type_id),
                Some(
                    "minecraft:turtle"
                        | "minecraft:axolotl"
                        | "minecraft:frog"
                        | "minecraft:tadpole"
                        | "minecraft:guardian"
                        | "minecraft:elder_guardian"
                )
            )
    }

    /// `Bat.isResting`.
    pub fn is_bat_resting(&self) -> bool {
        self.living.bat_resting
    }

    /// `Vex.isCharging`.
    pub fn is_vex_charging(&self) -> bool {
        self.living.vex_charging
    }

    /// `Allay.isDancing`.
    pub fn is_allay_dancing(&self) -> bool {
        self.living.allay_dancing
    }

    /// What a boat the local player rides keeps between ticks.
    pub fn boat_ride(&self) -> crate::boat_ride::RideState {
        self.boat_ride
    }

    /// What a saddled horse the local player rides keeps between ticks.
    pub fn mount_ride(&self) -> crate::mount_ride::MountState {
        self.mount_ride
    }

    /// Whether this is a camel or a camel husk.
    pub fn is_camel(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:camel" | "minecraft:camel_husk")
        )
    }

    /// Whether this is a horse, donkey, mule or undead horse, saddled or not.
    pub fn is_horse(&self) -> bool {
        crate::mount_ride::is_horse_type(self.type_id)
    }

    /// Whether this is a pig.
    pub fn is_pig(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:pig")
    }

    /// `Strider.setSuffocating`: the flag, and the cold's modifier on the movement speed (a third
    /// less of the base).
    pub(crate) fn set_strider_suffocating(&mut self, suffocating: bool) {
        self.living.strider_suffocating = suffocating;
        let Some((id, _)) = voxel_data::entity_type::attribute("minecraft:movement_speed") else {
            return;
        };
        let Ok(attribute_id) = i32::try_from(id) else {
            return;
        };
        let position = self
            .attributes
            .iter()
            .position(|state| state.attribute_id == attribute_id);
        let modifier_id = "minecraft:suffocating";
        if suffocating {
            let modifier =
                ClientAttributeModifierState::new(modifier_id.to_string(), f64::from(-0.34_f32), 1);
            match position {
                Some(index) => {
                    let modifiers = &mut self.attributes[index].modifiers;
                    modifiers.retain(|existing| existing.id != modifier_id);
                    modifiers.push(modifier);
                }
                None => {
                    let base = voxel_data::entity_type::name(self.type_id)
                        .and_then(|name| {
                            voxel_data::entity_type::default_attribute(
                                name,
                                "minecraft:movement_speed",
                            )
                        })
                        .unwrap_or(0.0);
                    self.attributes.push(ClientAttributeState::new(
                        attribute_id,
                        base,
                        vec![modifier],
                    ));
                }
            }
        } else if let Some(index) = position {
            self.attributes[index]
                .modifiers
                .retain(|existing| existing.id != modifier_id);
        }
    }

    /// Whether this is a strider in a saddle (which a warped fungus on a stick steers).
    pub fn is_saddled_strider(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:strider")
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }

    /// Whether this is a pig in a saddle (which a carrot on a stick steers).
    pub fn is_saddled_pig(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:pig")
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }

    /// Whether this is a horse, donkey, mule or undead horse in a saddle, which the player on it
    /// steers (`AbstractHorse.getControllingPassenger`).
    pub fn is_controllable_horse(&self) -> bool {
        crate::mount_ride::is_horse_type(self.type_id)
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }

    /// Whether this is a boat, a chest boat or a raft.
    pub fn is_boat(&self) -> bool {
        vanilla_boat_max_passengers(self.type_id).is_some()
    }

    /// A boat's hit, bubbles and paddles, if it has any.
    pub fn boat(&self) -> Option<&crate::boat::BoatState> {
        self.kind.boat()
    }

    /// Whether a boat's paddle on a side (zero left, one right) rows
    /// (`AbstractBoat.getPaddleState`): its data says so and someone rows it.
    pub fn boat_paddle_rows(&self, side: usize) -> bool {
        let flag = if side == 0 {
            self.paddle_left
        } else {
            self.paddle_right
        };
        flag && !self.passenger_ids.is_empty()
    }

    /// `AbstractBoat.tick` on the client, at the start of the tick (`under_water` is
    /// none for an entity that is not a boat).
    pub(crate) fn tick_boat(&mut self, under_water: Option<bool>) {
        let Some(under_water) = under_water else {
            return;
        };
        let paddling = [self.boat_paddle_rows(0), self.boat_paddle_rows(1)];
        let tick_count = self.tick_count;
        self.kind
            .boat_or_insert_with(Default::default)
            .tick(under_water, paddling, tick_count);
    }

    /// The part of a ridden boat's tick before its own movement.
    pub(crate) fn tick_boat_start(&mut self) {
        self.kind
            .boat_or_insert_with(Default::default)
            .tick_start(false);
    }

    /// The part of a ridden boat's tick after its movement: the paddles row and the bubbles shake.
    pub(crate) fn tick_boat_end(&mut self, paddling: [bool; 2]) {
        let tick_count = self.tick_count;
        self.kind
            .boat_or_insert_with(Default::default)
            .tick_end(paddling, tick_count);
    }
}
