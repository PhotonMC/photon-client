//! What a guardian does of its own on the client (`Guardian.aiStep`): its
//! tail's sway, the spikes' breathing, how long it has aimed its beam and
//! where its head turns to, and the flop out of water. The bubbles the
//! reference adds to the air are not drawn, but the draws that place them are
//! taken from the same source, so what is drawn after them stays in step.

use photon_core::math::mth_lerp_f32;
use photon_core::random::RandomSource;

/// `Guardian.getAmbientSoundInterval`.
const AMBIENT_SOUND_INTERVAL: i32 = 160;

/// What `Guardian.aiStep` reads of where the guardian is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GuardianPlace {
    pub in_water: bool,
    pub moving: bool,
    /// Its feet.
    pub position: (f64, f64, f64),
    pub eye_height: f32,
    /// The entity the beam aims at, if the world holds a living one by that id.
    pub target: Option<GuardianTarget>,
}

/// The entity a guardian's beam aims at (`getActiveAttackTarget`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GuardianTarget {
    /// Its feet.
    pub position: (f64, f64, f64),
    pub height: f32,
    pub eye_height: f32,
}

/// What `Guardian.aiStep` turns or pushes of the entity itself.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GuardianPose {
    pub y_rot: f32,
    pub x_rot: f32,
    pub y_head_rot: f32,
    pub motion: (f64, f64, f64),
    pub on_ground: bool,
}

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
    /// `Mob.ambientSoundTime`.
    ambient_sound_time: i32,
    /// `clientSideCachedAttackTarget`: once found, the entity is kept (and
    /// so stays where it was if the world drops it).
    cached_target: Option<GuardianTarget>,
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
            ambient_sound_time: 0,
            cached_target: None,
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
        self.cached_target = None;
    }

    /// `Guardian.aiStep` on the client, while it lives, up to `super.aiStep`.
    pub(crate) fn tick(
        &mut self,
        place: &GuardianPlace,
        has_target: bool,
        attack_duration: i32,
        pose: &mut GuardianPose,
    ) {
        let GuardianPlace {
            in_water, moving, ..
        } = *place;
        // `Mob.baseTick` rolls the chance of an ambient sound from the same
        // source each tick, before `aiStep`; the sound is not played here,
        // but it draws the two floats of its pitch.
        let roll = self.random.0.next_int_bound(1000).unwrap_or_default();
        let ambient_sound_time = self.ambient_sound_time;
        self.ambient_sound_time += 1;
        if roll < ambient_sound_time {
            self.ambient_sound_time = -AMBIENT_SOUND_INTERVAL;
            self.random.0.next_float();
            self.random.0.next_float();
        }
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
        if moving && in_water {
            // Two bubbles, each placed by a draw for every axis.
            for _ in 0..6 {
                self.random.0.next_double();
            }
        }
        if has_target {
            if self.attack_time < attack_duration {
                self.attack_time += 1;
            }
            if let Some(live) = place.target {
                self.cached_target = Some(live);
            }
            if let Some(target) = self.cached_target {
                self.aim_at(place, &target, attack_duration, pose);
            }
        }
        if in_water {
            // `setAirSupply(300)`: the air is not read here.
        } else if pose.on_ground {
            let random = &mut self.random.0;
            let x = (random.next_float() * 2.0 - 1.0) * 0.4;
            let z = (random.next_float() * 2.0 - 1.0) * 0.4;
            pose.motion.0 += f64::from(x);
            pose.motion.1 += 0.5;
            pose.motion.2 += f64::from(z);
            pose.y_rot = random.next_float() * 360.0;
            pose.on_ground = false;
        }
        if has_target {
            pose.y_rot = pose.y_head_rot;
        }
    }

    /// The part of `aiStep` that runs when the target is known: the head
    /// turns to it and the bubbles along the beam are placed.
    fn aim_at(
        &mut self,
        place: &GuardianPlace,
        target: &GuardianTarget,
        attack_duration: i32,
        pose: &mut GuardianPose,
    ) {
        let (x, y, z) = place.position;
        let eye_y = y + f64::from(place.eye_height);
        let (target_x, target_y, target_z) = target.position;
        let target_eye_y = target_y + f64::from(target.eye_height);
        crate::living::look_at_tick(
            (&mut pose.y_head_rot, &mut pose.x_rot),
            (x, eye_y, z),
            (target_x, target_eye_y, target_z),
            (90.0, 90.0),
        );
        let scale = f64::from(self.attack_animation_scale(0.0, attack_duration));
        let dx = target_x - x;
        let dy = target_y + f64::from(target.height) * 0.5 - eye_y;
        let dz = target_z - z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        let random = &mut self.random.0;
        let mut dist = random.next_double();
        while dist < length {
            dist += 1.8 - scale + random.next_double() * (1.7 - scale);
        }
    }

    /// The source the tail and the spikes out of water are drawn from, as the
    /// reference's seed holds it, to compare how many draws were taken.
    pub fn random_state(&self) -> u64 {
        self.random.0.state()
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
