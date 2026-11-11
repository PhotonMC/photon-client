//! The leaves blocks' falling leaves (`LeavesBlock.leafParticleChance` and the leaf each lets fall).

use crate::tables::leaves::{LEAVES, LEGACY_SPRUCE};

/// One row: the block, its chance of letting a leaf fall, the leaf's particle type and its colour.
pub type LeafRow = (&'static str, f32, &'static str, Option<i32>);

/// A leaves block's chance of letting a leaf fall each pick, the leaf's particle type (`tinted`:
/// one coloured by the block's tint) and its own colour; none for another block.
pub fn of(block: &str) -> Option<(f32, &'static str, Option<i32>)> {
    all()
        .into_iter()
        .find(|(name, ..)| *name == block)
        .map(|(_, chance, particle, color)| (chance, particle, color))
}

/// Every leaves block of the active version, in registry order.
pub fn all() -> Vec<LeafRow> {
    let mut rows = LEAVES.to_vec();
    if photon_core::versions::rules()
        .protocol
        .spruce_leaves_are_tinted
    {
        rows.insert(1, LEGACY_SPRUCE);
    }
    rows
}
