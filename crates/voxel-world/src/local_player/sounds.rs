//! The sounds the local player makes by moving (`Entity.applyMovementEmissionAndPlaySound`,
//! `Player.playStepSound`): a step for every stretch of ground walked, the muffled step and the
//! swim sound in water.

use voxel_core::binary::BlockPos;
use voxel_core::math::Vec3;

use super::{ClientEntityPose, Level, LocalPlayer};

/// What kind of sound a player's sound is, for the volume sliders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalSoundSource {
    Players,
    Blocks,
    Hostile,
    Neutral,
    Records,
    Ambient,
}

impl LocalSoundSource {
    /// `SoundSource.getName`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Players => "player",
            Self::Blocks => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Records => "record",
            Self::Ambient => "ambient",
        }
    }
}

/// A sound the local player asks for, at a place.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalSound {
    pub event: &'static str,
    pub source: LocalSoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub position: (f64, f64, f64),
}

/// The sounds waiting to be played. Which are waiting is no part of what a world is equal to.
#[derive(Debug, Clone, Default)]
pub struct SoundQueue(pub Vec<LocalSound>);

impl PartialEq for SoundQueue {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for SoundQueue {}

/// How far a stretch of ground is walked for a step: the movement scaled (`0.6F` is the float
/// `Entity.applyMovementEmissionAndPlaySound` multiplies by).
const STEP_SCALE: f32 = 0.6;

impl LocalPlayer {
    /// The sounds made since the last call.
    pub fn take_sounds(&mut self) -> Vec<LocalSound> {
        std::mem::take(&mut self.pending_sounds)
    }

    fn state_in_tag(level: &Level<'_>, pos: BlockPos, tag: &str) -> bool {
        let state = crate::block::state_or_air(level.state_id(pos).map_or(0, i64::from));
        level
            .world
            .block_state_registry
            .tags()
            .contains(usize::from(state.block), tag)
    }

    /// `Entity.playSound`: one of the player's own sounds, where it is.
    fn play_player_sound(&mut self, event: &'static str, volume: f32, pitch: f32) {
        self.pending_sounds.push(LocalSound {
            event,
            source: LocalSoundSource::Players,
            volume,
            pitch,
            position: (self.position.x, self.position.y, self.position.z),
        });
    }

    fn sound_type_of(
        level: &Level<'_>,
        pos: BlockPos,
    ) -> Option<&'static voxel_data::block_sound::SoundTypeRow> {
        let id = level.state_id(pos).map_or(0, i64::from);
        let state = crate::block::state_or_air(id);
        voxel_data::block_sound::of_state(state.block().name, u32::try_from(id).ok()?)
    }

    /// `Entity.playStepSound`: the block's step sound, a sixth as loud.
    fn play_block_step(&mut self, level: &Level<'_>, pos: BlockPos) {
        if let Some(sound_type) = Self::sound_type_of(level, pos) {
            self.play_player_sound(
                sound_type.step_sound(),
                sound_type.volume * 0.15,
                sound_type.pitch,
            );
        }
    }

    /// `Entity.playMuffledStepSound`.
    fn play_muffled_step(&mut self, level: &Level<'_>, pos: BlockPos) {
        if let Some(sound_type) = Self::sound_type_of(level, pos) {
            self.play_player_sound(
                sound_type.step_sound(),
                sound_type.volume * 0.05,
                sound_type.pitch * 0.8,
            );
        }
    }

    /// `Entity.playCombinationStepSounds`.
    fn play_combination_step(&mut self, level: &Level<'_>, primary: BlockPos, secondary: BlockPos) {
        if let Some(sound_type) = Self::sound_type_of(level, primary) {
            self.play_player_sound(
                sound_type.step_sound(),
                sound_type.volume * 0.15,
                sound_type.pitch,
            );
        }
        self.play_muffled_step(level, secondary);
    }

    /// `LivingEntity.causeFallDamage`, for the sounds: a fall that would hurt makes the player's
    /// small or big fall sound and the block's. Whether it would is the result.
    fn cause_fall_damage(&mut self, level: &Level<'_>, distance: f64, modifier: f32) -> bool {
        if self.abilities.may_fly {
            return false;
        }
        // The default safe fall distance (3) and fall damage multiplier (1).
        let damage = (distance + 1.0e-6 - 3.0) * f64::from(modifier) * 1.0;
        let damage = damage.floor() as i32;
        if damage <= 0 {
            return false;
        }
        let sound = if damage > 4 {
            "minecraft:entity.player.big_fall"
        } else {
            "minecraft:entity.player.small_fall"
        };
        self.play_player_sound(sound, 1.0, 1.0);
        self.play_block_fall_sound(level);
        true
    }

    /// `LivingEntity.playBlockFallSound`.
    fn play_block_fall_sound(&mut self, level: &Level<'_>) {
        let pos = BlockPos::new(
            self.position.x.floor() as i32,
            (self.position.y - f64::from(0.2_f32)).floor() as i32,
            self.position.z.floor() as i32,
        );
        if let Some(sound_type) = Self::sound_type_of(level, pos) {
            self.play_player_sound(
                sound_type.fall_sound(),
                sound_type.volume * 0.5,
                sound_type.pitch * 0.75,
            );
        }
    }

    /// `Block.fallOn` of the block landed on, for the sounds it makes (the slime block's fall
    /// damage is none, farmland and turtle eggs break only on the server).
    pub(super) fn fall_on(&mut self, level: &Level<'_>, pos: BlockPos, distance: f64) {
        use voxel_data::block::classes::BlockClass;
        let id = level.state_id(pos).map_or(0, i64::from);
        let state = crate::block::state_or_air(id);
        let block = state.block();
        let is = |class: BlockClass| block.class.lineage().any(|own| own == class);
        if is(BlockClass::HayBlock) {
            self.cause_fall_damage(level, distance, 0.2);
        } else if is(BlockClass::SlimeBlock) {
            // `causeFallDamage(distance, 0)` hurts nothing, so it makes no sound.
        } else if is(BlockClass::HoneyBlock) {
            self.play_player_sound("minecraft:block.honey_block.slide", 1.0, 1.0);
            if self.cause_fall_damage(level, distance, 0.2)
                && let Some(sound_type) = Self::sound_type_of(level, pos)
            {
                self.play_player_sound(
                    sound_type.fall_sound(),
                    sound_type.volume * 0.5,
                    sound_type.pitch * 0.75,
                );
            }
        } else if is(BlockClass::PowderSnowBlock) {
            if distance >= 4.0 {
                let sound = if distance < 7.0 {
                    "minecraft:entity.player.small_fall"
                } else {
                    "minecraft:entity.player.big_fall"
                };
                self.play_player_sound(sound, 1.0, 1.0);
            }
        } else if is(BlockClass::PointedDripstoneBlock) && Self::is_dripstone_tip_up(state) {
            self.cause_fall_damage(level, distance + 2.5, 2.0);
        } else if is(BlockClass::BedBlock) {
            self.cause_fall_damage(level, distance * 0.5, 1.0);
        } else {
            self.cause_fall_damage(level, distance, 1.0);
        }
    }

    /// Whether a pointed dripstone is an upward tip.
    fn is_dripstone_tip_up(state: &crate::block::StateInfo) -> bool {
        Self::property_of(state, "vertical_direction") == Some("up")
            && Self::property_of(state, "thickness") == Some("tip")
    }

    /// A block state's property value by name; `None` when the block has no such property.
    fn property_of(state: &crate::block::StateInfo, name: &str) -> Option<&'static str> {
        let block = state.block();
        let property = block.properties.iter().find(|own| own.name == name)?;
        block.value(u32::from(state.offset), property)
    }

    /// `BubbleColumnAmbientSoundHandler.tick`: the sound of getting into a bubble column (the
    /// first state of one under the player's feet, the box shrunk a little at the top).
    pub(super) fn tick_ambient_sounds(&mut self, level: &Level<'_>) {
        let area = self
            .bounding_box
            .inflate(0.0, -f64::from(0.4_f32), 0.0)
            .deflate(1.0e-6);
        let mut found = None;
        'search: for z in (area.min_z.floor() as i32)..=(area.max_z.floor() as i32) {
            for y in (area.min_y.floor() as i32)..=(area.max_y.floor() as i32) {
                for x in (area.min_x.floor() as i32)..=(area.max_x.floor() as i32) {
                    let id = level.state_id(BlockPos::new(x, y, z)).map_or(0, i64::from);
                    let state = crate::block::state_or_air(id);
                    if state.block().name == "minecraft:bubble_column" {
                        found = Some(state);
                        break 'search;
                    }
                }
            }
        }
        if let Some(state) = found {
            if !self.was_in_bubble_column && !self.ambient_first_tick && !self.is_spectator() {
                let drag_down = Self::property_of(state, "drag") == Some("true");
                self.play_player_sound(
                    if drag_down {
                        "minecraft:block.bubble_column.whirlpool_inside"
                    } else {
                        "minecraft:block.bubble_column.upwards_inside"
                    },
                    1.0,
                    1.0,
                );
            }
            self.was_in_bubble_column = true;
        } else {
            self.was_in_bubble_column = false;
        }
        self.ambient_first_tick = false;
    }

    /// `(random.nextFloat() - random.nextFloat()) * 0.2F + 1.0F`.
    fn hurt_pitch(&mut self) -> f32 {
        let first = self.random.next_float();
        (first - self.random.next_float()) * 0.2 + 1.0
    }

    /// `LivingEntity.handleDamageEvent` of the local player: it flinches, its limbs swing at once,
    /// and it makes the sound its damage type gives (`Player.getHurtSound`).
    pub fn handle_damage_event(&mut self, damage_type: &str) {
        self.walk.set_speed(1.5);
        self.invulnerable_time = 20;
        self.hurt_time = 10;
        self.hurt_duration = 10;
        let pitch = self.hurt_pitch();
        self.play_player_sound(
            voxel_data::damage_effect::hurt_sound(damage_type),
            1.0,
            pitch,
        );
    }

    /// `LivingEntity.handleEntityEvent(3)` of the local player: the death sound (the rest of
    /// dying is the server's, as `health` reaches zero).
    pub fn handle_death_event(&mut self) {
        let pitch = self.hurt_pitch();
        self.play_player_sound("minecraft:entity.player.death", 1.0, pitch);
    }

    /// `Entity.waterSwimSound`.
    fn water_swim_sound(&mut self) {
        let delta = self.delta;
        let slow = f64::from(0.2_f32);
        let speed = (delta.x * delta.x * slow + delta.y * delta.y + delta.z * delta.z * slow).sqrt()
            as f32
            * 0.35;
        let speed = speed.min(1.0);
        let pitch = self.random_pitch();
        self.play_player_sound("minecraft:entity.player.swim", speed, pitch);
    }

    /// `1.0F + (random.nextFloat() - random.nextFloat()) * 0.4F`.
    fn random_pitch(&mut self) -> f32 {
        let first = self.random.next_float();
        1.0 + (first - self.random.next_float()) * 0.4
    }

    /// `Entity.doWaterSplashEffect`, for the sound: a splash that is louder for a faster fall.
    pub(super) fn water_splash_sound(&mut self) {
        let delta = self.delta;
        let slow = f64::from(0.2_f32);
        let speed = (delta.x * delta.x * slow + delta.y * delta.y + delta.z * delta.z * slow).sqrt()
            as f32
            * 0.2;
        let speed = speed.min(1.0);
        let event = if speed < 0.25 {
            "minecraft:entity.player.splash"
        } else {
            "minecraft:entity.player.splash.high_speed"
        };
        let pitch = self.random_pitch();
        self.play_player_sound(event, speed, pitch);
    }

    /// `Entity.getPrimaryStepSoundBlockPos`: the block above, if it is one a step is heard through.
    fn primary_step_pos(level: &Level<'_>, affecting: BlockPos) -> BlockPos {
        let above = BlockPos::new(affecting.x, affecting.y + 1, affecting.z);
        if !Self::state_in_tag(level, above, "minecraft:inside_step_sound_blocks")
            && !Self::state_in_tag(level, above, "minecraft:combination_step_sound_blocks")
        {
            affecting
        } else {
            above
        }
    }

    /// `Player.playStepSound`.
    fn play_player_step(&mut self, level: &Level<'_>, on_pos: BlockPos) {
        if self.is_in_water() {
            self.water_swim_sound();
            self.play_muffled_step(level, on_pos);
            return;
        }
        let primary = Self::primary_step_pos(level, on_pos);
        if primary != on_pos {
            if Self::state_in_tag(level, primary, "minecraft:combination_step_sound_blocks") {
                self.play_combination_step(level, primary, on_pos);
            } else {
                self.play_block_step(level, primary);
            }
        } else {
            self.play_block_step(level, on_pos);
        }
    }

    /// `Entity.vibrationAndSoundEffectsFromBlock`, for the sounds: whether the block is one a step
    /// is made on (it is stood on, climbed, crept on or ridden on, and not swum in).
    fn step_on_block(
        &mut self,
        level: &Level<'_>,
        pos: BlockPos,
        movement: Vec3,
        sound: bool,
    ) -> bool {
        let info = level.info(pos);
        if info.is_air() {
            return false;
        }
        let climbable =
            info.flags & super::flag::CLIMBABLE != 0 || info.name == "minecraft:powder_snow";
        if (self.on_ground
            || climbable
            || self.pose == ClientEntityPose::Crouching && movement.y == 0.0)
            && !self.is_swimming()
        {
            if sound {
                self.play_player_step(level, pos);
            }
            return true;
        }
        false
    }

    /// `Entity.applyMovementEmissionAndPlaySound` for a player that is not flying or creeping on
    /// the ground (`Player.getMovementEmission`): the distance walked builds up, and each whole
    /// step of it is a step sound.
    pub(super) fn apply_movement_emission(
        &mut self,
        level: &Level<'_>,
        movement: Vec3,
        effect_pos: BlockPos,
    ) {
        if self.abilities.flying || self.on_ground && self.is_shift_key_down() {
            return;
        }
        let moved = (movement.length() * f64::from(STEP_SCALE)) as f32;
        let horizontal = (movement.horizontal_distance() * f64::from(STEP_SCALE)) as f32;
        let supporting = self.get_on_pos(level, 1.0e-5);
        let supporting_info = level.info(supporting);
        let climbing = supporting_info.flags & super::flag::CLIMBABLE != 0
            || supporting_info.name == "minecraft:powder_snow";
        self.move_dist += if climbing { moved } else { horizontal };
        if self.move_dist > self.next_step && !supporting_info.is_air() {
            let mut produced = self.step_on_block(level, effect_pos, movement, true);
            if supporting != effect_pos {
                produced |= self.step_on_block(level, supporting, movement, false);
            }
            if produced {
                self.next_step = (self.move_dist as i32 + 1) as f32;
            } else if self.is_in_water() {
                self.next_step = (self.move_dist as i32 + 1) as f32;
                self.water_swim_sound();
            }
        }
    }
}
