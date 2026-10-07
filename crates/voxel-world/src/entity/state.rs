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

    /// A boat's data: its hit, paddles and bubbles.
    pub(crate) fn set_boat_data(&mut self, data: crate::metadata::BoatData) {
        let boat = self.kind.boat_or_insert_with(Default::default);
        if let Some(hurt) = data.hurt_time {
            boat.hurt_time = hurt;
        }
        if let Some(dir) = data.hurt_dir {
            boat.hurt_dir = dir;
        }
        if let Some(damage) = data.damage {
            boat.damage = damage;
        }
        if let Some(bubbles) = data.bubble_time {
            boat.bubble_time = bubbles;
        }
        if let Some(left) = data.paddle_left {
            self.paddle_left = left;
        }
        if let Some(right) = data.paddle_right {
            self.paddle_right = right;
        }
    }

    /// `Parrot.getVariant`'s id.
    pub fn parrot_variant(&self) -> i32 {
        self.living.parrot_variant
    }

    /// A parrot's wing beats, if it has ticked.
    pub fn parrot(&self) -> Option<&crate::parrot::ParrotState> {
        self.kind.parrot()
    }

    /// `Parrot.calculateFlapping` after `super.aiStep`: the wings beat by
    /// whether the parrot is in the air, and one falling there sinks slowly.
    pub(crate) fn tick_parrot(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:parrot") {
            return;
        }
        let (on_ground, passenger) = (self.on_ground, self.vehicle_id.is_some());
        if !on_ground && self.motion.y.get() < 0.0 {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        self.kind
            .parrot_or_insert_with(Default::default)
            .tick(on_ground, passenger);
    }

    /// A bat's animation states, if it has ticked.
    pub fn bat(&self) -> Option<&crate::bat::BatState> {
        self.kind.bat()
    }

    /// An allay's progress, if it has ticked.
    pub fn allay(&self) -> Option<&crate::allay::AllayState> {
        self.kind.allay()
    }

    /// `Bat.tick` after `super.tick`: a hanging bat does not move and hangs
    /// from the block above it; one flying slows its fall and climb.
    pub(crate) fn tick_bat(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:bat") {
            return;
        }
        let resting = self.living.bat_resting;
        if resting {
            self.motion.x = JavaF64::new(0.0_f64);
            self.motion.y = JavaF64::new(0.0_f64);
            self.motion.z = JavaF64::new(0.0_f64);
            let height = voxel_data::entity_type::pick(self.type_id)
                .map_or(0.0, |pick| pick.size.1 * self.scale());
            let y = self.position.y.get();
            self.position.y = JavaF64::new(y.floor() + 1.0 - f64::from(height));
        } else {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        let tick_count = self.tick_count;
        self.kind
            .bat_or_insert_with(Default::default)
            .tick(resting, tick_count);
    }

    /// `Allay.tick`'s client half, after `super.tick`.
    pub(crate) fn tick_allay(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:allay") {
            return;
        }
        let holds = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let dancing = self.living.allay_dancing;
        self.kind
            .allay_or_insert_with(Default::default)
            .tick(holds, dancing);
    }

    /// `Ghast.isCharging`: the ghast has its mouth open for a fireball.
    pub fn is_ghast_charging(&self) -> bool {
        self.living.ghast_charging
    }

    /// `Phantom.getPhantomSize`.
    pub fn phantom_size(&self) -> i32 {
        self.living.phantom_size
    }

    /// A guardian's animations, if it has ticked.
    pub fn guardian(&self) -> Option<&crate::guardian::GuardianState> {
        self.kind.guardian()
    }

    pub(crate) fn is_guardian(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:guardian" | "minecraft:elder_guardian")
        )
    }

    /// `Guardian.getActiveAttackTarget`'s id: what the beam aims at, if anything.
    pub fn guardian_attack_target(&self) -> Option<i32> {
        (self.living.guardian_target != 0).then_some(self.living.guardian_target)
    }

    /// `Guardian.isMoving`.
    pub fn is_guardian_moving(&self) -> bool {
        self.living.guardian_moving
    }

    /// `Guardian.onSyncedDataUpdated`: when the data holds the attack target,
    /// the beam starts over.
    pub(crate) fn set_guardian_data(&mut self, moving: Option<bool>, target: Option<i32>) {
        if let Some(moving) = moving {
            self.living.guardian_moving = moving;
        }
        if let Some(target) = target {
            self.living.guardian_target = target;
            self.kind
                .guardian_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
                .restart_attack();
        }
    }

    /// `Guardian.aiStep`'s client half, while the guardian lives: the beam's
    /// target (which the world resolves from its id) is where the head turns.
    pub(crate) fn tick_guardian(&mut self, target: Option<crate::guardian::GuardianTarget>) {
        if !self.is_guardian() || !self.alive {
            return;
        }
        let eye_height = self
            .dimensions()
            .scale(self.scale(), self.scale())
            .eye_height();
        let place = crate::guardian::GuardianPlace {
            in_water: self.living.in_water,
            moving: self.living.guardian_moving,
            position: self.position(),
            eye_height,
            target,
        };
        let mut pose = crate::guardian::GuardianPose {
            y_rot: self.y_rot.get(),
            x_rot: self.x_rot.get(),
            y_head_rot: self.y_head_rot.get(),
            motion: self.motion(),
            on_ground: self.on_ground,
        };
        let has_target = self.living.guardian_target != 0;
        let duration = self.guardian_attack_duration();
        self.kind
            .guardian_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
            .tick(&place, has_target, duration, &mut pose);
        self.y_rot = JavaF32::new(pose.y_rot);
        self.x_rot = JavaF32::new(pose.x_rot);
        self.y_head_rot = JavaF32::new(pose.y_head_rot);
        self.motion = DVec3::new(pose.motion.0, pose.motion.1, pose.motion.2);
        self.on_ground = pose.on_ground;
    }

    /// `Guardian.getAttackDuration`, shorter for the elder.
    pub fn guardian_attack_duration(&self) -> i32 {
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:elder_guardian") {
            60
        } else {
            80
        }
    }

    /// Whether this is a frog.
    pub(crate) fn is_frog(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:frog")
    }

    /// A frog's animation states, if it has any.
    pub fn frog(&self) -> Option<&crate::frog::FrogState> {
        self.kind.frog()
    }

    /// `Frog.onSyncedDataUpdated(DATA_POSE)`: the pose the data names starts
    /// or stops the frog's animations.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose) {
        self.pose = pose;
        if self.is_frog() {
            let tick_count = self.tick_count;
            self.kind
                .frog_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_breeze() {
            let tick_count = self.tick_count;
            self.kind
                .breeze_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.kind
                .warden_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
    }

    /// A sniffer's state and animation states, if it has any.
    pub fn sniffer(&self) -> Option<&crate::sniffer::SnifferState> {
        self.kind.sniffer()
    }

    /// `Sniffer.onSyncedDataUpdated(DATA_STATE)`.
    pub(crate) fn set_sniffer_mode(&mut self, mode: crate::sniffer::SnifferMode) {
        let tick_count = self.tick_count;
        self.kind
            .sniffer_or_insert_with(Default::default)
            .set_mode(mode, tick_count);
    }

    /// `FireworkRocketEntity.isShotAtAngle`.
    pub fn is_shot_at_angle(&self) -> bool {
        self.shot_at_angle
    }

    /// Whether the type draws as an item facing the camera (`ThrownItemRenderer`, with its
    /// scale and whether it is fully lit), and so has its item among its data.
    pub fn thrown_item_look(&self) -> Option<(f32, bool)> {
        match voxel_data::entity_type::name(self.type_id)? {
            "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:experience_bottle"
            | "minecraft:lingering_potion"
            | "minecraft:snowball"
            | "minecraft:splash_potion" => Some((1.0, false)),
            "minecraft:eye_of_ender" => Some((1.0, true)),
            "minecraft:fireball" => Some((3.0, true)),
            "minecraft:small_fireball" => Some((0.75, true)),
            _ => None,
        }
    }

    /// Whether this is an arrow, a spectral arrow or a trident (`AbstractArrow`).
    pub fn is_abstract_arrow(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:arrow" | "minecraft:spectral_arrow" | "minecraft:trident")
        )
    }

    /// The block the entity was added in (`FallingBlockEntity.getStartPos`).
    pub fn start_block(&self) -> (i32, i32, i32) {
        self.start_block
    }

    /// The id of the entity this one is leashed to, if it is.
    pub fn leash_holder_id(&self) -> Option<i32> {
        (self.leash_holder_id != 0).then_some(self.leash_holder_id)
    }

    /// Whether this is a lightning bolt (`LightningBolt`).
    pub fn is_lightning_bolt(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:lightning_bolt")
    }

    /// A lightning bolt's shape and flashes, if it has begun to tick.
    pub fn lightning(&self) -> Option<&crate::lightning::LightningState> {
        self.kind.lightning()
    }

    pub(crate) fn lightning_finished(&self) -> bool {
        self.lightning_done
    }

    /// A display entity's data and render state.
    pub fn display(&self) -> Option<&crate::display::DisplayState> {
        self.kind.display()
    }

    /// Whether the entity has an `InterpolationHandler` (`getInterpolation` is not null): the
    /// living, the minecarts and the displays ease to the positions the server sends.
    pub(crate) fn has_interpolation_handler(&self) -> bool {
        self.living_kind().is_some() || self.is_minecart() || self.kind.display().is_some()
    }

    /// The length of the handler's easing (`InterpolationHandler.interpolationSteps`): three, or
    /// a display's own.
    pub(super) fn interpolation_length(&self) -> i32 {
        self.kind
            .display()
            .map_or(i32::from(LIVING_ENTITY_POSITION_LERP_STEPS), |display| {
                display.pos_rot_duration()
            })
    }

    /// Whether this is an item frame or a glow item frame (`ItemFrame`).
    pub fn is_item_frame(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:item_frame" | "minecraft:glow_item_frame")
        )
    }

    /// `ItemFrame.getRotation`.
    pub fn item_frame_rotation(&self) -> i32 {
        self.item_frame_rotation
    }

    /// Whether this is a block of primed tnt (`PrimedTnt`).
    pub fn is_primed_tnt(&self) -> bool {
        self.type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
    }

    /// A primed tnt's fuse and block, if it has any.
    pub fn primed_tnt(&self) -> Option<&crate::primed_tnt::PrimedTntState> {
        self.kind.primed_tnt()
    }

    pub(crate) fn primed_tnt_mut(&mut self) -> &mut crate::primed_tnt::PrimedTntState {
        self.kind.primed_tnt_or_insert_with(Default::default)
    }

    /// Whether this is a pair of evoker fangs.
    pub fn is_evoker_fangs(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:evoker_fangs")
    }

    /// Evoker fangs' bite, if it has begun to be told of.
    pub fn evoker_fangs(&self) -> Option<&crate::evoker_fangs::FangsState> {
        self.kind.evoker_fangs()
    }

    pub(crate) fn evoker_fangs_mut(&mut self) -> &mut crate::evoker_fangs::FangsState {
        self.kind.evoker_fangs_or_insert_with(Default::default)
    }

    /// An arrow's shake and data, if it has any.
    pub fn arrow(&self) -> Option<&crate::arrow::ArrowState> {
        self.kind.arrow()
    }

    pub(crate) fn arrow_mut(&mut self) -> &mut crate::arrow::ArrowState {
        self.kind.arrow_or_insert_with(Default::default)
    }

    /// Whether this is one of the minecarts (`AbstractMinecart`).
    pub fn is_minecart(&self) -> bool {
        voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .is_some()
    }

    /// A minecart's hit, carried block and fuel, if it has any.
    pub fn minecart(&self) -> Option<&crate::minecart::MinecartState> {
        self.kind
            .minecart_slot()
            .and_then(|slot| slot.state.as_ref())
    }

    pub(crate) fn minecart_mut(&mut self) -> &mut crate::minecart::MinecartState {
        self.kind
            .minecart_slot_or_insert_with(Default::default)
            .state
            .get_or_insert_with(Default::default)
    }

    /// `AbstractMinecart.getDisplayBlockState`: the block state the server named, else the
    /// type's own (the plain minecart shows none).
    pub fn minecart_display_block(&self) -> Option<u32> {
        let state = self.minecart().copied().unwrap_or_default();
        state.custom_display.or_else(|| {
            voxel_data::entity_type::name(self.type_id)
                .and_then(|name| crate::minecart::default_display_block(name, state.fuel))
        })
    }

    /// `AbstractMinecart.getDisplayOffset`, in sixteenths of a block.
    pub fn minecart_display_offset(&self) -> i32 {
        let default = voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .unwrap_or(6);
        self.minecart()
            .copied()
            .unwrap_or_default()
            .display_offset(default)
    }

    /// `AbstractMinecart.tick` on the client, with `OldMinecartBehavior.tick`'s client half: the
    /// hit runs down, then the position and turn ease to the last packet's while that lasts,
    /// else the turns are kept under a whole turn.
    pub(crate) fn tick_minecart(&mut self) {
        self.minecart_mut().tick();
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:tnt_minecart") {
            self.minecart_mut().burn_fuse();
        }
        if self.minecart_track().is_some() {
            self.tick_minecart_track();
        } else if self.is_interpolating() {
            self.tick_position_interpolation();
        } else {
            let (y_rot, x_rot) = (self.y_rot.get(), self.x_rot.get());
            self.x_rot = JavaF32::new(x_rot % 360.0);
            self.y_rot = JavaF32::new(y_rot % 360.0);
        }
    }

    /// `NewMinecartBehavior.lerpClientPositionAndRotation`: the steps the server sent are played
    /// back, putting the minecart where they lead.
    pub(super) fn tick_minecart_track(&mut self) {
        let (y_rot, x_rot) = self.body_rotation();
        let now = crate::minecart_track::MinecartStep {
            position: self.position(),
            movement: self.motion(),
            y_rot,
            x_rot,
            weight: 0.0,
        };
        let Some(place) = self
            .kind
            .minecart_slot_mut()
            .and_then(|slot| slot.track.as_mut())
            .and_then(|track| track.tick(now))
        else {
            return;
        };
        self.set_position(place.position);
        self.motion.x = JavaF64::new(place.movement.0);
        self.motion.y = JavaF64::new(place.movement.1);
        self.motion.z = JavaF64::new(place.movement.2);
        // `Entity.setXRot` and `setYRot` discard what is not finite.
        if place.x_rot.is_finite() {
            self.x_rot = JavaF32::new((place.x_rot % 360.0).clamp(-90.0, 90.0));
        }
        if place.y_rot.is_finite() {
            self.y_rot = JavaF32::new(place.y_rot);
        }
    }
}
