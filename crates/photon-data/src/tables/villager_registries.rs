//! The villager types and professions, with the hat each one's texture asks for.
//! Names are the registry paths, in wire order.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

use crate::villager::Hat;

#[rustfmt::skip]
pub static VILLAGER_TYPES: [(&str, Hat); 7] = [
    ("desert", Hat::Full),
    ("jungle", Hat::None),
    ("plains", Hat::None),
    ("savanna", Hat::None),
    ("snow", Hat::Full),
    ("swamp", Hat::None),
    ("taiga", Hat::None),
];

#[rustfmt::skip]
pub static VILLAGER_PROFESSIONS: [(&str, Hat); 15] = [
    ("none", Hat::None),
    ("armorer", Hat::None),
    ("butcher", Hat::Partial),
    ("cartographer", Hat::None),
    ("cleric", Hat::None),
    ("farmer", Hat::Full),
    ("fisherman", Hat::Full),
    ("fletcher", Hat::Full),
    ("leatherworker", Hat::None),
    ("librarian", Hat::Full),
    ("mason", Hat::None),
    ("nitwit", Hat::None),
    ("shepherd", Hat::Full),
    ("toolsmith", Hat::None),
    ("weaponsmith", Hat::None),
];
