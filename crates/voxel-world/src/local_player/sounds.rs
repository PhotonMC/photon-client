//! The sounds the local player makes by moving (`Entity.applyMovementEmissionAndPlaySound`,
//! `Player.playStepSound`): a step for every stretch of ground walked, the muffled step and the
//! swim sound in water.

use voxel_core::binary::BlockPos;
use voxel_core::math::Vec3;
use voxel_core::random::RandomSource;

use super::{ClientEntityPose, Level, LocalPlayer};

/// What kind of sound a player's sound is, for the volume sliders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalSoundSource {
    Players,
    Blocks,
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

    /// `Entity.waterSwimSound`.
    fn water_swim_sound(&mut self) {
        let delta = self.delta;
        let slow = f64::from(0.2_f32);
        let speed = (delta.x * delta.x * slow + delta.y * delta.y + delta.z * delta.z * slow).sqrt()
            as f32
            * 0.35;
        let speed = speed.min(1.0);
        let mut random = RandomSource::create(i64::from(self.tick_count));
        let pitch = 1.0 + (random.next_float() - random.next_float()) * 0.4;
        self.play_player_sound("minecraft:entity.player.swim", speed, pitch);
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
