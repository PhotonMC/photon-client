use super::*;

impl ClientEntity {
    /// `LivingEntity.getTicksUsingItem(partialTick)`: how long the item in use has been used.
    pub fn ticks_using_item(&self, partial_tick: f32) -> f32 {
        if self.is_using_item() {
            self.living.using_item_ticks as f32 + partial_tick
        } else {
            0.0
        }
    }

    /// `LivingEntity.onKineticHit` (entity event 2): the kick of a spear's hit starts, unless one
    /// began in the last ten ticks.
    pub(crate) fn on_kinetic_hit(&mut self, game_time: u64) {
        if self
            .living
            .kinetic_hit_time
            .is_none_or(|last| game_time.saturating_sub(last) > 10)
        {
            self.living.kinetic_hit_time = Some(game_time);
        }
    }

    /// The limbs' walk animation.
    pub fn walk_animation(&self) -> &crate::living::WalkAnimation {
        &self.walk
    }

    /// `getSwimAmount(partialTick)`.
    pub fn swim_amount(&self, partial_tick: f32) -> f32 {
        let old = self.old_swim_amount.get();
        old + partial_tick * (self.swim_amount.get() - old)
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

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    /// `LivingEntity.getFallFlyingTicks`.
    pub fn fall_flying_ticks(&self) -> i32 {
        self.living.fall_fly_ticks
    }

    /// `Entity.isInWater` for a mob that tracks it.
    pub fn is_in_water(&self) -> bool {
        self.living.in_water
    }

    /// `Entity.isCrouching`: the crouching pose.
    pub fn is_crouching(&self) -> bool {
        self.pose == ClientEntityPose::Crouching
    }

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

    pub fn hurt_time(&self) -> i32 {
        self.living.hurt_time
    }

    pub fn death_time(&self) -> i32 {
        self.living.death_time
    }

    /// `getHurtDir`: a player keeps the yaw its last hurt animation came with; any other living
    /// entity has none.
    pub fn hurt_dir(&self) -> f32 {
        f32::from_bits(self.living.hurt_dir)
    }

    pub fn health(&self) -> Option<f32> {
        self.health.map(f32::from_bits)
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

    pub fn effect_particles(&self) -> &[EffectParticle] {
        &self.effect_particles
    }

    pub fn sleeping_pos(&self) -> Option<photon_core::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    /// `Entity.getCustomName`: the component the server sent, as its NBT tag.
    pub fn custom_name(&self) -> Option<photon_core::nbt::FieldValue> {
        crate::metadata::custom_name_from_metadata(self.metadata.as_deref()?)?
    }

    /// `LivingEntityRenderer.isEntityUpsideDown` for the entities that are not avatars: a living
    /// entity named Dinnerbone or Grumm. An avatar's renderer asks its cape first.
    pub fn is_upside_down(&self) -> bool {
        self.is_living_entity() && !self.is_avatar() && self.has_upside_down_name()
    }

    /// A custom name of Dinnerbone or Grumm (the text of the name, whatever its style).
    pub fn has_upside_down_name(&self) -> bool {
        self.custom_name().is_some_and(|name| {
            let text = literal_text(&name);
            text == "Dinnerbone" || text == "Grumm"
        })
    }

    /// `Avatar.DATA_PLAYER_MODE_CUSTOMISATION`: the skin layers an avatar shows, a bit for each
    /// `PlayerModelPart` (none until the server sends them; a mannequin's start as all).
    pub fn model_parts(&self) -> u8 {
        self.metadata
            .as_deref()
            .and_then(crate::metadata::player_model_customisation_from_metadata)
            .unwrap_or(0)
    }

    /// `Avatar.isModelPartShown(CAPE)`: the cape bit of the skin layers.
    pub fn shows_cape(&self) -> bool {
        self.model_parts() & 1 != 0
    }

    /// `AvatarRenderer.isEntityUpsideDown` for a player: the cape shows and the profile is named
    /// Dinnerbone or Grumm.
    pub fn is_upside_down_player(&self, profile_name: &str) -> bool {
        self.shows_cape() && matches!(profile_name, "Dinnerbone" | "Grumm")
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

    /// `Entity.canSpawnSprintParticle` but for the fluids the world has to look up.
    pub(crate) fn sprints_on_foot(&self) -> bool {
        self.living_kind().is_some()
            && self.alive
            && self.is_sprinting()
            && !self.is_spectator()
            && !self.is_crouching()
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    /// `getMainArm`: a mob's left-handed flag, an avatar's data.
    pub fn main_arm_is_left(&self) -> bool {
        if self.is_avatar() {
            self.living.main_arm_left
        } else {
            self.is_left_handed()
        }
    }

    /// Whether the entity's water state is tracked (`Entity.isInWater`): the
    /// mobs that draw differently in water.
    pub(crate) fn tracks_water(&self) -> bool {
        self.is_squid()
            || self.is_fish()
            || matches!(
                photon_data::entity_type::name(self.type_id),
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

    /// `ItemEntity.bobOffs`.
    pub fn bob_offset(&self) -> f32 {
        self.bob_offset.get()
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

pub(crate) fn shared_flag_is_set(flags: u8, flag: u8) -> bool {
    flags & (1_u8 << flag) != 0
}

/// What an entity's tick and look keep besides its pose:
/// `LivingEntity.hurtTime` and `deathTime`, the registry id of the variant a
/// pig or a cow wears, and the value of an experience orb, once the server
/// has told them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct EntityState {
    pub(crate) hurt_time: i32,
    /// `Player.hurtDir`, as bits so the state stays `Eq`.
    pub(crate) hurt_dir: u32,
    pub(crate) death_time: i32,
    /// How many ticks the item in use has been used (`getTicksUsingItem`, whole).
    pub(crate) using_item_ticks: i32,
    /// `LivingEntity.lastKineticHitFeedbackTime`: the game time a spear's hit last kicked, if one
    /// has.
    pub(crate) kinetic_hit_time: Option<u64>,
    /// `LivingEntity.elytraAnimationState`.
    pub(crate) elytra: crate::elytra::ElytraState,
    /// `LivingEntity.fallFlyTicks`: how long the entity has glided.
    pub(crate) fall_fly_ticks: i32,
    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub(crate) previous_motion: [u64; 3],
    /// `ClientAvatarState`'s cloak and bob, which hang a player's cape.
    pub(crate) cloak: crate::cloak::Cloak,
    pub(crate) variant: Option<i32>,
    pub(crate) orb_value: Option<i32>,
    /// A piglin's dancing flag.
    pub(crate) dancing: bool,
    /// A piglin's crossbow-charging flag.
    pub(crate) charging_crossbow: bool,
    /// An avatar's main arm is the left.
    pub(crate) main_arm_left: bool,
    /// A villager's type, profession and level (registry ids), once told.
    pub(crate) villager_data: Option<[i32; 3]>,
    /// `AbstractVillager.getUnhappyCounter`.
    pub(crate) unhappy_counter: i32,
    /// `Raider.isCelebrating`, and whether a spellcaster is casting.
    pub(crate) celebrating: bool,
    pub(crate) casting_spell: bool,
    /// The block state an enderman carries (zero for none), and whether it is
    /// creepy (stared at).
    pub(crate) carried_state: i32,
    pub(crate) creepy: bool,
    /// Whether a snow golem wears its pumpkin; it does until the data says not.
    pub(crate) bare_head: bool,
    /// Which of its horns a goat has lost, until the data says it has none.
    pub(crate) lost_left_horn: bool,
    pub(crate) lost_right_horn: bool,
    /// Whether a goat screams.
    pub(crate) screaming_goat: bool,
    /// A mooshroom's variant, zero for red.
    pub(crate) mooshroom_type: i32,
    /// Whether a mooshroom holds the stew effects of a flower fed to it on this client.
    pub(crate) mooshroom_stew: bool,
    /// An ageable mob's age lock, and the ticks the client waits after a golden dandelion
    /// before it takes another (`AgeableMob.ageLockParticleTimer`).
    pub(crate) age_locked: bool,
    pub(crate) age_lock_timer: i32,
    /// A rabbit's variant, zero for brown.
    pub(crate) rabbit_type: i32,
    /// Whether a creeper is charged.
    pub(crate) powered: bool,
    /// A horse's variant data (the variant in the low byte, the markings in
    /// the next) and whether a chested one carries its chest.
    pub(crate) horse_variant: i32,
    pub(crate) horse_chest: bool,
    /// A bee's flags and the game time its anger ends.
    pub(crate) bee_flags: u8,
    pub(crate) anger_end: i64,
    /// A tamable animal's flags (1 sitting, 4 tame), a wolf's interest, and
    /// the dye of its collar, when the data has said (red otherwise).
    pub(crate) tamable_flags: u8,
    /// The owner a tamable animal names (`TamableAnimal.getOwnerReference`).
    pub(crate) owner: Option<photon_core::binary::Uuid>,
    pub(crate) wolf_interested: bool,
    /// A wolf's interest angle and shake.
    pub(crate) wolf: crate::mobs::wolf::WolfState,
    pub(crate) collar: Option<i32>,
    /// A cat's data (whether it lies, whether it is relaxed) and its own
    /// client state.
    pub(crate) cat_lying: bool,
    pub(crate) cat_relaxed: bool,
    pub(crate) cat: crate::mobs::cat::CatState,
    /// `Strider.isSuffocating`: it is out of lava and cold.
    pub(crate) strider_suffocating: bool,
    /// `Hoglin.isImmuneToZombification`.
    pub(crate) hoglin_immune: bool,
    /// `Entity.isInWater` for the mobs that track it.
    pub(crate) in_water: bool,
    /// `Entity.wasTouchingWater`: whether the entity's box found water last tick, so the tick it
    /// first does splashes.
    pub(crate) touching_water: bool,
    /// `EntityFluidInteraction.isInFluid(LAVA)` as of last tick.
    pub(crate) touching_lava: bool,
    /// `GlowSquid.getDarkTicksRemaining`.
    pub(crate) glow_dark_ticks: i32,
    /// `AbstractCubeMob.getSize` once the data has said (one before), and its squish.
    pub(crate) cube_size: Option<i32>,
    pub(crate) cube: crate::mobs::cube_mob::CubeMobState,
    /// A shulker's attach face (a direction's data value, down at first), raw
    /// peek and colour (`None` for the default), and its lid.
    pub(crate) shulker_face: i32,
    pub(crate) shulker_raw_peek: i8,
    pub(crate) shulker_color: Option<u8>,
    pub(crate) shulker: crate::mobs::shulker::ShulkerState,
    /// `Parrot.getVariant`'s id.
    pub(crate) parrot_variant: i32,
    /// `Bat.isResting`, `Vex.isCharging` and `Allay.isDancing`.
    pub(crate) bat_resting: bool,
    pub(crate) vex_charging: bool,
    pub(crate) allay_dancing: bool,
    /// `Ghast.isCharging` and `Phantom.getPhantomSize`.
    pub(crate) ghast_charging: bool,
    pub(crate) phantom_size: i32,
    /// `WitherSkull.isDangerous`: the blue skull, which flies slower.
    pub(crate) wither_skull_dangerous: bool,
    /// `Guardian.isMoving` and the id of `getActiveAttackTarget` (zero for none).
    pub(crate) guardian_moving: bool,
    pub(crate) guardian_target: i32,
    /// `Axolotl.getVariant`'s id and `Axolotl.isPlayingDead`.
    pub(crate) axolotl_variant: i32,
    pub(crate) axolotl_playing_dead: bool,
    /// `Turtle.hasEgg` and `Turtle.isLayingEgg`.
    pub(crate) turtle_has_egg: bool,
    pub(crate) turtle_laying_egg: bool,
    /// A salmon's size, a tropical fish's packed variant or a pufferfish's puff
    /// state, once the data has said.
    pub(crate) fish_variant: Option<i32>,
    /// A copper golem's `CopperGolemState` and its weathering (0 to 3).
    pub(crate) copper_golem_action: crate::mobs::copper_golem::GolemAction,
    pub(crate) copper_golem_weather: u8,
    /// A polar bear's standing animation.
    pub(crate) polar_bear: crate::mobs::polar_bear::PolarBearState,
    /// A fox's flags, variant and own client state.
    pub(crate) fox: crate::mobs::fox::FoxState,
    /// A sheep's wool and eating, a chicken's flap.
    pub(crate) animal: crate::mobs::animal::AnimalState,
    /// The arm's swing.
    pub(crate) swing: crate::living::SwingState,
    /// An armor stand's flags and poses.
    pub(crate) armor_stand: crate::objects::armor_stand::ArmorStandState,
}

impl EntityState {
    /// Takes the `TamableAnimal` data of a nautilus or parrot.
    pub(crate) fn apply_tamable(&mut self, data: crate::metadata::TamableData) {
        if let Some(flags) = data.flags {
            self.tamable_flags = flags;
        }
        if let Some(owner) = data.owner {
            self.owner = owner;
        }
    }

    /// Takes what a cat's data holds (see [`crate::metadata::CatData`]).
    pub(crate) fn apply_cat(&mut self, cat: crate::metadata::CatData) {
        if let Some(flags) = cat.flags {
            self.tamable_flags = flags;
        }
        if let Some(owner) = cat.owner {
            self.owner = owner;
        }
        if let Some(lying) = cat.lying {
            self.cat_lying = lying;
        }
        if let Some(relaxed) = cat.relaxed {
            self.cat_relaxed = relaxed;
        }
        if let Some(dye) = cat.collar {
            self.collar = Some(dye);
        }
    }

    /// Takes what a wolf's data holds (see [`crate::metadata::WolfData`]).
    pub(crate) fn apply_wolf(&mut self, wolf: crate::metadata::WolfData) {
        if let Some(flags) = wolf.flags {
            self.tamable_flags = flags;
        }
        if let Some(owner) = wolf.owner {
            self.owner = owner;
        }
        if let Some(interested) = wolf.interested {
            self.wolf_interested = interested;
        }
        if let Some(dye) = wolf.collar {
            self.collar = Some(dye);
        }
        if let Some(end) = wolf.anger_end {
            self.anger_end = end;
        }
    }
}

/// `Component.getString` of a component with no translations or keys: its own text and then its
/// extra components' (a plain string is a text component).
fn literal_text(component: &photon_core::nbt::FieldValue) -> String {
    use photon_core::nbt::FieldValue;
    match component {
        FieldValue::Str(text) => text.clone(),
        FieldValue::List(parts) => parts.iter().map(literal_text).collect(),
        FieldValue::Compound(fields) => fields
            .iter()
            .filter(|field| field.key == "text" || field.key == "extra")
            .map(|field| literal_text(&field.value))
            .collect(),
        _ => String::new(),
    }
}

impl EntityViewMut<'_> {
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
                self.pos.position.y = JavaF64::new(self.pos.position.y.get() + lift);
            } else {
                self.living.death_time += 1;
            }
        }
    }
}

impl EntityView<'_> {
    /// `Entity.isOnFire` on the client: the shared flag, for a type that can burn (a heart-bound
    /// creaking cannot). Not carried: an item whose stack resists fire.
    pub fn is_burning(&self) -> bool {
        self.is_on_fire()
            && !photon_data::entity_type::fire_immune(self.type_id)
            && !self
                .creaking()
                .is_some_and(|creaking| creaking.is_heart_bound())
    }
}

impl EntityView<'_> {
    /// `Entity.displayFireAnimation`: the flames a burning entity wears, except for a spectator.
    pub fn displays_fire(&self) -> bool {
        self.is_burning() && !self.is_spectator()
    }
}
