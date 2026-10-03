//! What a copper golem does of its own on the client: the animation states its
//! `CopperGolemState` data runs (`CopperGolem.setupAnimationStates`), and the
//! wait before the idle head spin that the reference draws from an unseeded
//! source.

use voxel_core::random::RandomSource;

use crate::animation_state::AnimationState;
use crate::rabbit::IdleRandom;

/// `AbstractGolem.getAmbientSoundInterval`.
const AMBIENT_SOUND_INTERVAL: i32 = 120;

/// `CopperGolemState`: what the golem is doing at a chest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GolemAction {
    #[default]
    Idle,
    GettingItem,
    GettingNoItem,
    DroppingItem,
    DroppingNoItem,
}

impl GolemAction {
    /// `CopperGolemState.BY_ID`: an id out of range is the first.
    pub(crate) fn from_id(id: i32) -> Self {
        match id {
            1 => Self::GettingItem,
            2 => Self::GettingNoItem,
            3 => Self::DroppingItem,
            4 => Self::DroppingNoItem,
            _ => Self::Idle,
        }
    }
}

/// The five animations of the model, each with its own start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CopperGolemPlaying {
    pub idle: AnimationState,
    pub get_item: AnimationState,
    pub get_no_item: AnimationState,
    pub drop_item: AnimationState,
    pub drop_no_item: AnimationState,
}

/// The states and what drives them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperGolemAnimations {
    pub playing: CopperGolemPlaying,
    idle_animation_start_tick: i32,
    /// `Mob.ambientSoundTime`.
    ambient_sound_time: i32,
    random: IdleRandom,
}

impl CopperGolemAnimations {
    /// A golem that has just appeared, with a source seeded by the clock.
    pub(crate) fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        Self {
            playing: CopperGolemPlaying::default(),
            idle_animation_start_tick: 0,
            ambient_sound_time: 0,
            random: IdleRandom::new(RandomSource::create(seed)),
        }
    }

    /// Fixes the source the idle wait is drawn from, so a comparison can be made.
    pub fn fix_seed(&mut self, seed: i64) {
        self.random.set_seed(seed);
    }

    /// One client tick: `Mob.baseTick` first asks, with the entity's source,
    /// whether the golem would make an ambient sound (it has none, so nothing
    /// else is drawn), then `setupAnimationStates` runs (not without AI).
    pub(crate) fn tick(
        &mut self,
        action: GolemAction,
        tick_count: i32,
        (no_ai, alive): (bool, bool),
    ) {
        if alive {
            let chance = self.random.next_int_bound(1000).unwrap_or(0);
            let before = self.ambient_sound_time;
            self.ambient_sound_time += 1;
            if chance < before {
                self.ambient_sound_time = -AMBIENT_SOUND_INTERVAL;
            }
        }
        if no_ai {
            return;
        }
        let interactions = [
            (GolemAction::GettingItem, &mut self.playing.get_item),
            (GolemAction::GettingNoItem, &mut self.playing.get_no_item),
            (GolemAction::DroppingItem, &mut self.playing.drop_item),
            (GolemAction::DroppingNoItem, &mut self.playing.drop_no_item),
        ];
        for (own, state) in interactions {
            if own == action {
                state.start_if_stopped(tick_count);
            } else {
                state.stop();
            }
        }
        if action != GolemAction::Idle {
            self.playing.idle.stop();
            self.idle_animation_start_tick = 0;
            return;
        }
        if self.idle_animation_start_tick == tick_count {
            self.playing.idle.start(tick_count);
        } else if self.idle_animation_start_tick == 0 {
            self.idle_animation_start_tick =
                tick_count + self.random.next_int_range(200, 240).unwrap_or(200);
        }
        // `tickCount == idleAnimationStartTick + 10.0F`, compared as floats.
        if tick_count as f32 == self.idle_animation_start_tick as f32 + 10.0 {
            // The head spin's sound would play here.
            self.idle_animation_start_tick = 0;
        }
    }
}
