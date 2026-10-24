//! Resource packs (`net.minecraft.server.packs`): where a pack's files come from, what its
//! `pack.mcmeta` says, and the stack of packs the game reads its assets through
//! (`MultiPackResourceManager`).

mod meta;
mod pack;
mod source;
mod stack;

pub use meta::{IdentifierPattern, PackFormat, format_range};
pub use pack::{Pack, PackError};
pub use source::{DirectorySource, Source, ZipSource};
pub use stack::{Resource, ResourceStack};
