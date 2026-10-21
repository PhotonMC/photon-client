//! The sound of eating and drinking (`Consumable.shouldEmitParticlesAndSounds` and
//! `emitParticlesAndSounds`, driven from `ItemStack.onUseTick`). The completion's own emit
//! (`onConsume`) runs on the server only, which sends that one on.

use super::*;

/// What the consumable of a stack in use sounds like.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeCue {
    /// `consumeTicks`: the seconds of the component in ticks.
    pub ticks: i32,
    /// The animation is the drinking one rather than the eating one.
    pub drink: bool,
    pub sound: &'static str,
    /// `hasConsumeParticles`.
    pub particles: bool,
}

impl ConsumeCue {
    /// `Consumable.shouldEmitParticlesAndSounds`: every fourth tick left, once the first fifth
    /// (a little more) of the time has passed. `remaining` counts down from the duration, and
    /// keeps counting down until the server ends the use.
    pub fn emits(&self, used_ticks: i32) -> bool {
        let wait = (self.ticks as f32 * 0.21875_f32) as i32;
        used_ticks > wait && (self.ticks - used_ticks) % 4 == 0
    }
}

/// `ItemStack.onUseTick` of the item the player is using, as far as it sounds.
impl LocalPlayer {
    pub(super) fn consume_tick(&mut self) {
        let cue = if self.hands.using_off_hand {
            self.hands.off_consume
        } else {
            self.hands.main_consume
        };
        let Some(cue) = cue.filter(|cue| cue.emits(self.use_edge.ticks)) else {
            return;
        };
        // `emitParticlesAndSounds`: the volumes and pitches of both kinds are drawn, then the
        // crumbs (five of them, four draws each).
        let eat_volume = if self.random.next_boolean() { 0.5 } else { 1.0 };
        let eat_pitch = self.random.triangle_f32(1.0, 0.2);
        let drink_pitch = self.random.next_float() * (1.0_f32 - 0.9_f32) + 0.9_f32;
        if cue.particles {
            for _ in 0..5 * 4 {
                self.random.next_float();
            }
        }
        let (volume, pitch) = if cue.drink {
            (0.5, drink_pitch)
        } else {
            (eat_volume, eat_pitch)
        };
        let at = self.position;
        self.pending_sounds.push(LocalSound::at(
            cue.sound,
            LocalSoundSource::Players,
            (volume, pitch),
            (at.x, at.y, at.z),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bread() -> ConsumeCue {
        ConsumeCue {
            ticks: 32,
            drink: false,
            sound: "minecraft:entity.generic.eat",
            particles: true,
        }
    }

    /// 32 ticks: the wait is 7 (`(int)(32 * 0.21875)`), so the first emit is the first tick of
    /// a multiple of four left after more than 7 have passed: 8 used, 24 left. It goes on past
    /// the end (negative ticks left) until the server ends the use.
    #[test]
    fn eating_sounds_every_fourth_tick_after_the_first_seventh() {
        let used: Vec<i32> = (0..40).filter(|used| bread().emits(*used)).collect();
        assert_eq!(used, [8, 12, 16, 20, 24, 28, 32, 36]);
    }

    fn using(cue: ConsumeCue) -> LocalPlayer {
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::ZERO,
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        player.hands.using_item = true;
        player.hands.main_consume = Some(cue);
        player
    }

    fn sounded(player: &mut LocalPlayer, ticks: i32) -> Vec<LocalSound> {
        for used in 0..ticks {
            player.use_edge.ticks = used;
            player.consume_tick();
        }
        player.take_sounds()
    }

    /// Eating is heard at the component's sound with half or full volume and a pitch near one;
    /// drinking at half volume and a pitch between 0.9 and one.
    #[test]
    fn the_volume_and_pitch_follow_the_animation() {
        let eaten = sounded(&mut using(bread()), 33);
        assert_eq!(eaten.len(), 7);
        for sound in &eaten {
            assert_eq!(sound.event, "minecraft:entity.generic.eat");
            assert!(sound.volume == 0.5 || sound.volume == 1.0);
            assert!((0.8..=1.2).contains(&sound.pitch));
            assert_eq!(sound.source, LocalSoundSource::Players);
        }
        let drink = ConsumeCue {
            drink: true,
            sound: "minecraft:entity.generic.drink",
            particles: false,
            ..bread()
        };
        for sound in sounded(&mut using(drink), 33) {
            assert_eq!(sound.volume, 0.5);
            assert!((0.9..1.0).contains(&sound.pitch));
        }
    }

    /// Nothing sounds when nothing is being used or the hand holds no consumable.
    #[test]
    fn a_hand_without_a_consumable_is_silent() {
        let mut player = using(bread());
        player.hands.main_consume = None;
        assert!(sounded(&mut player, 40).is_empty());
    }
}
