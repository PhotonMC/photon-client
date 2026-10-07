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

    /// The sound a block makes (`SoundType`) at its middle: its volume and pitch scaled as the event
    /// asks, whichever of the five sounds `pick` takes. Nothing for a state with no sound type.
    pub(crate) fn block_sound(
        &mut self,
        pos: (i32, i32, i32),
        state: i32,
        pick: fn(&voxel_data::block_sound::SoundTypeRow) -> &'static str,
        (volume, pitch): (fn(f32) -> f32, f32),
    ) {
        let id = i64::from(state);
        let info = block::state_or_air(id);
        let Some(sound_type) = u32::try_from(id)
            .ok()
            .and_then(|id| voxel_data::block_sound::of_state(info.block().name, id))
        else {
            return;
        };
        self.sounds.0.push(local_player::LocalSound {
            event: pick(sound_type),
            source: local_player::LocalSoundSource::Blocks,
            volume: volume(sound_type.volume),
            pitch: sound_type.pitch * pitch,
            position: (
                f64::from(pos.0) + 0.5,
                f64::from(pos.1) + 0.5,
                f64::from(pos.2) + 0.5,
            ),
        });
    }

    /// `BlockItem.place`'s sound: the block placed, a little softer than loud and lower.
    pub fn play_place_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            voxel_data::block_sound::SoundTypeRow::place_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        );
    }

    /// `MultiPlayerGameMode.continueDestroyBlock`'s hit sound: a block struck while it is broken.
    pub fn play_hit_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            voxel_data::block_sound::SoundTypeRow::hit_sound,
            (|volume| (volume + 1.0) / 8.0, 0.5),
        );
    }

    /// `LevelEventHandler.levelEvent`, for the events that make a sound the client knows: `2001`,
    /// a block broken (its state's number is the data).
    pub fn apply_level_event(&mut self, event: i32, pos: (i32, i32, i32), data: i32) {
        if event == 2001 && block::state_or_air(i64::from(data)).block().name != "minecraft:air" {
            self.block_sound(
                pos,
                data,
                voxel_data::block_sound::SoundTypeRow::break_sound,
                (|volume| (volume + 1.0) / 2.0, 0.8),
            );
        }
    }
}
