//! Item component patches as the network carries them.
//!
//! A stack's patch says which data components it adds, with their values,
//! and which it removes. The client has to read every value to find where
//! the stack ends, and to send a stack back it has to write each value
//! behind its length (`DataComponentPatch.DELIMITED_STREAM_CODEC`). Neither
//! needs the values themselves, so each component's stream codec is ported
//! as a skip: it moves over the bytes the reference's codec would read.
//!
//! A stack whose component predicate names a partial predicate is refused:
//! this port does not read those.

/// The bytes are not what the codec reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Malformed;

/// The registry ids of the data component types, named.
pub use crate::item::component_types as ids;

mod adventure;
pub use adventure::*;
mod components;
pub use components::*;
mod cursor;
pub use cursor::*;
pub mod hash;
pub mod tag;
