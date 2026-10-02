//! The entity type registry: each type's name, and its id (its place in the
//! list, which is what the wire carries).

use crate::generated::entity_picks::ENTITY_PICKS;
use crate::generated::entity_types::ENTITY_TYPES;

/// The name of the type with this id (`minecraft:pig`).
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| ENTITY_TYPES.get(index))
        .copied()
}

/// The id of the type with this name.
pub fn id(name: &str) -> Option<i32> {
    ENTITY_TYPES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| index as i32)
}

/// What the crosshair needs of an entity type, as a new entity of it answers
/// (`isPickable`, `getPickRadius`, `canBePickedFromInside`, and its size in
/// the standing pose with a baby's, each as width, height, eye height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityPick {
    pub pickable: bool,
    pub pick_radius: f32,
    pub from_inside: bool,
    pub size: (f32, f32, f32),
    /// Only the types a baby can be.
    pub baby: Option<(f32, f32, f32)>,
}

/// The crosshair's answers for the type with this id; the player has none.
pub fn pick(id: i32) -> Option<&'static EntityPick> {
    usize::try_from(id)
        .ok()
        .and_then(|index| ENTITY_PICKS.get(index))
        .and_then(Option::as_ref)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_ids_go_both_ways() {
        assert_eq!(name(156), Some("minecraft:player"));
        assert_eq!(id("minecraft:player"), Some(156));
        assert_eq!(id("minecraft:cow"), Some(30));
        assert_eq!(name(-1), None);
        assert_eq!(name(100_000), None);
        assert_eq!(id("minecraft:nope"), None);
    }
}
