//! What an end crystal does of its own on the client: its age, which turns and
//! bobs its glass (`EndCrystal.time`), and the data the server syncs (the block its
//! beam reaches, whether it shows its base).

use voxel_core::binary::BlockPos;
use voxel_core::random::RandomSource;

/// `EndCrystal`'s data ids: `Entity` has eight, then these.
pub const BEAM_TARGET_DATA_ID: u8 = 8;
pub const SHOW_BOTTOM_DATA_ID: u8 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCrystalState {
    /// `EndCrystal.time`: starts at a random whole number below a hundred thousand.
    time: i32,
    pub(crate) beam_target: Option<BlockPos>,
    pub(crate) show_bottom: bool,
}

impl EndCrystalState {
    /// A crystal as it is made: its time from the entity's own random source, which
    /// the reference does not seed.
    pub fn new(seed: i64) -> Self {
        let time = RandomSource::create(seed)
            .next_int_bound(100_000)
            .unwrap_or_default();
        Self {
            time,
            beam_target: None,
            show_bottom: true,
        }
    }

    pub fn time(&self) -> i32 {
        self.time
    }

    /// `EndCrystal.getBeamTarget`.
    pub fn beam_target(&self) -> Option<BlockPos> {
        self.beam_target
    }

    /// `EndCrystal.showsBottom`.
    pub fn shows_bottom(&self) -> bool {
        self.show_bottom
    }

    /// `EndCrystal.tick`.
    pub(crate) fn tick(&mut self) {
        self.time = self.time.wrapping_add(1);
    }
}
