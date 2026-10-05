//! What an ender dragon does of its own on the client: the flight history its
//! parts are posed from (`DragonFlightHistory`), how fast its wings beat, and
//! the phase the server syncs (`EnderDragon.aiStep`, `onSyncedDataUpdated`).

use voxel_core::math::{mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32};
use voxel_core::random::RandomSource;

/// `EnderDragon.DATA_PHASE`'s data id: `Mob` ends at 15.
pub const PHASE_DATA_ID: u8 = 16;

/// `DragonFlightHistory.LENGTH`.
const LENGTH: usize = 64;
const MASK: usize = LENGTH - 1;

/// `DragonFlightHistory.Sample`: a height and a turn, in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub y: f64,
    pub y_rot: f32,
}

/// `DragonFlightHistory`: the last sixty-four samples of where the dragon
/// was, one for each tick.
#[derive(Debug, Clone, PartialEq)]
pub struct FlightHistory {
    samples: [Sample; LENGTH],
    head: i32,
}

impl Default for FlightHistory {
    fn default() -> Self {
        Self {
            samples: [Sample { y: 0.0, y_rot: 0.0 }; LENGTH],
            head: -1,
        }
    }
}

impl FlightHistory {
    /// `record`: the first sample fills the whole history.
    pub fn record(&mut self, y: f64, y_rot: f32) {
        let sample = Sample { y, y_rot };
        if self.head < 0 {
            self.samples = [sample; LENGTH];
        }
        self.head += 1;
        if self.head == LENGTH as i32 {
            self.head = 0;
        }
        self.samples[self.head as usize] = sample;
    }

    /// `get(delay)`: the sample from this many ticks ago.
    pub fn get(&self, delay: i32) -> Sample {
        self.samples[(self.head.wrapping_sub(delay) as usize) & MASK]
    }

    /// `get(delay, partialTicks)`: between that sample and the one before it.
    pub fn get_at(&self, delay: i32, partial: f32) -> Sample {
        let (sample, old) = (self.get(delay), self.get(delay + 1));
        Sample {
            y: mth_lerp_f64(f64::from(partial), old.y, sample.y),
            y_rot: mth_rot_lerp_f32(partial, old.y_rot, sample.y_rot),
        }
    }
}

/// `EnderDragonPhase`, by the ids the server syncs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DragonPhase {
    HoldingPattern,
    StrafePlayer,
    LandingApproach,
    Landing,
    Takeoff,
    SittingFlaming,
    SittingScanning,
    SittingAttacking,
    ChargingPlayer,
    Dying,
    #[default]
    Hovering,
}

impl DragonPhase {
    /// `EnderDragonPhase.getById`: an id out of range is hovering, as the reference's
    /// manager starts from it.
    pub fn from_id(id: i32) -> Self {
        match id {
            0 => Self::HoldingPattern,
            1 => Self::StrafePlayer,
            2 => Self::LandingApproach,
            3 => Self::Landing,
            4 => Self::Takeoff,
            5 => Self::SittingFlaming,
            6 => Self::SittingScanning,
            7 => Self::SittingAttacking,
            8 => Self::ChargingPlayer,
            9 => Self::Dying,
            _ => Self::Hovering,
        }
    }

    /// `DragonPhaseInstance.isSitting`: the sitting phases and hovering.
    pub fn is_sitting(self) -> bool {
        matches!(
            self,
            Self::SittingFlaming | Self::SittingScanning | Self::SittingAttacking | Self::Hovering
        )
    }

    /// Landing or taking off (what the neck's pose follows then).
    pub fn is_landing_or_taking_off(self) -> bool {
        matches!(self, Self::Landing | Self::Takeoff)
    }
}

/// What a dragon's tick looks up of the end crystals in the world.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DragonCrystals {
    /// The nearest crystal whose box meets the dragon's, 32 blocks wider.
    pub nearest: Option<i32>,
    /// Whether the crystal the dragon holds is still in the world.
    pub current_exists: bool,
}

#[derive(Debug, Clone)]
pub struct EnderDragonState {
    pub history: FlightHistory,
    flap_time: f32,
    old_flap_time: f32,
    phase: DragonPhase,
    dragon_death_time: i32,
    /// `EnderDragon.nearestCrystal`, by its id: the crystal whose beam it is healed by.
    nearest_crystal: Option<i32>,
    /// The entity's own random source, which the reference does not seed.
    random: RandomSource,
}

impl EnderDragonState {
    pub fn new(seed: i64) -> Self {
        Self {
            history: FlightHistory::default(),
            flap_time: 0.0,
            old_flap_time: 0.0,
            phase: DragonPhase::default(),
            dragon_death_time: 0,
            nearest_crystal: None,
            random: RandomSource::create(seed),
        }
    }
}

impl PartialEq for EnderDragonState {
    fn eq(&self, other: &Self) -> bool {
        self.history == other.history
            && self.flap_time == other.flap_time
            && self.old_flap_time == other.old_flap_time
            && self.phase == other.phase
            && self.dragon_death_time == other.dragon_death_time
            && self.nearest_crystal == other.nearest_crystal
            && self.random.state() == other.random.state()
    }
}

impl EnderDragonState {
    pub fn phase(&self) -> DragonPhase {
        self.phase
    }

    /// `onSyncedDataUpdated(DATA_PHASE)`.
    pub(crate) fn set_phase(&mut self, phase: DragonPhase) {
        self.phase = phase;
    }

    /// `EnderDragon.nearestCrystal`: the id of the end crystal whose beam reaches it.
    pub fn nearest_crystal(&self) -> Option<i32> {
        self.nearest_crystal
    }

    /// `EnderDragon.checkCrystals`: a crystal that is gone is let go and one still there
    /// heals the dragon a health every ten ticks (the answer says whether it does), and
    /// one time in ten the nearest crystal is looked for again.
    pub(crate) fn check_crystals(&mut self, crystals: DragonCrystals, tick_count: i32) -> bool {
        let mut heal = false;
        if self.nearest_crystal.is_some() {
            if !crystals.current_exists {
                self.nearest_crystal = None;
            } else if tick_count % 10 == 0 {
                heal = true;
            }
        }
        if self.random.next_int_bound(10) == Ok(0) {
            self.nearest_crystal = crystals.nearest;
        }
        heal
    }

    /// `EnderDragon.dragonDeathTime`: the ticks it has been dying.
    pub fn dragon_death_time(&self) -> i32 {
        self.dragon_death_time
    }

    /// `EnderDragon.tickDeath` (its count; the rest is the server's).
    pub(crate) fn tick_death(&mut self) {
        self.dragon_death_time += 1;
    }

    /// `EnderDragon.flapTime` and `oFlapTime`.
    pub fn flap_time(&self) -> f32 {
        self.flap_time
    }

    pub fn old_flap_time(&self) -> f32 {
        self.old_flap_time
    }

    /// The flap time between two ticks (`Mth.lerp(partial, oFlapTime, flapTime)`).
    pub fn flap_time_at(&self, partial: f32) -> f32 {
        mth_lerp_f32(partial, self.old_flap_time, self.flap_time)
    }

    /// The start of `EnderDragon.aiStep` on the client: the flap time is kept.
    pub(crate) fn begin_ai_step(&mut self) {
        self.old_flap_time = self.flap_time;
    }

    /// What the rest of `aiStep` does while the dragon is not dying: the wings beat
    /// faster the slower it flies (and slower in a wall), steadily while it sits, and
    /// stop at the middle of their beat without AI; its height and turn are recorded.
    pub(crate) fn fly(&mut self, motion: (f64, f64, f64), no_ai: bool, place: (f64, f32)) {
        let horizontal = (motion.0 * motion.0 + motion.2 * motion.2).sqrt();
        let mut speed = 0.2_f32 / (horizontal as f32 * 10.0 + 1.0);
        speed *= 2.0_f64.powf(motion.1) as f32;
        if self.phase.is_sitting() {
            self.flap_time += 0.1;
        } else {
            // `inWall` is worked out by the server alone: never set here.
            self.flap_time += speed;
        }
        if no_ai {
            self.flap_time = 0.5;
        } else {
            self.history.record(place.0, place.1);
        }
    }
}

// The values are sums of finite steps: never NaN.
impl Eq for Sample {}
impl Eq for FlightHistory {}
impl Eq for EnderDragonState {}
