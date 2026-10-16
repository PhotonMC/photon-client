//! The block entities the level ticks (`Level.tickBlockEntities`): the order they were set up in,
//! the ones set up during a pass waiting for the next, and the ones removed dropped as the pass
//! reaches them.

use crate::block::piston::Pos;

/// One block entity's ticker: where it is, and whether the entity there was removed.
#[derive(Debug, Clone)]
struct Slot {
    pos: Pos,
    removed: bool,
}

/// `Level.blockEntityTickers` and `pendingBlockEntityTickers`.
#[derive(Debug, Clone, Default)]
pub(crate) struct BlockTickers {
    slots: Vec<Slot>,
    pending: Vec<Slot>,
    ticking: bool,
}

impl BlockTickers {
    /// `LevelChunk.updateBlockEntityTicker` for a block entity that has a ticker: one already
    /// ticking at the place is taken over by the new entity, and keeps its turn.
    pub(crate) fn register(&mut self, pos: Pos) {
        let live = |slot: &&mut Slot| slot.pos == pos && !slot.removed;
        if self
            .slots
            .iter_mut()
            .chain(self.pending.iter_mut())
            .find(live)
            .is_some()
        {
            return;
        }
        let slot = Slot {
            pos,
            removed: false,
        };
        if self.ticking {
            self.pending.push(slot);
        } else {
            self.slots.push(slot);
        }
    }

    /// `LevelChunk.removeBlockEntityTicker`: the ticker at a place is dropped when the pass
    /// next reaches it.
    pub(crate) fn unregister(&mut self, pos: Pos) {
        for slot in self.slots.iter_mut().chain(self.pending.iter_mut()) {
            if slot.pos == pos {
                slot.removed = true;
            }
        }
    }

    /// Where the tickers are that have not been removed.
    pub(crate) fn positions(&self) -> impl Iterator<Item = Pos> + '_ {
        self.slots
            .iter()
            .chain(&self.pending)
            .filter(|slot| !slot.removed)
            .map(|slot| slot.pos)
    }

    /// The start of a pass: what waited joins the list.
    pub(crate) fn begin_pass(&mut self) {
        self.ticking = true;
        let waiting = std::mem::take(&mut self.pending);
        self.slots.extend(waiting);
    }

    /// The end of a pass.
    pub(crate) fn end_pass(&mut self) {
        self.ticking = false;
    }

    /// The place of the ticker at `index` for the pass to tick, dropping the removed ones on the
    /// way; `None` at the end of the list.
    pub(crate) fn next_to_tick(&mut self, index: &mut usize) -> Option<Pos> {
        while *index < self.slots.len() {
            if self.slots[*index].removed {
                self.slots.remove(*index);
                continue;
            }
            let pos = self.slots[*index].pos;
            *index += 1;
            return Some(pos);
        }
        None
    }

    /// Whether the entity at a place has a ticker.
    #[cfg(test)]
    pub(crate) fn has(&self, pos: Pos) -> bool {
        self.slots
            .iter()
            .chain(&self.pending)
            .any(|slot| slot.pos == pos && !slot.removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pass(tickers: &mut BlockTickers) -> Vec<Pos> {
        tickers.begin_pass();
        let mut index = 0;
        let mut ticked = Vec::new();
        while let Some(pos) = tickers.next_to_tick(&mut index) {
            ticked.push(pos);
        }
        tickers.end_pass();
        ticked
    }

    #[test]
    fn tickers_tick_in_the_order_they_were_set_up() {
        let mut tickers = BlockTickers::default();
        tickers.register((3, 0, 0));
        tickers.register((1, 0, 0));
        tickers.register((2, 0, 0));
        assert_eq!(pass(&mut tickers), [(3, 0, 0), (1, 0, 0), (2, 0, 0)]);
    }

    #[test]
    fn a_ticker_set_up_during_a_pass_waits_for_the_next() {
        let mut tickers = BlockTickers::default();
        tickers.register((1, 0, 0));
        tickers.begin_pass();
        tickers.register((2, 0, 0));
        let mut index = 0;
        assert_eq!(tickers.next_to_tick(&mut index), Some((1, 0, 0)));
        assert_eq!(tickers.next_to_tick(&mut index), None);
        tickers.end_pass();
        assert_eq!(pass(&mut tickers), [(1, 0, 0), (2, 0, 0)]);
    }

    #[test]
    fn a_removed_ticker_is_dropped_and_a_new_one_goes_last() {
        let mut tickers = BlockTickers::default();
        tickers.register((1, 0, 0));
        tickers.register((2, 0, 0));
        tickers.unregister((1, 0, 0));
        tickers.register((1, 0, 0));
        assert_eq!(pass(&mut tickers), [(2, 0, 0), (1, 0, 0)]);
        assert!(tickers.has((1, 0, 0)));
    }

    #[test]
    fn an_entity_set_over_another_keeps_its_turn() {
        let mut tickers = BlockTickers::default();
        tickers.register((1, 0, 0));
        tickers.register((2, 0, 0));
        tickers.register((1, 0, 0));
        assert_eq!(pass(&mut tickers), [(1, 0, 0), (2, 0, 0)]);
    }
}
