//! What a block of primed tnt does of its own on the client (`PrimedTnt`): the fuse the server
//! synced, counted down each tick (the block is gone when it runs out), and the block state it
//! shows (the tnt block unless the server says another).

/// Where a primed tnt's fuse (an int) and block state (a block state) sit among its data
/// (`Entity` has eight).
pub const FUSE_DATA_ID: u8 = 8;
pub const BLOCK_STATE_DATA_ID: u8 = 9;

/// `EntityDataSerializers.BLOCK_STATE`.
pub const BLOCK_STATE_SERIALIZER_ID: i32 = 14;

/// `PrimedTnt.DEFAULT_FUSE_TIME`.
const DEFAULT_FUSE: i32 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimedTntState {
    fuse: i32,
    block_state: Option<u32>,
}

impl Default for PrimedTntState {
    fn default() -> Self {
        Self {
            fuse: DEFAULT_FUSE,
            block_state: None,
        }
    }
}

impl PrimedTntState {
    /// `PrimedTnt.getFuse`.
    pub fn fuse(&self) -> i32 {
        self.fuse
    }

    /// `PrimedTnt.getBlockState`: the state the server named, else the tnt block's own.
    pub fn block_state(&self) -> Option<u32> {
        self.block_state.or_else(|| {
            photon_data::block::registry::block("minecraft:tnt")
                .map(|block| block.default_state_id())
        })
    }

    pub(crate) fn set_fuse(&mut self, fuse: i32) {
        self.fuse = fuse;
    }

    pub(crate) fn set_block_state(&mut self, state: u32) {
        self.block_state = Some(state);
    }

    /// `PrimedTnt.tick`'s fuse: it runs down by one, and a fuse spent discards the block.
    pub(crate) fn tick(&mut self) {
        self.fuse -= 1;
    }

    /// Whether the fuse is spent (`fuse <= 0`).
    pub fn is_spent(&self) -> bool {
        self.fuse <= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fuse_runs_down_by_one_each_tick_and_is_spent_at_zero() {
        let mut state = PrimedTntState::default();
        assert_eq!(state.fuse(), 80);
        state.set_fuse(2);
        state.tick();
        assert!(!state.is_spent());
        state.tick();
        assert!(state.is_spent());
    }

    #[test]
    fn the_block_is_the_tnt_block_until_the_server_says_another() {
        let mut state = PrimedTntState::default();
        let tnt = state.block_state();
        assert!(tnt.is_some());
        state.set_block_state(1);
        assert_eq!(state.block_state(), Some(1));
    }
}
