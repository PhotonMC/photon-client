//! Where the game's behaviour differs by version ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)):
//! game code asks a named question here and never names a version.
//!
//! - [`rules`] is the declarative table of every yes/no difference, one row per rule grouped by
//!   domain; code reads a field of [`rules()`].
//! - [`behaviour`] keeps what is not a plain before/since flag: values that differ by version
//!   (data version, pack format, parser ids), the per-version table slot and the asset version
//!   override.

pub mod behaviour;
pub mod rules;

pub use rules::{Rules, rules};
