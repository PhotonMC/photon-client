//! Typed ids. A number that names one thing in one table is a type of its own, so that an entity
//! id cannot be passed where an entity type id is wanted. The number is public: a value is made as
//! `EntityId(7)` and read as `id.0`, at the places where the wire, a file or a test gives it as a
//! plain number. `Debug` and `Display` print the number alone, as the plain number printed.

use std::fmt;

/// Defines an id type: a newtype of the wire's `i32` with the number's own order, `Debug` and
/// `Display`.
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub i32);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

id_type!(
    /// An entity's id on the server's wire (`Entity.getId`), which is also the local player's.
    EntityId
);

id_type!(
    /// An entity type's id in the `minecraft:entity_type` registry (`EntityType`), the number
    /// the server sends with an added entity.
    EntityTypeId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entity_id_orders_and_prints_as_its_number() {
        assert!(EntityId(-3) < EntityId(2));
        assert_eq!(format!("{:?} {}", EntityId(7), EntityId(7)), "7 7");
        assert_eq!(
            format!("{:?} {}", EntityTypeId(156), EntityTypeId(156)),
            "156 156"
        );
    }
}
