//! The blocks fire catches on (`FireBlock.igniteOdds` and `burnOdds`).

use crate::tables::fire::FLAMMABLE;

/// A block's ignite and burn odds; none for a block fire does not catch on.
pub fn odds(block: &str) -> Option<(u8, u8)> {
    FLAMMABLE
        .iter()
        .find(|(name, ..)| *name == block)
        .map(|&(_, ignite, burn)| (ignite, burn))
}

/// Every block fire catches on, in registry order: its name, ignite and burn odds.
pub fn all() -> &'static [(&'static str, u8, u8)] {
    &FLAMMABLE
}
