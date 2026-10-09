//! What a horse, donkey, mule, undead horse, llama or camel does of its own on
//! the client (`AbstractHorse.tick`): the grazing, rearing and open-mouth
//! animations follow the flags the server sends, and the tail swishes now and
//! then.

use photon_core::random::RandomSource;

use crate::animation_state::AnimationState;
use photon_core::java_float::JavaF32;

/// `AbstractHorse.DATA_ID_FLAGS` bits.
const FLAG_TAME: u8 = 2;
const FLAG_EATING: u8 = 16;
const FLAG_STANDING: u8 = 32;
const FLAG_OPEN_MOUTH: u8 = 64;

/// `AbstractHorse.getAmbientSoundInterval`.
const AMBIENT_SOUND_INTERVAL: i32 = 400;

/// The animal's tail counter, which counts a swish out.
const TAIL_SWISH_TICKS: i32 = 8;

/// The source `aiStep` draws the tail's chance from: the entity's own, which
/// the reference does not seed. Two states never differ by it.
#[derive(Debug, Clone)]
struct TailRandom(RandomSource);

impl PartialEq for TailRandom {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for TailRandom {}

/// A camel's own client state (`Camel.setupAnimationStates` and what it reads):
/// five animation states, the idle timer, the dash flag the server sent and
/// the tick the pose last changed (negative while sitting).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CamelState {
    pub sit: AnimationState,
    pub sit_pose: AnimationState,
    pub sit_up: AnimationState,
    pub idle: AnimationState,
    pub dash: AnimationState,
    idle_timeout: i32,
    dashing: bool,
    last_pose_change_tick: i64,
    /// `Camel.dashCooldown`: ticks until it can dash again.
    dash_cooldown: i32,
}

/// How long a camel takes to sit down and to stand up, in ticks.
const SIT_DOWN_TICKS: i64 = 40;
const STAND_UP_TICKS: i64 = 52;

impl CamelState {
    pub(crate) fn set_dashing(&mut self, dashing: bool) {
        self.dashing = dashing;
    }

    pub(crate) fn set_last_pose_change_tick(&mut self, tick: i64) {
        self.last_pose_change_tick = tick;
    }

    /// `Camel.dashCooldown` (also `getJumpCooldown`).
    pub fn dash_cooldown(&self) -> i32 {
        self.dash_cooldown
    }

    pub(crate) fn set_dash_cooldown(&mut self, ticks: i32) {
        self.dash_cooldown = ticks;
    }

    /// `isCamelSitting`.
    pub fn sitting(&self) -> bool {
        self.last_pose_change_tick < 0
    }

    /// `Camel.refuseToMove`: sitting, or sitting down or standing up.
    pub fn refuses_to_move(&self, game_time: i64) -> bool {
        self.sitting() || self.in_pose_transition(game_time)
    }

    /// `Camel.standUp`: from sitting, it rises.
    pub(crate) fn stand_up(&mut self, game_time: i64) {
        if self.sitting() {
            self.last_pose_change_tick = game_time;
        }
    }

    /// `Camel.standUpInstantly`: standing, and for a while long enough to have stood.
    pub(crate) fn stand_up_instantly(&mut self, game_time: i64) {
        self.last_pose_change_tick = (game_time - 52 - 1).max(0);
    }

    /// `Camel.getBodyAnchorAnimationYOffset`'s pose: whether it sits and for how long.
    pub fn pose_for_anchor(&self, game_time: i64) -> (bool, i64) {
        (self.sitting(), self.pose_time(game_time))
    }

    /// `getPoseTime`: the game time since the pose last changed.
    fn pose_time(&self, game_time: i64) -> i64 {
        game_time.wrapping_sub(self.last_pose_change_tick.wrapping_abs())
    }

    /// Whether the camel sits and the game time since its pose last changed, as its leash
    /// offset reads them.
    pub fn leash_pose(&self, game_time: i64) -> (bool, i64) {
        (self.sitting(), self.pose_time(game_time))
    }

    /// `isInPoseTransition`.
    pub fn in_pose_transition(&self, game_time: i64) -> bool {
        self.pose_time(game_time)
            < if self.sitting() {
                SIT_DOWN_TICKS
            } else {
                STAND_UP_TICKS
            }
    }

    /// Whether the server has the camel dashing, as the client last had it.
    pub fn is_dashing(&self) -> bool {
        self.dashing
    }

    /// Whether the dash animation is running (`dashAnimationState.isStarted`),
    /// which keeps the walk still.
    pub fn is_dash_animation_started(&self) -> bool {
        self.dash.is_started()
    }

    /// `Camel.tick` after the horse's: a dash ends when the camel is on the
    /// ground or rides, then the animation states follow the pose.
    pub(crate) fn tick(
        &mut self,
        random: &mut RandomSource,
        (tick_count, game_time): (i32, i64),
        (on_ground, in_liquid, passenger): (bool, bool, bool),
    ) {
        if self.dashing && self.dash_cooldown < 50 && (on_ground || in_liquid || passenger) {
            self.dashing = false;
        }
        if self.dash_cooldown > 0 {
            self.dash_cooldown -= 1;
        }
        if self.idle_timeout <= 0 {
            self.idle_timeout = random.next_int_bound(40).unwrap_or(0) + 80;
            self.idle.start(tick_count);
        } else {
            self.idle_timeout -= 1;
        }
        let pose_time = self.pose_time(game_time);
        let sitting = self.sitting();
        if (pose_time < 0) != sitting {
            self.sit_up.stop();
            self.dash.stop();
            if sitting && (0..SIT_DOWN_TICKS).contains(&pose_time) {
                self.sit.start_if_stopped(tick_count);
                self.sit_pose.stop();
            } else {
                self.sit.stop();
                self.sit_pose.start_if_stopped(tick_count);
            }
        } else {
            self.sit.stop();
            self.sit_pose.stop();
            self.dash.animate_when(self.dashing, tick_count);
            self.sit_up.animate_when(
                self.in_pose_transition(game_time) && pose_time >= 0,
                tick_count,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorseState {
    pub camel: CamelState,
    flags: u8,
    eat: JavaF32,
    eat_old: JavaF32,
    stand: JavaF32,
    stand_old: JavaF32,
    mouth: JavaF32,
    mouth_old: JavaF32,
    tail_counter: i32,
    /// `AbstractHorse.standCounter`: ticks left of a rear, and `allowStandSliding`: a rider's jump
    /// let it rear while it is held still.
    stand_counter: i32,
    allow_stand_sliding: bool,
    /// `Mob.ambientSoundTime`.
    ambient_sound_time: i32,
    random: TailRandom,
}

fn lerp(old: JavaF32, new: JavaF32, partial: f32) -> f32 {
    let (old, new) = (old.get(), new.get());
    old + partial * (new - old)
}

impl HorseState {
    /// An animal that has just appeared: its random source seeded by the clock.
    pub(crate) fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        Self {
            camel: CamelState::default(),
            flags: 0,
            eat: JavaF32::new(0.0),
            eat_old: JavaF32::new(0.0),
            stand: JavaF32::new(0.0),
            stand_old: JavaF32::new(0.0),
            mouth: JavaF32::new(0.0),
            mouth_old: JavaF32::new(0.0),
            tail_counter: 0,
            stand_counter: 0,
            allow_stand_sliding: false,
            ambient_sound_time: 0,
            random: TailRandom(RandomSource::create(seed)),
        }
    }

    /// Fixes the source the tail's chance is drawn from, so a comparison can
    /// be made against the reference (which does not seed it).
    pub fn fix_seed(&mut self, seed: i64) {
        self.random.0.set_seed(seed);
    }

    pub(crate) fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    /// `AbstractHorse.standIfPossible` on a client that is its authority: it rears for twenty ticks
    /// (`setStanding`, which also stops its grazing).
    pub(crate) fn stand_if_possible(&mut self) {
        self.flags &= !FLAG_EATING;
        self.flags |= FLAG_STANDING;
        self.stand_counter = 20;
    }

    /// `AbstractHorse.allowStandSliding`.
    pub fn allow_stand_sliding(&self) -> bool {
        self.allow_stand_sliding
    }

    pub(crate) fn set_allow_stand_sliding(&mut self, allow: bool) {
        self.allow_stand_sliding = allow;
    }

    /// One client tick: `aiStep` may start a tail swish; `tick` counts it out
    /// and moves the three animations.
    pub(crate) fn tick(&mut self, alive: bool) {
        // `Mob.baseTick` first asks, with the same source, whether the animal
        // would make an ambient sound; the sound itself is not played here, but
        // choosing its pitch takes two draws.
        if alive {
            let chance = self.random.0.next_int_bound(1000).unwrap_or(0);
            let before = self.ambient_sound_time;
            self.ambient_sound_time += 1;
            if chance < before {
                self.ambient_sound_time = -AMBIENT_SOUND_INTERVAL;
                let _ = self.random.0.next_float();
                let _ = self.random.0.next_float();
            }
        }
        if self.random.0.next_int_bound(200) == Ok(0) {
            self.tail_counter = 1;
        }
        if self.tail_counter > 0 {
            self.tail_counter += 1;
            if self.tail_counter > TAIL_SWISH_TICKS {
                self.tail_counter = 0;
            }
        }
        // `tick`: a rear counts out.
        if self.stand_counter > 0 {
            self.stand_counter -= 1;
            if self.stand_counter <= 0 {
                self.flags &= !FLAG_STANDING;
                self.stand_counter = 0;
            }
        }
        let (eat, stand, mouth) = (self.eat.get(), self.stand.get(), self.mouth.get());
        self.eat_old = self.eat;
        let mut eat = if self.flags & FLAG_EATING != 0 {
            (eat + ((1.0 - eat) * 0.4 + 0.05)).min(1.0)
        } else {
            (eat + ((0.0 - eat) * 0.4 - 0.05)).max(0.0)
        };
        self.stand_old = self.stand;
        let stand = if self.flags & FLAG_STANDING != 0 {
            eat = 0.0;
            self.eat_old = JavaF32::new(eat);
            (stand + ((1.0 - stand) * 0.4 + 0.05)).min(1.0)
        } else {
            self.allow_stand_sliding = false;
            (stand + ((0.8 * stand * stand * stand - stand) * 0.6 - 0.05)).max(0.0)
        };
        self.eat = JavaF32::new(eat);
        self.stand = JavaF32::new(stand);
        self.mouth_old = self.mouth;
        let mouth = if self.flags & FLAG_OPEN_MOUTH != 0 {
            (mouth + ((1.0 - mouth) * 0.7 + 0.05)).min(1.0)
        } else {
            (mouth + ((0.0 - mouth) * 0.7 - 0.05)).max(0.0)
        };
        self.mouth = JavaF32::new(mouth);
    }

    /// A camel's own tick, after the horse's (see [`CamelState::tick`]).
    pub(crate) fn tick_camel(
        &mut self,
        (tick_count, game_time): (i32, i64),
        (on_ground, in_liquid, passenger): (bool, bool, bool),
    ) {
        self.camel.tick(
            &mut self.random.0,
            (tick_count, game_time),
            (on_ground, in_liquid, passenger),
        );
    }

    /// `getEatAnim`: how far the head is down to graze, between two ticks.
    pub fn eat_animation(&self, partial: f32) -> f32 {
        lerp(self.eat_old, self.eat, partial)
    }

    /// `standAnimO`: how far the animal was reared a tick ago, which a rider's seat reads.
    pub fn stand_animation_old(&self) -> f32 {
        self.stand_old.get()
    }

    /// Whether the animal rears (`AbstractHorse.isStanding`).
    /// `AbstractHorse.isTamed`.
    pub fn is_tamed(&self) -> bool {
        self.flags & FLAG_TAME != 0
    }

    pub fn is_standing(&self) -> bool {
        self.flags & FLAG_STANDING != 0
    }

    /// `getStandAnim`: how far the animal is reared.
    pub fn stand_animation(&self, partial: f32) -> f32 {
        lerp(self.stand_old, self.stand, partial)
    }

    /// `getMouthAnim`: how far the mouth is open.
    pub fn mouth_animation(&self, partial: f32) -> f32 {
        lerp(self.mouth_old, self.mouth, partial)
    }

    /// `tailCounter`: above zero while the tail swishes.
    pub fn tail_counter(&self) -> i32 {
        self.tail_counter
    }
}
