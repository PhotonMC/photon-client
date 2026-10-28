//! The sounds of the blocks (`SoundType`): the volume and pitch they play at, and the sound each
//! of breaking, stepping, placing, hitting and falling makes.

use crate::tables::block_sounds::{BLOCK_SOUND_TYPES, SOUND_TYPES, STATE_SOUND_TYPES};

/// One sound type (`SoundType`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundTypeRow {
    /// The reference's constant (`STONE`, `WOOD`).
    pub name: &'static str,
    pub volume: f32,
    pub pitch: f32,
    /// The sound events of breaking, stepping, placing, hitting and falling, in that order.
    pub sounds: [&'static str; 5],
}

impl SoundTypeRow {
    pub fn break_sound(&self) -> &'static str {
        self.sounds[0]
    }

    pub fn step_sound(&self) -> &'static str {
        self.sounds[1]
    }

    pub fn place_sound(&self) -> &'static str {
        self.sounds[2]
    }

    pub fn hit_sound(&self) -> &'static str {
        self.sounds[3]
    }

    pub fn fall_sound(&self) -> &'static str {
        self.sounds[4]
    }
}

/// Every sound type, by the reference's constant.
pub fn sound_types() -> impl Iterator<Item = &'static SoundTypeRow> {
    SOUND_TYPES.iter()
}

/// The sound type of a block (`BlockBehaviour.getSoundType` of its default state).
pub fn of_block(name: &str) -> Option<&'static SoundTypeRow> {
    let (_, index) = BLOCK_SOUND_TYPES.iter().find(|(block, _)| *block == name)?;
    SOUND_TYPES.get(usize::from(*index))
}

/// The sound type of a block state, given its block (the few states with another type are listed).
pub fn of_state(block: &str, state_id: u32) -> Option<&'static SoundTypeRow> {
    match STATE_SOUND_TYPES
        .iter()
        .find(|(state, _)| *state == state_id)
    {
        Some((_, index)) => SOUND_TYPES.get(usize::from(*index)),
        None => of_block(block),
    }
}

/// Every block with its sound type, in the registry's order.
pub fn blocks() -> impl Iterator<Item = (&'static str, &'static SoundTypeRow)> {
    BLOCK_SOUND_TYPES
        .iter()
        .filter_map(|(block, index)| Some((*block, SOUND_TYPES.get(usize::from(*index))?)))
}

/// The states whose sound type is not their block's.
pub fn state_overrides() -> impl Iterator<Item = (u32, &'static SoundTypeRow)> {
    STATE_SOUND_TYPES
        .iter()
        .filter_map(|(state, index)| Some((*state, SOUND_TYPES.get(usize::from(*index))?)))
}
