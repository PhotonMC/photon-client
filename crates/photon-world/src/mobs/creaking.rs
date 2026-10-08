//! What a creaking does of its own on the client: the animation states its
//! ticks keep (`Creaking.setupAnimationStates`), the events that start its
//! attack and its sway, and the blink of its eyes while it is torn down.

use photon_core::random::RandomSource;

use crate::animation_state::AnimationState;

/// `Creaking`'s data ids: `Mob` ends at 15, then the four it defines.
pub const CAN_MOVE_DATA_ID: u8 = 16;
pub const IS_ACTIVE_DATA_ID: u8 = 17;
pub const IS_TEARING_DOWN_DATA_ID: u8 = 18;
pub const HOME_POS_DATA_ID: u8 = 19;

/// The events a creaking answers itself: the attack and the sway of a hurt one.
pub const ATTACK_EVENT: i8 = 4;
pub const INVULNERABLE_EVENT: i8 = 66;

const ATTACK_ANIMATION_DURATION: i32 = 15;
const INVULNERABILITY_ANIMATION_DURATION: i32 = 8;

#[derive(Debug, Clone)]
pub struct CreakingState {
    pub attack: AnimationState,
    pub invulnerable: AnimationState,
    pub death: AnimationState,
    attack_remaining: i32,
    invulnerable_remaining: i32,
    pub(crate) can_move: bool,
    pub(crate) active: bool,
    pub(crate) tearing_down: bool,
    /// `Creaking.isHeartBound`: it has a home position.
    pub(crate) heart_bound: bool,
    eyes_glowing: bool,
    next_flicker_time: i32,
    /// The entity's own random source, which the reference does not seed.
    random: RandomSource,
}

impl PartialEq for CreakingState {
    fn eq(&self, other: &Self) -> bool {
        self.attack == other.attack
            && self.invulnerable == other.invulnerable
            && self.death == other.death
            && self.attack_remaining == other.attack_remaining
            && self.invulnerable_remaining == other.invulnerable_remaining
            && self.can_move == other.can_move
            && self.active == other.active
            && self.tearing_down == other.tearing_down
            && self.heart_bound == other.heart_bound
            && self.eyes_glowing == other.eyes_glowing
            && self.next_flicker_time == other.next_flicker_time
            && self.random.state() == other.random.state()
    }
}

impl Eq for CreakingState {}

impl CreakingState {
    pub fn new(seed: i64) -> Self {
        Self {
            attack: AnimationState::default(),
            invulnerable: AnimationState::default(),
            death: AnimationState::default(),
            attack_remaining: 0,
            invulnerable_remaining: 0,
            can_move: true,
            active: false,
            tearing_down: false,
            heart_bound: false,
            eyes_glowing: false,
            next_flicker_time: 0,
            random: RandomSource::create(seed),
        }
    }

    pub fn can_move(&self) -> bool {
        self.can_move
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_tearing_down(&self) -> bool {
        self.tearing_down
    }

    pub fn is_heart_bound(&self) -> bool {
        self.heart_bound
    }

    /// `Creaking.hasGlowingEyes`: the blink a torn-down creaking's eyes keep.
    pub fn has_glowing_eyes(&self) -> bool {
        self.eyes_glowing
    }

    /// `Creaking.handleEntityEvent`: whether the creaking answers the event itself.
    pub(crate) fn event(&mut self, event: i8) -> bool {
        match event {
            INVULNERABLE_EVENT => self.invulnerable_remaining = INVULNERABILITY_ANIMATION_DURATION,
            ATTACK_EVENT => self.attack_remaining = ATTACK_ANIMATION_DURATION,
            _ => return false,
        }
        true
    }

    /// `Creaking.aiStep`'s countdowns, then `Creaking.tick`'s client half, after
    /// the rest of the tick: the animation states follow them, and the eyes blink.
    pub(crate) fn tick(&mut self, tick_count: i32, death_time: i32) {
        self.invulnerable_remaining = (self.invulnerable_remaining - 1).max(0);
        self.attack_remaining = (self.attack_remaining - 1).max(0);
        self.attack
            .animate_when(self.attack_remaining > 0, tick_count);
        self.invulnerable
            .animate_when(self.invulnerable_remaining > 0, tick_count);
        self.death.animate_when(self.tearing_down, tick_count);
        self.check_eye_blink(death_time);
    }

    /// `Creaking.checkEyeBlink`.
    fn check_eye_blink(&mut self, death_time: i32) {
        if death_time > self.next_flicker_time {
            let (min, max) = if self.eyes_glowing {
                (2, 8)
            } else {
                (death_time / 4, death_time / 2)
            };
            let wait = self
                .random
                .next_int_between_inclusive(min, max)
                .unwrap_or(min);
            self.next_flicker_time = death_time + wait;
            self.eyes_glowing = !self.eyes_glowing;
        }
    }
}
