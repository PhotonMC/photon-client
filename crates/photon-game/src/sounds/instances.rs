//! The sounds that follow a thing in the world and change as it does (`MinecartSoundInstance`,
//! `RidingEntitySoundInstance`, `BeeSoundInstance`, `GuardianAttackSoundInstance`,
//! `SnifferSoundInstance`): what each makes of the thing it follows on every tick. The caller
//! reads the thing and plays the voice.

use super::SoundSource;
use super::engine::Attenuation;

/// What a ticking sound instance may change of itself (`AbstractSoundInstance`'s fields).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    pub position: (f64, f64, f64),
    pub volume: f32,
    pub pitch: f32,
    pub stopped: bool,
}

impl Voice {
    /// A voice that starts as `volume` loud and has not been placed yet.
    pub const fn new(volume: f32) -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            volume,
            pitch: 1.0,
            stopped: false,
        }
    }

    /// `x = (float) entity.getX()` and the rest: the place narrowed to floats.
    pub fn follow(&mut self, (x, y, z): (f64, f64, f64)) {
        self.position = (
            f64::from(x as f32),
            f64::from(y as f32),
            f64::from(z as f32),
        );
    }

    /// `AbstractTickableSoundInstance.stop`.
    fn stop(&mut self) {
        self.stopped = true;
    }
}

/// What an instance is made as (its constructor): the event, how it loops, whether it may start
/// at no volume and how it dies away with distance.
#[derive(Debug, Clone, Copy)]
pub struct Setup {
    /// The sound event, without the `minecraft:` namespace.
    pub event: &'static str,
    pub source: SoundSource,
    pub volume: f32,
    pub looping: bool,
    pub can_start_silent: bool,
    pub attenuation: Attenuation,
}

const fn setup(
    event: &'static str,
    source: SoundSource,
    (volume, looping, can_start_silent): (f32, bool, bool),
    attenuation: Attenuation,
) -> Setup {
    Setup {
        event,
        source,
        volume,
        looping,
        can_start_silent,
        attenuation,
    }
}

/// `MinecartSoundInstance`.
pub const MINECART: Setup = setup(
    "entity.minecart.riding",
    SoundSource::Neutral,
    (0.0, true, true),
    Attenuation::Linear,
);
/// `BeeFlyingSoundInstance` and `BeeAggressiveSoundInstance`.
pub const BEE_FLYING: Setup = setup(
    "entity.bee.loop",
    SoundSource::Neutral,
    (0.0, true, true),
    Attenuation::Linear,
);
pub const BEE_AGGRESSIVE: Setup = setup(
    "entity.bee.loop_aggressive",
    SoundSource::Neutral,
    (0.0, true, true),
    Attenuation::Linear,
);
/// `GuardianAttackSoundInstance`: heard the same wherever the guardian is.
pub const GUARDIAN_ATTACK: Setup = setup(
    "entity.guardian.attack",
    SoundSource::Hostile,
    (1.0, true, false),
    Attenuation::None,
);
/// `SnifferSoundInstance`.
pub const SNIFFER_DIGGING: Setup = setup(
    "entity.sniffer.digging",
    SoundSource::Neutral,
    (1.0, false, false),
    Attenuation::Linear,
);

/// `RidingMinecartSoundInstance`: the rolling heard from inside, with the air or under water.
pub const RIDING_MINECART: Setup = riding("entity.minecart.inside");
pub const RIDING_MINECART_UNDERWATER: Setup = riding("entity.minecart.inside.underwater");
/// `RidingEntitySoundInstance` of a happy ghast and of a nautilus.
pub const RIDING_GHAST: Setup = riding("entity.happy_ghast.riding");
pub const RIDING_NAUTILUS: Setup = riding("entity.nautilus.riding");

/// A sound heard the same wherever the vehicle is, loud as the vehicle is fast.
const fn riding(event: &'static str) -> Setup {
    setup(
        event,
        SoundSource::Neutral,
        (0.0, true, true),
        Attenuation::None,
    )
}

/// `Mth.lerp` on floats.
fn lerp(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * (to - from)
}

/// `Mth.clamp` on floats.
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min } else { value.min(max) }
}

/// `Mth.clampedLerp` on floats.
fn clamped_lerp(factor: f32, min: f32, max: f32) -> f32 {
    if factor < 0.0 {
        min
    } else if factor > 1.0 {
        max
    } else {
        lerp(factor, min, max)
    }
}

/// `Vec3.horizontalDistance` narrowed to a float, as the instances read it.
pub fn horizontal_speed(motion_x: f64, motion_z: f64) -> f32 {
    (motion_x * motion_x + motion_z * motion_z).sqrt() as f32
}

/// `Vec3.length` narrowed to a float.
pub fn speed(motion: (f64, f64, f64)) -> f32 {
    (motion.0 * motion.0 + motion.1 * motion.1 + motion.2 * motion.2).sqrt() as f32
}

/// A minecart as its rolling sound reads it.
#[derive(Debug, Clone, Copy)]
pub struct MinecartView {
    pub removed: bool,
    pub position: (f64, f64, f64),
    pub speed: f32,
    /// Off its rails under the new behaviour (`!isOnRails() && behavior instanceof New...`).
    pub off_rail: bool,
    /// `TickRateManager.runsNormally`.
    pub runs_normally: bool,
}

/// `MinecartSoundInstance.tick`. Its own `pitch` field hides the one the engine reads, so the
/// pitch stays at one however fast the minecart goes.
pub fn tick_minecart(voice: &mut Voice, view: &MinecartView) {
    if view.removed {
        voice.stop();
        return;
    }
    voice.follow(view.position);
    if view.speed >= 0.01 && view.runs_normally && !view.off_rail {
        voice.volume = lerp(clamp(view.speed, 0.0, 0.5), 0.0, 0.7);
    } else {
        voice.volume = 0.0;
    }
}

/// What a rider's own sound of a vehicle reads (`RidingEntitySoundInstance`).
#[derive(Debug, Clone, Copy)]
pub struct RidingView {
    pub removed: bool,
    /// The player is a passenger, and of this vehicle.
    pub rides_it: bool,
    /// The sound is for the water the other way round from where the one it follows is
    /// (`shouldNotPlayUnderwaterSound`).
    pub wrong_water: bool,
    pub speed: f32,
    /// `shoudlPlaySound`.
    pub allowed: bool,
    /// Where the vehicle is, when the sound is placed at it (26.3).
    pub follow: Option<(f64, f64, f64)>,
}

/// The loudness range of a riding sound (`volumeMin`, `volumeMax`, `volumeAmplifier`).
#[derive(Debug, Clone, Copy)]
pub struct RidingVolume {
    pub min: f32,
    pub max: f32,
    pub amplifier: f32,
}

/// `RidingEntitySoundInstance.tick`.
pub fn tick_riding(voice: &mut Voice, view: &RidingView, volume: RidingVolume) {
    if view.removed || !view.rides_it {
        voice.stop();
    } else if view.wrong_water {
        voice.volume = volume.min;
    } else {
        voice.volume = if view.speed >= 0.01 && view.allowed {
            volume.amplifier * clamped_lerp(view.speed, volume.min, volume.max)
        } else {
            volume.min
        };
        if let Some(at) = view.follow {
            voice.position = at;
        }
    }
}

/// A bee as its buzzing reads it.
#[derive(Debug, Clone, Copy)]
pub struct BeeView {
    pub removed: bool,
    pub position: (f64, f64, f64),
    pub speed: f32,
    pub baby: bool,
    pub angry: bool,
}

/// `BeeSoundInstance`: which of its two sounds this is, and whether it has handed over to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buzz {
    pub aggressive: bool,
    switched: bool,
}

impl Buzz {
    pub const fn new(aggressive: bool) -> Self {
        Self {
            aggressive,
            switched: false,
        }
    }

    /// `BeeSoundInstance.tick`: whether the other sound must be queued now.
    pub fn tick(&mut self, voice: &mut Voice, view: &BeeView) -> bool {
        let switch = if self.aggressive {
            !view.angry
        } else {
            view.angry
        };
        let handed_over = switch && !voice.stopped;
        if handed_over {
            self.switched = true;
        }
        if !view.removed && !self.switched {
            voice.follow(view.position);
            let (min, max) = if view.baby { (1.1, 1.5) } else { (0.7, 1.1) };
            if view.speed >= 0.01 {
                voice.pitch = lerp(clamp(view.speed, min, max), min, max);
                voice.volume = lerp(clamp(view.speed, 0.0, 0.5), 0.0, 1.2);
            } else {
                voice.pitch = 0.0;
                voice.volume = 0.0;
            }
        } else {
            voice.stop();
        }
        handed_over
    }
}

/// `GuardianAttackSoundInstance.tick`: `scale` is `getAttackAnimationScale(0)`; a client never
/// knows a guardian's target, so it is none.
pub fn tick_guardian_attack(
    voice: &mut Voice,
    removed: bool,
    position: (f64, f64, f64),
    scale: f32,
) {
    if removed {
        voice.stop();
        return;
    }
    voice.follow(position);
    voice.volume = 0.0 + 1.0 * scale * scale;
    voice.pitch = 0.7 + 0.5 * scale;
}

/// `SnifferSoundInstance.tick`: it sounds while the sniffer digs or searches.
pub fn tick_sniffer_digging(
    voice: &mut Voice,
    removed: bool,
    position: (f64, f64, f64),
    can_play_digging_sound: bool,
) {
    if removed || !can_play_digging_sound {
        voice.stop();
        return;
    }
    voice.follow(position);
    voice.volume = 1.0;
    voice.pitch = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    const HERE: (f64, f64, f64) = (1.5, 64.0, -3.1);

    fn cart(speed: f32, off_rail: bool) -> MinecartView {
        MinecartView {
            removed: false,
            position: HERE,
            speed,
            off_rail,
            runs_normally: true,
        }
    }

    #[test]
    fn a_minecart_rolls_louder_with_speed_and_is_silent_off_its_rails() {
        let mut voice = Voice::new(0.0);
        tick_minecart(&mut voice, &cart(0.25, false));
        assert_eq!(voice.volume, 0.25 * 0.7);
        assert_eq!(voice.pitch, 1.0);
        assert_eq!(voice.position.2, f64::from(-3.1_f32));
        tick_minecart(&mut voice, &cart(9.0, false));
        assert_eq!(voice.volume, 0.35);
        tick_minecart(&mut voice, &cart(0.25, true));
        assert_eq!(voice.volume, 0.0);
        tick_minecart(&mut voice, &cart(0.005, false));
        assert_eq!(voice.volume, 0.0);
    }

    #[test]
    fn a_removed_thing_ends_its_sound() {
        let mut voice = Voice::new(0.0);
        let mut view = cart(0.25, false);
        view.removed = true;
        tick_minecart(&mut voice, &view);
        assert!(voice.stopped);
    }

    #[test]
    fn a_rider_hears_the_vehicle_by_its_speed_until_it_gets_off() {
        let volume = RidingVolume {
            min: 0.0,
            max: 0.75,
            amplifier: 1.0,
        };
        let mut view = RidingView {
            removed: false,
            rides_it: true,
            wrong_water: false,
            speed: 0.375,
            allowed: true,
            follow: None,
        };
        let mut voice = Voice::new(0.0);
        tick_riding(&mut voice, &view, volume);
        assert_eq!(voice.volume, 0.28125);
        assert_eq!(voice.position, (0.0, 0.0, 0.0));
        view.follow = Some((1.5, 64.0, -2.0));
        tick_riding(&mut voice, &view, volume);
        assert_eq!(voice.position, (1.5, 64.0, -2.0));
        view.follow = Some((9.0, 9.0, 9.0));
        view.wrong_water = true;
        tick_riding(&mut voice, &view, volume);
        assert_eq!(voice.volume, 0.0);
        assert_eq!(voice.position, (1.5, 64.0, -2.0));
        view.rides_it = false;
        tick_riding(&mut voice, &view, volume);
        assert!(voice.stopped);
    }

    #[test]
    fn a_bee_hands_its_buzz_over_when_it_gets_angry() {
        let view = |angry| BeeView {
            removed: false,
            position: HERE,
            speed: 0.25,
            baby: false,
            angry,
        };
        let mut buzz = Buzz::new(false);
        let mut voice = Voice::new(0.0);
        assert!(!buzz.tick(&mut voice, &view(false)));
        assert!(!voice.stopped && voice.volume > 0.0);
        assert!(buzz.tick(&mut voice, &view(true)));
        assert!(voice.stopped);
    }

    #[test]
    fn a_still_bee_is_silent_and_a_baby_buzzes_higher() {
        let mut view = BeeView {
            removed: false,
            position: HERE,
            speed: 0.0,
            baby: false,
            angry: false,
        };
        let mut buzz = Buzz::new(false);
        let mut voice = Voice::new(0.0);
        buzz.tick(&mut voice, &view);
        assert_eq!((voice.volume, voice.pitch), (0.0, 0.0));
        view.speed = 0.5;
        buzz.tick(&mut voice, &view);
        let adult = voice.pitch;
        view.baby = true;
        buzz.tick(&mut voice, &view);
        assert!(voice.pitch > adult);
    }

    #[test]
    fn a_guardian_beam_hums_louder_as_it_charges() {
        let mut voice = Voice::new(1.0);
        tick_guardian_attack(&mut voice, false, HERE, 0.5);
        assert_eq!((voice.volume, voice.pitch), (0.25, 0.95));
        tick_guardian_attack(&mut voice, true, HERE, 0.5);
        assert!(voice.stopped);
    }

    #[test]
    fn a_sniffer_digs_loudly_only_while_it_digs() {
        let mut voice = Voice::new(1.0);
        tick_sniffer_digging(&mut voice, false, HERE, true);
        assert!(!voice.stopped);
        tick_sniffer_digging(&mut voice, false, HERE, false);
        assert!(voice.stopped);
    }
}
