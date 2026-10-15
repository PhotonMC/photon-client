//! Typed ids. A number that names one thing in one table is a type of its own, so that an entity
//! id cannot be passed where an entity type id is wanted. The number is public: a value is made as
//! `EntityId(7)` and read as `id.0`, at the places where the wire, a file or a test gives it as a
//! plain number. `Debug` and `Display` print the number alone, as the plain number printed.

use std::fmt;

/// An entity's id on the server's wire (`Entity.getId`), which is also the local player's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub i32);

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entity_id_orders_and_prints_as_its_number() {
        assert!(EntityId(-3) < EntityId(2));
        assert_eq!(format!("{:?} {}", EntityId(7), EntityId(7)), "7 7");
    }
}
