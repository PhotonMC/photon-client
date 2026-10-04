//! What a guardian does of its own on the client (`Guardian.aiStep`): its
//! tail's sway, the spikes' breathing, and how long it has aimed its beam.
//! What the reference draws from its source for particles and for a flop on
//! the ground is not drawn here, so the spikes out of water follow it only
//! while the guardian has neither.

use voxel_core::math::mth_lerp_f32;
use voxel_core::random::RandomSource;

/// The source the tail's start and the spikes out of water are drawn from: the
/// entity's own, which the reference does not seed. Two states never differ by it.
#[derive(Debug, Clone)]
struct EntityRandom(RandomSource);

impl PartialEq for EntityRandom {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for EntityRandom {}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardianState {
    tail_animation: f32,
    tail_animation_old: f32,
    tail_animation_speed: f32,
    spikes_animation: f32,
    spikes_animation_old: f32,
    attack_time: i32,
    random: EntityRandom,
}

/// The animations are finite, so the comparison is a total one.
impl Eq for GuardianState {}

impl GuardianState {
    /// A guardian that has just appeared: its tail starts where a source
    /// seeded by the clock says (the reference draws it, unseeded, in the
    /// constructor).
    pub(crate) fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        let mut random = RandomSource::create(seed);
        let tail = random.next_float();
        Self {
            tail_animation: tail,
            tail_animation_old: tail,
            tail_animation_speed: 0.0,
            spikes_animation: 0.0,
            spikes_animation_old: 0.0,
            attack_time: 0,
            random: EntityRandom(random),
        }
    }

    /// Fixes where the tail starts and the source the spikes are drawn from,
    /// which the reference draws from a source it does not seed, so a
    /// comparison can be made.
    pub fn fix_random(&mut self, tail: f32, seed: i64) {
        self.tail_animation = tail;
        self.tail_animation_old = tail;
        self.random.0.set_seed(seed);
    }

    /// `onSyncedDataUpdated(DATA_ID_ATTACK_TARGET)`: the beam starts over.
    pub(crate) fn restart_attack(&mut self) {
        self.attack_time = 0;
    }

    /// `Guardian.aiStep` on the client, while it lives.
    pub(crate) fn tick(
        &mut self,
        (in_water, moving, has_target): (bool, bool, bool),
        attack_duration: i32,
    ) {
        // `Mob.baseTick` draws the chance of an ambient sound from the same
        // source each tick, before `aiStep`.
        let _ = self.random.0.next_int_bound(1000);
        self.tail_animation_old = self.tail_animation;
        if !in_water {
            self.tail_animation_speed = 2.0;
        } else if moving {
            if self.tail_animation_speed < 0.5 {
                self.tail_animation_speed = 4.0;
            } else {
                self.tail_animation_speed += (0.5 - self.tail_animation_speed) * 0.1;
            }
        } else {
            self.tail_animation_speed += (0.125 - self.tail_animation_speed) * 0.2;
        }
        self.tail_animation += self.tail_animation_speed;
        self.spikes_animation_old = self.spikes_animation;
        if !in_water {
            self.spikes_animation = self.random.0.next_float();
        } else if moving {
            self.spikes_animation += (0.0 - self.spikes_animation) * 0.25;
        } else {
            self.spikes_animation += (1.0 - self.spikes_animation) * 0.06;
        }
        if has_target && self.attack_time < attack_duration {
            self.attack_time += 1;
        }
    }

    /// `getTailAnimation`.
    pub fn tail_animation(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(partial_tick, self.tail_animation_old, self.tail_animation)
    }

    /// `getSpikesAnimation`.
    pub fn spikes_animation(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(
            partial_tick,
            self.spikes_animation_old,
            self.spikes_animation,
        )
    }

    /// `getAttackAnimationScale`: how far the beam has charged, from zero to one.
    pub fn attack_animation_scale(&self, partial_tick: f32, attack_duration: i32) -> f32 {
        (self.attack_time as f32 + partial_tick) / attack_duration as f32
    }

    /// `getClientSideAttackTime`.
    pub fn attack_time(&self) -> i32 {
        self.attack_time
    }
}
