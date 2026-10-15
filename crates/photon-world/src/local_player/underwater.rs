//! The sounds of being under water (`LocalPlayer.updateIsUnderwater`,
//! `UnderwaterAmbientSoundHandler`): a sound for getting in and out, the loop that fades in while
//! under water, and the additions that come by chance.

use super::{LocalPlayer, LocalSound, LocalSoundSource};

/// A sound under water that has a life of its own in the sound engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderwaterSound {
    /// `UnderwaterAmbientSoundInstance`: the loop that fades with the water.
    Loop,
    /// `UnderwaterAmbientSoundInstances.SubSound`: one of the additions, over when the water is left.
    Addition(&'static str),
}

/// What the player keeps of its ambient sounds: the under-water ones and, for
/// `BubbleColumnAmbientSoundHandler`, whether the last tick was in a bubble column and whether
/// a tick has gone by.
#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    /// `LocalPlayer.wasUnderwater`.
    pub(super) was: bool,
    /// `UnderwaterAmbientSoundHandler.tickDelay`.
    pub(super) delay: i32,
    /// The sounds waiting for the engine.
    pub(super) pending: Vec<UnderwaterSound>,
    pub(super) in_bubble_column: bool,
    pub(super) started: bool,
}

impl LocalPlayer {
    /// `Player.updateIsUnderwater` with `LocalPlayer`'s sounds for the change.
    pub(super) fn update_is_underwater(&mut self) {
        let was = self.underwater.was;
        self.underwater.was = self.water.eyes_inside;
        if self.is_spectator() {
            return;
        }
        if !was && self.underwater.was {
            self.play_underwater_change("minecraft:ambient.underwater.enter");
            self.underwater.pending.push(UnderwaterSound::Loop);
        }
        if was && !self.underwater.was {
            self.play_underwater_change("minecraft:ambient.underwater.exit");
        }
    }

    fn play_underwater_change(&mut self, event: &'static str) {
        self.pending_sounds.push(LocalSound {
            event,
            source: LocalSoundSource::Ambient,
            volume: 1.0,
            pitch: 1.0,
            position: (self.position.x, self.position.y, self.position.z),
        });
    }

    /// `UnderwaterAmbientSoundHandler.tick`.
    pub(super) fn tick_underwater_additions(&mut self) {
        self.underwater.delay -= 1;
        if self.underwater.delay > 0 || !self.is_under_water() {
            return;
        }
        let chance = self.random.next_float();
        let event = if chance < 1.0e-4 {
            "minecraft:ambient.underwater.loop.additions.ultra_rare"
        } else if chance < 0.001 {
            "minecraft:ambient.underwater.loop.additions.rare"
        } else if chance < 0.01 {
            "minecraft:ambient.underwater.loop.additions"
        } else {
            return;
        };
        self.underwater.delay = 0;
        self.underwater
            .pending
            .push(UnderwaterSound::Addition(event));
    }

    /// The under-water sounds started since the last call.
    pub fn take_underwater_sounds(&mut self) -> Vec<UnderwaterSound> {
        std::mem::take(&mut self.underwater.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::ids::EntityId;

    #[test]
    fn getting_in_the_water_plays_the_sound_and_starts_the_loop() {
        let mut player = LocalPlayer::new(
            EntityId(1),
            photon_core::math::Vec3::ZERO,
            0.0,
            0.0,
            crate::ClientPlayerGameMode::Survival,
            false,
        );
        player.water.eyes_inside = true;
        player.update_is_underwater();
        let sounds = player.take_sounds();
        assert_eq!(sounds[0].event, "minecraft:ambient.underwater.enter");
        assert_eq!(player.take_underwater_sounds(), [UnderwaterSound::Loop]);
        player.water.eyes_inside = false;
        player.update_is_underwater();
        assert_eq!(
            player.take_sounds()[0].event,
            "minecraft:ambient.underwater.exit"
        );
    }
}
