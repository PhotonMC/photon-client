//! The leaves blocks' falling leaves (`LeavesBlock.leafParticleChance` and the leaf each lets fall).

use crate::tables::leaves::LEAVES;

/// A leaves block's chance of letting a leaf fall each pick, the leaf's particle type (`tinted`:
/// one coloured by the block's tint) and its own colour; none for another block.
pub fn of(block: &str) -> Option<(f32, &'static str, Option<i32>)> {
    LEAVES
        .iter()
        .find(|(name, ..)| *name == block)
        .map(|&(_, chance, particle, color)| (chance, particle, color))
}

/// Every leaves block, in registry order.
pub fn all() -> &'static [(&'static str, f32, &'static str, Option<i32>)] {
    &LEAVES
}
