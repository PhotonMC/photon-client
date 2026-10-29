//! What a lightning bolt does of its own on the client (`LightningBolt`): the seed its shape
//! is drawn from, how many more times it flashes, and when it is gone. The reference draws
//! the seed and the flashes from the entity's own random source, which it does not seed.

use photon_core::random::RandomSource;

/// `LightningBolt.START_LIFE`.
const START_LIFE: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningState {
    life: i32,
    flashes: i32,
    seed: i64,
    /// What the entity's random source is drawn from, and how many times it has been.
    entity_seed: i64,
    draws: i64,
    /// The pitches (as bits) of the thunder and the impact the first tick found to play.
    pitches: Option<[u32; 2]>,
    /// Whether the last tick left no flashes: the bolt is discarded (`LightningBolt.discard`).
    done: bool,
}

impl LightningState {
    /// A bolt as it is made: its seed, then the flashes it has left (one to three).
    pub fn new(entity_seed: i64) -> Self {
        let mut random = RandomSource::create(entity_seed);
        let seed = random.next_long();
        let flashes = random.next_int_bound(3).unwrap_or_default() + 1;
        Self {
            life: START_LIFE,
            flashes,
            seed,
            entity_seed,
            draws: 0,
            pitches: None,
            done: false,
        }
    }

    /// The next run of the entity's random source.
    fn random(&mut self) -> RandomSource {
        self.draws += 1;
        RandomSource::create(self.entity_seed ^ self.draws.wrapping_mul(0x2545_F491_4F6C_DD1D))
    }

    /// The pitches of the thunder and the impact the bolt strikes with, once, on its first tick.
    pub(crate) fn take_pitches(&mut self) -> Option<(f32, f32)> {
        self.pitches
            .take()
            .map(|[thunder, impact]| (f32::from_bits(thunder), f32::from_bits(impact)))
    }

    /// `LightningBolt.seed`: what the bolt's shape is drawn from.
    pub fn seed(&self) -> i64 {
        self.seed
    }

    /// `LightningBolt.life >= 0`: the bolt is lit, and flashes the sky.
    pub fn is_lit(&self) -> bool {
        self.life >= 0
    }

    /// `LightningBolt.tick` on the client: the life runs down; once it is spent, the bolt flashes
    /// again after a random wait with a new shape, or, with no flashes left, is discarded (true).
    pub(crate) fn tick(&mut self) -> bool {
        self.done = self.step();
        self.done
    }

    /// Whether the last tick discarded the bolt.
    pub(crate) fn finished(&self) -> bool {
        self.done
    }

    fn step(&mut self) -> bool {
        if self.life == START_LIFE {
            let mut random = self.random();
            let thunder = 0.8 + random.next_float() * 0.2;
            let impact = 0.5 + random.next_float() * 0.2;
            self.pitches = Some([thunder.to_bits(), impact.to_bits()]);
        }
        self.life -= 1;
        if self.life < 0 {
            if self.flashes == 0 {
                return true;
            }
            let mut random = self.random();
            let wait = random.next_int_bound(10).unwrap_or_default();
            if self.life < -wait {
                self.flashes -= 1;
                self.life = 1;
                self.seed = random.next_long();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bolt_flashes_and_is_then_discarded() {
        let mut bolt = LightningState::new(7);
        let first = bolt.seed();
        let mut ticks = 0;
        let mut seeds = std::collections::BTreeSet::from([first]);
        while !bolt.tick() {
            seeds.insert(bolt.seed());
            ticks += 1;
            assert!(ticks < 200, "a bolt does not last for ever");
        }
        // One to three flashes: the first seed and up to two new ones.
        assert!((1..=3).contains(&seeds.len()));
    }

    #[test]
    fn a_bolt_strikes_once_on_its_first_tick() {
        let mut bolt = LightningState::new(7);
        assert_eq!(bolt.take_pitches(), None);
        bolt.tick();
        let (thunder, impact) = bolt.take_pitches().expect("the first tick strikes");
        assert!((0.8..1.0).contains(&thunder) && (0.5..0.7).contains(&impact));
        while !bolt.tick() {
            assert_eq!(bolt.take_pitches(), None);
        }
    }

    #[test]
    fn a_bolt_lights_the_sky_while_its_life_is_not_spent() {
        let mut bolt = LightningState::new(7);
        // Made with a life of two: the first tick leaves one, the second none.
        assert!(!bolt.tick() && bolt.is_lit());
        assert!(!bolt.tick() && bolt.is_lit());
        // The third spends it: it is gone, or waits dark, or flashes again with a life of one.
        let gone = bolt.tick();
        assert_eq!(bolt.is_lit(), !gone && bolt.life == 1);
    }
}
