use crate::*;

impl ClientWorld {
    /// A sound the world makes at a place (`Level.playLocalSound`), for the sound engine to play.
    pub fn push_sound(&mut self, sound: local_player::LocalSound) {
        self.sounds.0.push(sound);
    }

    /// The sounds made since the last call, in order.
    pub fn take_sounds(&mut self) -> Vec<local_player::LocalSound> {
        std::mem::take(&mut self.sounds.0)
    }

    /// A sound the level plays at the local player's hearing (`ClientLevel.playSound`).
    pub fn play_local_sound(&mut self, sound: local_player::LocalSound) {
        self.sounds.0.push(sound);
    }

    /// The sound a block makes (`SoundType`) at its middle, played now.
    pub(crate) fn block_sound(
        &mut self,
        pos: (i32, i32, i32),
        state: i32,
        pick: fn(&photon_data::block_sound::SoundTypeRow) -> &'static str,
        scale: (fn(f32) -> f32, f32),
    ) {
        if let Some(sound) = block_sound_of(pos, state, pick, scale) {
            self.sounds.0.push(sound);
        }
    }

    /// `BlockItem.place`'s sound: the block placed, a little softer than loud and lower.
    pub fn play_place_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            photon_data::block_sound::SoundTypeRow::place_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        );
    }

    /// `MultiPlayerGameMode.continueDestroyBlock`'s hit sound: a block struck while it is broken.
    pub fn play_hit_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            photon_data::block_sound::SoundTypeRow::hit_sound,
            (|volume| (volume + 1.0) / 8.0, 0.5),
        );
    }

    /// `LevelEventHandler.levelEvent`, for the events that make a sound the client knows: `2001`,
    /// a block broken (its state's number is the data).
    /// The level events since the last call, in the order they came.
    pub fn take_level_events(&mut self) -> Vec<LevelEvent> {
        std::mem::take(&mut self.level_events.0)
    }

    /// The block and face the player swung at breaking, since the last call.
    pub fn take_breaking_effects(
        &mut self,
    ) -> Vec<((i32, i32, i32), photon_core::direction::Direction)> {
        std::mem::take(&mut self.breaking_effects)
    }

    /// `ClientLevel.addBreakingBlockEffect` asked for, as `Minecraft.continueAttack` asks.
    pub fn add_breaking_block_effect(
        &mut self,
        pos: (i32, i32, i32),
        face: photon_core::direction::Direction,
    ) {
        self.breaking_effects.push((pos, face));
    }

    /// `ClientLevel.levelEvent`: the event waits for the particle engine, which plays its sounds
    /// and makes its particles with the level's random.
    pub fn apply_level_event(&mut self, event: i32, pos: (i32, i32, i32), data: i32) {
        self.level_events
            .0
            .push(LevelEvent::Level { event, pos, data });
    }

    /// An explosion the server reports, for the particle engine to play.
    pub fn apply_explosion(&mut self, explosion: crate::particles::Explosion) {
        self.level_events
            .0
            .push(LevelEvent::Explosion(Box::new(explosion)));
    }

    /// `ClientLevel.blockEvent`, for what the client does with one beside a block entity's: a note
    /// block plays its instrument as it is now (`NoteBlock.triggerEvent`). False for another block.
    pub fn apply_note_block_event(&mut self, pos: (i32, i32, i32)) -> bool {
        let Some(state) = self.block_state_at(pos.0, pos.1, pos.2) else {
            return false;
        };
        if block::state_or_air(i64::from(state)).block().name != "minecraft:note_block" {
            return false;
        }
        let Some(sound) = block::rules::property(state, "instrument").and_then(note_block_sound)
        else {
            // `custom_head`: the head above's own sound is not read; it plays nothing.
            return false;
        };
        let tunable = !sound.contains(".imitate.");
        let note = block::rules::property(state, "note")
            .and_then(|note| note.parse().ok())
            .filter(|_| tunable);
        self.level_events
            .0
            .push(LevelEvent::Note { pos, sound, note });
        true
    }
}

/// The sound a note block's instrument plays (`NoteBlockInstrument.getSoundEvent`); none for
/// `custom_head`, whose sound is its head's.
fn note_block_sound(instrument: &str) -> Option<&'static str> {
    #[rustfmt::skip]
    const SOUNDS: [&str; 26] = [
        "harp", "basedrum", "snare", "hat", "bass", "flute", "bell", "guitar", "chime", "xylophone",
        "iron_xylophone", "cow_bell", "didgeridoo", "bit", "banjo", "pling", "trumpet",
        "trumpet_exposed", "trumpet_oxidized", "trumpet_weathered", "imitate.zombie",
        "imitate.skeleton", "imitate.creeper", "imitate.ender_dragon", "imitate.wither_skeleton",
        "imitate.piglin",
    ];
    #[rustfmt::skip]
    const EVENTS: [&str; 26] = [
        "minecraft:block.note_block.harp", "minecraft:block.note_block.basedrum",
        "minecraft:block.note_block.snare", "minecraft:block.note_block.hat",
        "minecraft:block.note_block.bass", "minecraft:block.note_block.flute",
        "minecraft:block.note_block.bell", "minecraft:block.note_block.guitar",
        "minecraft:block.note_block.chime", "minecraft:block.note_block.xylophone",
        "minecraft:block.note_block.iron_xylophone", "minecraft:block.note_block.cow_bell",
        "minecraft:block.note_block.didgeridoo", "minecraft:block.note_block.bit",
        "minecraft:block.note_block.banjo", "minecraft:block.note_block.pling",
        "minecraft:block.note_block.trumpet", "minecraft:block.note_block.trumpet_exposed",
        "minecraft:block.note_block.trumpet_oxidized",
        "minecraft:block.note_block.trumpet_weathered",
        "minecraft:block.note_block.imitate.zombie", "minecraft:block.note_block.imitate.skeleton",
        "minecraft:block.note_block.imitate.creeper",
        "minecraft:block.note_block.imitate.ender_dragon",
        "minecraft:block.note_block.imitate.wither_skeleton",
        "minecraft:block.note_block.imitate.piglin",
    ];
    let name = match instrument {
        "dragon" => "imitate.ender_dragon",
        "zombie" => "imitate.zombie",
        "skeleton" => "imitate.skeleton",
        "creeper" => "imitate.creeper",
        "wither_skeleton" => "imitate.wither_skeleton",
        "piglin" => "imitate.piglin",
        other => other,
    };
    SOUNDS
        .iter()
        .position(|sound| *sound == name)
        .map(|index| EVENTS[index])
}

/// The level events waiting for the particle engine. Which are waiting is no part of what a world is
/// equal to.
#[derive(Debug, Clone, Default)]
pub struct LevelEventQueue(pub(crate) Vec<LevelEvent>);

impl PartialEq for LevelEventQueue {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for LevelEventQueue {}

/// What the level plays when the particle engine takes it, in the order it came.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelEvent {
    /// One `ClientboundLevelEventPacket` (or a level event of the client's own).
    Level {
        event: i32,
        pos: (i32, i32, i32),
        data: i32,
    },
    /// A note block struck: its instrument's sound, and its note when it is tuned.
    Note {
        pos: (i32, i32, i32),
        sound: &'static str,
        note: Option<i32>,
    },
    /// An explosion the server reports (`ClientboundExplodePacket`).
    Explosion(Box<crate::particles::Explosion>),
}

impl LevelEvent {
    /// The sound of a block broken (2001): its break sound, louder and lower, at its middle;
    /// nothing for air or another event.
    pub fn break_sound(&self) -> Option<local_player::LocalSound> {
        let Self::Level {
            event: 2001,
            pos,
            data,
        } = *self
        else {
            return None;
        };
        if block::state_or_air(i64::from(data)).block().name == "minecraft:air" {
            return None;
        }
        block_sound_of(
            pos,
            data,
            photon_data::block_sound::SoundTypeRow::break_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        )
    }
}

/// The sound a block makes (`SoundType`) at its middle: its volume and pitch scaled as the event
/// asks, whichever of the five sounds `pick` takes. Nothing for a state with no sound type.
pub fn block_sound_of(
    pos: (i32, i32, i32),
    state: i32,
    pick: fn(&photon_data::block_sound::SoundTypeRow) -> &'static str,
    (volume, pitch): (fn(f32) -> f32, f32),
) -> Option<local_player::LocalSound> {
    let id = i64::from(state);
    let info = block::state_or_air(id);
    let sound_type = u32::try_from(id)
        .ok()
        .and_then(|id| photon_data::block_sound::of_state(info.block().name, id))?;
    Some(local_player::LocalSound {
        event: pick(sound_type),
        source: local_player::LocalSoundSource::Blocks,
        volume: volume(sound_type.volume),
        pitch: sound_type.pitch * pitch,
        position: (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        ),
    })
}
