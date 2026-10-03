//! What a panda does of its own on the client (`Panda.tick`): the data it
//! counts down or up itself (the unhappy, sneeze and eat counters, the roll),
//! how far it sits, lies on its back and rolls, and the chance it starts to
//! eat. Its flags and genes are synched data.

use voxel_core::math::mth_lerp_f32 as lerp_f32;
use voxel_core::random::RandomSource;

/// `Panda.DATA_ID_FLAGS` bits.
const SNEEZE: u8 = 2;
const ROLL: u8 = 4;
const SIT: u8 = 8;
const ON_BACK: u8 = 16;

/// How many ticks a roll lasts (`TOTAL_ROLL_STEPS`).
const TOTAL_ROLL_STEPS: i32 = 32;
/// A sneeze ends once its counter passes this.
const SNEEZE_TICKS: i32 = 20;

/// `Panda.Gene` ids that are recessive (brown, weak) and the worried one.
const BROWN: u8 = 4;
const WEAK: u8 = 5;
const WORRIED: u8 = 2;

/// The source the chance to eat is drawn from: the entity's own, which the
/// reference does not seed (made when first drawn from, by the clock). Two
/// states never differ by it.
#[derive(Debug, Clone, Default)]
struct EntityRandom(Option<RandomSource>);

impl PartialEq for EntityRandom {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for EntityRandom {}

impl EntityRandom {
    fn source(&mut self) -> &mut RandomSource {
        self.0.get_or_insert_with(|| {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as i64);
            RandomSource::create(seed)
        })
    }
}

/// What a panda's tick reads of the world around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PandaSurroundings {
    pub(crate) thundering: bool,
    pub(crate) holds_item: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PandaState {
    unhappy_counter: i32,
    sneeze_counter: i32,
    eat_counter: i32,
    main_gene: u8,
    hidden_gene: u8,
    flags: u8,
    sit_amount: SmoothAmount,
    on_back_amount: SmoothAmount,
    roll_amount: SmoothAmount,
    roll_counter: i32,
    random: EntityRandom,
}

/// An amount and the last tick's, which the drawing blends between.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SmoothAmount {
    now: f32,
    before: f32,
}

// The amounts are finite, so equality is total.
impl Eq for SmoothAmount {}

impl SmoothAmount {
    /// One tick towards one when `up`, else towards zero, at these speeds.
    fn step(&mut self, up: bool) {
        self.before = self.now;
        self.now = if up {
            (self.now + 0.15).min(1.0)
        } else {
            (self.now - 0.19).max(0.0)
        };
    }

    fn at(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.before, self.now)
    }
}

/// What a panda's data holds, each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PandaData {
    pub(crate) unhappy_counter: Option<i32>,
    pub(crate) sneeze_counter: Option<i32>,
    pub(crate) eat_counter: Option<i32>,
    pub(crate) main_gene: Option<u8>,
    pub(crate) hidden_gene: Option<u8>,
    pub(crate) flags: Option<u8>,
}

impl PandaState {
    pub(crate) fn apply(&mut self, data: PandaData) {
        if let Some(value) = data.unhappy_counter {
            self.unhappy_counter = value;
        }
        if let Some(value) = data.sneeze_counter {
            self.sneeze_counter = value;
        }
        if let Some(value) = data.eat_counter {
            self.eat_counter = value;
        }
        if let Some(value) = data.main_gene {
            self.main_gene = value;
        }
        if let Some(value) = data.hidden_gene {
            self.hidden_gene = value;
        }
        if let Some(value) = data.flags {
            self.flags = value;
        }
    }

    /// Fixes what the reference draws from an unseeded source, so a comparison
    /// can be made.
    pub fn fix_seed(&mut self, seed: i64) {
        self.random.0 = Some(RandomSource::create(seed));
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    /// `Panda.Gene.getVariantFromGenes`: the gene's id (zero, normal, for an
    /// id the genes lack); a recessive main gene shows only when the hidden
    /// one is the same.
    pub fn variant(&self) -> u8 {
        let gene = |id: u8| if id > 6 { 0 } else { id };
        let (main, hidden) = (gene(self.main_gene), gene(self.hidden_gene));
        if matches!(main, BROWN | WEAK) {
            if main == hidden { main } else { 0 }
        } else {
            main
        }
    }

    fn is_worried(&self) -> bool {
        self.variant() == WORRIED
    }

    /// `Panda.getUnhappyCounter`.
    pub fn unhappy_counter(&self) -> i32 {
        self.unhappy_counter
    }

    pub fn is_unhappy(&self) -> bool {
        self.unhappy_counter > 0
    }

    pub fn is_sneezing(&self) -> bool {
        self.flags & SNEEZE != 0
    }

    pub fn sneeze_counter(&self) -> i32 {
        self.sneeze_counter
    }

    pub fn is_eating(&self) -> bool {
        self.eat_counter > 0
    }

    pub fn is_sitting(&self) -> bool {
        self.flags & SIT != 0
    }

    pub fn is_on_back(&self) -> bool {
        self.flags & ON_BACK != 0
    }

    pub fn is_rolling(&self) -> bool {
        self.flags & ROLL != 0
    }

    /// `Panda.isScared`.
    pub fn is_scared(&self, thundering: bool) -> bool {
        self.is_worried() && thundering
    }

    pub fn roll_counter(&self) -> i32 {
        self.roll_counter
    }

    /// `Panda.getSitAmount`.
    pub fn sit_amount(&self, partial: f32) -> f32 {
        self.sit_amount.at(partial)
    }

    /// `Panda.getLieOnBackAmount`.
    pub fn lie_on_back_amount(&self, partial: f32) -> f32 {
        self.on_back_amount.at(partial)
    }

    /// `Panda.getRollAmount`.
    pub fn roll_amount(&self, partial: f32) -> f32 {
        self.roll_amount.at(partial)
    }

    /// One client tick (`Panda.tick` after `super.tick`), and whether the
    /// panda sets its pitch to zero (it does while it sits).
    pub(crate) fn tick(&mut self, around: PandaSurroundings) -> bool {
        // Whether the panda is in water is not tracked: it never is.
        if self.is_worried() {
            if around.thundering {
                self.set_flag(SIT, true);
                self.eat_counter = 0;
            } else if !self.is_eating() {
                self.set_flag(SIT, false);
            }
        }
        if self.unhappy_counter > 0 {
            self.unhappy_counter -= 1;
        }
        if self.is_sneezing() {
            self.sneeze_counter += 1;
            if self.sneeze_counter > SNEEZE_TICKS {
                self.set_flag(SNEEZE, false);
                self.sneeze_counter = 0;
            }
        }
        if self.is_rolling() {
            self.roll_counter += 1;
            if self.roll_counter > TOTAL_ROLL_STEPS {
                self.set_flag(ROLL, false);
            }
        } else {
            self.roll_counter = 0;
        }
        let sitting = self.is_sitting();
        self.sit_amount.step(sitting);
        self.handle_eating(around);
        let on_back = self.is_on_back();
        self.on_back_amount.step(on_back);
        let rolling = self.is_rolling();
        self.roll_amount.step(rolling);
        sitting
    }

    /// `Panda.handleEating`: it may start to eat what it holds while it sits,
    /// and counts how long it has eaten. The draws the reference's particles
    /// and sounds make are made too, so the source runs on as it does.
    fn handle_eating(&mut self, around: PandaSurroundings) {
        let scared = self.is_scared(around.thundering);
        if !self.is_eating()
            && self.is_sitting()
            && !scared
            && around.holds_item
            && self.random.source().next_int_bound(80) == Ok(1)
        {
            self.eat_counter = 1;
        } else if !around.holds_item || !self.is_sitting() {
            self.eat_counter = 0;
        }
        if self.is_eating() {
            if self.eat_counter % 5 == 0 {
                let random = self.random.source();
                // The sound's volume and pitch, then six bits of the item's
                // break particle (three draws of velocity, three of position).
                let _ = random.next_int_bound(2);
                random.next_float();
                random.next_float();
                for _ in 0..36 {
                    random.next_float();
                }
            }
            self.eat_counter += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTHING: PandaSurroundings = PandaSurroundings {
        thundering: false,
        holds_item: false,
    };

    fn with_flags(flags: u8) -> PandaState {
        let mut panda = PandaState::default();
        panda.apply(PandaData {
            flags: Some(flags),
            ..PandaData::default()
        });
        panda
    }

    #[test]
    fn a_recessive_gene_shows_only_when_both_genes_are_it() {
        let mut panda = PandaState::default();
        let genes = |main, hidden| PandaData {
            main_gene: Some(main),
            hidden_gene: Some(hidden),
            ..PandaData::default()
        };
        panda.apply(genes(BROWN, BROWN));
        assert_eq!(panda.variant(), BROWN);
        panda.apply(genes(BROWN, 1));
        assert_eq!(panda.variant(), 0);
        panda.apply(genes(3, WEAK));
        assert_eq!(panda.variant(), 3);
        panda.apply(genes(9, 9));
        assert_eq!(panda.variant(), 0);
    }

    #[test]
    fn a_sneeze_counts_up_and_ends_with_the_counter_reset() {
        let mut panda = with_flags(SNEEZE);
        for _ in 0..20 {
            panda.tick(NOTHING);
        }
        assert!(panda.is_sneezing());
        assert_eq!(panda.sneeze_counter(), 20);
        panda.tick(NOTHING);
        assert!(!panda.is_sneezing());
        assert_eq!(panda.sneeze_counter(), 0);
    }

    #[test]
    fn a_roll_lasts_thirty_two_ticks_and_the_counter_resets_after() {
        let mut panda = with_flags(ROLL);
        for _ in 0..32 {
            panda.tick(NOTHING);
        }
        assert!(panda.is_rolling());
        panda.tick(NOTHING);
        assert!(!panda.is_rolling());
        panda.tick(NOTHING);
        assert_eq!(panda.roll_counter(), 0);
    }

    #[test]
    fn a_worried_panda_sits_in_thunder_and_a_sitting_one_levels_its_pitch() {
        let mut panda = PandaState::default();
        panda.apply(PandaData {
            main_gene: Some(WORRIED),
            ..PandaData::default()
        });
        let thunder = PandaSurroundings {
            thundering: true,
            holds_item: false,
        };
        assert!(panda.tick(thunder));
        assert!(panda.is_sitting() && panda.is_scared(true));
        assert!(!panda.tick(NOTHING));
        assert!(!panda.is_sitting());
    }

    #[test]
    fn eating_runs_while_it_sits_with_something_and_stops_without() {
        let mut panda = with_flags(SIT);
        panda.fix_seed(7);
        panda.apply(PandaData {
            eat_counter: Some(1),
            ..PandaData::default()
        });
        let holding = PandaSurroundings {
            thundering: false,
            holds_item: true,
        };
        for _ in 0..4 {
            panda.tick(holding);
        }
        assert!(panda.is_eating());
        panda.tick(NOTHING);
        assert!(!panda.is_eating());
    }
}
