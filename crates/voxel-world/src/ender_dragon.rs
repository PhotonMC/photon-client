//! What an ender dragon does of its own on the client: the flight history its
//! parts are posed from (`DragonFlightHistory`), how fast its wings beat, and
//! the phase the server syncs (`EnderDragon.aiStep`, `onSyncedDataUpdated`).

use voxel_core::math::{mth_cos_f64, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32, mth_sin_f64};
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

/// `EnderDragonPart`: one of the boxes a dragon is hit through, by its name and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonPart {
    pub name: &'static str,
    pub width: f32,
    pub height: f32,
    /// Where the part stands (its feet), as the dragon's last tick set it.
    pub position: (f64, f64, f64),
}

/// The parts, in the order `EnderDragon.subEntities` has them: head, neck, body,
/// three tails, two wings. A part's id is the dragon's, plus its place here, plus one.
const PARTS: [(&str, f32, f32); 8] = [
    ("head", 1.0, 1.0),
    ("neck", 3.0, 3.0),
    ("body", 5.0, 3.0),
    ("tail", 2.0, 2.0),
    ("tail", 2.0, 2.0),
    ("tail", 2.0, 2.0),
    ("wing", 4.0, 2.0),
    ("wing", 4.0, 2.0),
];

/// `EnderDragon.rotWrap`.
fn rot_wrap(mut degrees: f32) -> f32 {
    while degrees >= 180.0 {
        degrees -= 360.0;
    }
    while degrees < -180.0 {
        degrees += 360.0;
    }
    degrees
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
    /// `EnderDragon.subEntities`.
    parts: [DragonPart; 8],
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
            parts: PARTS.map(|(name, width, height)| DragonPart {
                name,
                width,
                height,
                position: (0.0, 0.0, 0.0),
            }),
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
            && self.parts == other.parts
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

    /// `EnderDragon.getSubEntities`.
    pub fn parts(&self) -> &[DragonPart; 8] {
        &self.parts
    }

    /// `EnderDragon.getHeadYOffset`.
    fn head_y_offset(&self) -> f32 {
        if self.phase.is_sitting() {
            return -1.0;
        }
        (self.history.get(5).y - self.history.get(0).y) as f32
    }

    /// The end of `EnderDragon.aiStep`'s flying branch: every part is put at a place
    /// round the dragon that follows its flight history (`tickPart`): the body and the wings
    /// by its turn, the head and the neck by it and by how it climbs, the tails by the history behind.
    pub(crate) fn tick_parts(&mut self, at: (f64, f64, f64), y_rot: f32) {
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        let tilt = (self.history.get(5).y - self.history.get(10).y) as f32 * 10.0 * degrees;
        let (cc_tilt, ss_tilt) = (cos(tilt), sin(tilt));
        let rot1 = y_rot * degrees;
        let (ss1, cc1) = (sin(rot1), cos(rot1));
        // `yRotA` is the server's alone: always zero here.
        let rot2 = y_rot * degrees - 0.0 * 0.01;
        let (ss2, cc2) = (sin(rot2), cos(rot2));
        let y_offset = self.head_y_offset();
        let mut offsets = [(0.0_f64, 0.0_f64, 0.0_f64); 8];
        offsets[2] = (f64::from(ss1 * 0.5), 0.0, f64::from(-cc1 * 0.5));
        offsets[6] = (f64::from(cc1 * 4.5), 2.0, f64::from(ss1 * 4.5));
        offsets[7] = (f64::from(cc1 * -4.5), 2.0, f64::from(ss1 * -4.5));
        offsets[0] = (
            f64::from(ss2 * 6.5 * cc_tilt),
            f64::from(y_offset + ss_tilt * 6.5),
            f64::from(-cc2 * 6.5 * cc_tilt),
        );
        offsets[1] = (
            f64::from(ss2 * 5.5 * cc_tilt),
            f64::from(y_offset + ss_tilt * 5.5),
            f64::from(-cc2 * 5.5 * cc_tilt),
        );
        let p1 = self.history.get(5);
        for i in 0..3 {
            let p0 = self.history.get(12 + i * 2);
            let rot = y_rot * degrees + rot_wrap(p0.y_rot - p1.y_rot) * degrees;
            let (ss, cc) = (sin(rot), cos(rot));
            let dd = (i + 1) as f32 * 2.0;
            offsets[3 + i as usize] = (
                f64::from(-(ss1 * 1.5 + ss * dd) * cc_tilt),
                p0.y - p1.y - f64::from((dd + 1.5) * ss_tilt) + 1.5,
                f64::from((cc1 * 1.5 + cc * dd) * cc_tilt),
            );
        }
        for (part, offset) in self.parts.iter_mut().zip(offsets) {
            part.position = (at.0 + offset.0, at.1 + offset.1, at.2 + offset.2);
        }
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

    /// `EnderDragon.tickDeath` (its count and its lift; the rest is the server's).
    pub(crate) fn tick_death(&mut self, lift: f64) {
        self.dragon_death_time += 1;
        for part in &mut self.parts {
            part.position.1 += lift;
        }
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
