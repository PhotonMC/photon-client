//! The shadow each entity type's renderer sets up (`EntityRenderer.shadowRadius`, `shadowStrength`).

use crate::tables::entity_shadows::{SHADOWS, UNBUILT};

/// A type's renderer-set shadow radius and strength; none for a type with no renderer listed.
pub fn of(entity_type: &str) -> Option<(f32, f32)> {
    SHADOWS
        .iter()
        .find(|(name, ..)| *name == entity_type)
        .map(|&(_, radius, strength)| (radius, strength))
}

/// Every listed type: its name, radius and strength.
pub fn all() -> &'static [(&'static str, f32, f32)] {
    &SHADOWS
}

/// The types whose renderer the oracle could not build.
pub fn unbuilt() -> &'static [&'static str] {
    &UNBUILT
}
