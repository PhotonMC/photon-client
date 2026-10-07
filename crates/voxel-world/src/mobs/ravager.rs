//! What a ravager does of its own on the client (`Ravager.aiStep`,
//! `handleEntityEvent`): the attack, stun and roar counters, which only the
//! server's events and its own ticks move.

use crate::entity_random::EntityRandom;

/// The events: an attack swing, a stun, and the roar's burst.
pub(crate) const ATTACK_EVENT: i8 = 4;
pub(crate) const STUN_EVENT: i8 = 39;
pub(crate) const ROAR_EVENT: i8 = 69;

const ATTACK_TICKS: i32 = 10;
const STUN_TICKS: i32 = 40;
const ROAR_TICKS: i32 = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RavagerState {
    attack_tick: i32,
    stunned_tick: i32,
    roar_tick: i32,
    random: EntityRandom,
}

impl RavagerState {
    /// `Ravager.handleEntityEvent`; whether the event was one of its own. The
    /// roar's burst of particles draws forty times three gaussians.
    pub(crate) fn event(&mut self, event: i8) -> bool {
        match event {
            ATTACK_EVENT => self.attack_tick = ATTACK_TICKS,
            STUN_EVENT => self.stunned_tick = STUN_TICKS,
            ROAR_EVENT => {
                let random = self.random.source();
                for _ in 0..120 {
                    random.next_gaussian();
                }
            }
            _ => return false,
        }
        true
    }

    /// One client tick (`Ravager.aiStep` after `super.aiStep`): a living
    /// ravager's roar, attack and stun run down; the stun's effect draws a
    /// chance for each tick, and the roar follows the stun.
    pub(crate) fn tick(&mut self, alive: bool) {
        if !alive {
            return;
        }
        if self.roar_tick > 0 {
            self.roar_tick -= 1;
        }
        if self.attack_tick > 0 {
            self.attack_tick -= 1;
        }
        if self.stunned_tick > 0 {
            self.stunned_tick -= 1;
            let random = self.random.source();
            // `stunEffect`: one in six throws a cloud, from two more draws.
            if random.next_int_bound(6) == Ok(0) {
                random.next_double();
                random.next_double();
            }
            if self.stunned_tick == 0 {
                self.roar_tick = ROAR_TICKS;
            }
        }
    }

    /// Fixes what the reference draws from an unseeded source, so a comparison
    /// can be made.
    pub fn fix_seed(&mut self, seed: i64) {
        self.random.fix_seed(seed);
    }

    pub fn attack_tick(&self) -> i32 {
        self.attack_tick
    }

    pub fn stunned_tick(&self) -> i32 {
        self.stunned_tick
    }

    pub fn roar_tick(&self) -> i32 {
        self.roar_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_attack_swing_runs_ten_ticks() {
        let mut ravager = RavagerState::default();
        assert!(ravager.event(ATTACK_EVENT));
        for _ in 0..9 {
            ravager.tick(true);
        }
        assert_eq!(ravager.attack_tick(), 1);
        ravager.tick(true);
        assert_eq!(ravager.attack_tick(), 0);
        assert!(!ravager.event(1));
    }

    #[test]
    fn a_stun_ends_in_a_roar_of_twenty_ticks_and_a_dead_ravager_stands_still() {
        let mut ravager = RavagerState::default();
        ravager.fix_seed(1942);
        ravager.event(STUN_EVENT);
        ravager.tick(false);
        assert_eq!(ravager.stunned_tick(), 40);
        for _ in 0..40 {
            ravager.tick(true);
        }
        assert_eq!((ravager.stunned_tick(), ravager.roar_tick()), (0, 20));
        for _ in 0..20 {
            ravager.tick(true);
        }
        assert_eq!(ravager.roar_tick(), 0);
    }
}
