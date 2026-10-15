//! `ItemPickupParticle` and its group: what was picked up flies to whoever took it over three
//! ticks, drawn as it was when it was taken.

use crate::ClientItemStackState;
use photon_core::ids::EntityId;

/// `ItemPickupParticle.LIFE_TIME`.
const LIFE_TIME: i32 = 3;

/// What the taken entity looked like when it was taken (`extractEntity(from, 1.0F)`).
#[derive(Debug, Clone, PartialEq)]
pub enum PickedUp {
    /// A dropped item: its stack before it shrank, its age and its bob.
    Item {
        stack: ClientItemStackState,
        age_in_ticks: f32,
        bob_offset: f32,
    },
    /// An experience orb: its value and its age.
    Orb { value: i32, age_in_ticks: f32 },
    /// Anything else taken (an arrow, a trident): not drawn here.
    Other,
}

/// One entity flying to the one that took it.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemPickup {
    pub picked: PickedUp,
    /// Where the entity was taken from.
    pub from: (f64, f64, f64),
    /// Who took it: none for the local player.
    pub target: Option<EntityId>,
    target_now: (f64, f64, f64),
    target_old: (f64, f64, f64),
    life: i32,
    removed: bool,
}

/// `ItemPickupParticle.updatePosition`: halfway between the taker's feet and its eyes.
pub fn pickup_target((x, y, z): (f64, f64, f64), eye_height: f32) -> (f64, f64, f64) {
    (x, (y + (y + f64::from(eye_height))) / 2.0, z)
}

impl ItemPickup {
    /// `ItemPickupParticle(..)`: the taker's place is where it heads, now and before.
    pub fn new(
        (picked, from): (PickedUp, (f64, f64, f64)),
        target: Option<EntityId>,
        target_at: (f64, f64, f64),
    ) -> Self {
        Self {
            picked,
            from,
            target,
            target_now: target_at,
            target_old: target_at,
            life: 0,
            removed: false,
        }
    }

    /// `ItemPickupParticle.tick`: the third tick is its last; the taker's place is followed.
    pub fn tick(&mut self, target_at: (f64, f64, f64)) {
        self.life += 1;
        if self.life == LIFE_TIME {
            self.removed = true;
        }
        self.target_old = self.target_now;
        self.target_now = target_at;
    }

    /// Where the taker was last seen.
    pub(super) fn target_at(&self) -> (f64, f64, f64) {
        self.target_now
    }

    pub fn is_alive(&self) -> bool {
        !self.removed
    }

    /// `ItemPickupParticleGroup.ParticleInstance.fromParticle`: where it is drawn at a partial
    /// tick, the way to the taker eased in by the square of the time.
    pub fn position_at(&self, partial: f32) -> (f64, f64, f64) {
        let mut time = (self.life as f32 + partial) / LIFE_TIME as f32;
        time *= time;
        let lerp = |delta: f64, from: f64, to: f64| from + delta * (to - from);
        let partial = f64::from(partial);
        let target = (
            lerp(partial, self.target_old.0, self.target_now.0),
            lerp(partial, self.target_old.1, self.target_now.1),
            lerp(partial, self.target_old.2, self.target_now.2),
        );
        let time = f64::from(time);
        (
            lerp(time, self.from.0, target.0),
            lerp(time, self.from.1, target.1),
            lerp(time, self.from.2, target.2),
        )
    }
}
