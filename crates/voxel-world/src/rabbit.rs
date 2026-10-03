//! What a rabbit does of its own on the client: its hop (`jumpTicks`,
//! `jumpDuration`), and the animation states the hop and the idle head tilt
//! run on (`Rabbit.setupAnimationStates`).

use voxel_core::random::RandomSource;

use crate::animation_state::AnimationState;

/// How long a hop lasts, in ticks.
const JUMP_DURATION: i32 = 15;
/// How long a rabbit waits at least before it tilts its head.
const IDLE_MINIMAL_DURATION_TICKS: i32 = 180;

/// The source the idle timeout is drawn from: the entity's own, which the
/// reference does not seed. Two states never differ by it.
#[derive(Debug, Clone)]
struct IdleRandom(RandomSource);

impl PartialEq for IdleRandom {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for IdleRandom {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RabbitState {
    pub hop: AnimationState,
    pub idle_head_tilt: AnimationState,
    jump_ticks: i32,
    jump_duration: i32,
    idle_animation_timeout: i32,
    random: IdleRandom,
}

impl RabbitState {
    /// A rabbit that has just appeared: its idle timeout drawn from a source
    /// seeded by the clock.
    pub(crate) fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        let mut random = RandomSource::create(seed);
        let timeout = IDLE_MINIMAL_DURATION_TICKS + random.next_int_bound(40).unwrap_or(0);
        Self {
            hop: AnimationState::default(),
            idle_head_tilt: AnimationState::default(),
            jump_ticks: 0,
            jump_duration: 0,
            idle_animation_timeout: timeout,
            random: IdleRandom(random),
        }
    }

    /// Fixes what the reference draws from an unseeded source (the wait before
    /// the head tilt and the source after it), so a comparison can be made.
    pub fn fix_idle(&mut self, timeout: i32, seed: i64) {
        self.idle_animation_timeout = timeout;
        self.random.0.set_seed(seed);
    }

    /// `Rabbit.handleEntityEvent(1)`: a hop starts.
    pub(crate) fn start_hop(&mut self) {
        self.jump_duration = JUMP_DURATION;
        self.jump_ticks = 0;
    }

    /// One client tick: `baseTick` sets the animation states, then `aiStep`
    /// moves the hop along.
    pub(crate) fn tick(&mut self, tick_count: i32, no_ai: bool) {
        // No leash is tracked, so a rabbit is never held by one.
        if self.idle_animation_timeout <= 0 && !no_ai {
            self.idle_animation_timeout =
                IDLE_MINIMAL_DURATION_TICKS + self.random.0.next_int_bound(40).unwrap_or(0);
            self.idle_head_tilt.start(tick_count);
        } else if self.jump_ticks > 0 {
            self.hop.start_if_stopped(tick_count);
            self.idle_head_tilt.stop();
        } else {
            self.idle_animation_timeout -= 1;
            self.hop.stop();
        }
        if self.jump_ticks != self.jump_duration {
            self.jump_ticks += 1;
        } else if self.jump_duration != 0 {
            self.jump_ticks = 0;
            self.jump_duration = 0;
        }
    }

    /// `Rabbit.getJumpCompletion`.
    pub fn jump_completion(&self, partial: f32) -> f32 {
        if self.jump_duration == 0 {
            0.0
        } else {
            (self.jump_ticks as f32 + partial) / self.jump_duration as f32
        }
    }
}
