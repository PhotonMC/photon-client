//! The entity type registry: each type's name, and its id (its place in the
//! list, which is what the wire carries).

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
