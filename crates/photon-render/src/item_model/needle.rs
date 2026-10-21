//! The compass needle (`NeedleDirectionHelper`, `CompassAngleState`): a wobbler that eases the
//! needle once per game tick, and the turn that points it at a block.

use photon_core::math::positive_modulo_f32;

/// What a needle points at or follows: a compass's target (`CompassAngleState.CompassTarget`) or
/// a clock's time source (`Time.TimeSource`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeedleSource {
    None,
    Lodestone,
    Spawn,
    Recovery,
    TimeRandom,
    TimeDaytime,
    TimeMoonPhase,
}

/// What a `compass` range's property is named in front of: its target and `wobble`.
pub const COMPASS: &str = "minecraft:compass/";
/// What a `time` range's property is named in front of: its source and `wobble`.
pub const TIME: &str = "minecraft:time/";

impl NeedleSource {
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::Lodestone,
        Self::Spawn,
        Self::Recovery,
        Self::TimeRandom,
        Self::TimeDaytime,
        Self::TimeMoonPhase,
    ];

    pub fn is_time(self) -> bool {
        matches!(
            self,
            Self::TimeRandom | Self::TimeDaytime | Self::TimeMoonPhase
        )
    }

    /// The source a compass `target` or a clock `source` names.
    pub fn parse(name: &str, time: bool) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|source| source.is_time() == time && source.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Lodestone => "lodestone",
            Self::Spawn => "spawn",
            Self::Recovery => "recovery",
            Self::TimeRandom => "random",
            Self::TimeDaytime => "daytime",
            Self::TimeMoonPhase => "moon_phase",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The property a range dispatch is named by, for a source and its `wobble`.
    pub fn property(self, wobble: bool) -> String {
        let prefix = if self.is_time() { TIME } else { COMPASS };
        format!("{prefix}{}/{}", self.name(), u8::from(wobble))
    }

    /// The source and `wobble` a range dispatch's property names.
    pub fn from_property(property: &str) -> Option<(Self, bool)> {
        let (rest, time) = match property.strip_prefix(COMPASS) {
            Some(rest) => (rest, false),
            None => (property.strip_prefix(TIME)?, true),
        };
        let (source, wobble) = rest.split_once('/')?;
        Some((Self::parse(source, time)?, wobble == "1"))
    }

    /// What the wobbler multiplies its speed by (`newWobbler`): a compass is quicker.
    pub fn factor(self) -> f32 {
        if self.is_time() { 0.9 } else { 0.8 }
    }
}

/// The needle angles a draw reads, by target, as `f32` bits (a turn, from 0 to 1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Needles(pub [Option<u32>; 7]);

/// `NeedleDirectionHelper.Wobbler`: eases a rotation towards a target, once per game tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Wobbler {
    /// Whether it eases (`standardWobbler`) or follows its target (`nonWobbler`).
    wobble: bool,
    factor: f32,
    rotation: f32,
    delta_rotation: f32,
    last_update_tick: i64,
}

impl Wobbler {
    /// A wobbler that multiplies the speed by `factor` each update (`newWobbler`).
    pub fn new(wobble: bool, factor: f32) -> Self {
        Self {
            wobble,
            factor,
            ..Self::default()
        }
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Whether this game tick has not moved the needle yet; a plain needle always follows.
    pub fn should_update(&self, tick: i64) -> bool {
        !self.wobble || self.last_update_tick != tick
    }

    pub fn update(&mut self, tick: i64, target: f32) {
        if !self.wobble {
            self.rotation = target;
            return;
        }
        self.last_update_tick = tick;
        let step = positive_modulo_f32(target - self.rotation + 0.5, 1.0) - 0.5;
        self.delta_rotation += step * 0.1;
        self.delta_rotation *= self.factor;
        self.rotation = positive_modulo_f32(self.rotation + self.delta_rotation, 1.0);
    }
}

/// Where the holder stands and looks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Holder {
    pub position: [f64; 3],
    /// `getVisualRotationYInDegrees`.
    pub yaw: f32,
}

/// `getAngleFromEntityToPos`: the heading from the holder to the centre of a block, in turns.
fn angle_to(holder: &Holder, block: [i32; 3]) -> f32 {
    let dz = f64::from(block[2]) + 0.5 - holder.position[2];
    let dx = f64::from(block[0]) + 0.5 - holder.position[0];
    (dz.atan2(dx) / f64::from((std::f64::consts::PI * 2.0) as f32)) as f32
}

/// `getWrappedVisualRotationY`: the holder's yaw in turns, from 0 to 1.
fn wrapped_yaw(holder: &Holder) -> f32 {
    positive_modulo_f32(holder.yaw / 360.0, 1.0)
}

/// `getRotationTowardsNeedleSource`: the needle turn towards a block. The local player's needle
/// eases through its wobbler; anyone else's follows the heading at once.
pub fn rotation_towards(
    holder: &Holder,
    block: [i32; 3],
    local: Option<(&mut Wobbler, i64)>,
) -> f32 {
    let angle = angle_to(holder, block);
    let yaw = wrapped_yaw(holder);
    let target = match local {
        Some((wobbler, tick)) => {
            if wobbler.should_update(tick) {
                wobbler.update(tick, 0.5 - (yaw - 0.25));
            }
            angle + wobbler.rotation()
        }
        None => 0.5 - (yaw - 0.25 - angle),
    };
    positive_modulo_f32(target, 1.0)
}

/// `isValidNeedleSourcePos`, past the dimension: a block the holder is not standing in the
/// centre of.
pub fn is_valid_target(holder: &Holder, block: [i32; 3]) -> bool {
    let distance = (0..3)
        .map(|axis| {
            let delta = f64::from(block[axis]) + 0.5 - holder.position[axis];
            delta * delta
        })
        .sum::<f64>();
    distance >= f64::from(1.0E-5_f32)
}

/// `getRandomlySpinningRotation`: the needle with no target, from the spin and the draw's seed.
pub fn spinning_rotation(spin: f32, seed: i32) -> f32 {
    positive_modulo_f32(
        spin + seed.wrapping_mul(1_327_217_883) as f32 / 2.147_483_6E9,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_needle_follows_at_once_and_a_wobbling_one_waits_for_the_next_tick() {
        let mut plain = Wobbler::new(false, 0.8);
        plain.update(5, 0.25);
        assert!(plain.should_update(5));
        assert_eq!(plain.rotation(), 0.25);
        let mut wobbling = Wobbler::new(true, 0.8);
        assert!(!wobbling.should_update(0));
        wobbling.update(7, 0.25);
        assert!(!wobbling.should_update(7));
        assert!(wobbling.should_update(8));
    }
}
